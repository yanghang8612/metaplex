use {
    crate::{
        error::MetadataError,
        instruction::MetadataInstruction,
        state::{
            Key, MasterEdition, Metadata, EDITION, MAX_MASTER_EDITION_LEN, MAX_METADATA_LEN,
            MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH, PREFIX,
        },
        utils::{
            assert_initialized, assert_mint_authority_matches_mint, assert_rent_exempt,
            assert_update_authority_is_correct, create_or_allocate_account_raw,
            mint_limited_edition, spl_token_burn, spl_token_mint_to, transfer_mint_authority,
            TokenBurnParams, TokenMintToParams,
        },
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
    spl_token::state::{Account, Mint},
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = MetadataInstruction::try_from_slice(input)?;
    match instruction {
        MetadataInstruction::CreateMetadataAccount(args) => {
            msg!("Instruction: Create Metadata Accounts");
            process_create_metadata_accounts(
                program_id,
                accounts,
                args.data.name,
                args.data.symbol,
                args.data.uri,
            )
        }
        MetadataInstruction::UpdateMetadataAccount(args) => {
            msg!("Instruction: Update Metadata Accounts");
            process_update_metadata_accounts(program_id, accounts, args.uri, args.update_authority)
        }
        MetadataInstruction::CreateMasterEdition(args) => {
            msg!("Instruction: Create Master Edition");
            process_create_master_edition(program_id, accounts, args.max_supply)
        }
        MetadataInstruction::MintNewEditionFromMasterEditionViaToken => {
            msg!("Instruction: Mint New Edition from Master Edition Via Token");
            process_mint_new_edition_from_master_edition_via_token(program_id, accounts)
        }
    }
}

/// Create a new account instruction
pub fn process_create_metadata_accounts(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    symbol: String,
    uri: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    if name.len() > MAX_NAME_LENGTH {
        return Err(MetadataError::NameTooLong.into());
    }

    if symbol.len() > MAX_SYMBOL_LENGTH {
        return Err(MetadataError::SymbolTooLong.into());
    }

    if uri.len() > MAX_URI_LENGTH {
        return Err(MetadataError::UriTooLong.into());
    }

    let mint: Mint = assert_initialized(mint_info)?;
    assert_mint_authority_matches_mint(&mint, mint_authority_info)?;

    let metadata_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (metadata_key, metadata_bump_seed) =
        Pubkey::find_program_address(metadata_seeds, program_id);
    let metadata_authority_signer_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        &[metadata_bump_seed],
    ];

    if metadata_account_info.key != &metadata_key {
        return Err(MetadataError::InvalidMetadataKey.into());
    }

    create_or_allocate_account_raw(
        *program_id,
        metadata_account_info,
        rent_info,
        system_account_info,
        payer_account_info,
        MAX_METADATA_LEN,
        metadata_authority_signer_seeds,
    )?;

    let mut metadata: Metadata = try_from_slice_unchecked(&metadata_account_info.data.borrow())?;
    metadata.mint = *mint_info.key;
    metadata.key = Key::MetadataV1;
    metadata.data.name = name;
    metadata.data.symbol = symbol;
    metadata.data.uri = uri;
    metadata.update_authority = *update_authority_info.key;

    metadata.serialize(&mut *metadata_account_info.data.borrow_mut())?;

    Ok(())
}

/// Update existing account instruction
pub fn process_update_metadata_accounts(
    _: &Pubkey,
    accounts: &[AccountInfo],
    uri: Option<String>,
    update_authority: Option<Pubkey>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let mut metadata: Metadata = try_from_slice_unchecked(&metadata_account_info.data.borrow())?;

    assert_update_authority_is_correct(&metadata, update_authority_info)?;

    if let Some(val) = uri {
        if val.len() > MAX_URI_LENGTH {
            return Err(MetadataError::UriTooLong.into());
        }
        metadata.data.uri = val;
    }

    if let Some(val) = update_authority {
        metadata.update_authority = val;
    }

    metadata.serialize(&mut *metadata_account_info.data.borrow_mut())?;
    Ok(())
}

