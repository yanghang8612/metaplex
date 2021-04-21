use {
    crate::{
        error::MetaplexError,
        instruction::MetaplexInstruction,
        state::{
            AuctionManager, AuctionManagerSettings, AuctionManagerStatus, EditionType, Key,
            NonWinningConstraint, OriginalAuthorityLookup, WinningConfig, WinningConfigState,
            WinningConstraint, MAX_AUCTION_MANAGER_SIZE, PREFIX,
        },
        utils::{
            assert_authority_correct, assert_initialized, assert_owned_by,
            assert_store_safety_vault_manager_match, check_and_transfer_edition_master_mint,
            common_metadata_checks, common_redeem_checks, common_redeem_finish,
            common_winning_config_checks, create_or_allocate_account_raw, issue_start_auction,
            shift_authority_back_to_originating_user, spl_token_mint_to, spl_token_transfer,
            transfer_metadata_ownership, transfer_safety_deposit_box_items, CommonRedeemReturn,
            CommonWinningConfigCheckReturn, TokenMintToParams, TokenTransferParams,
        },
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        msg,
        pubkey::Pubkey,
    },
    spl_auction::processor::AuctionData,
    spl_token::state::{Account, Mint},
    spl_token_metadata::{
        state::{MasterEdition, Metadata},
        utils::assert_update_authority_is_correct,
    },
    spl_token_vault::state::{SafetyDepositBox, Vault, VaultState},
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
        MetaplexInstruction::RedeemBid => {
            msg!("Instruction: Redeem Bid");
            process_redeem_bid(program_id, accounts)
        }
        MetaplexInstruction::RedeemMasterEditionBid => {
            msg!("Instruction: Redeem Master Edition Bid");
            process_redeem_master_edition_bid(program_id, accounts)
        }
        MetaplexInstruction::RedeemLimitedEditionBid => {
            msg!("Instruction: Redeem Limited Edition Bid");
            process_redeem_limited_edition_bid(program_id, accounts)
        }
        MetaplexInstruction::RedeemOpenEditionBid => {
            msg!("Instruction: Redeem Open Edition Bid");
            process_redeem_open_edition_bid(program_id, accounts)
        }
        MetaplexInstruction::StartAuction => {
            msg!("Instruction: Start Auction");
            process_start_auction(program_id, accounts)
        }
    }
}

