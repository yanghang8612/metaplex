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
    /// Multiple cases are covered here:
    ///    1. You are redeeming a bid for a normal token
    ///    2. You are looking to mint a Limited Edition from a Master Edition template
    ///    3. You are looking to gain ownership of a Master Edition itself so you control the Master Edition going forward
    ///    4. Not really a fourth case, but you may be receiving an open edition in all cases, and in cancelled bid case.
    ///
    /// NOTE: If you are redeeming a newly minted Limited Edition, you must actually supply a destination account containing a token from a brand new
    /// mint. We do not provide the token to you. Our job with this action is to christen this mint + token combo as an official Limited Edition.
    /// If you won "multiple" Limited Editions with a single bid, you will need to call this endpoint N times, for N Limited Editions, each with a different destination/mint combo.
    /// Please realize this scenario is DIFFERENT FROM redeeming a Limited Edition that someone put up for auction that they themselves got somewhere else, like a previous auction.
    /// In that case, you only need to provide a destination account of the same mint type of the Limited Edition mint, and it's just as if
    /// it were any other token (case #1.)
    ///
    /// Depending on which case you are calling for, you will append a different set of accounts to the end of the account list, as each case requires
    /// different information, and authorities, to complete it's work.* Please see a footnote at the bottom about Open editions.
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
    ///   13. `[]` Rent sysvar
    ///   14. `[]` System
    ///   15. `[]` Clock sysvar.
    ///   
    ///   Case 1: Redeeming bid for normal token:
    ///
    ///   16. `[]` PDA-based Transfer authority to move the tokens from the store to the destination seed ['vault', program_id]
    ///        but please note that this is a PDA relative to the Token Vault program, with the 'vault' prefix
    ///
    ///   Case 2: Redeeming bid for Limited Edition:
    ///
    ///   16. `[writable]` New Limited Edition Metadata (pda of ['metadata', program id, newly made mint id]) - remember PDA is relative to token metadata program
    ///   17. `[writable]` Mint of destination account. This needs to be a newly created mint and the destination account
    ///                   needs to have exactly one token in it already. We will simply "grant" the limited edition status on this token.
    ///   18. `[signer]` Destination mint authority - this account is optional, and will only be used/checked if you are receiving a newly minted limited edition.
    ///   19. `[]` Master Metadata (pda of ['metadata', program id, master mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   20. `[]` Master Name-Symbol (pda of ['metadata', program id, name, symbol']) - remember PDA is relative to token metadata program
    ///   21. `[]` New Limited Edition (pda of ['metadata', program id, newly made mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   22. `[]` Master Edition (pda of ['metadata', program id, master mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   23. `[]` Original authority on the Master Metadata, which can be gotten via reading off the key from lookup of OriginalAuthorityLookup struct with
    ///            key of (pda of ['metaplex', auction key, master metadata key]).
    ///            We'll use this to grant back authority to the owner of the master metadata if we no longer need it after this latest minting.
    ///   24. `[]` Original authority Lookup key - pda of ['metaplex', auction key, master metadata key]
    ///
    ///   Case 3: Redeeming a bid to gain ownership of a Master Edition itself:
    ///
    ///   16. `[writable]` Master Metadata account (pda of ['metadata', program id, master mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   17. `[writable]` Name symbol tuple account
    ///           (This account is optional, and will only be used if metadata is unique, otherwise this account key will be ignored no matter it's value)
    ///   18. `[]` New authority for Master Metadata - If you are taking ownership of a Master Edition in and of itself, or a Limited Edition that isn't newly minted for you during this auction
    ///             ie someone else had it minted for themselves in a prior auction or through some other means, this is the account the metadata for these tokens will be delegated to
    ///             after this transaction. Otherwise this account will be ignored.
    ///   19. `[]` PDA-based Transfer authority to move the tokens from the store to the destination seed ['vault', program_id]
    ///        but please note that this is a PDA relative to the Token Vault program, with the 'vault' prefix
    ///
    ///
    ///   OPEN EDITIONS: FURTHERMORE, if you are expecting to receive an open edition token out of this, because this auction supports that, you'll need to add accounts of the same kind and order
    ///   as in Case 2 but for the Open Edition coin at the end of the list. This means that if you are expecting to get an open edition coin, make a destination account with a new mint,
    ///   slap a single coin in it, and then pass up this:
    ///
    ///   x.    `[writable]` New Open Edition Metadata (pda of ['metadata', program id, newly made mint id]) - remember PDA is relative to token metadata program
    ///   xi.   `[writable]` Mint of destination account. This needs to be a newly created mint and the destination account
    ///                   needs to have exactly one token in it already. We will simply "grant" the limited edition status on this token.
    ///   xii.  `[signer]` Destination mint authority - this account is optional, and will only be used/checked if you are receiving a newly minted limited edition.
    ///   xiii. `[]` Master Metadata (pda of ['metadata', program id, master mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   xiii. `[]` Destination account with a single token in it
    ///   xiv.  `[]` New Limited Edition (pda of ['metadata', program id, newly made mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   xv.   `[]` Master Edition (pda of ['metadata', program id, master mint id, 'edition']) - remember PDA is relative to token metadata program
    ///   xvi.  `[]` Original authority on the Master Metadata, which can be gotten via reading off the key from lookup of OriginalAuthorityLookup struct with
    ///            key of (pda of ['metaplex', auction key, master metadata key]).
    ///            We'll use this to grant back authority to the owner of the master metadata if we no longer need it after this latest minting.
    ///   xvii. `[]` Original authority Lookup key - pda of ['metaplex', auction key, master metadata key]
    ///
    ///   Notice this looks similar to case 2, except instead of a Name Symbol you're passing in an additional destination account,
    ///   but for the open edition you expect to receive. This means that it is fully reasonable to make a RedeemBid call that has accounts #16-24 containing keys for the Limited Edition winning bid you won
    ///   plus #24-30 containing keys for the Open Edition you got as a thank you for bidding, and at the end you'll have two tokens in two different accounts from two different mints, one a Limited Edition and one
    ///   an open edition.
    RedeemBid,
}
/*
/// Creates an InitMetaplex instruction
#[allow(clippy::too_many_arguments)]
pub fn create_init_vault_instruction(
    program_id: Pubkey,
    fraction_mint: Pubkey,
    redeem_treasury: Pubkey,
    fraction_treasury: Pubkey,
    vault: Pubkey,
    vault_authority: Pubkey,
    external_price_account: Pubkey,
    allow_further_share_creation: bool,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(fraction_mint, false),
            AccountMeta::new(redeem_treasury, false),
            AccountMeta::new(fraction_treasury, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(vault_authority, false),
            AccountMeta::new_readonly(external_price_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetaplexInstruction::InitMetaplex(InitMetaplexArgs {
            allow_further_share_creation,
        })
        .try_to_vec()
        .unwrap(),
    }
}
*/
