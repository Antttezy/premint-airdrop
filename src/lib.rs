pub mod error;
pub mod instruction;
pub mod pda;
pub mod processor;
pub mod state;
pub mod util;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("AtQLjceKsgF99N6qYZCWgKr689neWd4285J7TrmuSHQe");
