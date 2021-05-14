use {
    crate::{
        error::MetaplexError,
        state::{
            AuctionManager, AuctionManagerStatus, BidRedemptionTicket, Key,
            OriginalAuthorityLookup, Store, WhitelistedCreator, WinningConfig, WinningConfigState,
            PREFIX,
        },
    },
    borsh::BorshSerialize,
    solana_program::{
        account_info::AccountInfo,
        borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_option::COption,
        program_pack::{IsInitialized, Pack},
        pubkey::Pubkey,
        system_instruction,
        sysvar::{rent::Rent, Sysvar},
    },
    spl_auction::{
        instruction::start_auction_instruction,
        processor::{start_auction::StartAuctionArgs, AuctionData, AuctionState, BidderMetadata},
    },
    spl_token::{
        instruction::{set_authority, AuthorityType},
        state::{Account, Mint},
    },
    spl_token_metadata::{
        instruction::update_metadata_accounts,
        state::{MasterEdition, Metadata, EDITION},
    },
    spl_token_vault::{instruction::create_withdraw_tokens_instruction, state::SafetyDepositBox},
    std::convert::TryInto,
};

/// assert initialized account
pub fn assert_initialized<T: Pack + IsInitialized>(
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        Err(MetaplexError::Uninitialized.into())
    } else {
        Ok(account)
    }
}

pub fn assert_rent_exempt(rent: &Rent, account_info: &AccountInfo) -> ProgramResult {
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        Err(MetaplexError::NotRentExempt.into())
    } else {
        Ok(())
    }
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        Err(MetaplexError::IncorrectOwner.into())
    } else {
        Ok(())
    }
}

pub fn assert_signer(account_info: &AccountInfo) -> ProgramResult {
    if !account_info.is_signer {
        Err(ProgramError::MissingRequiredSignature)
    } else {
        Ok(())
    }
}

pub fn assert_store_safety_vault_manager_match(
    auction_manager: &AuctionManager,
    safety_deposit: &SafetyDepositBox,
    vault_info: &AccountInfo,
    store_info: &AccountInfo,
) -> ProgramResult {
    if auction_manager.vault != *vault_info.key {
        return Err(MetaplexError::AuctionManagerVaultMismatch.into());
    }

    if safety_deposit.vault != *vault_info.key {
        return Err(MetaplexError::SafetyDepositBoxVaultMismatch.into());
    }

    if safety_deposit.store != *store_info.key {
        return Err(MetaplexError::SafetyDepositBoxStoreMismatch.into());
    }
    Ok(())
}

pub fn assert_at_least_one_creator_matches_or_store_public(
    program_id: &Pubkey,
    auction_manager: &AuctionManager,
    metadata: &Metadata,
    whitelisted_creator_info: &AccountInfo,
    store_info: &AccountInfo,
) -> ProgramResult {
    if let Some(creators) = &metadata.data.creators {
        let store: Store = try_from_slice_unchecked(&store_info.data.borrow_mut())?;
        if store.public {
            return Ok(());
        }

        // does it exist? It better!
        let existing_whitelist_creator: WhitelistedCreator =
            try_from_slice_unchecked(&whitelisted_creator_info.data.borrow_mut())?;

        if !existing_whitelist_creator.activated {
            return Err(MetaplexError::WhitelistedCreatorInactive.into());
        }

        for creator in creators {
            // Now find at least one creator that can make this pda in the list
            let (key, _) = Pubkey::find_program_address(
                &[
                    PREFIX.as_bytes(),
                    program_id.as_ref(),
                    auction_manager.store.as_ref(),
                    creator.address.as_ref(),
                ],
                program_id,
            );
            if key == *whitelisted_creator_info.key {
                return Ok(());
            }
        }
        return Err(MetaplexError::InvalidWhitelistedCreator.into());
    }

    Ok(())
}

