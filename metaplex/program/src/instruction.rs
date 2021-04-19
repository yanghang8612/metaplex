use {
    crate::state::AuctionManagerSettings,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        sysvar,
    },
};

/// Instructions supported by the Fraction program.
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum MetaplexInstruction {
    /// Initializes an Auction Manager
    ///   0. `[writable]` Uninitialized, unallocated auction manager account with pda of ['metaplex', auction_key from auction referenced below]
    ///   1. `[]` Activated vault account with authority set to auction manager account (this will be checked)
    ///           Note in addition that this vault account should have authority set to this program's pda of ['metaplex', auction_key]
    ///   2. `[]` Auction with auctioned item being set to the vault given and authority set to this program's pda of ['metaplex', auction_key]
    ///   3. `[]` External Pricing Account which must be owned by this program
    ///   4. `[]` Open edition MasterEdition account (optional - only if using this feature)
    ///   5. `[]` Open edition Mint account (optional - only if using this feature)
    ///   6. `[]` Authority for the Auction Manager
    ///   7. `[signer]` Payer
    ///   8. `[]` Token program
    ///   9. `[]` Token vault program
    ///   10. `[]` Token metadata program
    ///   11. `[]` Auction program
    ///   12. `[]` System sysvar
    ///   13. `[]` Rent sysvar
    InitAuctionManager(AuctionManagerSettings),

    /// Validates that a given safety deposit box has in it contents that match the expected WinningConfig in the auction manager.
    /// A stateful call, this will error out if you call it a second time after validation has occurred.
    ///   0. `[writable]` Auction manager
    ///   1. `[writable]` Metadata account
    ///   2. `[writable]` Name symbol tuple account
    ///           (This account is optional, and will only be used if metadata is unique, otherwise this account key will be ignored no matter it's value)
    ///   3. `[writable]` Original authority lookup - unallocated uninitialized pda account with seed ['metaplex', auction key, metadata key]
    ///                   We will store original authority here to return it later.
    ///   4. `[]` Safety deposit box account
    ///   5. `[]` Store account of safety deposit box
    ///   6. `[]` Mint account of the token in the safety deposit box
    ///   7. `[]` Edition OR MasterEdition record key
    ///           Remember this does not need to be an existing account (may not be depending on token), just is a pda with seed
    ///            of ['metadata', program id, master mint id, 'edition']. - remember PDA is relative to token metadata program.
    ///   8. `[]` Vault account
    ///   9. `[signer]` Authority
    ///   10. `[signer]` Metadata Authority
    ///   11. `[signer]` Payer
    ///   12. `[]` Token metadata program
    ///   13. `[]` System
    ///   14. `[]` Rent sysvar
    ValidateSafetyDepositBox,

    /// Note: This requires that auction manager be in a Running state.
    ///
    /// If an auction is complete, you can redeem your bid for a specific item here. If you are the first to do this,
    /// The auction manager will switch from Running state to Disbursing state. If you are the last, this may change
    /// the auction manager state to Finished provided that no authorities remain to be delegated for Master Edition tokens.
    ///
    /// NOTE: Please note that it is totally possible to redeem a bid 2x - once for a prize you won and once at the RedeemOpenEditionBid point for an open edition
    /// that comes as a 'token of appreciation' for bidding. They are not mutually exclusive unless explicitly set to be that way.
    ///
    ///   0. `[writable]` Auction manager
    ///   1. `[writable]` Store of safety deposit box account
    ///   2. `[writable]` Destination account.
    ///   3. `[writable]` Bid redemption key -
    ///        Just a PDA with seed ['metaplex', auction_key, bidder_metadata_key] that we will allocate to mark that you redeemed your bid
    ///   4. `[]` Safety deposit box account
    ///   5. `[]` Fraction mint of the vault
    ///   6. `[]` Vault account
    ///   7. `[]` Auction
    ///   8. `[]` Your BidderMetadata account
    ///   9. `[signer]` Your Bidder account
    ///   10. `[signer]` Payer
    ///   11. `[]` Token program
    ///   12. `[]` Token Vault program
    ///   13. `[]` Token metadata program
    ///   14. `[]` System
    ///   15. `[]` Rent sysvar
    ///   16. `[]` Clock sysvar.
    ///   17. `[]` PDA-based Transfer authority to move the tokens from the store to the destination seed ['vault', program_id]
    ///        but please note that this is a PDA relative to the Token Vault program, with the 'vault' prefix
    RedeemBid,

    /// Note: This requires that auction manager be in a Running state.
    ///
    /// If an auction is complete, you can redeem your bid for the actual Master Edition itself if it's for that prize here.
    /// If you are the first to do this, the auction manager will switch from Running state to Disbursing state.
    /// If you are the last, this may change the auction manager state to Finished provided that no authorities remain to be delegated for Master Edition tokens.
    ///
    /// NOTE: Please note that it is totally possible to redeem a bid 2x - once for a prize you won and once at the RedeemOpenEditionBid point for an open edition
    /// that comes as a 'token of appreciation' for bidding. They are not mutually exclusive unless explicitly set to be that way.
    ///
    ///   0. `[writable]` Auction manager
    ///   1. `[writable]` Store of safety deposit box account
    ///   2. `[writable]` Destination account.
    ///   3. `[writable]` Bid redemption key -
    ///        Just a PDA with seed ['metaplex', auction_key, bidder_metadata_key] that we will allocate to mark that you redeemed your bid
    ///   4. `[]` Safety deposit box account
    ///   5. `[]` Fraction mint of the vault
    ///   6. `[]` Vault account
    ///   7. `[]` Auction
    ///   8. `[]` Your BidderMetadata account
    ///   9. `[signer]` Your Bidder account
    ///   10. `[signer]` Payer
    ///   11. `[]` Token program
    ///   12. `[]` Token Vault program
    ///   13. `[]` Token metadata program
    ///   14. `[]` System
    ///   15. `[]` Rent sysvar
    ///   16. `[]` Clock sysvar.
    ///   17. `[writable]` Master Metadata account (pda of ['metadata', program id, master mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   18. `[writable]` Master Name symbol tuple account
    ///           (This account is optional, and will only be used if metadata is unique, otherwise this account key will be ignored no matter it's value)
    ///   19. `[]` New authority for Master Metadata - If you are taking ownership of a Master Edition in and of itself, or a Limited Edition that isn't newly minted for you during this auction
    ///             ie someone else had it minted for themselves in a prior auction or through some other means, this is the account the metadata for these tokens will be delegated to
    ///             after this transaction. Otherwise this account will be ignored.
    ///   20. `[]` PDA-based Transfer authority to move the tokens from the store to the destination seed ['vault', program_id]
    ///        but please note that this is a PDA relative to the Token Vault program, with the 'vault' prefix
    RedeemMasterEditionBid,

    /// Note: This requires that auction manager be in a Running state.
    ///
    /// If an auction is complete, you can redeem your bid for a Limited Edition here if the bid is valid for that type. If you are the first to do this,
    /// The auction manager will switch from Running state to Disbursing state. If you are the last, this may change
    /// the auction manager state to Finished provided that no authorities remain to be delegated for Master Edition tokens.
    ///
    /// NOTE: Please note that it is totally possible to redeem a bid 2x - once for a prize you won and once at the RedeemOpenEditionBid point for an open edition
    /// that comes as a 'token of appreciation' for bidding. They are not mutually exclusive unless explicitly set to be that way.
    ///
    /// NOTE: Since you are receiving a newly minted Limited Edition, you must actually supply a destination account containing a token from a brand new
    /// mint. We do not provide the token to you. Our job with this action is to christen this mint + token combo as an official Limited Edition.
    /// If you won "multiple" Limited Editions with a single bid, you will need to call this endpoint N times, for N Limited Editions, each with a different destination/mint combo.
    /// Please realize this scenario is DIFFERENT FROM redeeming a Limited Edition that someone put up for auction that they themselves got somewhere else, like a previous auction.
    /// In that case, you only need to provide a destination account of the same mint type of the Limited Edition mint, and it's just as if
    /// it were any other token, so you would be using the RedeemBid endpoint, not this one.
    ///
    ///   0. `[writable]` Auction manager
    ///   1. `[writable]` Store of safety deposit box account
    ///   2. `[writable]` Destination account.
    ///   3. `[writable]` Bid redemption key -
    ///        Just a PDA with seed ['metaplex', auction_key, bidder_metadata_key] that we will allocate to mark that you redeemed your bid
    ///   4. `[]` Safety deposit box account
    ///   5. `[]` Fraction mint of the vault
    ///   6. `[]` Vault account
    ///   7. `[]` Auction
    ///   8. `[]` Your BidderMetadata account
    ///   9. `[signer]` Your Bidder account
    ///   10. `[signer]` Payer
    ///   11. `[]` Token program
    ///   12. `[]` Token Vault program
    ///   13. `[]` Token metadata program
    ///   14. `[]` System
    ///   15. `[]` Rent sysvar
    ///   16. `[]` Clock sysvar.
    ///   17. `[writable]` New Limited Edition Metadata (pda of ['metadata', program id, newly made mint id]) - remember PDA is relative to token metadata program
    ///   18. `[writable]` Mint of destination account. This needs to be a newly created mint and the destination account
    ///                   needs to have exactly one token in it already. We will simply "grant" the limited edition status on this token.
    ///   19. `[signer]` Destination mint authority - this account is optional, and will only be used/checked if you are receiving a newly minted limited edition.
    ///   20. `[]` Master Metadata (pda of ['metadata', program id, master mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   21. `[]` Master Name-Symbol (pda of ['metadata', program id, name, symbol']) - remember PDA is relative to token metadata program
    ///   22. `[]` New Limited Edition (pda of ['metadata', program id, newly made mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   23. `[]` Master Edition (pda of ['metadata', program id, master mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   24. `[]` Original authority on the Master Metadata, which can be gotten via reading off the key from lookup of OriginalAuthorityLookup struct with
    ///            key of (pda of ['metaplex', auction key, master metadata key]).
    ///            We'll use this to grant back authority to the owner of the master metadata if we no longer need it after this latest minting.
    ///   25. `[]` Original authority Lookup key - pda of ['metaplex', auction key, master metadata key]
    RedeemLimitedEditionBid,

    /// Note: This requires that auction manager be in a Running state.
    ///
    /// If an auction is complete, you can redeem your bid for an Open Edition token if it is eligible. If you are the first to do this,
    /// The auction manager will switch from Running state to Disbursing state. If you are the last, this may change
    /// the auction manager state to Finished provided that no authorities remain to be delegated for Master Edition tokens.
    ///
    /// NOTE: Please note that it is totally possible to redeem a bid 2x - once for a prize you won and once at this end point for a open edition
    /// that comes as a 'token of appreciation' for bidding. They are not mutually exclusive unless explicitly set to be that way.
    ///
    /// NOTE: If you are redeeming a newly minted Open Edition, you must actually supply a destination account containing a token from a brand new
    /// mint. We do not provide the token to you. Our job with this action is to christen this mint + token combo as an official Open Edition.
    ///
    ///   0. `[writable]` Auction manager
    ///   1. `[writable]` Store of safety deposit box account
    ///   2. `[writable]` Destination account.
    ///   3. `[writable]` Bid redemption key -
    ///        Just a PDA with seed ['metaplex', auction_key, bidder_metadata_key] that we will allocate to mark that you redeemed your bid
    ///   4. `[]` Safety deposit box account
    ///   5. `[]` Fraction mint of the vault
    ///   6. `[]` Vault account
    ///   7. `[]` Auction
    ///   8. `[]` Your BidderMetadata account
    ///   9. `[signer]` Payer
    ///   10. `[]` Token program
    ///   11. `[]` Token Vault program
    ///   12. `[]` Token metadata program
    ///   13. `[]` System
    ///   14. `[]` Rent sysvar
    ///   15. `[]` Clock sysvar.
    ///   16. `[writable]` New Open Edition Metadata (pda of ['metadata', program id, newly made mint id]) - remember PDA is relative to token metadata program
    ///   17. `[writable]` Mint of destination account. This needs to be a newly created mint and the destination account
    ///                   needs to have exactly one token in it already. We will simply "grant" the open edition status on this token.
    ///   18. `[signer]` Destination mint authority - this account is optional, and will only be used/checked if you are receiving a newly minted open edition.
    ///   19. `[]` Master Metadata (pda of ['metadata', program id, master mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   20. `[]` New Open Edition (pda of ['metadata', program id, newly made mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   21. `[]` Master Edition (pda of ['metadata', program id, master mint id, 'edition']) - remember PDA is relative to token metadata program
    RedeemOpenEditionBid,

    /// If the auction manager is in Validated state, it can invoke the start command via calling this command here.
    ///
    ///   0. `[writable]` Auction manager
    ///   1. `[writable]` Auction
    ///   2. `[signer]` Auction manager authority
    ///   3. `[]` Auction program
    StartAuction,
}

