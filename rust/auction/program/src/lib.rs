#![allow(warnings)]

mod errors;
pub mod utils;

pub mod entrypoint;
pub mod instruction;
pub mod processor;

/// Prefix used in PDA derivations to avoid collisions with other programs.
pub const PREFIX: &str = "auction";
pub const BUY_NOW: &str = "buy now";
pub const BONFIDA_SOL_VAULT: &str = "GcWEQ9K78FV7LEHteFVciYApERk5YvQuFDQPk1yYJVXi";
pub const REF_SHARE: u64 = 20;

solana_program::declare_id!("AVWV7vdWbLqXiLKFaP19GhYurhwxaLp2qRBSjT5tR5vT");