pub fn assert_authority_correct(
    auction_manager: &AuctionManager,
    authority_info: &AccountInfo,
) -> ProgramResult {
    if auction_manager.authority != *authority_info.key {
        return Err(MetaplexError::AuctionManagerAuthorityMismatch.into());
    }

    assert_signer(authority_info)?;

    Ok(())
}
/// Create account almost from scratch, lifted from
/// https://github.com/solana-labs/solana-program-library/blob/7d4873c61721aca25464d42cc5ef651a7923ca79/associated-token-account/program/src/processor.rs#L51-L98
#[inline(always)]
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports);
        invoke(
            &system_instruction::transfer(&payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    msg!("Allocate space for the account");
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        &[new_account_info.clone(), system_program_info.clone()],
        &[&signer_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        &[new_account_info.clone(), system_program_info.clone()],
        &[&signer_seeds],
    )?;
    msg!("Completed assignation!");

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn transfer_safety_deposit_box_items<'a>(
    token_vault_program: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    safety_deposit_box: AccountInfo<'a>,
    store: AccountInfo<'a>,
    vault: AccountInfo<'a>,
    fraction_mint: AccountInfo<'a>,
    vault_authority: AccountInfo<'a>,
    transfer_authority: AccountInfo<'a>,
    rent: AccountInfo<'a>,
    amount: u64,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &create_withdraw_tokens_instruction(
            *token_vault_program.key,
            *destination.key,
            *safety_deposit_box.key,
            *store.key,
            *vault.key,
            *fraction_mint.key,
            *vault_authority.key,
            *transfer_authority.key,
            amount,
        ),
        &[
            token_vault_program,
            destination,
            safety_deposit_box,
            store,
            vault,
            fraction_mint,
            vault_authority,
            transfer_authority,
            rent,
        ],
        &[&signer_seeds],
    )?;

    Ok(())
}

pub fn issue_start_auction<'a>(
    auction_program: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    auction: AccountInfo<'a>,
    clock: AccountInfo<'a>,
    vault: Pubkey,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &start_auction_instruction(
            *auction_program.key,
            *authority.key,
            StartAuctionArgs { resource: vault },
        ),
        &[auction_program, authority, auction, clock],
        &[&signer_seeds],
    )?;

    Ok(())
}

pub fn transfer_metadata_ownership<'a>(
    token_metadata_program: AccountInfo<'a>,
    metadata_info: AccountInfo<'a>,
    update_authority: AccountInfo<'a>,
    new_update_authority: AccountInfo<'a>,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &update_metadata_accounts(
            *token_metadata_program.key,
            *metadata_info.key,
            *update_authority.key,
            Some(*new_update_authority.key),
            None,
        ),
        &[
            update_authority,
            new_update_authority,
            metadata_info,
            token_metadata_program,
        ],
        &[&signer_seeds],
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn check_and_transfer_edition_master_mint<'a>(
    master_edition_mint_info: &AccountInfo<'a>,
    master_edition_master_mint_info: &AccountInfo<'a>,
    master_edition_info: &AccountInfo<'a>,
    auction_manager_info: &AccountInfo<'a>,
    token_metadata_program_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    master_edition_master_mint_authority_info: &AccountInfo<'a>,
    authority_seeds: &[&[u8]],
) -> ProgramResult {
    // Make sure it's a real mint
    let _mint: Mint = assert_initialized(master_edition_mint_info)?;
    let master_mint: Mint = assert_initialized(master_edition_master_mint_info)?;

    let edition_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        token_metadata_program_info.key.as_ref(),
        &master_edition_mint_info.key.as_ref(),
        spl_token_metadata::state::EDITION.as_bytes(),
    ];

    let (edition_key, _) =
        Pubkey::find_program_address(edition_seeds, &token_metadata_program_info.key);
    if edition_key != *master_edition_info.key {
        return Err(MetaplexError::InvalidEditionAddress.into());
    }

    let master_edition: MasterEdition =
        try_from_slice_unchecked(&master_edition_info.data.borrow_mut())?;
    if master_edition.max_supply.is_some() {
        return Err(MetaplexError::CantUseLimitedSupplyEditionsWithOpenEditionAuction.into());
    }

    if master_edition.master_mint != *master_edition_master_mint_info.key {
        return Err(MetaplexError::MasterEditionMasterMintMismatch.into());
    }

    assert_signer(master_edition_master_mint_authority_info)?;

    if let COption::Some(authority) = master_mint.mint_authority {
        if authority != *master_edition_master_mint_authority_info.key {
            return Err(MetaplexError::MasterMintAuthorityMismatch.into());
        }
    }

    if let COption::Some(authority) = master_mint.freeze_authority {
        if authority != *master_edition_master_mint_authority_info.key {
            return Err(MetaplexError::MasterMintAuthorityMismatch.into());
        }
    }

    transfer_mint_authority(
        authority_seeds,
        auction_manager_info.key,
        &auction_manager_info,
        &master_edition_master_mint_info,
        &master_edition_master_mint_authority_info,
        token_program_info,
    )?;

    Ok(())
}

