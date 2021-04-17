use {
    crate::{
        error::MetaplexError,
        state::{AuctionManager, PREFIX},
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_pack::{IsInitialized, Pack},
        pubkey::Pubkey,
        system_instruction,
        sysvar::{rent::Rent, Sysvar},
    },
    spl_token_metadata::{
        instruction::{mint_new_edition_from_master_edition, transfer_update_authority},
        state::Metadata,
    },
    spl_token_vault::{instruction::create_withdraw_tokens_instruction, state::SafetyDepositBox},
    std::{convert::TryInto, slice::Iter},
};

/// assert initialized account
pub fn assert_initialized<T: Pack + IsInitialized>(
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        Err(MetaplexError::Uninitialized.into())
    } else {
        Ok(account)
    }
}

pub fn assert_rent_exempt(rent: &Rent, account_info: &AccountInfo) -> ProgramResult {
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        Err(MetaplexError::NotRentExempt.into())
    } else {
        Ok(())
    }
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        Err(MetaplexError::IncorrectOwner.into())
    } else {
        Ok(())
    }
}

pub fn assert_store_safety_vault_manager_match(
    auction_manager: &AuctionManager,
    safety_deposit: &SafetyDepositBox,
    vault_info: &AccountInfo,
    store_info: &AccountInfo,
) -> ProgramResult {
    if auction_manager.vault != *vault_info.key {
        return Err(MetaplexError::AuctionManagerVaultMismatch.into());
    }

    if safety_deposit.vault != *vault_info.key {
        return Err(MetaplexError::SafetyDepositBoxVaultMismatch.into());
    }

    if safety_deposit.store != *store_info.key {
        return Err(MetaplexError::SafetyDepositBoxStoreMismatch.into());
    }
    Ok(())
}

pub fn assert_authority_correct(
    auction_manager: &AuctionManager,
    authority_info: &AccountInfo,
) -> ProgramResult {
    if auction_manager.authority != *authority_info.key {
        return Err(MetaplexError::AuctionManagerAuthorityMismatch.into());
    }

    if !authority_info.is_signer {
        return Err(MetaplexError::AuctionManagerAuthorityIsNotSigner.into());
    }

    Ok(())
}
/// Create account almost from scratch, lifted from
/// https://github.com/solana-labs/solana-program-library/blob/7d4873c61721aca25464d42cc5ef651a7923ca79/associated-token-account/program/src/processor.rs#L51-L98
#[inline(always)]
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports);
        invoke(
            &system_instruction::transfer(&payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    msg!("Allocate space for the account");
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        &[new_account_info.clone(), system_program_info.clone()],
        &[&signer_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        &[new_account_info.clone(), system_program_info.clone()],
        &[&signer_seeds],
    )?;
    msg!("Completed assignation!");

    Ok(())
}

pub fn transfer_safety_deposit_box_items<'a>(
    token_vault_program: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    safety_deposit_box: AccountInfo<'a>,
    store: AccountInfo<'a>,
    vault: AccountInfo<'a>,
    fraction_mint: AccountInfo<'a>,
    vault_authority: AccountInfo<'a>,
    transfer_authority: AccountInfo<'a>,
    amount: u64,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &create_withdraw_tokens_instruction(
            *token_vault_program.key,
            *destination.key,
            *safety_deposit_box.key,
            *store.key,
            *vault.key,
            *fraction_mint.key,
            *vault_authority.key,
            *transfer_authority.key,
            amount,
        ),
        &[
            token_vault_program,
            destination,
            safety_deposit_box,
            store,
            vault,
            fraction_mint,
            vault_authority,
            transfer_authority,
        ],
        &[&signer_seeds],
    )?;

    Ok(())
}

pub fn mint_edition_from_account_iterator<'a>(
    program_id: Pubkey,
    auction_manager_info: &AccountInfo<'a>,
    token_metadata_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    account_info_iter: &mut Iter<AccountInfo<'a>>,
) -> ProgramResult {
    let new_metadata_info = next_account_info(account_info_iter)?;
    let destination_mint_info = next_account_info(account_info_iter)?;
    let destination_mint_authority_info = next_account_info(account_info_iter)?;
    let master_metadata_info = next_account_info(account_info_iter)?;
    let new_edition_info = next_account_info(account_info_iter)?;
    let master_edition_info = next_account_info(account_info_iter)?;

    let seeds = &[PREFIX.as_bytes(), &auction_manager_info.key.as_ref()];
    let (_, bump_seed) = Pubkey::find_program_address(seeds, &program_id);
    let mint_seeds = &[
        PREFIX.as_bytes(),
        &auction_manager_info.key.as_ref(),
        &[bump_seed],
    ];

    mint_edition(
        token_metadata_program_info.clone(),
        new_metadata_info.clone(),
        new_edition_info.clone(),
        master_edition_info.clone(),
        destination_mint_info.clone(),
        destination_mint_authority_info.clone(),
        payer_info.clone(),
        auction_manager_info.clone(),
        master_metadata_info.clone(),
        mint_seeds,
    )?;

    Ok(())
}