pub fn process_redeem_open_edition_bid(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let destination_info = next_account_info(account_info_iter)?;
    let bid_redemption_info = next_account_info(account_info_iter)?;
    let safety_deposit_info = next_account_info(account_info_iter)?;
    let fraction_mint_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let auction_info = next_account_info(account_info_iter)?;
    let bidder_metadata_info = next_account_info(account_info_iter)?;
    let bidder_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let token_vault_program_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;

    // add checks for master mint/metadata/master edition combo. pull
    // assert
    let master_metadata_info = next_account_info(account_info_iter)?;
    let master_mint_info = next_account_info(account_info_iter)?;
    let master_edition_info = next_account_info(account_info_iter)?;
    let transfer_authority_info = next_account_info(account_info_iter)?;
    let CommonRedeemReturn {
        mut auction_manager,
        redemption_bump_seed,
        bidder_metadata,
        safety_deposit,
        auction,
        rent: _rent,
        destination,
    } = common_redeem_checks(
        program_id,
        auction_manager_info,
        store_info,
        destination_info,
        bid_redemption_info,
        safety_deposit_info,
        fraction_mint_info,
        vault_info,
        auction_info,
        bidder_metadata_info,
        bidder_info,
        payer_info,
        token_program_info,
        token_vault_program_info,
        token_metadata_program_info,
        rent_info,
        system_info,
        clock_info,
        true,
    )?;

    common_metadata_checks(
        master_metadata_info,
        master_edition_info,
        token_metadata_program_info,
        master_mint_info,
        &safety_deposit,
        &destination,
    )?;

    let mut gets_open_edition = auction_manager.settings.open_edition_config != None
        && auction_manager.settings.open_edition_non_winning_constraint
            != NonWinningConstraint::NoOpenEdition;

    if !bidder_metadata.cancelled {
        if let Some(winning_index) = auction.bid_state.is_winner(bidder_metadata.bidder_pubkey) {
            if winning_index < auction_manager.settings.winning_configs.len() {
                // Okay, so they placed in the auction winning prizes section!
                gets_open_edition = auction_manager.settings.open_edition_winner_constraint
                    == WinningConstraint::OpenEditionGiven;
            }
        }
    }

    if gets_open_edition {
        let seeds = &[PREFIX.as_bytes(), &auction_manager.auction.as_ref()];
        let (_, bump_seed) = Pubkey::find_program_address(seeds, &program_id);
        let mint_seeds = &[
            PREFIX.as_bytes(),
            &auction_manager.auction.as_ref(),
            &[bump_seed],
        ];

        spl_token_mint_to(TokenMintToParams {
            mint: master_mint_info.clone(),
            destination: destination_info.clone(),
            amount: 1,
            authority: auction_manager_info.clone(),
            authority_signer_seeds: mint_seeds,
            token_program: token_program_info.clone(),
        })?;

        if let Some(open_edition_fixed_price) = auction_manager.settings.open_edition_fixed_price {
            spl_token_transfer(TokenTransferParams {
                source: bidder_info.clone(),
                destination: auction_manager_info.clone(),
                amount: open_edition_fixed_price,
                authority: transfer_authority_info.clone(),
                authority_signer_seeds: mint_seeds,
                token_program: token_program_info.clone(),
            })?
        }
    }

    common_redeem_finish(
        program_id,
        &mut auction_manager,
        auction_manager_info,
        bidder_metadata_info,
        rent_info,
        system_info,
        payer_info,
        bid_redemption_info,
        redemption_bump_seed,
        false,
        true,
    )?;
    Ok(())
}

