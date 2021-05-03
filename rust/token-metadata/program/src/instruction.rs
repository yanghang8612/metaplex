use {
    crate::state::Data,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        sysvar,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
/// Args for update call
pub struct UpdateMetadataAccountArgs {
    pub uri: String,
    // Ignored when NameSymbolTuple present
    pub non_unique_specific_update_authority: Option<Pubkey>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
/// Args for create call
pub struct CreateMetadataAccountArgs {
    /// Note that unique metadatas are disabled for now.
    pub allow_duplication: bool,
    pub data: Data,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CreateMasterEditionArgs {
    /// If set, means that no more than this number of editions can ever be minted. This is immutable.
    pub max_supply: Option<u64>,
}

/// Instructions supported by the Metadata program.
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum MetadataInstruction {
    /// Create NameSymbolTuple (optional) and  Metadata objects.
    ///   0. `[writable]`  NameSymbolTuple key (pda of ['metadata', program id, name, symbol])
    ///   1. `[writable]`  Metadata key (pda of ['metadata', program id, mint id])
    ///   2. `[]` Mint of token asset
    ///   3. `[signer]` Mint authority
    ///   4. `[signer]` payer
    ///   5. `[signer]` update authority info (Signer is optional - only required if NameSymbolTuple exists)
    ///   6. `[]` System program
    ///   7. `[]` Rent info
    CreateMetadataAccounts(CreateMetadataAccountArgs),

    /// Update an  Metadata (name/symbol are unchangeable)
    ///   0. `[writable]` Metadata account
    ///   1. `[signer]` Update authority key
    ///   2. `[]`  NameSymbolTuple account key (pda of ['metadata', program id, name, symbol])
    ///            (does not need to exist if Metadata is of the duplicatable type)
    UpdateMetadataAccounts(UpdateMetadataAccountArgs),

    /// Transfer Update Authority
    ///   0. `[writable]`  NameSymbolTuple account or Metadata account (if duplicatable)
    ///   1. `[signer]` Current Update authority key
    ///   2. `[]`  New Update authority account key
    TransferUpdateAuthority,

    /// Register a Metadata as a Master Edition, which means Editions can be minted.
    /// Henceforth, no further tokens will be mintable from this primary mint. Will throw an error if more than one
    /// token exists, and will throw an error if less than one token exists in this primary mint.
    ///   0. `[writable]` Unallocated edition account with address as pda of ['metadata', program id, mint, 'edition']
    ///   1. `[writable]` Metadata mint
    ///   2. `[writable]` Master mint - A mint you control that can mint tokens that can be exchanged for limited editions of your
    ///       master edition via the MintNewEditionFromMasterEditionViaToken endpoint, like a one time authority.
    ///   3. `[signer]` Current Update authority key on metadata
    ///   4. `[signer]` Mint authority on the metadata's mint - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
    ///   5. `[]` Metadata account
    ///   6. `[]` Name symbol account (optional), will be used if update authority on metadata is None
    ///   7. `[signer]` payer
    ///   8. `[]` Token program
    ///   9. `[]` System program
    ///   10. `[]` Rent info
    ///   11. `[writable]` Optional Fixed supply master mint authorization token account - if using max supply, must provide this.
    ///                    All tokens ever in existence will be dumped here in one go, you must own this account, and you will be unable
    ///                    to mint new authorization tokens going forward.
    ///   12. `[signer]`   Master mint authority - must be provided if using max supply. THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY.
    CreateMasterEdition(CreateMasterEditionArgs),

    /// Given a master edition, mint a new edition from it, if max_supply not already maxed out. Update authority set to update authority of original.
    /// If you want to move it, transfer it yourself. Note that Edition coins cannot be unique, by definition, since they have same name/symbols.
    ///   0. `[writable]` New Metadata key (pda of ['metadata', program id, mint id])
    ///   1. `[writable]` New Edition (pda of ['metadata', program id, mint id, 'edition'])
    ///   2. `[writable]` Master Record Edition (pda of ['metadata', program id, master mint id, 'edition'])
    ///   3. `[writable]` Mint of new token - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
    ///   4. `[signer]` Mint authority
    ///   5. `[signer]` payer
    ///   6. `[signer]` update authority info of master metadata account
    ///   7. `[]` Master record metadata account
    ///   8. `[]` Token program
    ///   9. `[]` System program
    ///   10. `[]` Rent info
    MintNewEditionFromMasterEdition,

    /// Given a master edition, mint a new edition from it, if max_supply not already maxed out. Update authority set to update authority of original.
    /// If you want to move it, transfer it yourself. Note that Edition coins cannot be unique, by definition, since they have same name/symbols.
    ///   0. `[writable]` New Metadata key (pda of ['metadata', program id, mint id])
    ///   1. `[writable]` New Edition (pda of ['metadata', program id, mint id, 'edition'])
    ///   2. `[writable]` Master Record Edition (pda of ['metadata', program id, master mint id, 'edition'])
    ///   3. `[writable]` Mint of new token - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
    ///   4. `[signer]` Mint authority of new mint
    ///   5. `[writable]` Master Mint of master record edition
    ///   6. `[writable]` Token account containing master mint token to be transferred
    ///   7. `[signer]` Burn authority for this token
    ///   8. `[signer]` payer
    ///   9. `[]` update authority info of master metadata account
    ///   10. `[]` Master record metadata account
    ///   11. `[]` Token program
    ///   12. `[]` System program
    ///   13. `[]` Rent info
    MintNewEditionFromMasterEditionViaToken,
}

/// Creates an CreateMetadataAccounts instruction
#[allow(clippy::too_many_arguments)]
pub fn create_metadata_accounts(
    program_id: Pubkey,
    name_symbol_account: Pubkey,
    metadata_account: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    payer: Pubkey,
    update_authority: Pubkey,
    name: String,
    symbol: String,
    uri: String,
    allow_duplication: bool,
    update_authority_is_signer: bool,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(name_symbol_account, false),
            AccountMeta::new(metadata_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(update_authority, update_authority_is_signer),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetadataInstruction::CreateMetadataAccounts(CreateMetadataAccountArgs {
            data: Data { name, symbol, uri },
            allow_duplication,
        })
        .try_to_vec()
        .unwrap(),
    }
}

/// update metadata account instruction
pub fn update_metadata_accounts(
    program_id: Pubkey,
    metadata_account: Pubkey,
    name_symbol_account: Pubkey,
    update_authority: Pubkey,
    non_unique_specific_update_authority: Option<Pubkey>,
    uri: String,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata_account, false),
            AccountMeta::new_readonly(update_authority, true),
            AccountMeta::new_readonly(name_symbol_account, false),
        ],
        data: MetadataInstruction::UpdateMetadataAccounts(UpdateMetadataAccountArgs {
            uri,
            non_unique_specific_update_authority,
        })
        .try_to_vec()
        .unwrap(),
    }
}

