use crate::errors::AuctionError;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    clock::{Slot, UnixTimestamp},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod cancel_bid;
pub mod create_auction;
pub mod place_bid;
pub mod set_authority;
pub mod start_auction;

pub use cancel_bid::*;
pub use create_auction::*;
pub use place_bid::*;
pub use start_auction::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    use crate::instruction::AuctionInstruction;
    use cancel_bid::cancel_bid;
    use create_auction::create_auction;
    use place_bid::place_bid;
    use start_auction::start_auction;

    match AuctionInstruction::try_from_slice(input)? {
        AuctionInstruction::CreateAuction(args) => {
            msg!("+ Processing CreateAuction");
            create_auction(program_id, accounts, args)
        }
        AuctionInstruction::StartAuction(args) => {
            msg!("+ Processing StartAuction");
            start_auction(program_id, accounts, args)
        }
        AuctionInstruction::PlaceBid(args) => {
            msg!("+ Processing PlaceBid");
            place_bid(program_id, accounts, args)
        }
        AuctionInstruction::CancelBid(args) => {
            msg!("+ Processing Cancelbid");
            cancel_bid(program_id, accounts)
        }
        AuctionInstruction::SetAuthority => {
            msg!("+ Processing SetAuthority");
            cancel_bid(program_id, accounts)
        }
    }
}

/// Structure containing timing configuration, I.E when the auction ends.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct AuctionTiming {}

// The two extra 8's are present, one 8 is for the Vec's amount of elements and one is for the max usize in
// bid state.
pub const BASE_AUCTION_DATA_SIZE: usize = 32 + 32 + 32 + 8 + 8 + 1 + 9 + 9 + 9 + 9;
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct AuctionData {
    /// Pubkey of the authority with permission to modify this auction.
    pub authority: Pubkey,
    /// Pubkey of the resource being bid on.
    pub resource: Pubkey,
    /// Token mint for the SPL token being used to bid
    pub token_mint: Pubkey,
    /// The time the last bid was placed, used to keep track of auction timing.
    pub last_bid: Option<Slot>,
    /// Slot time the auction was officially ended by.
    pub ended_at: Option<Slot>,
    /// End time is the cut-off point that the auction is forced to end by.
    pub end_auction_at: Option<Slot>,
    /// Gap time is the amount of time in slots after the previous bid at which the auction ends.
    pub end_auction_gap: Option<Slot>,
    /// The state the auction is in, whether it has started or ended.
    pub state: AuctionState,
    /// Auction Bids, each user may have one bid open at a time.
    pub bid_state: BidState,
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
pub struct Bid(Pubkey, u64);

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
                    msg!("{} < {}", top.1, bid.1);
                    if top.1 < bid.1 {
                        bids.retain(|b| b.0 != bid.0);
                        bids.push(bid);
                        if bids.len() > *max {
                            bids.remove(0);
                        }
                        return Ok(());
                    }
                    return Ok(());
                }
                _ => {
                    msg!("Pushing bid onto stack");
                    bids.push(bid);
                    return Ok(());
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
    pub fn is_winner(&self, key: Pubkey) -> Option<usize> {
        match self {
            // Presense in the winner list is enough to check win state.
            BidState::EnglishAuction { ref bids, max } => bids.iter().position(|bid| bid.0 == key),
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
    /// These fields not technically required for the backend but used on the front end to index and search bidder pots
    /// quickly, like indices in a database...
    /// originating bidder acct
    pub bidder_act: Pubkey,
    /// auction account
    pub auction_act: Pubkey,
}
