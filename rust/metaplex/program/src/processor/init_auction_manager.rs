use {
    crate::{
        error::MetaplexError,
        state::{
            AuctionManager, AuctionManagerSettings, AuctionManagerStatus, Key, Store,
            WinningConfigState, MAX_AUCTION_MANAGER_SIZE, PREFIX,
        },
        utils::{
            assert_derivation, assert_initialized, assert_owned_by, create_or_allocate_account_raw,
        },
    },
    borsh::BorshSerialize,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
    },
    spl_auction::processor::AuctionData,
    spl_token::state::Account,
    spl_token_vault::state::{Vault, VaultState},
};

pub fn process_init_auction_manager(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    auction_manager_settings: AuctionManagerSettings,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let auction_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let accept_payment_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let vault: Vault = try_from_slice_unchecked(&vault_info.data.borrow_mut())?;
    let auction: AuctionData = try_from_slice_unchecked(&auction_info.data.borrow_mut())?;
    let accept_payment: Account = assert_initialized(accept_payment_info)?;
    // Assert it is real
    let store: Store = try_from_slice_unchecked(&store_info.data.borrow_mut())?;

    assert_owned_by(vault_info, &store.token_vault_program)?;
    assert_owned_by(auction_info, &store.auction_program)?;
    assert_owned_by(store_info, program_id)?;

    if vault.authority != *auction_manager_info.key {
        return Err(MetaplexError::VaultAuthorityMismatch.into());
    }

    if auction.authority != *auction_manager_info.key {
        return Err(MetaplexError::AuctionAuthorityMismatch.into());
    }

    let bump_seed = assert_derivation(
        program_id,
        auction_manager_info,
        &[PREFIX.as_bytes(), &auction_info.key.as_ref()],
    )?;

    assert_derivation(
        &store.auction_program,
        auction_info,
        &[
            spl_auction::PREFIX.as_bytes(),
            &store.auction_program.as_ref(),
            &vault_info.key.as_ref(),
        ],
    )?;

    if auction.token_mint != accept_payment.mint {
        return Err(MetaplexError::AuctionAcceptPaymentMintMismatch.into());
    }

    if accept_payment.owner != *auction_manager_info.key {
        return Err(MetaplexError::AcceptPaymentOwnerMismatch.into());
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
        if winning_config.safety_deposit_box_index > vault.token_type_count {
            return Err(MetaplexError::InvalidSafetyDepositBox.into());
        }

        winning_config_states.push(WinningConfigState {
            amount_minted: 0,
            validated: false,
            claimed: false,
        })
    }

    let authority_seeds = &[PREFIX.as_bytes(), &auction_info.key.as_ref(), &[bump_seed]];

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
    auction_manager.store = *store_info.key;
    auction_manager.state.status = AuctionManagerStatus::Initialized;
    auction_manager.settings = auction_manager_settings;
    auction_manager.vault = *vault_info.key;
    auction_manager.auction = *auction_info.key;
    auction_manager.authority = *authority_info.key;
    auction_manager.accept_payment = *accept_payment_info.key;
    auction_manager.state.winning_configs_validated = 0;
    auction_manager.state.winning_config_states = winning_config_states;
    auction_manager.serialize(&mut *auction_manager_info.data.borrow_mut())?;

    Ok(())
}
