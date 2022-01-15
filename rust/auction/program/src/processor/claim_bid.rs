//! Claim bid winnings into a target SPL account, only the authorised key can do this, though the
//! target can be any SPL account.

use std::str::FromStr;

use crate::{
    errors::AuctionError,
    processor::{AuctionData, BidderMetadata, BidderPot},
    utils::{
        assert_account_key, assert_derivation, assert_initialized, assert_owned_by, assert_signer,
        create_or_allocate_account_raw, spl_token_transfer, TokenTransferParams,
    },
    BONFIDA_SOL_VAULT, BUY_NOW, PREFIX, REF_SHARE,
};

use super::BuyNowData;

use {
    borsh::{try_from_slice_with_schema, BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        msg,
        program::invoke_signed,
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        system_instruction,
        sysvar::{clock::Clock, Sysvar},
    },
    spl_token::state::Account,
};

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ClaimBidArgs {
    pub resource: Pubkey,
    pub fee_percentage: u64, // * 10_000
}

struct Accounts<'a, 'b: 'a> {
    destination: &'a AccountInfo<'b>,
    bidder_pot_token: &'a AccountInfo<'b>,
    bidder_pot: &'a AccountInfo<'b>,
    authority: &'a AccountInfo<'b>,
    auction: &'a AccountInfo<'b>,
    bidder: &'a AccountInfo<'b>,
    mint: &'a AccountInfo<'b>,
    clock_sysvar: &'a AccountInfo<'b>,
    token_program: &'a AccountInfo<'b>,
    bonfida_vault: &'a AccountInfo<'b>,
    buy_now: &'a AccountInfo<'b>,
    bonfida_sol_vault: &'a AccountInfo<'b>,
    referrer: Option<&'a AccountInfo<'b>>,
}

fn parse_accounts<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let account_iter = &mut accounts.iter();
    let accounts = Accounts {
        destination: next_account_info(account_iter)?,
        bidder_pot_token: next_account_info(account_iter)?,
        bidder_pot: next_account_info(account_iter)?,
        authority: next_account_info(account_iter)?,
        auction: next_account_info(account_iter)?,
        bidder: next_account_info(account_iter)?,
        mint: next_account_info(account_iter)?,
        clock_sysvar: next_account_info(account_iter)?,
        token_program: next_account_info(account_iter)?,
        bonfida_vault: next_account_info(account_iter)?,
        buy_now: next_account_info(account_iter)?,
        bonfida_sol_vault: next_account_info(account_iter)?,
        referrer: next_account_info(account_iter).ok(),
    };

    assert_owned_by(accounts.auction, program_id)?;
    assert_owned_by(accounts.mint, &spl_token::id())?;
    assert_owned_by(accounts.destination, &spl_token::id())?;
    assert_owned_by(accounts.bidder_pot_token, &spl_token::id())?;
    assert_signer(accounts.authority)?;

    assert_account_key(
        accounts.bonfida_sol_vault,
        &Pubkey::from_str(BONFIDA_SOL_VAULT).unwrap(),
    )
    .unwrap();

    Ok(accounts)
}

