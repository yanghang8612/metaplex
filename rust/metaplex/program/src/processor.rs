use {
    crate::{
        error::MetaplexError,
        instruction::MetaplexInstruction,
        state::{EditionType, PREFIX},
        utils::{
            common_redeem_checks, common_redeem_finish, common_winning_config_checks,
            transfer_metadata_ownership, transfer_safety_deposit_box_items, CommonRedeemCheckArgs,
            CommonRedeemFinishArgs, CommonRedeemReturn, CommonWinningConfigCheckReturn,
        },
    },
    borsh::BorshDeserialize,
    claim_bid::process_claim_bid,
    empty_payment_account::process_empty_payment_account,
    init_auction_manager::process_init_auction_manager,
    redeem_open_edition_bid::process_redeem_open_edition_bid,
    set_store::process_set_store,
    set_whitelisted_creator::process_set_whitelisted_creator,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        pubkey::Pubkey,
    },
    start_auction::process_start_auction,
    validate_open_edition::process_validate_open_edition,
    validate_safety_deposit_box::process_validate_safety_deposit_box,
};

pub mod claim_bid;
pub mod empty_payment_account;
pub mod init_auction_manager;
pub mod redeem_open_edition_bid;
pub mod set_store;
pub mod set_whitelisted_creator;
pub mod start_auction;
pub mod validate_open_edition;
pub mod validate_safety_deposit_box;

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
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
        MetaplexInstruction::RedeemOpenEditionBid => {
            msg!("Instruction: Redeem Open Edition Bid");
            process_redeem_open_edition_bid(program_id, accounts)
        }
        MetaplexInstruction::StartAuction => {
            msg!("Instruction: Start Auction");
            process_start_auction(program_id, accounts)
        }
        MetaplexInstruction::ClaimBid => {
            msg!("Instruction: Claim Bid");
            process_claim_bid(program_id, accounts)
        }
        MetaplexInstruction::EmptyPaymentAccount => {
            msg!("Instruction: Empty Payment Account");
            process_empty_payment_account(program_id, accounts)
        }
        MetaplexInstruction::SetStore(args) => {
            msg!("Instruction: Set Store");
            process_set_store(program_id, accounts, args.public)
        }
        MetaplexInstruction::SetWhitelistedCreator(args) => {
            msg!("Instruction: Set Whitelisted Creator");
            process_set_whitelisted_creator(program_id, accounts, args.activated)
        }
        MetaplexInstruction::ValidateOpenEdition => {
            msg!("Instruction: Validate Open Edition");
            process_validate_open_edition(program_id, accounts)
        }
    }
}