pub fn transfer_mint_authority<'a>(
    new_authority_seeds: &[&[u8]],
    new_authority_key: &Pubkey,
    new_authority_info: &AccountInfo<'a>,
    mint_info: &AccountInfo<'a>,
    mint_authority_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
) -> ProgramResult {
    msg!("Setting mint authority");
    invoke_signed(
        &set_authority(
            token_program_info.key,
            mint_info.key,
            Some(new_authority_key),
            AuthorityType::MintTokens,
            mint_authority_info.key,
            &[&mint_authority_info.key],
        )
        .unwrap(),
        &[
            mint_authority_info.clone(),
            mint_info.clone(),
            token_program_info.clone(),
            new_authority_info.clone(),
        ],
        &[new_authority_seeds],
    )?;
    msg!("Setting freeze authority");
    invoke_signed(
        &set_authority(
            token_program_info.key,
            mint_info.key,
            Some(&new_authority_key),
            AuthorityType::FreezeAccount,
            mint_authority_info.key,
            &[&mint_authority_info.key],
        )
        .unwrap(),
        &[
            mint_authority_info.clone(),
            mint_info.clone(),
            token_program_info.clone(),
            new_authority_info.clone(),
        ],
        &[new_authority_seeds],
    )?;

    Ok(())
}

pub struct CommonRedeemReturn {
    pub redemption_bump_seed: u8,
    pub auction_manager: AuctionManager,
    pub safety_deposit: SafetyDepositBox,
    pub auction: AuctionData,
    pub bidder_metadata: BidderMetadata,
    pub rent: Rent,
    pub destination: Account,
    pub bidder_pot_pubkey: Pubkey,
}