/// Create master edition
pub fn process_create_master_edition(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    max_supply: Option<u64>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let edition_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let master_mint_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let metadata_account_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_info)?;

    let metadata: Metadata = try_from_slice_unchecked(&metadata_account_info.data.borrow())?;
    let mint: Mint = assert_initialized(mint_info)?;
    let master_mint: Mint = assert_initialized(master_mint_info)?;

    let edition_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ];
    let (edition_key, bump_seed) = Pubkey::find_program_address(edition_seeds, program_id);

    if edition_key != *edition_account_info.key {
        return Err(MetadataError::InvalidEditionKey.into());
    }

    assert_mint_authority_matches_mint(&mint, mint_authority_info)?;
    assert_mint_authority_matches_mint(&master_mint, mint_authority_info)?;

    if metadata.mint != *mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    if master_mint.decimals != 0 {
        return Err(MetadataError::MasterMintDecimalsShouldBeZero.into());
    }

    if mint.decimals != 0 {
        return Err(MetadataError::EditionMintDecimalsShouldBeZero.into());
    }

    assert_update_authority_is_correct(&metadata, update_authority_info)?;

    if mint.supply != 1 {
        return Err(MetadataError::EditionsMustHaveExactlyOneToken.into());
    }

    let edition_authority_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &mint_info.key.as_ref(),
        EDITION.as_bytes(),
        &[bump_seed],
    ];

    create_or_allocate_account_raw(
        *program_id,
        edition_account_info,
        rent_info,
        system_account_info,
        payer_account_info,
        MAX_MASTER_EDITION_LEN,
        edition_authority_seeds,
    )?;

    let mut edition: MasterEdition = try_from_slice_unchecked(&edition_account_info.data.borrow())?;

    edition.key = Key::MasterEditionV1;
    edition.supply = 0;
    edition.max_supply = max_supply;
    edition.master_mint = *master_mint_info.key;
    edition.serialize(&mut *edition_account_info.data.borrow_mut())?;

    // While you can't mint any more of your master record, you can
    // mint as many limited editions as you like, and coins to permission others
    // to mint one of them in the future.
    transfer_mint_authority(
        edition_authority_seeds,
        &edition_key,
        edition_account_info,
        mint_info,
        mint_authority_info,
        token_program_info,
    )?;

    if let Some(supply) = max_supply {
        // We need to enact limited supply protocol.
        let auth_token_acct_info = next_account_info(account_info_iter)?;
        let master_mint_authority_info = next_account_info(account_info_iter)?;

        let auth_token_acct: Account = assert_initialized(auth_token_acct_info)?;
        if auth_token_acct.mint != *master_mint_info.key {
            return Err(MetadataError::MasterMintAuthorizationAccountMismatch.into());
        }
        assert_rent_exempt(rent, auth_token_acct_info)?;
        if auth_token_acct.owner != *update_authority_info.key {
            return Err(MetadataError::AuthorizationTokenAccountOwnerMismatch.into());
        }

        spl_token_mint_to(TokenMintToParams {
            mint: master_mint_info.clone(),
            destination: auth_token_acct_info.clone(),
            amount: supply,
            authority: master_mint_authority_info.clone(),
            authority_signer_seeds: &[],
            token_program: token_program_info.clone(),
        })?;

        transfer_mint_authority(
            edition_authority_seeds,
            &edition_key,
            edition_account_info,
            master_mint_info,
            master_mint_authority_info,
            token_program_info,
        )?;
    }
    Ok(())
}

pub fn process_mint_new_edition_from_master_edition_via_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let new_metadata_account_info = next_account_info(account_info_iter)?;
    let new_edition_account_info = next_account_info(account_info_iter)?;
    let master_edition_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let master_mint_info = next_account_info(account_info_iter)?;
    let master_token_account_info = next_account_info(account_info_iter)?;
    let burn_authority = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let master_metadata_account_info = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let token_account: Account = assert_initialized(master_token_account_info)?;
    let master_edition: MasterEdition =
        try_from_slice_unchecked(&master_edition_account_info.data.borrow())?;

    if master_edition.master_mint != *master_mint_info.key {
        return Err(MetadataError::MasterMintMismatch.into());
    }

    if token_account.mint != *master_mint_info.key {
        return Err(MetadataError::TokenAccountMintMismatch.into());
    }

    if token_account.amount < 1 {
        return Err(MetadataError::NotEnoughTokens.into());
    }

    spl_token_burn(TokenBurnParams {
        mint: master_mint_info.clone(),
        source: master_token_account_info.clone(),
        amount: 1,
        authority: burn_authority.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_account_info.clone(),
    })?;

    mint_limited_edition(
        program_id,
        new_metadata_account_info,
        new_edition_account_info,
        master_edition_account_info,
        mint_info,
        mint_authority_info,
        payer_account_info,
        update_authority_info,
        master_metadata_account_info,
        token_program_account_info,
        system_account_info,
        rent_info,
    )?;
    Ok(())
}
