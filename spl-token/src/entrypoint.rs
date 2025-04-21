//! Program entrypoint

use crate::{error::TokenError, processor::Processor};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    program_error::PrintProgramError, pubkey::Pubkey,
};



// #[cfg(any(not(feature = "no-entrypoint"), feature = "inline"))]
// #[cfg(not(feature = "no-entrypoint"))]
#[cfg(any(not(feature = "no-entrypoint"), feature = "inline"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<TokenError>();
        return Err(error);
    }
    Ok(())
}
