use crate::processor::Processor;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    pubkey::Pubkey,
};
use solana_program::program_error::PrintProgramError;
use crate::error::PoolError;


#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);


fn process_instruction<'g>(
    program_id: &'g Pubkey,
    accounts: &'g [AccountInfo<'g>],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = Processor::process_instruction(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<PoolError>();
        return Err(error);
    }
    Ok(())
}
