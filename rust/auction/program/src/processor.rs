use crate::errors::AuctionError;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    clock::{Slot, UnixTimestamp},
    entrypoint::ProgramResult,
    hash::Hash,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Declare submodules, each contains a single handler for each instruction variant in the program.
pub mod cancel_bid;
pub mod claim_bid;
pub mod create_auction;
pub mod end_auction;
pub mod place_bid;
pub mod set_authority;
pub mod start_auction;

// Re-export submodules handlers + associated types for other programs to consume.
pub use cancel_bid::*;
pub use claim_bid::*;
pub use create_auction::*;
pub use end_auction::*;
pub use place_bid::*;
pub use set_authority::*;
pub use start_auction::*;

pub const EXCLUSIVE_AUCTION_AUTHORITY: &str = "jCebN34bUfdeUYJT13J1yG16XWQpt5PDx6Mse9GUqhR";

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    use crate::instruction::AuctionInstruction;
    match AuctionInstruction::try_from_slice(input)? {
        AuctionInstruction::CancelBid(args) => cancel_bid(program_id, accounts, args),
        AuctionInstruction::ClaimBid(args) => claim_bid(program_id, accounts, args),
        AuctionInstruction::CreateAuction(args) => create_auction(program_id, accounts, args),
        AuctionInstruction::EndAuction(args) => end_auction(program_id, accounts, args),
        AuctionInstruction::PlaceBid(args) => place_bid(program_id, accounts, args),
        AuctionInstruction::SetAuthority => set_authority(program_id, accounts),
        AuctionInstruction::StartAuction(args) => start_auction(program_id, accounts, args),
    }
}

/// Structure with pricing floor data.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum PriceFloor {
    /// Due to borsh on the front end disallowing different arguments in enums, we have to make sure data is
    /// same size across all three
    /// No price floor, any bid is valid.
    None([u8; 32]),
    /// Explicit minimum price, any bid below this is rejected.
    MinimumPrice([u64; 4]),
    /// Hidden minimum price, revealed at the end of the auction.
    BlindedPrice(Hash),
}

// The two extra 8's are present, one 8 is for the Vec's amount of elements and one is for the max
// usize in bid state.
pub const BASE_AUCTION_DATA_SIZE: usize = 32 + 32 + 32 + 9 + 9 + 9 + 9 + 1 + 32 + 1 + 8 + 8;

pub const HARDCODED_GAP_TIME: i64 = 600;
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct AuctionData {
    /// Pubkey of the authority with permission to modify this auction.
    pub authority: Pubkey,
    /// Pubkey of the resource being bid on.
    /// TODO try to bring this back some day. Had to remove this due to a stack access violation bug
    /// interactin that happens in metaplex during redemptions due to some low level rust error
    /// that happens when AuctionData has too many fields. This field was the least used.
    ///pub resource: Pubkey,
    /// Token mint for the SPL token being used to bid
    pub resource: Pubkey,
    /// Token mint for the SPL token being used to bid
    pub token_mint: Pubkey,
    /// The time the last bid was placed, used to keep track of auction timing.
    pub last_bid: Option<UnixTimestamp>,
    /// Unix timestamp the auction was officially ended by.
    pub ended_at: Option<UnixTimestamp>,
    /// End time is the cut-off point that the auction is forced to end by.
    pub end_auction_at: Option<UnixTimestamp>,
    /// Gap time is the amount of time in seconds after the previous bid at which the auction ends.
    pub end_auction_gap: Option<UnixTimestamp>,
    /// Minimum price for any bid to meet.
    pub price_floor: PriceFloor,
    /// The state the auction is in, whether it has started or ended.
    pub state: AuctionState,
    /// Auction Bids, each user may have one bid open at a time.
    pub bid_state: BidState,
}

impl AuctionData {
    pub fn ended(&self, now: UnixTimestamp) -> Result<bool, ProgramError> {
        // If there is an end time specified, handle conditions.
        return match (self.ended_at, self.end_auction_gap) {
            // Both end and gap present, means a bid can still be placed post-auction if it is
            // within the gap time.
            (Some(end), Some(gap)) => {
                // Check if the bid is within the gap between the last bidder.
                if let Some(last) = self.last_bid {
                    let next_bid_time = match last.checked_add(HARDCODED_GAP_TIME) {
                        Some(val) => val,
                        None => return Err(AuctionError::NumericalOverflowError.into()),
                    };
                    Ok(now > end && now > next_bid_time)
                } else {
                    Ok(now > end)
                }
            }

            // Simply whether now has passed the end.
            (Some(end), None) => Ok(now > end),

            // No other end conditions.
            _ => Ok(false),
        };
    }

    pub fn is_winner(&self, key: &Pubkey) -> Option<usize> {
        let minimum = match self.price_floor {
            PriceFloor::MinimumPrice(min) => min[0],
            _ => 0,
        };
        self.bid_state.is_winner(key, minimum)
    }
}

/// Define valid auction state transitions.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum AuctionState {
    Created,
    Started,
    Ended,
}

impl AuctionState {
    pub fn create() -> Self {
        AuctionState::Created
    }