pub struct CommonRedeemCheckArgs<'a> {
    pub program_id: &'a Pubkey,
    pub auction_manager_info: &'a AccountInfo<'a>,
    pub store_info: &'a AccountInfo<'a>,
    pub destination_info: &'a AccountInfo<'a>,
    pub bid_redemption_info: &'a AccountInfo<'a>,
    pub safety_deposit_info: &'a AccountInfo<'a>,
    pub vault_info: &'a AccountInfo<'a>,
    pub auction_info: &'a AccountInfo<'a>,
    pub bidder_metadata_info: &'a AccountInfo<'a>,
    pub bidder_info: &'a AccountInfo<'a>,
    pub token_program_info: &'a AccountInfo<'a>,
    pub token_vault_program_info: &'a AccountInfo<'a>,
    pub token_metadata_program_info: &'a AccountInfo<'a>,
    pub rent_info: &'a AccountInfo<'a>,
    pub is_open_edition: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn common_redeem_checks(
    args: CommonRedeemCheckArgs,
) -> Result<CommonRedeemReturn, ProgramError> {
    let CommonRedeemCheckArgs {
        program_id,
        auction_manager_info,
        store_info,
        destination_info,
        bid_redemption_info,
        safety_deposit_info,
        vault_info,
        auction_info,
        bidder_metadata_info,
        bidder_info,
        token_program_info,
        token_vault_program_info,
        token_metadata_program_info,
        rent_info,
        is_open_edition,
    } = args;

    let rent = &Rent::from_account_info(&rent_info)?;

    if !bid_redemption_info.data_is_empty() {
        let bid_redemption: BidRedemptionTicket =
            try_from_slice_unchecked(&bid_redemption_info.data.borrow_mut())?;
        if (is_open_edition && bid_redemption.open_edition_redeemed)
            || (!is_open_edition && bid_redemption.bid_redeemed)
        {
            return Err(MetaplexError::BidAlreadyRedeemed.into());
        }
    }

    let mut auction_manager: AuctionManager =
        try_from_slice_unchecked(&auction_manager_info.data.borrow_mut())?;
    let safety_deposit: SafetyDepositBox =
        try_from_slice_unchecked(&safety_deposit_info.data.borrow_mut())?;
    let auction: AuctionData = try_from_slice_unchecked(&auction_info.data.borrow_mut())?;
    let bidder_metadata: BidderMetadata =
        try_from_slice_unchecked(&bidder_metadata_info.data.borrow_mut())?;
    // Is it initialized and an actual Account?
    let destination: Account = assert_initialized(&destination_info)?;

    assert_signer(bidder_info)?;
    assert_owned_by(&destination_info, token_program_info.key)?;
    assert_owned_by(&auction_manager_info, &program_id)?;
    assert_store_safety_vault_manager_match(
        &auction_manager,
        &safety_deposit,
        &vault_info,
        &store_info,
    )?;
    // looking out for you!
    assert_rent_exempt(rent, &destination_info)?;

    if auction_manager.auction != *auction_info.key {
        return Err(MetaplexError::AuctionManagerAuctionMismatch.into());
    }

    if auction_manager.token_program != *token_program_info.key {
        return Err(MetaplexError::AuctionManagerTokenProgramMismatch.into());
    }

    if auction_manager.token_vault_program != *token_vault_program_info.key {
        return Err(MetaplexError::AuctionManagerTokenVaultProgramMismatch.into());
    }

    if auction_manager.token_metadata_program != *token_metadata_program_info.key {
        return Err(MetaplexError::AuctionManagerTokenMetadataProgramMismatch.into());
    }

    if auction.state != AuctionState::Ended {
        return Err(MetaplexError::AuctionHasNotEnded.into());
    }

    // No-op if already set.
    auction_manager.state.status = AuctionManagerStatus::Disbursing;

    let redemption_path = [
        PREFIX.as_bytes(),
        auction_manager.auction.as_ref(),
        bidder_metadata_info.key.as_ref(),
    ];
    let (redemption_key, redemption_bump_seed) =
        Pubkey::find_program_address(&redemption_path, &program_id);

    if redemption_key != *bid_redemption_info.key {
        return Err(MetaplexError::BidRedemptionMismatch.into());
    }

    let meta_path = [
        spl_auction::PREFIX.as_bytes(),
        auction_manager.auction_program.as_ref(),
        auction_info.key.as_ref(),
        bidder_metadata.bidder_pubkey.as_ref(),
        "metadata".as_bytes(),
    ];

    let (meta_key, _) = Pubkey::find_program_address(&meta_path, &auction_manager.auction_program);

    if meta_key != *bidder_metadata_info.key {
        return Err(MetaplexError::InvalidBidderMetadata.into());
    }

    if bidder_metadata.bidder_pubkey != *bidder_info.key {
        return Err(MetaplexError::BidderMetadataBidderMismatch.into());
    }

    let bidder_pot_seeds = &[
        spl_auction::PREFIX.as_bytes(),
        &auction_manager.auction_program.as_ref(),
        &auction_manager.auction.as_ref(),
        bidder_metadata.bidder_pubkey.as_ref(),
    ];
    let (bidder_pot_pubkey, _) =
        Pubkey::find_program_address(bidder_pot_seeds, &auction_manager.auction_program);

    Ok(CommonRedeemReturn {
        redemption_bump_seed,
        auction_manager,
        auction,
        bidder_metadata,
        safety_deposit,
        rent: *rent,
        destination,
        bidder_pot_pubkey,
    })
}

