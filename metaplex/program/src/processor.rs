use {
    crate::{
        error::MetaplexError,
        instruction::MetaplexInstruction,
        state::{
            AuctionManager, AuctionManagerSettings, AuctionManagerState, AuctionManagerStatus,
            EditionType, Key, WinningConfig, MAX_AUCTION_MANAGER_SIZE, PREFIX,
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
    spl_token_metadata::state::{Edition, MasterEdition, Metadata},
    spl_token_vault::state::{ExternalPriceAccount, SafetyDepositBox, Vault, VaultState},
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
        MetaplexInstruction::ValidateSafetyDepositBox => {
            msg!("Instruction: Validate Safety Deposit Box");
            process_validate_safety_deposit_box(program_id, accounts)
        }
    }
}
pub fn process_validate_safety_deposit_box(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let safety_deposit_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let edition_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;

    let mut auction_manager: AuctionManager =
        try_from_slice_unchecked(&auction_manager_info.data.borrow_mut())?;
    let safety_deposit: SafetyDepositBox =
        try_from_slice_unchecked(&safety_deposit_info.data.borrow_mut())?;
    let store: Account = assert_initialized(store_info)?;
    let metadata: Metadata = try_from_slice_unchecked(&metadata_info.data.borrow_mut())?;
    // Is it a real vault?
    let _vault: Vault = try_from_slice_unchecked(&vault_info.data.borrow_mut())?;
    // Is it a real mint?
    let _mint: Mint = assert_initialized(mint_info)?;

    if auction_manager.authority != *authority_info.key {
        return Err(MetaplexError::AuctionManagerAuthorityMismatch.into());
    }

    if *mint_info.key != safety_deposit.token_mint {
        return Err(MetaplexError::SafetyDepositBoxMintMismatch.into());
    }

    // We want to ensure that the mint you are using with this token is one
    // we can actually transfer to and from using our token program invocations, which
    // we can check by asserting ownership by the token program we recorded in init.
    if *mint_info.owner != auction_manager.token_program {
        return Err(MetaplexError::TokenProgramMismatch.into());
    }

    if !authority_info.is_signer {
        return Err(MetaplexError::AuctionManagerAuthorityIsNotSigner.into());
    }

    if auction_manager.vault != *vault_info.key {
        return Err(MetaplexError::AuctionManagerVaultMismatch.into());
    }

    if safety_deposit.vault != *vault_info.key {
        return Err(MetaplexError::SafetyDepositBoxVaultMismatch.into());
    }

    if safety_deposit.store != *store_info.key {
        return Err(MetaplexError::SafetyDepositBoxStoreMismatch.into());
    }

    if safety_deposit.token_mint != metadata.mint {
        return Err(MetaplexError::SafetyDepositBoxMetadataMismatch.into());
    }

    let mut winning_config_option: Option<WinningConfig> = None;
    for n in 0..auction_manager.settings.winning_configs.len() {
        let possible_config = auction_manager.settings.winning_configs[n];
        if possible_config.safety_deposit_box_index == safety_deposit.order {
            winning_config_option = Some(possible_config);
            break;
        }
    }

    let mut winning_config = match winning_config_option {
        Some(config) => config,
        None => return Err(MetaplexError::SafetyDepositBoxNotUsedInAuction.into()),
    };

    let edition_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        auction_manager.token_metadata_program.as_ref(),
        &edition_info.key.as_ref(),
        spl_token_metadata::state::EDITION.as_bytes(),
    ];

    let (edition_key, _) =
        Pubkey::find_program_address(edition_seeds, &auction_manager.token_metadata_program);

    match winning_config.edition_type {
        EditionType::NA => {
            if store.amount < winning_config.amount.into() {
                return Err(MetaplexError::NotEnoughTokensToSupplyWinners.into());
            }
        }
        EditionType::MasterEdition | EditionType::MasterEditionAsTemplate => {
            if edition_key != *edition_info.key {
                return Err(MetaplexError::InvalidEditionAddress.into());
            }

            let master_edition: MasterEdition =
                try_from_slice_unchecked(&edition_info.data.borrow_mut())?;

            if let Some(max_supply) = master_edition.max_supply {
                let remaining_supply = match max_supply.checked_sub(master_edition.supply) {
                    Some(val) => val,
                    None => return Err(MetaplexError::NumericalOverflowError.into()),
                };

                if winning_config.edition_type == EditionType::MasterEditionAsTemplate
                    && remaining_supply < winning_config.amount.into()
                {
                    return Err(MetaplexError::NotEnoughEditionsAvailableForAuction.into());
                }
            }

            if store.amount != 1 {
                return Err(MetaplexError::StoreIsEmpty.into());
            }

            if winning_config.edition_type == EditionType::MasterEdition
                && winning_config.amount != 1
            {
                return Err(MetaplexError::CannotAuctionOffMoreThanOneOfMasterEditionItself.into());
            }
        }
        EditionType::Edition => {
            if edition_key != *edition_info.key {
                return Err(MetaplexError::InvalidEditionAddress.into());
            }

            // Check that it serializes
            let _edition: Edition = try_from_slice_unchecked(&edition_info.data.borrow_mut())?;

            if store.amount != 1 {
                return Err(MetaplexError::StoreIsEmpty.into());
            }

            if winning_config.amount != 1 {
                return Err(MetaplexError::CannotAuctionOffMoreThanOneOfLimitedEdition.into());
            }
        }
    }

    winning_config.validated = true;
    auction_manager.state.safety_deposit_boxes_validated = match auction_manager
        .state
        .safety_deposit_boxes_validated
        .checked_add(1)
    {
        Some(val) => val,
        None => return Err(MetaplexError::NumericalOverflowError.into()),
    };

    if auction_manager.state.safety_deposit_boxes_validated
        == auction_manager.settings.winning_configs.len() as u8
    {
        auction_manager.state.status = AuctionManagerStatus::Validated
    }

    auction_manager.serialize(&mut *auction_manager_info.data.borrow_mut())?;

    Ok(())
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
    let open_master_edition_mint_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;
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
        // Make sure it's a real mint
        let _mint: Mint = assert_initialized(open_master_edition_mint_info)?;

        let edition_seeds = &[
            spl_token_metadata::state::PREFIX.as_bytes(),
            token_metadata_program_info.key.as_ref(),
            &open_master_edition_mint_info.key.as_ref(),
            spl_token_metadata::state::EDITION.as_bytes(),
        ];

        let (edition_key, bump_seed) =
            Pubkey::find_program_address(edition_seeds, &token_metadata_program_info.key);
        if edition_key != *open_master_edition_info.key {
            return Err(MetaplexError::InvalidEditionAddress.into());
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
    auction_manager.authority = *authority_info.key;
    auction_manager.token_program = *token_program_info.key;
    auction_manager.token_metadata_program = *token_metadata_program_info.key;
    auction_manager.state.status = AuctionManagerStatus::Initialized;
    auction_manager.state.safety_deposit_boxes_validated = 0;
    auction_manager.serialize(&mut *auction_manager_info.data.borrow_mut())?;

    Ok(())
}
