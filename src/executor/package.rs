mod system_program;
mod program_utils;
mod external;
mod inline;

pub use external::ExternalPackage;
pub use system_program::SystemProgramPackage;
pub use inline::InlinePackage;


use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    syscalls::Syscalls
};


pub trait Package {
    fn execute<'e>(&self, accounts: &'e [AccountInfo<'e>], instruction_data: &'e [u8], program_id: &'e Pubkey, syscalls: Box<dyn Syscalls>) -> ProgramResult;
}
