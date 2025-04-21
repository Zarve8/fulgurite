use crate::executor::Package;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::{ProgramResult, ProcessInstruction},
    pubkey::Pubkey,
    syscalls::Syscalls,
};


pub struct InlinePackage {
    entrypoint: ProcessInstruction
}

impl InlinePackage {
    pub fn new(entrypoint: ProcessInstruction) -> Self {
        Self { entrypoint }
    }
}

impl Package for InlinePackage {
    fn execute<'e>(&self, accounts: &'e [AccountInfo<'e>], instruction_data: &'e [u8], program_id: &'e Pubkey, syscalls: Box<dyn Syscalls>) -> ProgramResult {
        (self.entrypoint)(program_id, accounts, instruction_data, syscalls)
    }
}
