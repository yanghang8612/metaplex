use {
    crate::{
        error::MetaplexError,
        instruction::MetaplexInstruction,
        state::{AuctionManagerSettings, PREFIX},
        utils::{assert_initialized, assert_owned_by, assert_rent_exempt},
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        msg,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::Sysvar,
    },
    spl_auction::processor::AuctionData,
    spl_token::state::{Account, Mint},
    spl_token_vault::state::{ExternalPriceAccount, Vault},
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = MetaplexInstruction::try_from_slice(input)?;
    match instruction {
        MetaplexInstruction::InitAuctionManager(auction_manager_settings) => {
            msg!("Instruction: Init Auction Manager");
            process_init_auction_manager(program_id, accounts, auction_manager_settings)
        }
    }
}

pub fn process_init_auction_manager(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    auction_manager_settings: AuctionManagerSettings,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let auction_manager_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let auction_info = next_account_info(account_info_iter)?;
    let external_pricing_account_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

    // Just verifying this is a real account that serializes
    let _external_price_account: ExternalPriceAccount =
        try_from_slice_unchecked(&external_pricing_account_info.data.borrow_mut())?;
    let vault: Vault = try_from_slice_unchecked(&vault_info.data.borrow_mut())?;
    let auction: AuctionData = try_from_slice_unchecked(&auction_info.data.borrow_mut())?;

    assert_rent_exempt(rent, external_pricing_account_info)?;

    let seeds = &[PREFIX.as_bytes(), &auction_info.key.as_ref()];
    let (auction_authority, _) = Pubkey::find_program_address(seeds, &program_id);

    if vault.authority != auction_authority {
        return Err(MetaplexError::VaultAuthorityMismatch.into());
    }

    if *auction_manager_info.key != auction_authority {
        return Err(MetaplexError::AuctionManagerKeyMismatch.into());
    }

    if external_pricing_account_info.owner != program_id {
        return Err(MetaplexError::ExternalPriceAccountOwnerMismatch.into());
    }

    if vault.pricing_lookup_address != *external_pricing_account_info.key {
        return Err(MetaplexError::VaultExternalPricingMismatch.into());
    }

    if auction.

    vault.serialize(&mut *vault_info.data.borrow_mut())?;

    Ok(())
}
