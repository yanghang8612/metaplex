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
    ///   1. `[]` Safety deposit box account
    ///   2. `[]` Store account of safety deposit box
    ///   3. `[]` Mint account of the token in the safety deposit box
    ///   4. `[]` Metadata account
    ///   5. `[]` Name symbol tuple account
    ///           (This account is optional, and will only be used if metadata is unique, otherwise this account key will be ignored no matter it's value)
    ///   6. `[]` Edition OR MasterEdition record (if this safety deposit item contains a mint that has either)
    ///   7. `[]` Vault account
    ///   8. `[signer]` Authority
    ///   9. `[signer]` Metadata Authority
    ValidateSafetyDepositBox,

    /// Note: This requires that auction manager be in a Running state.
    /// If an auction is complete, you can redeem your bid for a specific item here. If you are the first to do this,
    /// The auction manager will switch from Running state to Disbursing state. If you are the last, this may change
    /// the auction manager state to Finished provided that no authorities remain to be delegated for Master Edition tokens.
    ///   0. `[writable]` Auction manager
    ///   1. `[writable]` Store of safety deposit box account
    ///   2. `[writable]` Destination account
    ///   3. `[]` Safety deposit box account
    ///   4. `[]` Fraction mint of the vault
    ///   5. `[]` Vault account
    ///   6. `[]` Auction
    ///   7. `[]` Your BidderMetadata account
    ///   8. `[signer]` Authority of the auction manager
    ///   9. `[]` Token program
    ///   10. `[]` Token Vault program
    ///   11. `[]` Rent sysvar
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