pub struct CommonRedeemFinishArgs<'a> {
    pub program_id: &'a Pubkey,
    pub auction_manager: AuctionManager,
    pub auction_manager_info: &'a AccountInfo<'a>,
    pub bidder_metadata_info: &'a AccountInfo<'a>,
    pub rent_info: &'a AccountInfo<'a>,
    pub system_info: &'a AccountInfo<'a>,
    pub payer_info: &'a AccountInfo<'a>,
    pub bid_redemption_info: &'a AccountInfo<'a>,
    pub redemption_bump_seed: u8,
    pub bid_redeemed: bool,
    pub open_edition_redeemed: bool,
}
#[allow(clippy::too_many_arguments)]
pub fn common_redeem_finish(args: CommonRedeemFinishArgs) -> ProgramResult {
    let CommonRedeemFinishArgs {
        program_id,
        mut auction_manager,
        auction_manager_info,
        bidder_metadata_info,
        rent_info,
        system_info,
        payer_info,
        bid_redemption_info,
        redemption_bump_seed,
        bid_redeemed,
        open_edition_redeemed,
    } = args;

    if bid_redeemed || open_edition_redeemed {
        let redemption_seeds = &[
            PREFIX.as_bytes(),
            auction_manager.auction.as_ref(),
            bidder_metadata_info.key.as_ref(),
            &[redemption_bump_seed],
        ];

        if bid_redemption_info.data_is_empty() {
            create_or_allocate_account_raw(
                *program_id,
                &bid_redemption_info,
                &rent_info,
                &system_info,
                &payer_info,
                3,
                redemption_seeds,
            )?;
        }

        let mut bid_redemption: BidRedemptionTicket =
            try_from_slice_unchecked(&bid_redemption_info.data.borrow_mut())?;
        bid_redemption.key = Key::BidRedemptionTicketV1;

        if open_edition_redeemed {
            bid_redemption.open_edition_redeemed = true
        } else if bid_redeemed {
            bid_redemption.bid_redeemed = true
        }
        bid_redemption.serialize(&mut *bid_redemption_info.data.borrow_mut())?;
    }

    let mut open_claims = false;
    for n in 0..auction_manager.state.winning_config_states.len() {
        if !auction_manager.state.winning_config_states[n].claimed {
            open_claims = true;
            break;
        }
    }

    if !open_claims
        && auction_manager
            .state
            .master_editions_with_authorities_remaining_to_return
            == 0
    {
        auction_manager.state.status = AuctionManagerStatus::Finished
    }

    auction_manager.serialize(&mut *auction_manager_info.data.borrow_mut())?;

    Ok(())
}

pub struct CommonWinningConfigCheckReturn {
    pub winning_config: WinningConfig,
    pub winning_config_state: WinningConfigState,
    pub transfer_authority: Pubkey,
}

pub fn common_winning_config_checks(
    auction_manager: &AuctionManager,
    safety_deposit: &SafetyDepositBox,
    winning_index: usize,
) -> Result<CommonWinningConfigCheckReturn, ProgramError> {
    let winning_config = auction_manager.settings.winning_configs[winning_index];
    let winning_config_state = auction_manager.state.winning_config_states[winning_index];
    if winning_config_state.claimed {
        return Err(MetaplexError::PrizeAlreadyClaimed.into());
    }

    if winning_config.safety_deposit_box_index != safety_deposit.order {
        return Err(MetaplexError::SafetyDepositIndexMismatch.into());
    }

    let transfer_authority_seeds = [
        spl_token_vault::state::PREFIX.as_bytes(),
        &auction_manager.token_vault_program.as_ref(),
    ];
    let (transfer_authority, _) = Pubkey::find_program_address(
        &transfer_authority_seeds,
        &&auction_manager.token_vault_program,
    );

    Ok(CommonWinningConfigCheckReturn {
        winning_config,
        winning_config_state,
        transfer_authority,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn shift_authority_back_to_originating_user<'a>(
    program_id: &Pubkey,
    auction_manager: &AuctionManager,
    auction_manager_info: &AccountInfo<'a>,
    master_metadata_info: &AccountInfo<'a>,
    original_authority: &AccountInfo<'a>,
    original_authority_lookup_info: &AccountInfo<'a>,
    master_mint_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    authority_seeds: &[&[u8]],
) -> ProgramResult {
    let original_authority_lookup_seeds = &[
        PREFIX.as_bytes(),
        &auction_manager.auction.as_ref(),
        master_metadata_info.key.as_ref(),
    ];

    let (expected_key, _) =
        Pubkey::find_program_address(original_authority_lookup_seeds, &program_id);

    if expected_key != *original_authority_lookup_info.key {
        return Err(MetaplexError::OriginalAuthorityLookupKeyMismatch.into());
    }

    let original_authority_lookup: OriginalAuthorityLookup =
        try_from_slice_unchecked(&original_authority_lookup_info.data.borrow_mut())?;
    if original_authority_lookup.original_authority != *original_authority.key {
        return Err(MetaplexError::OriginalAuthorityMismatch.into());
    }
    transfer_mint_authority(
        authority_seeds,
        original_authority.key,
        original_authority,
        master_mint_info,
        auction_manager_info,
        token_program_info,
    )?;

    Ok(())
}

// TODO due to a weird stack access violation bug we had to remove the args struct from this method
// to get redemptions working again after integrating new Auctions program. Try to bring it back one day
#[inline(always)]
pub fn spl_token_transfer<'a: 'b, 'b>(
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    amount: u64,
    authority: AccountInfo<'a>,
    authority_signer_seeds: &'b [&'b [u8]],
    token_program: AccountInfo<'a>,
) -> ProgramResult {
    let result = invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination, authority, token_program],
        &[authority_signer_seeds],
    );

    result.map_err(|_| MetaplexError::TokenTransferFailed.into())
}