pub fn claim_bid(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: ClaimBidArgs,
) -> ProgramResult {
    msg!("+ Processing ClaimBid");
    let accounts = parse_accounts(program_id, accounts)?;
    let clock = Clock::from_account_info(accounts.clock_sysvar)?;

    // The account within the pot must be owned by us.
    let actual_account: Account = assert_initialized(accounts.bidder_pot_token)?;
    if actual_account.owner != *accounts.auction.key || &actual_account.owner != program_id {
        return Err(AuctionError::BidderPotTokenAccountOwnerMismatch.into());
    }

    // Derive and load Auction.
    let auction_bump = assert_derivation(
        program_id,
        accounts.auction,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            args.resource.as_ref(),
        ],
    )?;

    let auction_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        args.resource.as_ref(),
        &[auction_bump],
    ];

    // Load the auction and verify this bid is valid.
    let auction: AuctionData = try_from_slice_unchecked(&accounts.auction.data.borrow())?;

    // User must have won the auction in order to claim their funds. Check early as the rest of the
    // checks will be for nothing otherwise.
    if auction.is_winner(accounts.bidder_pot.key).is_none() {
        return Err(AuctionError::InvalidState.into());
    }

    // Auction must have ended.
    if !auction.ended(clock.unix_timestamp)? {
        return Err(AuctionError::InvalidState.into());
    }

    // The mint provided in this claim must match the one the auction was initialized with.
    if auction.token_mint != *accounts.mint.key {
        return Err(AuctionError::IncorrectMint.into());
    }

    // Derive Pot address, this account wraps/holds an SPL account to transfer tokens into.
    let pot_seeds = [
        PREFIX.as_bytes(),
        program_id.as_ref(),
        accounts.auction.key.as_ref(),
        accounts.bidder.key.as_ref(),
    ];

    let pot_bump = assert_derivation(program_id, accounts.bidder_pot, &pot_seeds)?;

    let bump_authority_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        accounts.auction.key.as_ref(),
        accounts.bidder.key.as_ref(),
        &[pot_bump],
    ];

    // If the bidder pot account is empty, this bid is invalid.
    if accounts.bidder_pot.data_is_empty() {
        return Err(AuctionError::BidderPotDoesNotExist.into());
    }

    // Confirm we're looking at the real SPL account for this bidder.
    let mut bidder_pot: BidderPot =
        try_from_slice_unchecked(&accounts.bidder_pot.data.borrow_mut())?;
    if bidder_pot.bidder_pot != *accounts.bidder_pot_token.key {
        return Err(AuctionError::BidderPotTokenAccountOwnerMismatch.into());
    }

    // Calculate fees
    let fees = args
        .fee_percentage
        .checked_mul(actual_account.amount)
        .ok_or(AuctionError::NumericalOverflowError)?
        .checked_div(10000)
        .ok_or(AuctionError::NumericalOverflowError)?;
    let ref_fees = if accounts.referrer.is_some() {
        fees.checked_mul(REF_SHARE)
            .ok_or(AuctionError::NumericalOverflowError)?
            .checked_div(100)
            .ok_or(AuctionError::NumericalOverflowError)?
    } else {
        0
    };

    let rest_amount = actual_account
        .amount
        .checked_sub(fees)
        .ok_or(AuctionError::NumericalOverflowError)?;
    let fees = fees
        .checked_sub(ref_fees)
        .ok_or(AuctionError::NumericalOverflowError)?;

    // Stopgap measure
    if &actual_account.owner != program_id {
        // Transfer SPL bid balance back to the user and the bonfida vault
        spl_token_transfer(TokenTransferParams {
            source: accounts.bidder_pot_token.clone(),
            destination: accounts.destination.clone(),
            authority: accounts.auction.clone(),
            authority_signer_seeds: auction_seeds,
            token_program: accounts.token_program.clone(),
            amount: rest_amount,
        })?;
        spl_token_transfer(TokenTransferParams {
            source: accounts.bidder_pot_token.clone(),
            destination: accounts.bonfida_vault.clone(),
            authority: accounts.auction.clone(),
            authority_signer_seeds: auction_seeds,
            token_program: accounts.token_program.clone(),
            amount: fees,
        })?;

        if ref_fees != 0 {
            spl_token_transfer(TokenTransferParams {
                source: accounts.bidder_pot_token.clone(),
                destination: accounts.referrer.unwrap().clone(),
                authority: accounts.auction.clone(),
                authority_signer_seeds: auction_seeds,
                token_program: accounts.token_program.clone(),
                amount: ref_fees,
            })?;
        }
    }

    bidder_pot.emptied = true;
    bidder_pot.serialize(&mut *accounts.bidder_pot.data.borrow_mut())?;

    // Collect lamports from the buy_now account as an additional fee for this type of sales
    if buy_now_account_exists(program_id, &args.resource, &accounts.buy_now) {
        let mut target_lamports = accounts.bonfida_sol_vault.lamports.borrow_mut();
        let mut buy_now_lamports = accounts.buy_now.lamports.borrow_mut();

        **target_lamports += **buy_now_lamports;
        **buy_now_lamports = 0;
    }

    Ok(())
}

fn buy_now_account_exists(program_id: &Pubkey, resource: &Pubkey, buy_now: &AccountInfo) -> bool {
    let buy_now_path = [BUY_NOW.as_bytes(), program_id.as_ref(), resource.as_ref()];
    let (buy_now_key, buy_now_bump) = Pubkey::find_program_address(&buy_now_path, program_id);
    assert_account_key(buy_now, &buy_now_key).unwrap();
    if buy_now.data_len() != 0 {
        // If the account exists it must be owned by the program
        assert_owned_by(buy_now, program_id).unwrap();
        // To prevent issues when the domains are directly resold before being gc
        BuyNowData {
            max_price: u64::MAX,
        }
        .serialize(&mut *buy_now.data.borrow_mut())
        .unwrap();
        return true;
    }
    false
}
