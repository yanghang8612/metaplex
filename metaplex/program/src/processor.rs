use {
    crate::{
        error::MetaplexError,
        instruction::MetaplexInstruction,
        state::{
            AuctionManager, AuctionManagerSettings, AuctionManagerState, AuctionManagerStatus, Key,
            MAX_AUCTION_MANAGER_SIZE, PREFIX,
        },
        utils::{
            assert_initialized, assert_owned_by, assert_rent_exempt, create_or_allocate_account_raw,
        },
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        msg,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::Sysvar,
    },
    spl_auction::processor::AuctionData,
    spl_token::state::{Account, Mint},
    spl_token_metadata::state::MasterEdition,
    spl_token_vault::state::{ExternalPriceAccount, Vault, VaultState},
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = MetaplexInstruction::try_from_slice(input)?;
    match instruction {
        MetaplexInstruction::InitAuctionManager(auction_manager_settings) => {
            msg!("Instruction: Init Auction Manager");
            process_init_auction_manager(program_id, accounts, auction_manager_settings)
        }
    }
}

pub fn process_init_auction_manager(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    auction_manager_settings: AuctionManagerSettings,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let auction_info = next_account_info(account_info_iter)?;
    let external_pricing_account_info = next_account_info(account_info_iter)?;
    let open_master_edition_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_info)?;

    // Just verifying this is a real account that serializes
    let _external_price_account: ExternalPriceAccount =
        try_from_slice_unchecked(&external_pricing_account_info.data.borrow_mut())?;
    let vault: Vault = try_from_slice_unchecked(&vault_info.data.borrow_mut())?;
    let auction: AuctionData = try_from_slice_unchecked(&auction_info.data.borrow_mut())?;

    assert_rent_exempt(rent, external_pricing_account_info)?;

    let seeds = &[PREFIX.as_bytes(), &auction_info.key.as_ref()];
    let (auction_authority, bump_seed) = Pubkey::find_program_address(seeds, &program_id);

    if vault.authority != auction_authority {
        return Err(MetaplexError::VaultAuthorityMismatch.into());
    }

    if *auction_manager_info.key != auction_authority {
        return Err(MetaplexError::AuctionManagerKeyMismatch.into());
    }

    if auction.authority != auction_authority {
        return Err(MetaplexError::AuctionAuthorityMismatch.into());
    }

    if external_pricing_account_info.owner != program_id {
        return Err(MetaplexError::ExternalPriceAccountOwnerMismatch.into());
    }

    if vault.pricing_lookup_address != *external_pricing_account_info.key {
        return Err(MetaplexError::VaultExternalPricingMismatch.into());
    }

    if auction.resource != *vault_info.key {
        return Err(MetaplexError::AuctionVaultMismatch.into());
    }

    if vault.state != VaultState::Active {
        return Err(MetaplexError::VaultNotActive.into());
    }

    if vault.token_type_count == 0 {
        return Err(MetaplexError::VaultCannotEmpty.into());
    }

    for n in 0..auction_manager_settings.winning_configs.len() {
        let winning_config = &auction_manager_settings.winning_configs[n];
        if winning_config.safety_deposit_box_index > vault.token_type_count.into() {
            return Err(MetaplexError::InvalidSafetyDepositBox.into());
        }
    }

    if let Some(open_edition_config) = auction_manager_settings.open_edition_config {
        if open_edition_config > vault.token_type_count {
            return Err(MetaplexError::InvalidSafetyDepositBox.into());
        }
        let open_master_edition: MasterEdition =
            try_from_slice_unchecked(&open_master_edition_info.data.borrow_mut())?;
        if let Some(_) = open_master_edition.max_supply {
            return Err(MetaplexError::CantUseLimitedSupplyEditionsWithOpenEditionAuction.into());
        }
    }

    let authority_seeds = &[PREFIX.as_bytes(), &auction_info.key.as_ref(), &[bump_seed]];

    create_or_allocate_account_raw(
        *program_id,
        auction_manager_info,
        rent_info,
        system_info,
        payer_info,
        MAX_AUCTION_MANAGER_SIZE,
        authority_seeds,
    )?;

    let mut auction_manager: AuctionManager =
        try_from_slice_unchecked(&auction_manager_info.data.borrow_mut())?;

    auction_manager.key = Key::AuctionManagerV1;
    auction_manager.settings = auction_manager_settings;
    auction_manager.vault = *vault_info.key;
    auction_manager.auction = *auction_info.key;
    auction_manager.state.status = AuctionManagerStatus::Initialized;
    auction_manager.state.safety_deposit_boxes_validated = 0;
    auction_manager.serialize(&mut *auction_manager_info.data.borrow_mut())?;

    Ok(())
}
