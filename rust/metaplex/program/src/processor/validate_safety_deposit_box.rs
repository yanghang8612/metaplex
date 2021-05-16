use {
    crate::{
        error::MetaplexError,
        state::{
            AuctionManager, AuctionManagerStatus, EditionType, Key, OriginalAuthorityLookup, Store,
            WinningConfig, WinningConfigState, PREFIX,
        },
        utils::{
            assert_at_least_one_creator_matches_or_store_public, assert_authority_correct,
            assert_initialized, assert_owned_by, assert_store_safety_vault_manager_match,
            create_or_allocate_account_raw, transfer_metadata_ownership,
        },
    },
    borsh::BorshSerialize,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
    },
    spl_token::state::{Account, Mint},
    spl_token_metadata::{
        state::{MasterEdition, Metadata},
        utils::assert_update_authority_is_correct,
    },
    spl_token_vault::state::{SafetyDepositBox, Vault},
};

pub fn process_validate_safety_deposit_box(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let original_authority_lookup_info = next_account_info(account_info_iter)?;
    let whitelisted_creator_info = next_account_info(account_info_iter)?;
    let auction_manager_store_info = next_account_info(account_info_iter)?;
    let safety_deposit_info = next_account_info(account_info_iter)?;
    let safety_deposit_token_store_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let edition_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let metadata_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let mut auction_manager: AuctionManager =
        try_from_slice_unchecked(&auction_manager_info.data.borrow_mut())?;
    let safety_deposit: SafetyDepositBox =
        try_from_slice_unchecked(&safety_deposit_info.data.borrow_mut())?;
    let safety_deposit_token_store: Account = assert_initialized(safety_deposit_token_store_info)?;
    let metadata: Metadata = try_from_slice_unchecked(&metadata_info.data.borrow_mut())?;
    let store: Store = try_from_slice_unchecked(&auction_manager_store_info.data.borrow_mut())?;
    // Is it a real vault?
    let _vault: Vault = try_from_slice_unchecked(&vault_info.data.borrow_mut())?;
    // Is it a real mint?
    let _mint: Mint = assert_initialized(mint_info)?;

    assert_owned_by(auction_manager_info, program_id)?;
    assert_update_authority_is_correct(&metadata, metadata_authority_info)?;
    assert_authority_correct(&auction_manager, authority_info)?;
    assert_store_safety_vault_manager_match(
        &auction_manager,
        &safety_deposit,
        vault_info,
        safety_deposit_token_store_info,
    )?;
    assert_at_least_one_creator_matches_or_store_public(
        program_id,
        &auction_manager,
        &metadata,
        whitelisted_creator_info,
        auction_manager_store_info,
    )?;

    if auction_manager.store != *auction_manager_store_info.key {
        return Err(MetaplexError::AuctionManagerStoreMismatch.into());
    }

    if *mint_info.key != safety_deposit.token_mint {
        return Err(MetaplexError::SafetyDepositBoxMintMismatch.into());
    }

    if *token_metadata_program_info.key != store.token_metadata_program {
        return Err(MetaplexError::AuctionManagerTokenMetadataProgramMismatch.into());
    }

    // We want to ensure that the mint you are using with this token is one
    // we can actually transfer to and from using our token program invocations, which
    // we can check by asserting ownership by the token program we recorded in init.
    if *mint_info.owner != store.token_program {
        return Err(MetaplexError::TokenProgramMismatch.into());
    }

    let mut winning_configs: Vec<WinningConfig> = vec![];
    let mut winning_config_states: Vec<WinningConfigState> = vec![];

    for n in 0..auction_manager.settings.winning_configs.len() {
        let possible_config = auction_manager.settings.winning_configs[n];
        if possible_config.safety_deposit_box_index == safety_deposit.order {
            winning_configs.push(possible_config);
            winning_config_states.push(auction_manager.state.winning_config_states[n]);
        }
    }

    if winning_configs.is_empty() {
        return Err(MetaplexError::SafetyDepositBoxNotUsedInAuction.into());
    }

    // At this point we know we have at least one config and they may have different amounts but all
    // point at the same safety deposit box and so have the same edition type.
    let edition_type = winning_configs[0].edition_type;
    let mut total_amount_requested: u64 = 0;
    for curr in &winning_configs {
        total_amount_requested = match total_amount_requested.checked_add(curr.amount.into()) {
            Some(val) => val,
            None => return Err(MetaplexError::NumericalOverflowError.into()),
        };
    }

    let edition_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        store.token_metadata_program.as_ref(),
        &metadata.mint.as_ref(),
        spl_token_metadata::state::EDITION.as_bytes(),
    ];

    let (edition_key, _) =
        Pubkey::find_program_address(edition_seeds, &store.token_metadata_program);

    let seeds = &[PREFIX.as_bytes(), &auction_manager.auction.as_ref()];
    let (_, bump_seed) = Pubkey::find_program_address(seeds, &program_id);
    let authority_seeds = &[
        PREFIX.as_bytes(),
        &auction_manager.auction.as_ref(),
        &[bump_seed],
    ];
    // Supply logic check
    match edition_type {
        EditionType::OpenEdition => {
            if safety_deposit.token_mint != metadata.mint {
                return Err(MetaplexError::SafetyDepositBoxMetadataMismatch.into());
            }
            if edition_key != *edition_info.key {
                return Err(MetaplexError::InvalidEditionAddress.into());
            }

            if safety_deposit_token_store.amount != 1 {
                return Err(MetaplexError::StoreIsEmpty.into());
            }

            transfer_metadata_ownership(
                token_metadata_program_info.clone(),
                metadata_info.clone(),
                metadata_authority_info.clone(),
                auction_manager_info.clone(),
                authority_seeds,
            )?;
        }
        EditionType::MasterEdition => {
            if safety_deposit.token_mint != metadata.mint {
                return Err(MetaplexError::SafetyDepositBoxMetadataMismatch.into());
            }
            if edition_key != *edition_info.key {
                return Err(MetaplexError::InvalidEditionAddress.into());
            }

            if safety_deposit_token_store.amount != 1 {
                return Err(MetaplexError::StoreIsEmpty.into());
            }

            let original_authority_lookup_seeds = &[
                PREFIX.as_bytes(),
                &auction_manager.auction.as_ref(),
                metadata_info.key.as_ref(),
            ];

            let (expected_key, original_bump_seed) =
                Pubkey::find_program_address(original_authority_lookup_seeds, &program_id);
            let original_authority_seeds = &[
                PREFIX.as_bytes(),
                &auction_manager.auction.as_ref(),
                metadata_info.key.as_ref(),
                &[original_bump_seed],
            ];

            if expected_key != *original_authority_lookup_info.key {
                return Err(MetaplexError::OriginalAuthorityLookupKeyMismatch.into());
            }

            // We may need to transfer authority back, or to the new owner, so we need to keep track
            // of original ownership
            create_or_allocate_account_raw(
                *program_id,
                original_authority_lookup_info,
                rent_info,
                system_info,
                payer_info,
                33,
                original_authority_seeds,
            )?;

            let mut original_authority_lookup: OriginalAuthorityLookup =
                try_from_slice_unchecked(&original_authority_lookup_info.data.borrow_mut())?;
            original_authority_lookup.key = Key::OriginalAuthorityLookupV1;

            original_authority_lookup.original_authority = *metadata_authority_info.key;

            transfer_metadata_ownership(
                token_metadata_program_info.clone(),
                metadata_info.clone(),
                metadata_authority_info.clone(),
                auction_manager_info.clone(),
                authority_seeds,
            )?;

            original_authority_lookup
                .serialize(&mut *original_authority_lookup_info.data.borrow_mut())?;
            auction_manager
                .state
                .master_editions_with_authorities_remaining_to_return = match auction_manager
                .state
                .master_editions_with_authorities_remaining_to_return
                .checked_add(1)
            {
                Some(val) => val,
                None => return Err(MetaplexError::NumericalOverflowError.into()),
            };
        }
        EditionType::Na => {
            if safety_deposit.token_mint != metadata.mint {
                return Err(MetaplexError::SafetyDepositBoxMetadataMismatch.into());
            }
            if safety_deposit_token_store.amount < total_amount_requested {
                return Err(MetaplexError::NotEnoughTokensToSupplyWinners.into());
            }
        }
        EditionType::LimitedEdition => {
            if edition_key != *edition_info.key {
                return Err(MetaplexError::InvalidEditionAddress.into());
            }
            let master_edition: MasterEdition =
                try_from_slice_unchecked(&edition_info.data.borrow_mut())?;
            if safety_deposit.token_mint != master_edition.master_mint {
                return Err(MetaplexError::SafetyDepositBoxMasterMintMismatch.into());
            }

            if safety_deposit_token_store.amount != total_amount_requested {
                return Err(MetaplexError::NotEnoughTokensToSupplyWinners.into());
            }
        }
    }

    for state in &mut winning_config_states {
        state.validated = true;
    }

    auction_manager.state.winning_configs_validated = match auction_manager
        .state
        .winning_configs_validated
        .checked_add(winning_configs.len() as u8)
    {
        Some(val) => val,
        None => return Err(MetaplexError::NumericalOverflowError.into()),
    };

    if auction_manager.state.winning_configs_validated
        == auction_manager.settings.winning_configs.len() as u8
    {
        auction_manager.state.status = AuctionManagerStatus::Validated
    }

    auction_manager.serialize(&mut *auction_manager_info.data.borrow_mut())?;

    Ok(())
}
