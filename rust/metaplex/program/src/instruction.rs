use {
    crate::state::AuctionManagerSettings,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        sysvar,
    },
};
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct SetStoreArgs {
    pub public: bool,
}
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct SetWhitelistedCreatorArgs {
    pub activated: bool,
}

/// Instructions supported by the Fraction program.
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum MetaplexInstruction {
    /// Initializes an Auction Manager
    //
    ///   0. `[writable]` Uninitialized, unallocated auction manager account with pda of ['metaplex', auction_key from auction referenced below]
    ///   1. `[]` Combined vault account with authority set to auction manager account (this will be checked)
    ///           Note in addition that this vault account should have authority set to this program's pda of ['metaplex', auction_key]
    ///   2. `[]` Auction with auctioned item being set to the vault given and authority set to this program's pda of ['metaplex', auction_key]
    ///   3. `[]` Authority for the Auction Manager
    ///   4. `[signer]` Payer
    ///   5. `[]` Accept payment account of same token mint as the auction for taking payment for open editions, owner should be auction manager key
    ///   6. `[]` Store that this auction manager will belong to
    ///   7. `[]` System sysvar    
    ///   8. `[]` Rent sysvar
    InitAuctionManager(AuctionManagerSettings),

    /// Validates that a given safety deposit box has in it contents that match the expected WinningConfig in the auction manager.
    /// A stateful call, this will error out if you call it a second time after validation has occurred.
    ///   0. `[writable]` Auction manager
    ///   1. `[writable]` Metadata account
    ///   2. `[writable]` Original authority lookup - unallocated uninitialized pda account with seed ['metaplex', auction key, metadata key]
    ///                   We will store original authority here to return it later.
    ///   3. `[]` A whitelisted creator entry for the store of this auction manager pda of ['metaplex', store key, creator key]
    ///   where creator key comes from creator list of metadata, any will do
    ///   4. `[]` The auction manager's store key
    ///   5. `[]` Safety deposit box account
    ///   6. `[]` Safety deposit box storage account where the actual nft token is stored
    ///   7. `[]` Mint account of the token in the safety deposit box
    ///   8. `[]` Edition OR MasterEdition record key
    ///           Remember this does not need to be an existing account (may not be depending on token), just is a pda with seed
    ///            of ['metadata', program id, master mint id, 'edition']. - remember PDA is relative to token metadata program.
    ///   9. `[]` Vault account
    ///   10. `[signer]` Authority
    ///   11. `[signer]` Metadata Authority
    ///   12. `[signer]` Payer
    ///   13. `[]` Token metadata program
    ///   14. `[]` System
    ///   15. `[]` Rent sysvar
    ///   16. `[writable]` Limited edition Master Mint account (optional - only if using sending Limited Edition)
    ///   17. `[signer]` Limited edition Master Mint Authority account, this will TEMPORARILY TRANSFER MINTING AUTHORITY to the auction manager
    ///         until all limited editions have been redeemed for authority tokens.
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
    ///   1. `[writable]` Safety deposit token storage account
    ///   2. `[writable]` Destination account.
    ///   3. `[writable]` Bid redemption key -
    ///        Just a PDA with seed ['metaplex', auction_key, bidder_metadata_key] that we will allocate to mark that you redeemed your bid
    ///   4. `[writable]` Safety deposit box account
    ///   5. `[writable]` Vault account
    ///   6. `[writable]` Fraction mint of the vault
    ///   7. `[]` Auction
    ///   8. `[]` Your BidderMetadata account
    ///   9. `[signer optional]` Your Bidder account - Only needs to be signer if payer does not own
    ///   10. `[signer]` Payer
    ///   11. `[]` Token program
    ///   12. `[]` Token Vault program
    ///   13. `[]` Token metadata program
    ///   14. `[]` Store
    ///   15. `[]` System
    ///   16. `[]` Rent sysvar
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
    ///   1. `[writable]` Safety deposit token storage account
    ///   2. `[writable]` Destination account.
    ///   3. `[writable]` Bid redemption key -
    ///        Just a PDA with seed ['metaplex', auction_key, bidder_metadata_key] that we will allocate to mark that you redeemed your bid
    ///   4. `[writable]` Safety deposit box account
    ///   5. `[writable]` Vault account
    ///   6. `[writable]` Fraction mint of the vault
    ///   7. `[]` Auction
    ///   8. `[]` Your BidderMetadata account
    ///   9. `[signer optional]` Your Bidder account - Only needs to be signer if payer does not own
    ///   10. `[signer]` Payer
    ///   11. `[]` Token program
    ///   12. `[]` Token Vault program
    ///   13. `[]` Token metadata program
    ///   14. `[]` Store
    ///   15. `[]` System
    ///   16. `[]` Rent sysvar
    ///   17. `[writable]` Master Metadata account (pda of ['metadata', program id, master mint id]) - remember PDA is relative to token metadata program
    ///           (This account is optional, and will only be used if metadata is unique, otherwise this account key will be ignored no matter it's value)
    ///   18. `[]` New authority for Master Metadata - If you are taking ownership of a Master Edition in and of itself, or a Limited Edition that isn't newly minted for you during this auction
    ///             ie someone else had it minted for themselves in a prior auction or through some other means, this is the account the metadata for these tokens will be delegated to
    ///             after this transaction. Otherwise this account will be ignored.
    ///   19. `[]` PDA-based Transfer authority to move the tokens from the store to the destination seed ['vault', program_id]
    ///        but please note that this is a PDA relative to the Token Vault program, with the 'vault' prefix
    RedeemMasterEditionBid,

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
    ///   1. `[writable]` Safety deposit token storage account
    ///   2. `[writable]` Destination account for limited edition authority token. Must be same mint as master edition master mint.
    ///   3. `[writable]` Bid redemption key -
    ///        Just a PDA with seed ['metaplex', auction_key, bidder_metadata_key] that we will allocate to mark that you redeemed your bid
    ///   4. `[]` Safety deposit box account
    ///   5. `[]` Vault account
    ///   6. `[]` Fraction mint of the vault
    ///   7. `[]` Auction
    ///   8. `[]` Your BidderMetadata account
    ///   9. `[signer optional/writable]` Your Bidder account - Only needs to be signer if payer does not own
    ///   10. `[signer]` Payer
    ///   11. `[]` Token program
    ///   12. `[]` Token Vault program
    ///   13. `[]` Token metadata program
    ///   14. `[]` Store
    ///   15. `[]` System
    ///   16. `[]` Rent sysvar
    ///   17. `[]` Master Metadata (pda of ['metadata', program id, metadata mint id]) - remember PDA is relative to token metadata program
    ///   18. `[writable]` Master mint on the master edition - this is the mint used to produce one-time use tokens to give permission to make one limited edition.
    ///   19. `[writable]` Master Edition (pda of ['metadata', program id, metadata mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   20. `[signer]` Transfer authority to move the payment in the auction's token_mint coin from the bidder account for the open_edition_fixed_price
    ///             on the auction manager to the auction manager account itself.
    ///   21.  `[writable]` The accept payment account for the auction manager
    ///   22.  `[writable]` The token account you will potentially pay for the open edition bid with if necessary
    RedeemOpenEditionBid,

    /// If the auction manager is in Validated state, it can invoke the start command via calling this command here.
    ///
    ///   0. `[writable]` Auction manager
    ///   1. `[writable]` Auction
    ///   3. `[signer]` Auction manager authority
    ///   4. `[]` Store key
    ///   5. `[]` Auction program
    ///   6. `[]` Clock sysvar
    StartAuction,

    /// If the auction manager is in a Disbursing or Finished state, then this means Auction must be in Ended state.
    /// Then this end point can be used as a signed proxy to use auction manager's authority over the auction to claim bid funds
    /// into the accept payment account on the auction manager for a given bid. Auction has no opinions on how bids are redeemed,
    /// only that they exist, have been paid, and have a winning place. It is up to the implementer of the auction to determine redemption,
    /// and auction manager does this via bid redemption tickets and the vault contract which ensure the user always
    /// can get their NFT once they have paid. Therefore, once they have paid, and the auction is over, the artist can claim
    /// funds at any time without any danger to the user of losing out on their NFT, because the AM will honor their bid with an NFT
    /// at ANY time.
    ///
    ///   0. `[writable]` The accept payment account on the auction manager
    ///   1. `[writable]` The bidder pot token account
    ///   2. `[writable]` The bidder pot pda account [seed of ['auction', program_id, auction key, bidder key] -
    ///           relative to the auction program, not auction manager
    ///   3. `[]` The auction
    ///   4. `[]` The bidder wallet
    ///   5. `[]` Token mint of the auction
    ///   6. `[]` Vault
    ///   7. `[]` Auction manager
    ///   8. `[]` Store
    ///   9. `[]` Auction program
    ///   10. `[]` Clock sysvar
    ///   11. `[]` Token program
    ClaimBid,

    /// At any time, the auction manager authority may empty whatever funds are in the accept payment account
    /// on the auction manager. Funds come here from fixed price payments for open editions, and from draining bid payments
    /// from the auction.
    ///
    ///   0. `[writable]` The accept payment account on the auction manager
    ///   1. `[writable]` The destination account of same mint type as the accept payment account
    ///   2. `[signer]` Authority OF the auction manager - this is the account that can control the AM
    ///   3. `[]` Auction manager
    ///   4. `[]` Token program
    ///   5. `[]` Rent sysvar
    EmptyPaymentAccount,

    /// Given a signer wallet, create a store with pda ['metaplex', wallet] (if it does not exist) and/or update it
    /// (if it already exists). Stores can be set to open (anybody can publish) or closed (publish only via whitelist).
    ///
    ///   0. `[writable]` The store key, seed of ['metaplex', admin wallet]
    ///   1. `[signer]`  The admin wallet
    ///   2. `[signer]`  Payer
    ///   3. `[]` Token program
    ///   4. `[]` Token vault program
    ///   5. `[]` Token metadata program
    ///   6. `[]` Auction program
    ///   7. `[]` System
    ///   8. `[]` Rent sysvar
    SetStore(SetStoreArgs),

    /// Given an existing store, add or update an existing whitelisted creator for the store. This creates
    /// a PDA with seed ['metaplex', store key, creator key] if it does not already exist to store attributes there.
    ///
    ///   0. `[writable]` The whitelisted creator pda key, seed of ['metaplex', store key, creator key]
    ///   1. `[signer]`  The admin wallet
    ///   2. `[signer]`  Payer
    ///   3. `[]` The creator key
    ///   4. `[]` The store key, seed of ['metaplex', admin wallet]
    ///   5. `[]` System
    ///   6. `[]` Rent sysvar
    SetWhitelistedCreator(SetWhitelistedCreatorArgs),

    ///   Validates an open edition (if present) on the Auction Manager. Because of the differing mechanics of an open
    ///   edition, it needs to be validated at a different endpoint than a normal safety deposit box.
    ///   1. `[writable]` Auction manager
    ///   2. `[writable]` Open edition metadata
    ///   3. `[writable]` Open edition Mint account
    ///   4. `[writable]` Open edition Master Mint account (optional - only if your NFT has a master edition)
    ///   5. `[signer]` Open edition Master Mint Authority account, this will PERMANENTLY TRANSFER MINTING
    ///        AUTHORITY TO AUCTION MANAGER. You can still mint your own editions via your own personal authority however. (optional - only if your NFT has a master edition)
    ///   6. `[signer]` Open edition authority
    ///   7. `[signer]` Authority for the Auction Manager
    ///   8. `[]` Open edition MasterEdition account (optional - only if your NFT has one)
    ///   9. `[]` A whitelisted creator entry for this store for the open edition
    ///       pda of ['metaplex', store key, creator key] where creator key comes from creator list of metadata
    ///   10. `[]` The auction manager's store
    ///   11. `[]` Vault
    ///   12. `[]` Token program
    ///   13. `[]` Token metadata program
    ValidateOpenEdition,
}

