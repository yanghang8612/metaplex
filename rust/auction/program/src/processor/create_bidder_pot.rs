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
    bidder_pot_token: &'a AccountInfo<'b>,
    transfer_authority: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    rent: &'a AccountInfo<'b>,
    system: &'a AccountInfo<'b>,
}

fn parse_account<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let account_iter = &mut accounts.iter();

    let accounts = Accounts {
        auction: next_account_info(account_iter)?,
        bidder_pot: next_account_info(account_iter)?,
        bidder_pot_token: next_account_info(account_iter)?,
        bidder: next_account_info(account_iter)?,
        transfer_authority: next_account_info(account_iter)?,
        payer: next_account_info(account_iter)?,
        rent: next_account_info(account_iter)?,
        system: next_account_info(account_iter)?,
    };

    assert_owned_by(accounts.auction, program_id)?;
    assert_signer(accounts.bidder)?;
    assert_signer(accounts.payer)?;
    assert_signer(accounts.transfer_authority)?;
    assert_account_key(accounts.rent, &solana_program::sysvar::rent::ID)?;
    assert_account_key(accounts.system, &solana_program::system_program::ID)?;
    assert_owned_by(accounts.bidder_pot_token, &spl_token::id())?;
    return Ok(accounts);
}

pub fn create_bidder_pot<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> ProgramResult {
    msg!("+ Processing CreateBidderPot");
    let accounts = parse_account(program_id, accounts)?;

    let actual_account: Account = assert_initialized(accounts.bidder_pot_token)?;
    if actual_account.owner != *accounts.auction.key {
        return Err(AuctionError::BidderPotTokenAccountOwnerMismatch.into());
    }

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

    let bump_authority_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        accounts.auction.key.as_ref(),
        accounts.bidder.key.as_ref(),
        &[pot_bump],
    ];

    if accounts.bidder_pot.data_is_empty() {
        create_or_allocate_account_raw(
            *program_id,
            accounts.bidder_pot,
            accounts.rent,
            accounts.system,
            accounts.payer,
            mem::size_of::<BidderPot>(),
            bump_authority_seeds,
        )?;
        let mut pot: BidderPot = try_from_slice_unchecked(&accounts.bidder_pot.data.borrow_mut())?;
        pot.bidder_pot = *accounts.bidder_pot_token.key;
        pot.bidder_act = *accounts.bidder.key;
        pot.auction_act = *accounts.auction.key;
        pot.serialize(&mut *accounts.bidder_pot.data.borrow_mut())?;
    } else {
        msg!("Bidder pot already exists");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    Ok(())
}
