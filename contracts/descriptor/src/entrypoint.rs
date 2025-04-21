use crate::processor::Processor;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    pubkey::Pubkey,
};


#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);


fn process_instruction<'g>(
    program_id: &'g Pubkey,
    accounts: &'g [AccountInfo<'g>],
    instruction_data: &[u8],
) -> ProgramResult {
    Processor::process_instruction(program_id, accounts, instruction_data)
}