pub fn process_redeem_master_edition_bid(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let destination_info = next_account_info(account_info_iter)?;
    let bid_redemption_info = next_account_info(account_info_iter)?;
    let safety_deposit_info = next_account_info(account_info_iter)?;
    let fraction_mint_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let auction_info = next_account_info(account_info_iter)?;
    let bidder_metadata_info = next_account_info(account_info_iter)?;
    let bidder_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let token_vault_program_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;

    let metadata_info = next_account_info(account_info_iter)?;
    let name_symbol_info = next_account_info(account_info_iter)?;
    let new_metadata_authority_info = next_account_info(account_info_iter)?;
    let transfer_authority_info = next_account_info(account_info_iter)?;

    let CommonRedeemReturn {
        mut auction_manager,
        redemption_bump_seed,
        bidder_metadata,
        safety_deposit,
        auction,
        rent: _rent,
        destination: _destination,
    } = common_redeem_checks(
        program_id,
        auction_manager_info,
        store_info,
        destination_info,
        bid_redemption_info,
        safety_deposit_info,
        fraction_mint_info,
        vault_info,
        auction_info,
        bidder_metadata_info,
        bidder_info,
        payer_info,
        token_program_info,
        token_vault_program_info,
        token_metadata_program_info,
        rent_info,
        system_info,
        clock_info,
        false,
    )?;

    if !bidder_metadata.cancelled {
        if let Some(winning_index) = auction.bid_state.is_winner(bidder_metadata.bidder_pubkey) {
            if winning_index < auction_manager.settings.winning_configs.len() {
                let CommonWinningConfigCheckReturn {
                    winning_config,
                    mut winning_config_state,
                    transfer_authority,
                    vault_bump_seed,
                } = common_winning_config_checks(&auction_manager, &safety_deposit, winning_index)?;

                if winning_config.edition_type != EditionType::MasterEdition {
                    return Err(MetaplexError::WrongBidEndpointForPrize.into());
                }
                // Someone is selling off their master edition. We need to transfer it, as well as ownership of their
                // metadata.

                let auction_seeds = &[PREFIX.as_bytes(), &auction_manager.auction.as_ref()];
                let (_, auction_bump_seed) =
                    Pubkey::find_program_address(auction_seeds, &program_id);
                let auction_authority_seeds = &[
                    PREFIX.as_bytes(),
                    &auction_manager.auction.as_ref(),
                    &[auction_bump_seed],
                ];

                let vault_authority_seeds = &[
                    spl_token_vault::state::PREFIX.as_bytes(),
                    &auction_manager.token_vault_program.as_ref(),
                    &[vault_bump_seed],
                ];

                let metadata: Metadata =
                    try_from_slice_unchecked(&metadata_info.data.borrow_mut())?;

                if transfer_authority != *transfer_authority_info.key {
                    return Err(MetaplexError::InvalidTransferAuthority.into());
                }

                transfer_metadata_ownership(
                    &metadata,
                    token_metadata_program_info.clone(),
                    metadata_info.clone(),
                    name_symbol_info.clone(),
                    auction_manager_info.clone(),
                    new_metadata_authority_info.clone(),
                    auction_authority_seeds,
                )?;

                transfer_safety_deposit_box_items(
                    token_vault_program_info.clone(),
                    destination_info.clone(),
                    safety_deposit_info.clone(),
                    store_info.clone(),
                    vault_info.clone(),
                    fraction_mint_info.clone(),
                    auction_manager_info.clone(),
                    transfer_authority_info.clone(),
                    1,
                    vault_authority_seeds,
                )?;

                auction_manager
                    .state
                    .master_editions_with_authorities_remaining_to_return = match auction_manager
                    .state
                    .master_editions_with_authorities_remaining_to_return
                    .checked_sub(1)
                {
                    Some(val) => val,
                    None => return Err(MetaplexError::NumericalOverflowError.into()),
                };

                winning_config_state.claimed = true;
            }
        }
    };

    common_redeem_finish(
        program_id,
        &mut auction_manager,
        auction_manager_info,
        bidder_metadata_info,
        rent_info,
        system_info,
        payer_info,
        bid_redemption_info,
        redemption_bump_seed,
        true,
        false,
    )?;

    Ok(())
}