/// Creates an InitAuctionManager instruction
#[allow(clippy::too_many_arguments)]
pub fn create_init_auction_manager_instruction(
    program_id: Pubkey,
    auction_manager: Pubkey,
    vault: Pubkey,
    auction: Pubkey,
    auction_manager_authority: Pubkey,
    payer: Pubkey,
    accept_payment_account_key: Pubkey,
    store: Pubkey,
    settings: AuctionManagerSettings,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(auction, false),
            AccountMeta::new_readonly(auction_manager_authority, false),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(accept_payment_account_key, false),
            AccountMeta::new_readonly(store, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetaplexInstruction::InitAuctionManager(settings)
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an ValidateOpenEdition instruction
#[allow(clippy::too_many_arguments)]
pub fn create_validate_open_edition_instruction(
    program_id: Pubkey,
    auction_manager: Pubkey,
    vault: Pubkey,
    open_edition_metadata: Pubkey,
    open_edition_authority: Pubkey,
    open_edition_mint: Pubkey,
    open_edition_master_edition: Pubkey,
    open_edition_master_mint: Pubkey,
    open_edition_master_mint_authority: Pubkey,
    whitelisted_creator: Pubkey,
    auction_manager_authority: Pubkey,
    store: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new(open_edition_metadata, false),
            AccountMeta::new(open_edition_mint, false),
            AccountMeta::new(open_edition_master_mint, false),
            AccountMeta::new_readonly(open_edition_master_mint_authority, true),
            AccountMeta::new_readonly(open_edition_authority, true),
            AccountMeta::new_readonly(auction_manager_authority, true),
            AccountMeta::new_readonly(open_edition_master_edition, false),
            AccountMeta::new_readonly(whitelisted_creator, false),
            AccountMeta::new_readonly(store, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token_metadata::id(), false),
        ],
        data: MetaplexInstruction::ValidateOpenEdition
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
    original_authority_lookup: Pubkey,
    whitelisted_creator: Pubkey,
    store: Pubkey,
    safety_deposit_box: Pubkey,
    safety_deposit_token_store: Pubkey,
    safety_deposit_mint: Pubkey,
    edition: Pubkey,
    vault: Pubkey,
    auction_manager_authority: Pubkey,
    metadata_authority: Pubkey,
    payer: Pubkey,
    master_mint: Option<Pubkey>,
    master_mint_authority: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(auction_manager, false),
        AccountMeta::new(metadata, false),
        AccountMeta::new(original_authority_lookup, false),
        AccountMeta::new_readonly(whitelisted_creator, false),
        AccountMeta::new_readonly(store, false),
        AccountMeta::new_readonly(safety_deposit_box, false),
        AccountMeta::new_readonly(safety_deposit_token_store, false),
        AccountMeta::new_readonly(safety_deposit_mint, false),
        AccountMeta::new_readonly(edition, false),
        AccountMeta::new_readonly(vault, false),
        AccountMeta::new_readonly(auction_manager_authority, true),
        AccountMeta::new_readonly(metadata_authority, true),
        AccountMeta::new_readonly(payer, true),
        AccountMeta::new_readonly(spl_token_metadata::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    if let Some(key) = master_mint {
        accounts.push(AccountMeta::new(key, false))
    }

    if let Some(key) = master_mint_authority {
        accounts.push(AccountMeta::new_readonly(key, true))
    }

    Instruction {
        program_id,
        accounts,
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
    safety_deposit_token_store: Pubkey,
    destination: Pubkey,
    bid_redemption: Pubkey,
    safety_deposit_box: Pubkey,
    vault: Pubkey,
    fraction_mint: Pubkey,
    auction: Pubkey,
    bidder_metadata: Pubkey,
    bidder: Pubkey,
    payer: Pubkey,
    store: Pubkey,
    transfer_authority: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new(safety_deposit_token_store, false),
            AccountMeta::new(destination, false),
            AccountMeta::new(bid_redemption, false),
            AccountMeta::new(safety_deposit_box, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(fraction_mint, false),
            AccountMeta::new_readonly(auction, false),
            AccountMeta::new_readonly(bidder_metadata, false),
            AccountMeta::new_readonly(bidder, true),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token_vault::id(), false),
            AccountMeta::new_readonly(spl_token_metadata::id(), false),
            AccountMeta::new_readonly(store, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
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
    safety_deposit_token_store: Pubkey,
    destination: Pubkey,
    bid_redemption: Pubkey,
    safety_deposit_box: Pubkey,
    vault: Pubkey,
    fraction_mint: Pubkey,
    auction: Pubkey,
    bidder_metadata: Pubkey,
    bidder: Pubkey,
    payer: Pubkey,
    store: Pubkey,
    master_metadata: Pubkey,
    new_metadata_authority: Pubkey,
    transfer_authority: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new(safety_deposit_token_store, false),
            AccountMeta::new(destination, false),
            AccountMeta::new(bid_redemption, false),
            AccountMeta::new(safety_deposit_box, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(fraction_mint, false),
            AccountMeta::new_readonly(auction, false),
            AccountMeta::new_readonly(bidder_metadata, false),
            AccountMeta::new_readonly(bidder, true),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token_vault::id(), false),
            AccountMeta::new_readonly(spl_token_metadata::id(), false),
            AccountMeta::new_readonly(store, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new(master_metadata, false),
            AccountMeta::new_readonly(new_metadata_authority, false),
            AccountMeta::new_readonly(transfer_authority, false),
        ],
        data: MetaplexInstruction::RedeemMasterEditionBid
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an RedeemOpenEditionBid instruction
#[allow(clippy::too_many_arguments)]
pub fn create_redeem_open_edition_bid_instruction(
    program_id: Pubkey,
    auction_manager: Pubkey,
    safety_deposit_token_store: Pubkey,
    destination: Pubkey,
    bid_redemption: Pubkey,
    safety_deposit_box: Pubkey,
    vault: Pubkey,
    fraction_mint: Pubkey,
    auction: Pubkey,
    bidder_metadata: Pubkey,
    bidder: Pubkey,
    payer: Pubkey,
    store: Pubkey,
    master_metadata: Pubkey,
    master_mint: Pubkey,
    master_edition: Pubkey,
    transfer_authority: Pubkey,
    accept_payment: Pubkey,
    paying_token_account: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new(safety_deposit_token_store, false),
            AccountMeta::new(destination, false),
            AccountMeta::new(bid_redemption, false),
            AccountMeta::new_readonly(safety_deposit_box, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(fraction_mint, false),
            AccountMeta::new_readonly(auction, false),
            AccountMeta::new_readonly(bidder_metadata, false),
            AccountMeta::new_readonly(bidder, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token_vault::id(), false),
            AccountMeta::new_readonly(spl_token_metadata::id(), false),
            AccountMeta::new_readonly(store, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(master_metadata, false),
            AccountMeta::new(master_mint, false),
            AccountMeta::new(master_edition, false),
            AccountMeta::new_readonly(transfer_authority, true),
            AccountMeta::new(accept_payment, false),
            AccountMeta::new(paying_token_account, false),
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
    store: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(auction_manager, false),
            AccountMeta::new(auction, false),
            AccountMeta::new_readonly(auction_manager_authority, true),
            AccountMeta::new_readonly(store, false),
            AccountMeta::new_readonly(spl_auction::id(), false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
        ],
        data: MetaplexInstruction::StartAuction.try_to_vec().unwrap(),
    }
}

/// Creates an SetStore instruction
pub fn create_set_store_instruction(
    program_id: Pubkey,
    store: Pubkey,
    admin: Pubkey,
    payer: Pubkey,
    public: bool,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(store, false),
        AccountMeta::new_readonly(admin, true),
        AccountMeta::new_readonly(payer, true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_token_vault::id(), false),
        AccountMeta::new_readonly(spl_token_metadata::id(), false),
        AccountMeta::new_readonly(spl_auction::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];
    Instruction {
        program_id,
        accounts,
        data: MetaplexInstruction::SetStore(SetStoreArgs { public })
            .try_to_vec()
            .unwrap(),
    }
}
