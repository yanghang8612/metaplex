use {
    crate::{
        state::{Key, WhitelistedCreator, MAX_WHITELISTED_CREATOR_SIZE, PREFIX},
        utils::{assert_derivation, assert_signer, create_or_allocate_account_raw},
    },
    borsh::BorshSerialize,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
    },
};

pub fn process_set_whitelisted_creator<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    activated: bool,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let creator_info = next_account_info(account_info_iter)?;
    let admin_wallet_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    assert_signer(payer_info)?;
    assert_signer(admin_wallet_info)?;

    assert_derivation(
        program_id,
        store_info,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            admin_wallet_info.key.as_ref(),
        ],
    )?;

    let creator_bump = assert_derivation(
        program_id,
        creator_info,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            store_info.key.as_ref(),
            creator_info.key.as_ref(),
        ],
    )?;

    if creator_info.data_is_empty() {
        create_or_allocate_account_raw(
            *program_id,
            store_info,
            rent_info,
            system_info,
            payer_info,
            MAX_WHITELISTED_CREATOR_SIZE,
            &[
                PREFIX.as_bytes(),
                program_id.as_ref(),
                store_info.key.as_ref(),
                creator_info.key.as_ref(),
                &[creator_bump],
            ],
        )?;
    }

    let mut whitelisted_creator: WhitelistedCreator =
        try_from_slice_unchecked(&creator_info.data.borrow_mut())?;

    whitelisted_creator.key = Key::WhitelistedCreatorV1;
    whitelisted_creator.activated = activated;

    whitelisted_creator.serialize(&mut *creator_info.data.borrow_mut())?;
    Ok(())
}