pub fn process_redeem_limited_edition_bid(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let destination_info = next_account_info(account_info_iter)?;
    let bid_redemption_info = next_account_info(account_info_iter)?;
    let safety_deposit_info = next_account_info(account_info_iter)?;
    let fraction_mint_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let auction_info = next_account_info(account_info_iter)?;
    let bidder_metadata_info = next_account_info(account_info_iter)?;
    let bidder_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let token_vault_program_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;

    let master_metadata_info = next_account_info(account_info_iter)?;
    let master_mint_info = next_account_info(account_info_iter)?;
    let master_edition_info = next_account_info(account_info_iter)?;
    let original_authority = next_account_info(account_info_iter)?;
    let original_authority_lookup_info = next_account_info(account_info_iter)?;

    let CommonRedeemReturn {
        mut auction_manager,
        redemption_bump_seed,
        bidder_metadata,
        safety_deposit,
        auction,
        rent: _rent,
        destination,
    } = common_redeem_checks(
        program_id,
        auction_manager_info,
        store_info,
        destination_info,
        bid_redemption_info,
        safety_deposit_info,
        fraction_mint_info,
        vault_info,
        auction_info,
        bidder_metadata_info,
        bidder_info,
        payer_info,
        token_program_info,
        token_vault_program_info,
        token_metadata_program_info,
        rent_info,
        system_info,
        clock_info,
        false,
    )?;

    common_metadata_checks(
        master_metadata_info,
        master_edition_info,
        token_metadata_program_info,
        master_mint_info,
        &safety_deposit,
        &destination,
    )?;

    // There is only one case where a follow up call needs to be made, and that's when we have multiple limited editions
    // that need to be minted across multiple destination accounts. So this may change to false in that circumstance,
    // to allow the user to call this again to mint a second, third, limited edition etc.
    let mut bid_redeemed = true;
    if !bidder_metadata.cancelled {
        if let Some(winning_index) = auction.bid_state.is_winner(bidder_metadata.bidder_pubkey) {
            if winning_index < auction_manager.settings.winning_configs.len() {
                let CommonWinningConfigCheckReturn {
                    winning_config,
                    mut winning_config_state,
                    transfer_authority: _transfer_authority,
                    vault_bump_seed: _vault_bump_seed,
                } = common_winning_config_checks(&auction_manager, &safety_deposit, winning_index)?;

                if winning_config.edition_type != EditionType::LimitedEdition {
                    return Err(MetaplexError::WrongBidEndpointForPrize.into());
                }

                // In this case we need to mint a limited edition for you!

                let seeds = &[PREFIX.as_bytes(), &auction_manager.auction.as_ref()];
                let (_, bump_seed) = Pubkey::find_program_address(seeds, &program_id);
                let mint_seeds = &[
                    PREFIX.as_bytes(),
                    &auction_manager.auction.as_ref(),
                    &[bump_seed],
                ];

                spl_token_mint_to(TokenMintToParams {
                    mint: master_mint_info.clone(),
                    destination: destination_info.clone(),
                    amount: 1,
                    authority: auction_manager_info.clone(),
                    authority_signer_seeds: mint_seeds,
                    token_program: token_program_info.clone(),
                })?;

                winning_config_state.amount_minted =
                    match winning_config_state.amount_minted.checked_add(1) {
                        Some(val) => val,
                        None => return Err(MetaplexError::NumericalOverflowError.into()),
                    };

                if winning_config_state.amount_minted == winning_config.amount {
                    winning_config_state.claimed = true;

                    // we might be able to shift authority back now. check to see others that share also
                    // are all claimed?
                    let mut any_unclaimed = false;
                    for n in 0..auction_manager.state.winning_config_states.len() {
                        let config = auction_manager.settings.winning_configs[n];
                        let state = auction_manager.state.winning_config_states[n];
                        if config.safety_deposit_box_index
                            == winning_config.safety_deposit_box_index
                            && state.claimed == false
                        {
                            any_unclaimed = true;
                            break;
                        }
                    }

                    if !any_unclaimed {
                        shift_authority_back_to_originating_user(
                            program_id,
                            &auction_manager,
                            auction_manager_info,
                            master_metadata_info,
                            original_authority,
                            original_authority_lookup_info,
                            master_mint_info,
                            token_program_info,
                            mint_seeds,
                        )?;
                    }
                } else {
                    // We need to allow the user to make another call to RedeemBid with a new destination_account
                    // For another limited edition!
                    bid_redeemed = false;
                }
            }
        }
    }

    common_redeem_finish(
        program_id,
        &mut auction_manager,
        auction_manager_info,
        bidder_metadata_info,
        rent_info,
        system_info,
        payer_info,
        bid_redemption_info,
        redemption_bump_seed,
        bid_redeemed,
        false,
    )?;
    Ok(())
}

