use {
    crate::{
        error::MetaplexError,
        state::{
            AuctionManager, AuctionManagerStatus, BidRedemptionTicket, OriginalAuthorityLookup,
            WinningConfig, WinningConfigState, PREFIX,
        },
    },
    borsh::BorshSerialize,
    solana_program::{
        account_info::AccountInfo,
        borsh::try_from_slice_unchecked,
        clock::Clock,
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_pack::{IsInitialized, Pack},
        pubkey::Pubkey,
        system_instruction,
        sysvar::{rent::Rent, Sysvar},
    },
    spl_auction::{
        instruction::start_auction,
        processor::{start_auction::StartAuctionArgs, AuctionData, BidderMetadata},
    },
    spl_token::state::Account,
    spl_token_metadata::{
        instruction::{mint_new_edition_from_master_edition, transfer_update_authority},
        state::Metadata,
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

pub fn assert_authority_correct(
    auction_manager: &AuctionManager,
    authority_info: &AccountInfo,
) -> ProgramResult {
    if auction_manager.authority != *authority_info.key {
        return Err(MetaplexError::AuctionManagerAuthorityMismatch.into());
    }

    if !authority_info.is_signer {
        return Err(MetaplexError::AuctionManagerAuthorityIsNotSigner.into());
    }

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

pub fn transfer_safety_deposit_box_items<'a>(
    token_vault_program: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    safety_deposit_box: AccountInfo<'a>,
    store: AccountInfo<'a>,
    vault: AccountInfo<'a>,
    fraction_mint: AccountInfo<'a>,
    vault_authority: AccountInfo<'a>,
    transfer_authority: AccountInfo<'a>,
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
        ],
        &[&signer_seeds],
    )?;

    Ok(())
}

pub fn mint_edition<'a>(
    token_metadata_program: AccountInfo<'a>,
    metadata: AccountInfo<'a>,
    edition: AccountInfo<'a>,
    master_edition: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    mint_authority: AccountInfo<'a>,
    payer: AccountInfo<'a>,
    master_update_authority: AccountInfo<'a>,
    master_metadata: AccountInfo<'a>,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &mint_new_edition_from_master_edition(
            *token_metadata_program.key,
            *metadata.key,
            *edition.key,
            *master_edition.key,
            *mint.key,
            *mint_authority.key,
            *payer.key,
            *master_update_authority.key,
            *master_metadata.key,
        ),
        &[
            metadata,
            edition,
            master_edition,
            mint,
            mint_authority,
            payer,
            master_update_authority,
            master_metadata,
            token_metadata_program,
        ],
        &[&signer_seeds],
    )?;

    Ok(())
}

pub fn issue_start_auction<'a>(
    auction_program: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    auction: AccountInfo<'a>,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &start_auction(
            *auction_program.key,
            *authority.key,
            StartAuctionArgs {
                resource: *auction.key,
            },
        ),
        &[auction_program, authority, auction],
        &[&signer_seeds],
    )?;

    Ok(())
}

pub fn transfer_metadata_ownership<'a>(
    metadata: &Metadata,
    token_metadata_program: AccountInfo<'a>,
    metadata_info: AccountInfo<'a>,
    name_symbol: AccountInfo<'a>,
    update_authority: AccountInfo<'a>,
    new_update_authority: AccountInfo<'a>,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    let transferring_obj = match metadata.non_unique_specific_update_authority {
        Some(_) => metadata_info,
        None => name_symbol,
    };
    invoke_signed(
        &transfer_update_authority(
            *token_metadata_program.key,
            *transferring_obj.key,
            *update_authority.key,
            *new_update_authority.key,
        ),
        &[
            update_authority,
            new_update_authority,
            transferring_obj,
            token_metadata_program,
        ],
        &[&signer_seeds],
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
}

