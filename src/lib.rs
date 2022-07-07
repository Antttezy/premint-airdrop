pub mod error;
pub mod instruction;
pub mod pda;
pub mod processor;
pub mod state;
pub mod util;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("M1zWUwPXAoA77vQV4m7hGnZN9qtWM4BYZrA63EzBiyA");