pub fn common_metadata_checks(
    master_metadata_info: &AccountInfo,
    master_edition_info: &AccountInfo,
    token_metadata_program_info: &AccountInfo,
    master_mint_info: &AccountInfo,
    safety_deposit: &SafetyDepositBox,
    destination: &Account,
) -> ProgramResult {
    let master_metadata: Metadata =
        try_from_slice_unchecked(&master_metadata_info.data.borrow_mut())?;
    let master_edition: MasterEdition =
        try_from_slice_unchecked(&master_edition_info.data.borrow_mut())?;

    if safety_deposit.token_mint != master_metadata.mint {
        return Err(MetaplexError::SafetyDepositBoxMetadataMismatch.into());
    }

    assert_edition_valid(
        &token_metadata_program_info.key,
        &master_metadata.mint,
        master_edition_info,
    )?;

    if master_edition.master_mint != *master_mint_info.key {
        return Err(MetaplexError::MasterEditionMintMismatch.into());
    }

    if master_edition.master_mint != *master_mint_info.key {
        return Err(MetaplexError::MasterEditionMintMismatch.into());
    }

    if destination.mint != master_edition.master_mint {
        return Err(MetaplexError::DestinationMintMismatch.into());
    }

    Ok(())
}

pub fn assert_edition_valid(
    program_id: &Pubkey,
    mint: &Pubkey,
    edition_account_info: &AccountInfo,
) -> ProgramResult {
    let edition_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        program_id.as_ref(),
        &mint.as_ref(),
        EDITION.as_bytes(),
    ];
    let (edition_key, _) = Pubkey::find_program_address(edition_seeds, program_id);
    if edition_key != *edition_account_info.key {
        return Err(MetaplexError::InvalidEditionKey.into());
    }

    Ok(())
}

// TODO due to a weird stack access violation bug we had to remove the args struct from this method
// to get redemptions working again after integrating new Auctions program. Try to bring it back one day.
pub fn spl_token_mint_to<'a: 'b, 'b>(
    mint: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    amount: u64,
    authority: AccountInfo<'a>,
    authority_signer_seeds: &'b [&'b [u8]],
    token_program: AccountInfo<'a>,
) -> ProgramResult {
    let result = invoke_signed(
        &spl_token::instruction::mint_to(
            token_program.key,
            mint.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[mint, destination, authority, token_program],
        &[authority_signer_seeds],
    );
    result.map_err(|_| MetaplexError::TokenMintToFailed.into())
}

pub fn assert_derivation(
    program_id: &Pubkey,
    account: &AccountInfo,
    path: &[&[u8]],
) -> Result<u8, ProgramError> {
    let (key, bump) = Pubkey::find_program_address(&path, program_id);
    if key != *account.key {
        return Err(MetaplexError::DerivedKeyInvalid.into());
    }
    Ok(bump)
}
