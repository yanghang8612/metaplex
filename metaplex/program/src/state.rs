use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::pubkey::Pubkey,
};
/// prefix used for PDAs to avoid certain collision attacks (https://en.wikipedia.org/wiki/Collision_attack#Chosen-prefix_collision_attack)
pub const PREFIX: &str = "metaplex";

pub const MAX_WINNERS: usize = 200;
pub const MAX_WINNER_SIZE: usize = 4 * MAX_WINNERS;
pub const MAX_AUCTION_MANAGER_SIZE: usize =
    1 + 32 + 32 + 32 + 32 + 32 + 1 + 1 + 1 + 1 + MAX_WINNER_SIZE + 2 + 9;

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum Key {
    AuctionManagerV1,
}

/// An Auction Manager can support an auction that is an English auction and limited edition and open edition
/// all at once. Need to support all at once. We use u8 keys to point to safety deposit indices in Vault
/// as opposed to the pubkeys to save on space. Ordering of safety deposits is guaranteed fixed by vault
/// implementation.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct AuctionManager {
    pub key: Key,

    pub authority: Pubkey,

    pub auction: Pubkey,

    pub vault: Pubkey,

    pub token_metadata_program: Pubkey,

    pub token_program: Pubkey,

    pub state: AuctionManagerState,

    pub settings: AuctionManagerSettings,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct AuctionManagerState {
    pub status: AuctionManagerStatus,
    /// When all boxes are validated the auction is started and auction manager moves to Running
    pub safety_deposit_boxes_validated: u8,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct AuctionManagerSettings {
    /// Setups:
    /// 1. Winners get open edition + not charged extra
    /// 2. Winners dont get open edition
    pub open_edition_winner_constraint: WinnerConstraint,

    /// Setups:
    /// 1. Losers get open edition for free
    /// 2. Losers get open edition but pay fixed price
    /// 3. Losers get open edition but pay bid price
    pub open_edition_non_winning_constraint: NonWinningConstraint,

    /// The safety deposit box index in the vault containing the winning items, in order of place
    /// The same index can appear multiple times if that index contains n tokens for n appearances (this will be checked)
    pub winning_configs: Vec<WinningConfig>,

    /// The safety deposit box index in the vault containing the template for the open edition
    pub open_edition_config: Option<u8>,

    /// Setting this field disconnects the open edition's price from the bid. Any bid you submit, regardless
    /// of amount, charges you the same fixed price. NOTE: This field supersedes open_edition_reserve_price.
    pub open_edition_fixed_price: Option<u64>,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub enum WinnerConstraint {
    NoOpenEdition,
    OpenEditionGiven,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub enum NonWinningConstraint {
    NoOpenEdition,
    GivenForFixedPrice,
    GivenForBidPrice,
}

#[repr(C)]
#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Copy)]
pub enum EditionType {
    // Not an edition
    NA,
    /// Means you are auctioning off the master edition record
    MasterEdition,
    /// Means you are using the master edition to print off new editions during the auction (limited or open edition)
    MasterEditionAsTemplate,
    /// Means you are indicating this is an Edition, not a Master Edition, and you are auctioning it
    Edition,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Copy)]
pub struct WinningConfig {
    pub safety_deposit_box_index: u8,
    pub amount: u8,
    /// Each safety deposit box needs to be validated via endpoint before auction manager will agree to let auction begin.
    pub validated: bool,
    pub edition_type: EditionType,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub enum AuctionManagerStatus {
    Initialized,
    Validated,
    Running,
    Disbursing,
    Finished,
}