    #[inline(always)]
    pub fn start(self) -> Result<Self, ProgramError> {
        match self {
            AuctionState::Created => Ok(AuctionState::Started),
            _ => Err(AuctionError::AuctionTransitionInvalid.into()),
        }
    }

    #[inline(always)]
    pub fn end(self) -> Result<Self, ProgramError> {
        match self {
            AuctionState::Started => Ok(AuctionState::Ended),
            _ => Err(AuctionError::AuctionTransitionInvalid.into()),
        }
    }
}

/// Bids associate a bidding key with an amount bid.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Bid(pub Pubkey, pub u64);

/// BidState tracks the running state of an auction, each variant represents a different kind of
/// auction being run.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum BidState {
    EnglishAuction { bids: Vec<Bid>, max: usize },
    OpenEdition { bids: Vec<Bid>, max: usize },
}

/// Bidding Implementations.
///
/// English Auction: this stores only the current winning bids in the auction, pruning cancelled
/// and lost bids over time.
///
/// Open Edition: All bids are accepted, cancellations return money to the bidder and always
/// succeed.
impl BidState {
    pub fn new_english(n: usize) -> Self {
        BidState::EnglishAuction {
            bids: vec![],
            max: n,
        }
    }

    pub fn new_open_edition() -> Self {
        BidState::OpenEdition {
            bids: vec![],
            max: 0,
        }
    }

    /// Push a new bid into the state, this succeeds only if the bid is larger than the current top
    /// winner stored. Crappy list information to start with.
    pub fn place_bid(&mut self, bid: Bid) -> Result<(), ProgramError> {
        match self {
            // In a capped auction, track the limited number of winners.
            BidState::EnglishAuction { ref mut bids, max } => match bids.last() {
                Some(top) => {
                    let outbid = bid.1.saturating_sub(top.1);
                    let minimum_bid_increment = top.1 / 20;
                    if outbid > minimum_bid_increment || bids.len() < *max {
                        bids.retain(|b| b.0 != bid.0);
                        bids.push(bid);
                        if bids.len() > *max {
                            bids.remove(0);
                        }
                        Ok(())
                    } else {
                        msg!("This bid fails to make it onto the winner stack.");
                        Err(AuctionError::BidTooSmall.into())
                    }
                }
                _ => {
                    msg!("Pushing bid onto stack");
                    bids.push(bid);
                    Ok(())
                }
            },

            // In an open auction, bidding simply succeeds.
            BidState::OpenEdition { bids, max } => Ok(()),
        }
    }

    /// Cancels a bid, if the bid was a winning bid it is removed, if the bid is invalid the
    /// function simple no-ops.
    pub fn cancel_bid(&mut self, key: Pubkey) -> Result<(), ProgramError> {
        match self {
            BidState::EnglishAuction { ref mut bids, max } => {
                bids.retain(|b| b.0 != key);
                Ok(())
            }

            // In an open auction, cancelling simply succeeds. It's up to the manager of an auction
            // to decide what to do with open edition bids.
            BidState::OpenEdition { bids, max } => Ok(()),
        }
    }

    /// Check if a pubkey is currently a winner.
    pub fn is_winner(&self, key: &Pubkey, min: u64) -> Option<usize> {
        match self {
            // Presense in the winner list is enough to check win state.
            BidState::EnglishAuction { bids, max } => {
                bids.iter().position(|bid| &bid.0 == key && bid.1 > min)
            }
            // There are no winners in an open edition, it is up to the auction manager to decide
            // what to do with open edition bids.
            BidState::OpenEdition { bids, max } => None,
        }
    }
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum WinnerLimit {
    Unlimited(usize),
    Capped(usize),
}

pub const BIDDER_METADATA_LEN: usize = 32 + 32 + 8 + 8 + 1;
/// Models a set of metadata for a bidder, meant to be stored in a PDA. This allows looking up
/// information about a bidder regardless of if they have won, lost or cancelled.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct BidderMetadata {
    // Relationship with the bidder who's metadata this covers.
    pub bidder_pubkey: Pubkey,
    // Relationship with the auction this bid was placed on.
    pub auction_pubkey: Pubkey,
    // Amount that the user bid.
    pub last_bid: u64,
    // Tracks the last time this user bid.
    pub last_bid_timestamp: UnixTimestamp,
    // Whether the last bid the user made was cancelled. This should also be enough to know if the
    // user is a winner, as if cancelled it implies previous bids were also cancelled.
    pub cancelled: bool,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct BidderPot {
    /// Points at actual pot that is a token account
    pub bidder_pot: Pubkey,
    /// Originating bidder account
    pub bidder_act: Pubkey,
    /// Auction account
    pub auction_act: Pubkey,
    /// emptied or not
    pub emptied: bool,
}
#[repr(C)]
#[derive(Clone, BorshDeserialize, BorshSerialize, PartialEq, Debug)]
pub struct BuyNowData {
    /// Max price of the auction i.e buy now price
    pub max_price: u64,
}

pub const BUY_NOW_DATA_LEN: usize = 8;