/// transfer update authority instruction
pub fn transfer_update_authority(
    program_id: Pubkey,
    object: Pubkey,
    update_authority: Pubkey,
    new_update_authority: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(object, false),
            AccountMeta::new_readonly(update_authority, true),
            AccountMeta::new_readonly(new_update_authority, false),
        ],
        data: MetadataInstruction::TransferUpdateAuthority
            .try_to_vec()
            .unwrap(),
    }
}

/// creates a create_master_edition instruction
#[allow(clippy::too_many_arguments)]
pub fn create_master_edition(
    program_id: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
    master_mint: Pubkey,
    update_authority: Pubkey,
    mint_authority: Pubkey,
    metadata: Pubkey,
    name_symbol_account: Pubkey,
    payer: Pubkey,
    max_supply: Option<u64>,
    auth_holding_account: Option<Pubkey>,
    master_mint_authority: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(edition, false),
        AccountMeta::new(mint, false),
        AccountMeta::new(master_mint, false),
        AccountMeta::new_readonly(update_authority, true),
        AccountMeta::new_readonly(mint_authority, true),
        AccountMeta::new_readonly(metadata, false),
        AccountMeta::new_readonly(name_symbol_account, false),
        AccountMeta::new_readonly(payer, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    if let Some(acct) = auth_holding_account {
        accounts.push(AccountMeta::new(acct, false));
    }

    if let Some(auth) = master_mint_authority {
        accounts.push(AccountMeta::new_readonly(auth, true));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::CreateMasterEdition(CreateMasterEditionArgs { max_supply })
            .try_to_vec()
            .unwrap(),
    }
}

/// creates a mint_new_edition_from_master_edition instruction
#[allow(clippy::too_many_arguments)]
pub fn mint_new_edition_from_master_edition(
    program_id: Pubkey,
    metadata: Pubkey,
    edition: Pubkey,
    master_edition: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    payer: Pubkey,
    master_update_authority: Pubkey,
    master_metadata: Pubkey,
) -> Instruction {
    Instruction {
        program_id,

        accounts: vec![
            AccountMeta::new(metadata, false),
            AccountMeta::new(edition, false),
            AccountMeta::new(master_edition, false),
            AccountMeta::new(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(master_update_authority, true),
            AccountMeta::new_readonly(master_metadata, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetadataInstruction::MintNewEditionFromMasterEdition
            .try_to_vec()
            .unwrap(),
    }
}

/// creates a mint_new_edition_from_master_edition instruction
#[allow(clippy::too_many_arguments)]
pub fn mint_new_edition_from_master_edition_via_token(
    program_id: Pubkey,
    metadata: Pubkey,
    edition: Pubkey,
    master_edition: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    master_mint: Pubkey,
    master_token_account: Pubkey,
    burn_authority: Pubkey,
    payer: Pubkey,
    master_update_authority: Pubkey,
    master_metadata: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata, false),
            AccountMeta::new(edition, false),
            AccountMeta::new(master_edition, false),
            AccountMeta::new(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new(master_mint, false),
            AccountMeta::new(master_token_account, false),
            AccountMeta::new_readonly(burn_authority, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(master_update_authority, true),
            AccountMeta::new_readonly(master_metadata, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetadataInstruction::MintNewEditionFromMasterEditionViaToken
            .try_to_vec()
            .unwrap(),
    }
}
