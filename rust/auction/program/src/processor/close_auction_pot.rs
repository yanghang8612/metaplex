use std::str::FromStr;

use crate::{
    errors::AuctionError,
    processor::BidderPot,
    utils::{
        assert_account_key, assert_derivation, assert_initialized, assert_owned_by, assert_signer,
        create_or_allocate_account_raw,
    },
    PREFIX,
};

use borsh::BorshSerialize;
use solana_program::program::{invoke, invoke_signed};

use super::EXCLUSIVE_AUCTION_AUTHORITY;
use {
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_instruction,
        system_instruction::create_account,
        sysvar::{rent, Sysvar},
    },
    spl_token::state::Account,
    std::mem,
};

struct Accounts<'a, 'b: 'a> {
    auction: &'a AccountInfo<'b>,
    bidder_pot: &'a AccountInfo<'b>,
    bidder: &'a AccountInfo<'b>,
    destination: &'a AccountInfo<'b>,
    system: &'a AccountInfo<'b>,
    authority: &'a AccountInfo<'b>,
}

fn parse_account<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let account_iter = &mut accounts.iter();

    let accounts = Accounts {
        auction: next_account_info(account_iter)?,
        bidder_pot: next_account_info(account_iter)?,
        bidder: next_account_info(account_iter)?,
        destination: next_account_info(account_iter)?,
        system: next_account_info(account_iter)?,
        authority: next_account_info(account_iter)?,
    };

    assert_owned_by(accounts.auction, program_id)?;
    assert_signer(accounts.authority)?;
    assert_account_key(
        accounts.authority,
        &Pubkey::from_str(EXCLUSIVE_AUCTION_AUTHORITY).unwrap(),
    )?;
    assert_account_key(accounts.system, &solana_program::system_program::ID)?;
    return Ok(accounts);
}

pub fn close_auction_pot<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
    resource: Pubkey,
) -> ProgramResult {
    msg!("+ Processing CloseBidderPot");
    let accounts = parse_account(program_id, accounts)?;

    // Derive bidder pot key (to get the nonce)
    let pot_bump = assert_derivation(
        program_id,
        accounts.bidder_pot,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            accounts.auction.key.as_ref(),
            accounts.bidder.key.as_ref(),
        ],
    )?;
    let pot_seeds = [
        PREFIX.as_bytes(),
        program_id.as_ref(),
        accounts.auction.key.as_ref(),
        accounts.bidder.key.as_ref(),
        &[pot_bump],
    ];

    // Derive auction key
    let (auction_key, auction_bump) = Pubkey::find_program_address(
        &[PREFIX.as_bytes(), program_id.as_ref(), &resource.to_bytes()],
        program_id,
    );
    let auction_seeds = [
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &resource.to_bytes(),
        &[auction_bump],
    ];

    if !accounts.bidder_pot.data_is_empty() && !accounts.auction.data_is_empty() {
        let bidder_transfer_instr = system_instruction::transfer(
            &accounts.bidder_pot.key,
            &accounts.destination.key,
            accounts.bidder_pot.lamports(),
        );
        invoke_signed(
            &bidder_transfer_instr,
            &[
                accounts.system.clone(),
                accounts.bidder_pot.clone(),
                accounts.destination.clone(),
            ],
            &[&pot_seeds],
        );
        let auction_transfer_instr = system_instruction::transfer(
            &accounts.auction.key,
            &accounts.destination.key,
            accounts.auction.lamports(),
        );
        invoke_signed(
            &bidder_transfer_instr,
            &[
                accounts.system.clone(),
                accounts.bidder_pot.clone(),
                accounts.destination.clone(),
            ],
            &[&auction_seeds],
        );
    } else {
        msg!("Bidder pot does not exists");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