pub fn common_redeem_checks(
    program_id: &Pubkey,
    auction_manager_info: &AccountInfo,
    store_info: &AccountInfo,
    destination_info: &AccountInfo,
    bid_redemption_info: &AccountInfo,
    safety_deposit_info: &AccountInfo,
    _fraction_mint_info: &AccountInfo,
    vault_info: &AccountInfo,
    auction_info: &AccountInfo,
    bidder_metadata_info: &AccountInfo,
    bidder_info: &AccountInfo,
    _payer_info: &AccountInfo,
    token_program_info: &AccountInfo,
    token_vault_program_info: &AccountInfo,
    token_metadata_program_info: &AccountInfo,
    rent_info: &AccountInfo,
    _system_info: &AccountInfo,
    clock_info: &AccountInfo,
    is_open_edition: bool,
) -> Result<CommonRedeemReturn, ProgramError> {
    let clock = &Clock::from_account_info(clock_info)?;
    let rent = &Rent::from_account_info(rent_info)?;

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
    let _destination: Account = assert_initialized(destination_info)?;

    assert_owned_by(destination_info, token_program_info.key)?;
    assert_owned_by(auction_manager_info, program_id)?;
    assert_store_safety_vault_manager_match(
        &auction_manager,
        &safety_deposit,
        vault_info,
        store_info,
    )?;
    // looking out for you!
    assert_rent_exempt(rent, destination_info)?;

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

    if let Some(end_time) = auction.end_time_slot {
        if end_time < clock.slot {
            return Err(MetaplexError::AuctionHasNotEnded.into());
        }
    } else {
        return Err(MetaplexError::AuctionHasNoEndTime.into());
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

    if !bidder_info.is_signer {
        return Err(MetaplexError::BidderIsNotSigner.into());
    }

    Ok(CommonRedeemReturn {
        redemption_bump_seed,
        auction_manager,
        auction,
        bidder_metadata,
        safety_deposit,
        rent: *rent,
    })
}

pub fn common_redeem_finish<'a>(
    program_id: &Pubkey,
    auction_manager: &mut AuctionManager,
    auction_manager_info: &AccountInfo<'a>,
    bidder_metadata_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    system_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    bid_redemption_info: &AccountInfo<'a>,
    redemption_bump_seed: u8,
    bid_redeemed: bool,
    open_edition_redeemed: bool,
) -> ProgramResult {
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
                bid_redemption_info,
                rent_info,
                system_info,
                payer_info,
                2,
                redemption_seeds,
            )?;
        }

        let mut bid_redemption: BidRedemptionTicket =
            try_from_slice_unchecked(&bid_redemption_info.data.borrow_mut())?;

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
    pub vault_bump_seed: u8,
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
    let (transfer_authority, vault_bump_seed) = Pubkey::find_program_address(
        &transfer_authority_seeds,
        &&auction_manager.token_vault_program,
    );

    Ok(CommonWinningConfigCheckReturn {
        winning_config_state,
        winning_config,
        transfer_authority,
        vault_bump_seed,
    })
}

pub fn shift_authority_back_to_originating_user<'a>(
    program_id: &Pubkey,
    auction_manager: &AuctionManager,
    auction_manager_info: &AccountInfo<'a>,
    master_metadata_info: &AccountInfo<'a>,
    master_name_symbol_info: &AccountInfo<'a>,
    original_authority: &AccountInfo<'a>,
    original_authority_lookup_info: &AccountInfo<'a>,
    token_metadata_program_info: &AccountInfo<'a>,
) -> ProgramResult {
    let original_authority_lookup_seeds = &[
        PREFIX.as_bytes(),
        &auction_manager.auction.as_ref(),
        master_metadata_info.key.as_ref(),
    ];

    let (expected_key, original_bump_seed) =
        Pubkey::find_program_address(original_authority_lookup_seeds, &program_id);

    let original_authority_seeds = &[
        PREFIX.as_bytes(),
        &auction_manager.auction.as_ref(),
        master_metadata_info.key.as_ref(),
        &[original_bump_seed],
    ];

    if expected_key != *original_authority_lookup_info.key {
        return Err(MetaplexError::OriginalAuthorityLookupKeyMismatch.into());
    }

    let original_authority_lookup: OriginalAuthorityLookup =
        try_from_slice_unchecked(&original_authority_lookup_info.data.borrow_mut())?;
    if original_authority_lookup.original_authority != *original_authority.key {
        return Err(MetaplexError::OriginalAuthorityMismatch.into());
    }

    let master_metadata: Metadata =
        try_from_slice_unchecked(&master_metadata_info.data.borrow_mut())?;

    transfer_metadata_ownership(
        &master_metadata,
        token_metadata_program_info.clone(),
        master_metadata_info.clone(),
        master_name_symbol_info.clone(),
        auction_manager_info.clone(),
        original_authority.clone(),
        original_authority_seeds,
    )?;

    Ok(())
}