pub fn process_redeem_master_edition_bid<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let safety_deposit_token_store_info = next_account_info(account_info_iter)?;
    let destination_info = next_account_info(account_info_iter)?;
    let bid_redemption_info = next_account_info(account_info_iter)?;
    let safety_deposit_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let fraction_mint_info = next_account_info(account_info_iter)?;
    let auction_info = next_account_info(account_info_iter)?;
    let bidder_metadata_info = next_account_info(account_info_iter)?;
    let bidder_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let token_vault_program_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let metadata_info = next_account_info(account_info_iter)?;
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
        bidder_pot_pubkey,
        store,
    } = common_redeem_checks(CommonRedeemCheckArgs {
        program_id,
        auction_manager_info,
        safety_deposit_token_store_info,
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
        store_info,
        rent_info,
        is_open_edition: false,
    })?;

    if !bidder_metadata.cancelled {
        if let Some(winning_index) = auction.is_winner(&bidder_pot_pubkey) {
            if winning_index < auction_manager.settings.winning_configs.len() {
                let CommonWinningConfigCheckReturn {
                    winning_config,
                    mut winning_config_state,
                    transfer_authority,
                } = common_winning_config_checks(
                    &auction_manager,
                    &store,
                    &safety_deposit,
                    winning_index,
                )?;

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

                if transfer_authority != *transfer_authority_info.key {
                    return Err(MetaplexError::InvalidTransferAuthority.into());
                }

                transfer_metadata_ownership(
                    token_metadata_program_info.clone(),
                    metadata_info.clone(),
                    auction_manager_info.clone(),
                    new_metadata_authority_info.clone(),
                    auction_authority_seeds,
                )?;

                transfer_safety_deposit_box_items(
                    token_vault_program_info.clone(),
                    destination_info.clone(),
                    safety_deposit_info.clone(),
                    safety_deposit_token_store_info.clone(),
                    vault_info.clone(),
                    fraction_mint_info.clone(),
                    auction_manager_info.clone(),
                    transfer_authority_info.clone(),
                    rent_info.clone(),
                    1,
                    auction_authority_seeds,
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

    common_redeem_finish(CommonRedeemFinishArgs {
        program_id,
        auction_manager,
        auction_manager_info,
        bidder_metadata_info,
        rent_info,
        system_info,
        payer_info,
        bid_redemption_info,
        redemption_bump_seed,
        bid_redeemed: true,
        open_edition_redeemed: false,
    })?;

    Ok(())
}

pub fn process_redeem_bid<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let safety_deposit_token_store_info = next_account_info(account_info_iter)?;
    let destination_info = next_account_info(account_info_iter)?;
    let bid_redemption_info = next_account_info(account_info_iter)?;
    let safety_deposit_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let fraction_mint_info = next_account_info(account_info_iter)?;
    let auction_info = next_account_info(account_info_iter)?;
    let bidder_metadata_info = next_account_info(account_info_iter)?;
    let bidder_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let token_vault_program_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let transfer_authority_info = next_account_info(account_info_iter)?;

    let CommonRedeemReturn {
        auction_manager,
        redemption_bump_seed,
        bidder_metadata,
        safety_deposit,
        auction,
        rent: _rent,
        destination: _destination,
        bidder_pot_pubkey,
        store,
    } = common_redeem_checks(CommonRedeemCheckArgs {
        program_id,
        auction_manager_info,
        safety_deposit_token_store_info,
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
        store_info,
        is_open_edition: false,
    })?;

    if !bidder_metadata.cancelled {
        if let Some(winning_index) = auction.is_winner(&bidder_pot_pubkey) {
            if winning_index < auction_manager.settings.winning_configs.len() {
                // Okay, so they placed in the auction winning prizes section!
                let CommonWinningConfigCheckReturn {
                    winning_config,
                    mut winning_config_state,
                    transfer_authority,
                } = common_winning_config_checks(
                    &auction_manager,
                    &store,
                    &safety_deposit,
                    winning_index,
                )?;

                if winning_config.edition_type != EditionType::Na
                    && winning_config.edition_type != EditionType::LimitedEdition
                {
                    return Err(MetaplexError::WrongBidEndpointForPrize.into());
                }

                let auction_seeds = &[PREFIX.as_bytes(), &auction_manager.auction.as_ref()];

                let (_, auction_bump_seed) =
                    Pubkey::find_program_address(auction_seeds, program_id);

                let auction_auth_seeds = &[
                    PREFIX.as_bytes(),
                    &auction_manager.auction.as_ref(),
                    &[auction_bump_seed],
                ];

                if transfer_authority != *transfer_authority_info.key {
                    return Err(MetaplexError::InvalidTransferAuthority.into());
                }

                transfer_safety_deposit_box_items(
                    token_vault_program_info.clone(),
                    destination_info.clone(),
                    safety_deposit_info.clone(),
                    safety_deposit_token_store_info.clone(),
                    vault_info.clone(),
                    fraction_mint_info.clone(),
                    auction_manager_info.clone(),
                    transfer_authority_info.clone(),
                    rent_info.clone(),
                    winning_config.amount as u64,
                    auction_auth_seeds,
                )?;
                winning_config_state.claimed = true;
            }
        }
    }

    common_redeem_finish(CommonRedeemFinishArgs {
        program_id,
        auction_manager,
        auction_manager_info,
        bidder_metadata_info,
        rent_info,
        system_info,
        payer_info,
        bid_redemption_info,
        redemption_bump_seed,
        bid_redeemed: true,
        open_edition_redeemed: false,
    })?;
    Ok(())
}
