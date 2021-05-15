use {
    crate::{
        state::{Key, Store, MAX_STORE_SIZE, PREFIX},
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

pub fn process_set_store<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    public: bool,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let store_info = next_account_info(account_info_iter)?;
    let admin_wallet_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let token_vault_program_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;
    let auction_program_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    assert_signer(payer_info)?;
    assert_signer(admin_wallet_info)?;

    let store_bump = assert_derivation(
        program_id,
        store_info,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            admin_wallet_info.key.as_ref(),
        ],
    )?;

    if store_info.data_is_empty() {
        create_or_allocate_account_raw(
            *program_id,
            store_info,
            rent_info,
            system_info,
            payer_info,
            MAX_STORE_SIZE,
            &[
                PREFIX.as_bytes(),
                program_id.as_ref(),
                admin_wallet_info.key.as_ref(),
                &[store_bump],
            ],
        )?;
    }

    let mut store: Store = try_from_slice_unchecked(&store_info.data.borrow_mut())?;

    store.key = Key::StoreV1;
    store.public = public;
    store.token_program = *token_program_info.key;
    store.token_vault_program = *token_vault_program_info.key;
    store.token_metadata_program = *token_metadata_program_info.key;
    store.auction_program = *auction_program_info.key;
    store.serialize(&mut *store_info.data.borrow_mut())?;
    Ok(())
}