pub fn process_redeem_bid(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let destination_info = next_account_info(account_info_iter)?;
    let bid_redemption_info = next_account_info(account_info_iter)?;
    let safety_deposit_info = next_account_info(account_info_iter)?;
    let fraction_mint_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let auction_info = next_account_info(account_info_iter)?;
    let bidder_metadata_info = next_account_info(account_info_iter)?;
    let bidder_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let token_vault_program_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;

    let transfer_authority_info = next_account_info(account_info_iter)?;

    let CommonRedeemReturn {
        mut auction_manager,
        redemption_bump_seed,
        bidder_metadata,
        safety_deposit,
        auction,
        rent: _rent,
        destination: _destination,
    } = common_redeem_checks(
        program_id,
        auction_manager_info,
        store_info,
        destination_info,
        bid_redemption_info,
        safety_deposit_info,
        fraction_mint_info,
        vault_info,
        auction_info,
        bidder_metadata_info,
        bidder_info,
        payer_info,
        token_program_info,
        token_vault_program_info,
        token_metadata_program_info,
        rent_info,
        system_info,
        clock_info,
        false,
    )?;

    if !bidder_metadata.cancelled {
        if let Some(winning_index) = auction.bid_state.is_winner(bidder_metadata.bidder_pubkey) {
            if winning_index < auction_manager.settings.winning_configs.len() {
                // Okay, so they placed in the auction winning prizes section!
                let CommonWinningConfigCheckReturn {
                    winning_config,
                    mut winning_config_state,
                    transfer_authority,
                    vault_bump_seed,
                } = common_winning_config_checks(&auction_manager, &safety_deposit, winning_index)?;

                if winning_config.edition_type != EditionType::NA {
                    return Err(MetaplexError::WrongBidEndpointForPrize.into());
                }

                let vault_authority_seeds = &[
                    spl_token_vault::state::PREFIX.as_bytes(),
                    &auction_manager.token_vault_program.as_ref(),
                    &[vault_bump_seed],
                ];

                if transfer_authority != *transfer_authority_info.key {
                    return Err(MetaplexError::InvalidTransferAuthority.into());
                }

                transfer_safety_deposit_box_items(
                    token_vault_program_info.clone(),
                    destination_info.clone(),
                    safety_deposit_info.clone(),
                    store_info.clone(),
                    vault_info.clone(),
                    fraction_mint_info.clone(),
                    auction_manager_info.clone(),
                    transfer_authority_info.clone(),
                    winning_config.amount as u64,
                    vault_authority_seeds,
                )?;
                winning_config_state.claimed = true;
            }
        }
    }

    common_redeem_finish(
        program_id,
        &mut auction_manager,
        auction_manager_info,
        bidder_metadata_info,
        rent_info,
        system_info,
        payer_info,
        bid_redemption_info,
        redemption_bump_seed,
        true,
        false,
    )?;
    Ok(())
}

