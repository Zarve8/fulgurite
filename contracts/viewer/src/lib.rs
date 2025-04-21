pub mod instruction;
mod processor;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;