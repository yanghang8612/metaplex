use {
    crate::{
        error::MetaplexError,
        state::{AuctionManager, AuctionManagerStatus, PREFIX},
        utils::{
            assert_at_least_one_creator_matches_or_store_public, assert_authority_correct,
            assert_derivation, assert_owned_by, check_and_transfer_edition_master_mint,
        },
    },
    borsh::BorshSerialize,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
    },
    spl_token_metadata::utils::assert_update_authority_is_correct,
    spl_token_vault::state::Vault,
};

pub fn process_validate_open_edition(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let open_edition_metadata_info = next_account_info(account_info_iter)?;
    let open_edition_mint_info = next_account_info(account_info_iter)?;
    let open_edition_master_mint_info = next_account_info(account_info_iter)?;
    let open_edition_master_mint_authority_info = next_account_info(account_info_iter)?;
    let open_edition_authority_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let open_master_edition_info = next_account_info(account_info_iter)?;
    let whitelisted_creator_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;

    let mut auction_manager: AuctionManager =
        try_from_slice_unchecked(&auction_manager_info.data.borrow_mut())?;

    assert_authority_correct(&auction_manager, authority_info)?;

    let vault: Vault = try_from_slice_unchecked(&vault_info.data.borrow_mut())?;

    assert_owned_by(auction_manager_info, program_id)?;

    if auction_manager.store != *store_info.key {
        return Err(MetaplexError::AuctionManagerStoreMismatch.into());
    }

    if auction_manager.vault != *vault_info.key {
        return Err(MetaplexError::AuctionManagerVaultMismatch.into());
    }

    let bump_seed = assert_derivation(
        program_id,
        auction_manager_info,
        &[PREFIX.as_bytes(), &auction_manager.auction.as_ref()],
    )?;

    let authority_seeds = &[
        PREFIX.as_bytes(),
        &auction_manager.auction.as_ref(),
        &[bump_seed],
    ];

    if let Some(open_edition_config) = auction_manager.settings.open_edition_config {
        if open_edition_config > vault.token_type_count {
            return Err(MetaplexError::InvalidSafetyDepositBox.into());
        }

        let open_edition_metadata =
            try_from_slice_unchecked(&open_edition_metadata_info.data.borrow_mut())?;
        assert_update_authority_is_correct(&open_edition_metadata, open_edition_authority_info)?;

        assert_at_least_one_creator_matches_or_store_public(
            program_id,
            &auction_manager,
            &open_edition_metadata,
            whitelisted_creator_info,
            store_info,
        )?;

        check_and_transfer_edition_master_mint(
            open_edition_mint_info,
            open_edition_master_mint_info,
            open_master_edition_info,
            auction_manager_info,
            token_metadata_program_info,
            token_program_info,
            open_edition_master_mint_authority_info,
            authority_seeds,
        )?;

        if auction_manager.settings.winning_configs.is_empty() {
            auction_manager.state.status = AuctionManagerStatus::Validated;
        }
        auction_manager.serialize(&mut *auction_manager_info.data.borrow_mut())?;
    }

    Ok(())
}
