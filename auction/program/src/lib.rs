mod entrypoint;
mod errors;
mod instruction;
pub mod processor;
mod utils;

/// Prefix used in PDA derivations to avoid collisions with other programs.
const PREFIX: &str = "auction";
