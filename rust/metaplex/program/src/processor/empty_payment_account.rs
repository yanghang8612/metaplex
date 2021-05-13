use solana_program::sysvar::Sysvar;

use {
    crate::{
        error::MetaplexError,
        state::{AuctionManager, PREFIX},
        utils::{
            assert_authority_correct, assert_initialized, assert_owned_by, assert_rent_exempt,
            spl_token_transfer,
        },
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
        rent::Rent,
    },
    spl_token::state::Account,
};

pub fn process_empty_payment_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let accept_payment_info = next_account_info(account_info_iter)?;
    let destination_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let auction_manager_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(&rent_info)?;

    let auction_manager: AuctionManager =
        try_from_slice_unchecked(&auction_manager_info.data.borrow_mut())?;

    let destination: Account = assert_initialized(destination_info)?;
    let accept_payment: Account = assert_initialized(accept_payment_info)?;

    assert_authority_correct(&auction_manager, authority_info)?;
    assert_owned_by(auction_manager_info, program_id)?;
    assert_owned_by(destination_info, token_program_info.key)?;
    assert_rent_exempt(rent, destination_info)?;

    if auction_manager.accept_payment != *accept_payment_info.key {
        return Err(MetaplexError::AcceptPaymentMismatch.into());
    }

    if destination.mint != accept_payment.mint {
        return Err(MetaplexError::AcceptPaymentDestinationMintMismatch.into());
    }

    let seeds = &[PREFIX.as_bytes(), &auction_manager.auction.as_ref()];
    let (key, bump_seed) = Pubkey::find_program_address(seeds, &program_id);
    let authority_seeds = &[
        PREFIX.as_bytes(),
        &auction_manager.auction.as_ref(),
        &[bump_seed],
    ];

    if key != *auction_manager_info.key {
        return Err(MetaplexError::AuctionManagerKeyMismatch.into());
    }

    spl_token_transfer(
        accept_payment_info.clone(),
        destination_info.clone(),
        accept_payment.amount,
        auction_manager_info.clone(),
        authority_seeds,
        token_program_info.clone(),
    )?;

    Ok(())
}
