use {
    crate::{
        error::MetaplexError,
        state::{NonWinningConstraint, WinningConstraint, PREFIX},
        utils::{
            assert_initialized, common_metadata_checks, common_redeem_checks, common_redeem_finish,
            spl_token_mint_to, spl_token_transfer, CommonRedeemCheckArgs, CommonRedeemFinishArgs,
            CommonRedeemReturn,
        },
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        pubkey::Pubkey,
    },
    spl_token::state::Account,
};

#[allow(clippy::unnecessary_cast)]
#[allow(clippy::absurd_extreme_comparisons)]
pub fn process_redeem_open_edition_bid<'a>(
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
    // We keep it here to keep API base identical to the other redeem calls for ease of use by callers
    let _fraction_mint_info = next_account_info(account_info_iter)?;
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
    let master_metadata_info = next_account_info(account_info_iter)?;
    let master_mint_info = next_account_info(account_info_iter)?;
    let master_edition_info = next_account_info(account_info_iter)?;
    let transfer_authority_info = next_account_info(account_info_iter)?;
    let accept_payment_info = next_account_info(account_info_iter)?;
    let bidder_token_account_info = next_account_info(account_info_iter)?;

    let CommonRedeemReturn {
        auction_manager,
        redemption_bump_seed,
        bidder_metadata,
        safety_deposit,
        auction,
        rent: _rent,
        destination,
        bidder_pot_pubkey,
        store: _store,
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
        is_open_edition: true,
    })?;
    common_metadata_checks(
        master_metadata_info,
        master_edition_info,
        token_metadata_program_info,
        master_mint_info,
        &safety_deposit,
        &destination,
    )?;

    let bidder_token: Account = assert_initialized(bidder_token_account_info)?;

    if bidder_token.mint != auction.token_mint {
        return Err(MetaplexError::AcceptPaymentMintMismatch.into());
    }

    if *accept_payment_info.key != auction_manager.accept_payment {
        return Err(MetaplexError::AcceptPaymentMismatch.into());
    }

    let mut gets_open_edition = auction_manager.settings.open_edition_config != None
        && auction_manager.settings.open_edition_non_winning_constraint
            != NonWinningConstraint::NoOpenEdition;

    if !bidder_metadata.cancelled {
        if let Some(winning_index) = auction.is_winner(&bidder_pot_pubkey) {
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
        spl_token_mint_to(
            master_mint_info.clone(),
            destination_info.clone(),
            1,
            auction_manager_info.clone(),
            mint_seeds,
            token_program_info.clone(),
        )?;
        let mut price: u64 = 0;
        if let Some(open_edition_fixed_price) = auction_manager.settings.open_edition_fixed_price {
            price = open_edition_fixed_price;
        } else if auction_manager.settings.open_edition_non_winning_constraint
            == NonWinningConstraint::GivenForBidPrice
        {
            price = bidder_metadata.last_bid;
        }

        if bidder_token.amount.saturating_sub(price) < 0 as u64 {
            return Err(MetaplexError::NotEnoughBalanceForOpenEdition.into());
        }

        if price > 0 {
            spl_token_transfer(
                bidder_token_account_info.clone(),
                accept_payment_info.clone(),
                price,
                transfer_authority_info.clone(),
                mint_seeds,
                token_program_info.clone(),
            )?;
        }
    } else {
        return Err(MetaplexError::NotEligibleForOpenEdition.into());
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
        bid_redeemed: false,
        open_edition_redeemed: true,
    })?;
    Ok(())
}
