#![allow(warnings)]

mod errors;
mod utils;

pub mod entrypoint;
pub mod instruction;
pub mod processor;

/// Prefix used in PDA derivations to avoid collisions with other programs.
pub const PREFIX: &str = "auction";

solana_program::declare_id!("DBPY5XNr398qXCWkri9qaSar3kPgzHCkfa8r8agRKgsw");