/// Creates an InitAuctionManager instruction
#[allow(clippy::too_many_arguments)]
pub fn create_init_auction_manager_instruction(
    program_id: Pubkey,
    auction_manager: Pubkey,
    vault: Pubkey,
    auction: Pubkey,
    external_pricing_account: Pubkey,
    open_edition_master_edition: Pubkey,
    open_edition_mint: Pubkey,
    auction_manager_authority: Pubkey,
    payer: Pubkey,
    token_vault_program: Pubkey,
    auction_program: Pubkey,
    settings: AuctionManagerSettings,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(auction, false),
            AccountMeta::new_readonly(external_pricing_account, false),
            AccountMeta::new_readonly(open_edition_master_edition, false),
            AccountMeta::new_readonly(open_edition_mint, false),
            AccountMeta::new_readonly(auction_manager_authority, false),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(token_vault_program, false),
            AccountMeta::new_readonly(spl_token_metadata::id(), false),
            AccountMeta::new_readonly(auction_program, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetaplexInstruction::InitAuctionManager(settings)
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an ValidateSafetyDepositBox instruction
#[allow(clippy::too_many_arguments)]
pub fn create_validate_safety_deposit_box_instruction(
    program_id: Pubkey,
    auction_manager: Pubkey,
    metadata: Pubkey,
    name_symbol: Pubkey,
    original_authority_lookup: Pubkey,
    safety_deposit_box: Pubkey,
    store: Pubkey,
    safety_deposit_mint: Pubkey,
    edition: Pubkey,
    vault: Pubkey,
    auction_manager_authority: Pubkey,
    metadata_authority: Pubkey,
    payer: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new(metadata, false),
            AccountMeta::new(name_symbol, false),
            AccountMeta::new(original_authority_lookup, false),
            AccountMeta::new_readonly(safety_deposit_box, false),
            AccountMeta::new_readonly(store, false),
            AccountMeta::new_readonly(safety_deposit_mint, false),
            AccountMeta::new_readonly(edition, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(auction_manager_authority, true),
            AccountMeta::new_readonly(metadata_authority, true),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(spl_token_metadata::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetaplexInstruction::ValidateSafetyDepositBox
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an RedeemBid instruction
#[allow(clippy::too_many_arguments)]
pub fn create_redeem_bid_instruction(
    program_id: Pubkey,
    auction_manager: Pubkey,
    store: Pubkey,
    destination: Pubkey,
    bid_redemption: Pubkey,
    safety_deposit_box: Pubkey,
    fraction_mint: Pubkey,
    vault: Pubkey,
    auction: Pubkey,
    bidder_metadata: Pubkey,
    bidder: Pubkey,
    payer: Pubkey,
    token_vault_program: Pubkey,
    transfer_authority: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new(store, false),
            AccountMeta::new(destination, false),
            AccountMeta::new(bid_redemption, false),
            AccountMeta::new_readonly(safety_deposit_box, false),
            AccountMeta::new_readonly(fraction_mint, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(auction, false),
            AccountMeta::new_readonly(bidder_metadata, false),
            AccountMeta::new_readonly(bidder, true),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(token_vault_program, false),
            AccountMeta::new_readonly(spl_token_metadata::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new_readonly(transfer_authority, false),
        ],
        data: MetaplexInstruction::RedeemBid.try_to_vec().unwrap(),
    }
}

/// Creates an RedeemMasterEditionBid instruction
#[allow(clippy::too_many_arguments)]
pub fn create_redeem_master_edition_bid_instruction(
    program_id: Pubkey,
    auction_manager: Pubkey,
    store: Pubkey,
    destination: Pubkey,
    bid_redemption: Pubkey,
    safety_deposit_box: Pubkey,
    fraction_mint: Pubkey,
    vault: Pubkey,
    auction: Pubkey,
    bidder_metadata: Pubkey,
    bidder: Pubkey,
    payer: Pubkey,
    token_vault_program: Pubkey,
    master_metadata: Pubkey,
    master_name_symbol: Pubkey,
    new_metadata_authority: Pubkey,
    transfer_authority: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new(store, false),
            AccountMeta::new(destination, false),
            AccountMeta::new(bid_redemption, false),
            AccountMeta::new_readonly(safety_deposit_box, false),
            AccountMeta::new_readonly(fraction_mint, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(auction, false),
            AccountMeta::new_readonly(bidder_metadata, false),
            AccountMeta::new_readonly(bidder, true),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(token_vault_program, false),
            AccountMeta::new_readonly(spl_token_metadata::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new(master_metadata, false),
            AccountMeta::new(master_name_symbol, false),
            AccountMeta::new_readonly(new_metadata_authority, false),
            AccountMeta::new_readonly(transfer_authority, false),
        ],
        data: MetaplexInstruction::RedeemMasterEditionBid
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an RedeemLimitedEditionBid instruction
#[allow(clippy::too_many_arguments)]
pub fn create_redeem_limited_edition_bid_instruction(
    program_id: Pubkey,
    auction_manager: Pubkey,
    store: Pubkey,
    destination: Pubkey,
    bid_redemption: Pubkey,
    safety_deposit_box: Pubkey,
    fraction_mint: Pubkey,
    vault: Pubkey,
    auction: Pubkey,
    bidder_metadata: Pubkey,
    bidder: Pubkey,
    payer: Pubkey,
    token_vault_program: Pubkey,
    new_limited_edition_metadata: Pubkey,
    destination_mint: Pubkey,
    destination_mint_authority: Pubkey,
    master_metadata: Pubkey,
    master_name_symbol: Pubkey,
    new_limited_edition: Pubkey,
    master_edition: Pubkey,
    original_authority: Pubkey,
    original_authority_lookup: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new(store, false),
            AccountMeta::new(destination, false),
            AccountMeta::new(bid_redemption, false),
            AccountMeta::new_readonly(safety_deposit_box, false),
            AccountMeta::new_readonly(fraction_mint, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(auction, false),
            AccountMeta::new_readonly(bidder_metadata, false),
            AccountMeta::new_readonly(bidder, true),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(token_vault_program, false),
            AccountMeta::new_readonly(spl_token_metadata::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new(new_limited_edition_metadata, false),
            AccountMeta::new(destination_mint, false),
            AccountMeta::new_readonly(destination_mint_authority, true),
            AccountMeta::new_readonly(master_metadata, false),
            AccountMeta::new_readonly(master_name_symbol, false),
            AccountMeta::new_readonly(new_limited_edition, false),
            AccountMeta::new_readonly(master_edition, false),
            AccountMeta::new_readonly(original_authority, false),
            AccountMeta::new_readonly(original_authority_lookup, false),
        ],
        data: MetaplexInstruction::RedeemLimitedEditionBid
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an RedeemOpenEditionBid instruction
#[allow(clippy::too_many_arguments)]
pub fn create_redeem_open_edition_bid_instruction(
    program_id: Pubkey,
    auction_manager: Pubkey,
    store: Pubkey,
    destination: Pubkey,
    bid_redemption: Pubkey,
    safety_deposit_box: Pubkey,
    fraction_mint: Pubkey,
    vault: Pubkey,
    auction: Pubkey,
    bidder_metadata: Pubkey,
    bidder: Pubkey,
    payer: Pubkey,
    token_vault_program: Pubkey,
    new_open_edition_metadata: Pubkey,
    destination_mint: Pubkey,
    destination_mint_authority: Pubkey,
    master_metadata: Pubkey,
    new_open_edition: Pubkey,
    master_edition: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new(store, false),
            AccountMeta::new(destination, false),
            AccountMeta::new(bid_redemption, false),
            AccountMeta::new_readonly(safety_deposit_box, false),
            AccountMeta::new_readonly(fraction_mint, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(auction, false),
            AccountMeta::new_readonly(bidder_metadata, false),
            AccountMeta::new_readonly(bidder, true),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(token_vault_program, false),
            AccountMeta::new_readonly(spl_token_metadata::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new(new_open_edition_metadata, false),
            AccountMeta::new(destination_mint, false),
            AccountMeta::new_readonly(destination_mint_authority, true),
            AccountMeta::new_readonly(master_metadata, false),
            AccountMeta::new_readonly(new_open_edition, false),
            AccountMeta::new_readonly(master_edition, false),
        ],
        data: MetaplexInstruction::RedeemOpenEditionBid
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an StartAuction instruction
#[allow(clippy::too_many_arguments)]
pub fn create_start_auction_instruction(
    program_id: Pubkey,
    auction_manager: Pubkey,
    auction: Pubkey,
    auction_manager_authority: Pubkey,
    auction_program: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new(auction, false),
            AccountMeta::new_readonly(auction_manager_authority, true),
            AccountMeta::new_readonly(auction_program, false),
        ],
        data: MetaplexInstruction::StartAuction.try_to_vec().unwrap(),
    }
}
