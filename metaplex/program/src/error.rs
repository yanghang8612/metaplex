//! Error types

use {
    num_derive::FromPrimitive,
    solana_program::{
        decode_error::DecodeError,
        msg,
        program_error::{PrintProgramError, ProgramError},
    },
    thiserror::Error,
};

/// Errors that may be returned by the Metaplex program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum MetaplexError {
    /// Invalid instruction data passed in.
    #[error("Failed to unpack instruction data")]
    InstructionUnpackError,

    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,

    /// Already initialized
    #[error("Already initialized")]
    AlreadyInitialized,

    /// Uninitialized
    #[error("Uninitialized")]
    Uninitialized,

    /// Account does not have correct owner
    #[error("Account does not have correct owner")]
    IncorrectOwner,

    /// NumericalOverflowError
    #[error("NumericalOverflowError")]
    NumericalOverflowError,

    /// Token transfer failed
    #[error("Token transfer failed")]
    TokenTransferFailed,

    /// Invalid transfer authority provided
    #[error("Invalid transfer authority provided")]
    InvalidTransferAuthority,

    /// Vault's authority does not match the expected pda with seed ['metaplex', auction_key]
    #[error("Vault's authority does not match the expected ['metaplex', auction_key]")]
    VaultAuthorityMismatch,

    /// Auction's authority does not match the expected pda with seed ['metaplex', auction_key]
    #[error(
        "Auction's authority does not match the expected pda with seed ['metaplex', auction_key]"
    )]
    AuctionAuthorityMismatch,

    /// The authority passed to the call does not match the authority on the auction manager!
    #[error(
        "The authority passed to the call does not match the authority on the auction manager!"
    )]
    AuctionManagerAuthorityMismatch,

    /// Auction Manager does not have the appropriate pda key with seed ['metaplex', auction_key]
    #[error(
        "Auction Manager does not have the appropriate pda key with seed ['metaplex', auction_key]"
    )]
    AuctionManagerKeyMismatch,

    /// Auction is not auctioning off the vault given!
    #[error("Auction is not auctioning off the vault given!")]
    AuctionVaultMismatch,

    /// Vault given does not match that on given auction manager!
    #[error("Vault given does not match that on given auction manager!")]
    AuctionManagerVaultMismatch,

    /// The safety deposit box given does not belong to the given vault!
    #[error("The safety deposit box given does not belong to the given vault!")]
    SafetyDepositBoxVaultMismatch,

    /// The store given does not belong to the safety deposit box given!
    #[error("The store given does not belong to the safety deposit box given!")]
    SafetyDepositBoxStoreMismatch,

    /// The metadata given does not match the mint on the safety deposit box given!
    #[error("The metadata given does not match the mint on the safety deposit box given!")]
    SafetyDepositBoxMetadataMismatch,

    /// The mint given does not match the mint on the given safety deposit box!
    #[error("The mint given does not match the mint on the given safety deposit box!")]
    SafetyDepositBoxMintMismatch,

    /// The token metadata program given does not match the token metadata program on this auction manager!
    #[error("The token metadata program given does not match the token metadata program on this auction manager!")]
    AuctionManagerTokenMetadataProgramMismatch,

    /// The mint is owned by a different token program than the one used by this auction manager!
    #[error(
        "The mint is owned by a different token program than the one used by this auction manager!"
    )]
    TokenProgramMismatch,

    /// The auction given does not match the auction on the auction manager!
    #[error("The auction given does not match the auction on the auction manager!")]
    AuctionManagerAuctionMismatch,

    /// The auction program given does not match the auction program on the auction manager!
    #[error(
        "The auction program given does not match the auction program on the auction manager!"
    )]
    AuctionManagerAuctionProgramMismatch,

    /// The token program given does not match the token program on the auction manager!
    #[error("The token program given does not match the token program on the auction manager!")]
    AuctionManagerTokenProgramMismatch,

    /// The token vault program given does not match the token vault program on the auction manager!
    #[error("The token vault program given does not match the token vault program on the auction manager!")]
    AuctionManagerTokenVaultProgramMismatch,

    /// Only combined vaults may be used in auction managers!
    #[error("Only combined vaults may be used in auction managers!")]
    VaultNotCombined,

    /// Cannot auction off an empty vault!
    #[error("Cannot auction off an empty vault!")]
    VaultCannotEmpty,

    /// Listed a safety deposit box index that does not exist in this vault
    #[error("Listed a safety deposit box index that does not exist in this vault")]
    InvalidSafetyDepositBox,

    /// Cant use a limited supply edition for an open edition as you may run out of editions to print
    #[error("Cant use a limited supply edition for an open edition as you may run out of editions to print")]
    CantUseLimitedSupplyEditionsWithOpenEditionAuction,

    /// This safety deposit box is not listed as a prize in this auction manager!
    #[error("This safety deposit box is not listed as a prize in this auction manager!")]
    SafetyDepositBoxNotUsedInAuction,

    /// Auction Manager Authority needs to be signer for this action!
    #[error("Auction Manager Authority needs to be signer for this action!")]
    AuctionManagerAuthorityIsNotSigner,

    /// Either you have given a non-existent edition address or you have given the address to a different token-metadata program than was used to make this edition!
    #[error("Either you have given a non-existent edition address or you have given the address to a different token-metadata program than was used to make this edition!")]
    InvalidEditionAddress,

    /// There are not enough editions available for this auction!
    #[error("There are not enough editions available for this auction!")]
    NotEnoughEditionsAvailableForAuction,

    /// The store in the safety deposit is empty, so you have nothing to auction!
    #[error("The store in the safety deposit is empty, so you have nothing to auction!")]
    StoreIsEmpty,

    /// Not enough tokens to supply winners!
    #[error("Not enough tokens to supply winners!")]
    NotEnoughTokensToSupplyWinners,

    /// The auction manager must own the payoff account!
    #[error("The auction manager must own the payoff account!")]
    AuctionManagerMustOwnPayoffAccount,

    /// The auction manager must own the oustanding shares  account!
    #[error("The auction manager must own the oustanding shares account!")]
    AuctionManagerMustOwnOutstandingSharesAccount,

    /// This bidder metadata does not have the expected PDA address
    #[error("This bidder metadata does not have the expected PDA address")]
    InvalidBidderMetadata,

    /// The safety deposit box for your winning bid placement does not match the safety deposit box you provided!
    #[error("The safety deposit box for your winning bid placement does not match the safety deposit box you provided!")]
    SafetyDepositIndexMismatch,

    /// This prize has already been claimed!
    #[error("This prize has already been claimed!")]
    PrizeAlreadyClaimed,

    /// The bid redemption key does not match the expected PDA with seed ['metaplex', auction key, bidder metadata key]
    #[error("The bid redemption key does not match the expected PDA with seed ['metaplex', auction key, bidder metadata key]")]
    BidRedemptionMismatch,

    /// This bid has already been redeemed!
    #[error("This bid has already been redeemed!")]
    BidAlreadyRedeemed,

    /// Auction has not ended yet!
    #[error("Auction has not ended yet!")]
    AuctionHasNotEnded,

    /// The original authority lookup does not match the expected PDA of ['metaplex', auction key, metadata key]
    #[error("The original authority lookup does not match the expected PDA of ['metaplex', auction key, metadata key]")]
    OriginalAuthorityLookupKeyMismatch,

    /// The original authority given does not match that on the original authority lookup account!
    #[error("The original authority given does not match that on the original authority lookup account!")]
    OriginalAuthorityMismatch,

    /// The prize you are attempting to claim needs to be claimed from a different endpoint than this one.
    #[error("The prize you are attempting to claim needs to be claimed from a different endpoint than this one.")]
    WrongBidEndpointForPrize,

    /// The bidder given is not the bidder on the bidder metadata!
    #[error("The bidder given is not the bidder on the bidder metadata!")]
    BidderMetadataBidderMismatch,

    /// The bidder is not the signer on this transaction!
    #[error("The bidder is not the signer on this transaction!")]
    BidderIsNotSigner,

    /// Master mint given does not match the mint on master edition!
    #[error("Master mint given does not match the mint on master edition!")]
    MasterEditionMintMismatch,

    /// Destination does not have the proper mint!
    #[error("Destination does not have the proper mint!")]
    DestinationMintMismatch,

    /// Invalid edition key
    #[error("Invalid edition key")]
    InvalidEditionKey,

    /// Token mint to failed
    #[error("Token mint to failed")]
    TokenMintToFailed,

    /// Master mint authority must be signer to transfer minting authority to auction manager
    #[error(
        "Master mint authority must be signer to transfer minting authority to auction manager"
    )]
    MasterMintAuthorityMustBeSigner,

    /// The master mint authority provided does not match that on the mint
    #[error("The master mint authority provided does not match that on the mint")]
    MasterMintAuthorityMismatch,

    /// The master mint provided does not match that on the master edition provided
    #[error("The master mint provided does not match that on the master edition provided")]
    MasterEditionMasterMintMismatch,

    /// The accept payment account for this auction manager must match the auction's token mint!
    #[error(
        "The accept payment account for this auction manager must match the auction's token mint!"
    )]
    AuctionAcceptPaymentMintMismatch,

    /// The accept payment owner must be the auction manager!
    #[error("The accept payment owner must be the auction manager!")]
    AcceptPaymentOwnerMismatch,

    /// The accept payment given does not match the accept payment account on the auction manager!
    #[error("The accept payment given does not match the accept payment account on the auction manager!")]
    AcceptPaymentMismatch,

    /// You are not eligible for an open edition!
    #[error("You are not eligible for an open edition!")]
    NotEligibleForOpenEdition,
}

impl PrintProgramError for MetaplexError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<MetaplexError> for ProgramError {
    fn from(e: MetaplexError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MetaplexError {
    fn type_of() -> &'static str {
        "Metaplex Error"
    }
}
