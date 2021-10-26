use crate::{
    processor::BidderPot,
    utils::{
        assert_account_key, assert_derivation, assert_owned_by, assert_signer,
        create_or_allocate_account_raw,
    },
    PREFIX,
};

use {
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_instruction,
        system_instruction::create_account,
        sysvar::{rent, Sysvar},
    },
    std::mem,
};

struct Accounts<'a, 'b: 'a> {
    auction: &'a AccountInfo<'b>,
    bidder_pot: &'a AccountInfo<'b>,
    bidder: &'a AccountInfo<'b>,
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
    return Ok(accounts);
}

pub fn create_bidder_pot<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> ProgramResult {
    msg!("+ Processing CreateBidderPot");
    let accounts = parse_account(program_id, accounts)?;

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
    } else {
        msg!("Bidder pot already exists");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    Ok(())
}
