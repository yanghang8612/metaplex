#![allow(warnings)]

mod errors;
mod utils;

pub mod entrypoint;
pub mod instruction;
pub mod processor;

/// Prefix used in PDA derivations to avoid collisions with other programs.
pub const PREFIX: &str = "auction";

solana_program::declare_id!("8penQkW1ByRHECArWpV5W7htAKkcieY3dXhz4XxFVnu5");
