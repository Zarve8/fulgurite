pub mod instruction;
pub mod counter;
mod processor;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;