pub fn process_validate_safety_deposit_box(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let name_symbol_info = next_account_info(account_info_iter)?;
    let original_authority_lookup_info = next_account_info(account_info_iter)?;
    let safety_deposit_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let edition_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let metadata_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

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

    assert_owned_by(auction_manager_info, program_id)?;
    assert_update_authority_is_correct(
        &metadata,
        metadata_info,
        Some(name_symbol_info),
        metadata_authority_info,
    )?;
    assert_authority_correct(&auction_manager, authority_info)?;
    assert_store_safety_vault_manager_match(
        &auction_manager,
        &safety_deposit,
        vault_info,
        store_info,
    )?;

    if *mint_info.key != safety_deposit.token_mint {
        return Err(MetaplexError::SafetyDepositBoxMintMismatch.into());
    }

    if *token_metadata_program_info.key != auction_manager.token_metadata_program {
        return Err(MetaplexError::AuctionManagerTokenMetadataProgramMismatch.into());
    }

    // We want to ensure that the mint you are using with this token is one
    // we can actually transfer to and from using our token program invocations, which
    // we can check by asserting ownership by the token program we recorded in init.
    if *mint_info.owner != auction_manager.token_program {
        return Err(MetaplexError::TokenProgramMismatch.into());
    }

    if safety_deposit.token_mint != metadata.mint {
        return Err(MetaplexError::SafetyDepositBoxMetadataMismatch.into());
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

    if winning_configs.len() == 0 {
        return Err(MetaplexError::SafetyDepositBoxNotUsedInAuction.into());
    }

    // At this point we know we have at least one config and they may have different amounts but all
    // point at the same safety deposit box and so have the same edition type.
    let edition_type = winning_configs[0].edition_type;
    let mut total_amount_requested: u64 = 0;
    for n in 0..winning_configs.len() {
        let curr = winning_configs[n];
        total_amount_requested = match total_amount_requested.checked_add(curr.amount.into()) {
            Some(val) => val,
            None => return Err(MetaplexError::NumericalOverflowError.into()),
        };
    }

    let edition_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        auction_manager.token_metadata_program.as_ref(),
        &edition_info.key.as_ref(),
        spl_token_metadata::state::EDITION.as_bytes(),
    ];

    let (edition_key, _) =
        Pubkey::find_program_address(edition_seeds, &auction_manager.token_metadata_program);

    // Supply logic check
    match edition_type {
        EditionType::NA | EditionType::MasterEdition => {
            if store.amount < total_amount_requested.into() {
                return Err(MetaplexError::NotEnoughTokensToSupplyWinners.into());
            }
        }
        EditionType::LimitedEdition => {
            let master_edition: MasterEdition =
                try_from_slice_unchecked(&edition_info.data.borrow_mut())?;

            if let Some(max_supply) = master_edition.max_supply {
                let remaining_supply = match max_supply.checked_sub(master_edition.supply) {
                    Some(val) => val,
                    None => return Err(MetaplexError::NumericalOverflowError.into()),
                };
                if remaining_supply < total_amount_requested {
                    return Err(MetaplexError::NotEnoughEditionsAvailableForAuction.into());
                }
            }
        }
    }

    // Other checks/logic common to both Edition enum types
    match edition_type {
        EditionType::MasterEdition | EditionType::LimitedEdition => {
            if edition_key != *edition_info.key {
                return Err(MetaplexError::InvalidEditionAddress.into());
            }

            if store.amount != 1 {
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

            create_or_allocate_account_raw(
                *program_id,
                original_authority_lookup_info,
                rent_info,
                system_info,
                payer_info,
                32,
                original_authority_seeds,
            )?;

            let mut original_authority_lookup: OriginalAuthorityLookup =
                try_from_slice_unchecked(&original_authority_lookup_info.data.borrow_mut())?;

            let seeds = &[PREFIX.as_bytes(), &auction_manager.auction.as_ref()];
            let (_, bump_seed) = Pubkey::find_program_address(seeds, &program_id);
            let authority_seeds = &[
                PREFIX.as_bytes(),
                &auction_manager.auction.as_ref(),
                &[bump_seed],
            ];

            match edition_type {
                EditionType::MasterEdition => {
                    original_authority_lookup.original_authority = *metadata_authority_info.key;

                    transfer_metadata_ownership(
                        &metadata,
                        token_metadata_program_info.clone(),
                        metadata_info.clone(),
                        name_symbol_info.clone(),
                        metadata_authority_info.clone(),
                        auction_manager_info.clone(),
                        authority_seeds,
                    )?;
                }
                EditionType::LimitedEdition => {
                    let master_mint_info = next_account_info(account_info_iter)?;
                    let master_mint_authority_info = next_account_info(account_info_iter)?;
                    // For limited edition we dont need ownership, just minting power for authorization tokens.
                    original_authority_lookup.original_authority = *master_mint_authority_info.key;

                    check_and_transfer_edition_master_mint(
                        mint_info,
                        master_mint_info,
                        edition_info,
                        auction_manager_info,
                        token_metadata_program_info,
                        token_program_info,
                        master_mint_authority_info,
                        authority_seeds,
                    )?;
                }
                _ => {}
            }

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
        _ => {}
    }

    for n in 0..winning_config_states.len() {
        winning_config_states[n].validated = true;
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

pub fn process_start_auction(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let auction_manager_info = next_account_info(account_info_iter)?;
    let auction_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let auction_program_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;

    let mut auction_manager: AuctionManager =
        try_from_slice_unchecked(&auction_manager_info.data.borrow_mut())?;
    assert_authority_correct(&auction_manager, authority_info)?;

    assert_owned_by(auction_info, &auction_manager.auction_program)?;
    assert_owned_by(auction_manager_info, program_id)?;

    if auction_manager.auction != *auction_info.key {
        return Err(MetaplexError::AuctionManagerAuctionMismatch.into());
    }

    if auction_manager.auction_program != *auction_program_info.key {
        return Err(MetaplexError::AuctionManagerAuctionProgramMismatch.into());
    }
    let seeds = &[PREFIX.as_bytes(), &auction_manager.auction.as_ref()];
    let (_, bump_seed) = Pubkey::find_program_address(seeds, &program_id);
    let authority_seeds = &[
        PREFIX.as_bytes(),
        &auction_manager.auction.as_ref(),
        &[bump_seed],
    ];

    issue_start_auction(
        auction_program_info.clone(),
        auction_manager_info.clone(),
        auction_info.clone(),
        clock_info.clone(),
        auction_manager.vault,
        authority_seeds,
    )?;

    auction_manager.state.status = AuctionManagerStatus::Running;

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
    let open_edition_metadata_info = next_account_info(account_info_iter)?;
    let open_edition_name_symbol_info = next_account_info(account_info_iter)?;
    let open_edition_authority_info = next_account_info(account_info_iter)?;
    let open_master_edition_info = next_account_info(account_info_iter)?;
    let open_edition_mint_info = next_account_info(account_info_iter)?;
    let open_edition_master_mint_info = next_account_info(account_info_iter)?;
    let open_edition_master_mint_authority_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let token_vault_program_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;
    let auction_program_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let vault: Vault = try_from_slice_unchecked(&vault_info.data.borrow_mut())?;
    let auction: AuctionData = try_from_slice_unchecked(&auction_info.data.borrow_mut())?;
    assert_owned_by(vault_info, token_vault_program_info.key)?;
    assert_owned_by(auction_info, auction_program_info.key)?;

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

    if auction.resource != *vault_info.key {
        return Err(MetaplexError::AuctionVaultMismatch.into());
    }

    if vault.state != VaultState::Combined {
        return Err(MetaplexError::VaultNotCombined.into());
    }

    if vault.token_type_count == 0 {
        return Err(MetaplexError::VaultCannotEmpty.into());
    }

    let mut winning_config_states: Vec<WinningConfigState> = vec![];
    for n in 0..auction_manager_settings.winning_configs.len() {
        let winning_config = &auction_manager_settings.winning_configs[n];
        if winning_config.safety_deposit_box_index > vault.token_type_count.into() {
            return Err(MetaplexError::InvalidSafetyDepositBox.into());
        }

        winning_config_states.push(WinningConfigState {
            amount_minted: 0,
            validated: false,
            claimed: false,
        })
    }

    let authority_seeds = &[PREFIX.as_bytes(), &auction_info.key.as_ref(), &[bump_seed]];

    if let Some(open_edition_config) = auction_manager_settings.open_edition_config {
        if open_edition_config > vault.token_type_count {
            return Err(MetaplexError::InvalidSafetyDepositBox.into());
        }

        let open_edition_metadata =
            try_from_slice_unchecked(&open_edition_metadata_info.data.borrow_mut())?;
        assert_update_authority_is_correct(
            &open_edition_metadata,
            open_edition_metadata_info,
            Some(open_edition_name_symbol_info),
            open_edition_authority_info,
        )?;

        check_and_transfer_edition_master_mint(
            open_edition_mint_info,
            open_edition_master_mint_info,
            open_master_edition_info,
            auction_manager_info,
            token_metadata_program_info,
            token_program_info,
            open_edition_master_mint_authority_info,
            authority_seeds,
        )?;
    }

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
    auction_manager.token_vault_program = *token_vault_program_info.key;
    auction_manager.token_metadata_program = *token_metadata_program_info.key;
    auction_manager.auction_program = *auction_program_info.key;
    auction_manager.state.status = AuctionManagerStatus::Initialized;
    auction_manager.state.winning_configs_validated = 0;
    auction_manager.state.winning_config_states = winning_config_states;
    auction_manager.serialize(&mut *auction_manager_info.data.borrow_mut())?;

    Ok(())
}
