#![allow(warnings)]

mod errors;
mod utils;

pub mod entrypoint;
pub mod instruction;
pub mod processor;
pub mod constant;

/// Prefix used in PDA derivations to avoid collisions with other programs.
pub const PREFIX: &str = "lottery";

solana_program::declare_id!("auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8");