pub fn mint_edition<'a>(
    token_metadata_program: AccountInfo<'a>,
    metadata: AccountInfo<'a>,
    edition: AccountInfo<'a>,
    master_edition: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    mint_authority: AccountInfo<'a>,
    payer: AccountInfo<'a>,
    master_update_authority: AccountInfo<'a>,
    master_metadata: AccountInfo<'a>,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &mint_new_edition_from_master_edition(
            *token_metadata_program.key,
            *metadata.key,
            *edition.key,
            *master_edition.key,
            *mint.key,
            *mint_authority.key,
            *payer.key,
            *master_update_authority.key,
            *master_metadata.key,
        ),
        &[
            metadata,
            edition,
            master_edition,
            mint,
            mint_authority,
            payer,
            master_update_authority,
            master_metadata,
            token_metadata_program,
        ],
        &[&signer_seeds],
    )?;

    Ok(())
}

pub fn transfer_metadata_ownership<'a>(
    metadata: &Metadata,
    token_metadata_program: AccountInfo<'a>,
    metadata_info: AccountInfo<'a>,
    name_symbol: AccountInfo<'a>,
    update_authority: AccountInfo<'a>,
    new_update_authority: AccountInfo<'a>,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    let transferring_obj = match metadata.non_unique_specific_update_authority {
        Some(_) => metadata_info,
        None => name_symbol,
    };
    invoke_signed(
        &transfer_update_authority(
            *token_metadata_program.key,
            *transferring_obj.key,
            *update_authority.key,
            *new_update_authority.key,
        ),
        &[
            update_authority,
            new_update_authority,
            transferring_obj,
            token_metadata_program,
        ],
        &[&signer_seeds],
    )?;

    Ok(())
}

/// Issue a spl_token `Transfer` instruction.
#[inline(always)]
pub fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> ProgramResult {
    let TokenTransferParams {
        source,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let result = invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination, authority, token_program],
        &[authority_signer_seeds],
    );
    result.map_err(|_| MetaplexError::TokenTransferFailed.into())
}

/// Issue a spl_token `MintTo` instruction.
pub fn spl_token_mint_to(params: TokenMintToParams<'_, '_>) -> ProgramResult {
    let TokenMintToParams {
        mint,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let result = invoke_signed(
        &spl_token::instruction::mint_to(
            token_program.key,
            mint.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[mint, destination, authority, token_program],
        &[authority_signer_seeds],
    );
    result.map_err(|_| MetaplexError::TokenMintToFailed.into())
}

/// Issue a spl_token `Burn` instruction.
#[inline(always)]
pub fn spl_token_burn(params: TokenBurnParams<'_, '_>) -> ProgramResult {
    let TokenBurnParams {
        mint,
        source,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let result = invoke_signed(
        &spl_token::instruction::burn(
            token_program.key,
            source.key,
            mint.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, mint, authority, token_program],
        &[authority_signer_seeds],
    );
    result.map_err(|_| MetaplexError::TokenBurnFailed.into())
}

///TokenTransferParams
pub struct TokenTransferParams<'a: 'b, 'b> {
    /// source
    pub source: AccountInfo<'a>,
    /// destination
    pub destination: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: &'b [&'b [u8]],
    /// token_program
    pub token_program: AccountInfo<'a>,
}
/// TokenMintToParams
pub struct TokenMintToParams<'a: 'b, 'b> {
    /// mint
    pub mint: AccountInfo<'a>,
    /// destination
    pub destination: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: &'b [&'b [u8]],
    /// token_program
    pub token_program: AccountInfo<'a>,
}
/// TokenBurnParams
pub struct TokenBurnParams<'a: 'b, 'b> {
    /// mint
    pub mint: AccountInfo<'a>,
    /// source
    pub source: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: &'b [&'b [u8]],
    /// token_program
    pub token_program: AccountInfo<'a>,
}
