#![allow(warnings)]

mod errors;
mod utils;

pub mod entrypoint;
pub mod instruction;
pub mod processor;

/// Prefix used in PDA derivations to avoid collisions with other programs.
pub const PREFIX: &str = "auction";

solana_program::declare_id!("7av4YU7SU9wUgL54XrZjYzcGyg9zB9CWifizXyyjpGfE");
