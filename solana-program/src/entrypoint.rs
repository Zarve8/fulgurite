use {
    crate::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, syscalls::Syscalls},
    std::{
        result::Result as ResultGeneric,
    },
};


pub const SUCCESS: u64 = 0;
pub const HEAP_START_ADDRESS: u64 = 0x300000000;
pub const HEAP_LENGTH: usize = 32 * 1024;
pub const NON_DUP_MARKER: u8 = u8::MAX;
pub const MAX_PERMITTED_DATA_INCREASE: usize = 1_024 * 10;
pub const BPF_ALIGN_OF_U128: usize = 8;


pub type ProgramResult = ResultGeneric<(), ProgramError>;

pub type ProcessInstruction =
    fn(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8], Box<dyn Syscalls>) -> ProgramResult;


#[macro_export]
macro_rules! entrypoint {
    ($process_instruction:ident) => {

        #[cfg(not(feature="inline"))]
        /// # Safety
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint<'g>(program_id: &'g $crate::pubkey::Pubkey, accounts: &'g [$crate::account_info::AccountInfo<'g>], instruction_data: &'g [u8], mut syscalls: Box<dyn $crate::syscalls::Syscalls>) -> ProgramResult {
            let default_syscalls = $crate::syscalls::SYSCALLS.swap(&mut syscalls, std::sync::atomic::Ordering::Relaxed);

            let res = $process_instruction(program_id, accounts, instruction_data);

            let _ = $crate::syscalls::SYSCALLS.swap(default_syscalls, std::sync::atomic::Ordering::Relaxed);
            std::mem::forget(syscalls);

            res
        }

        #[cfg(feature="inline")]
        pub fn entrypoint(program_id: &$crate::pubkey::Pubkey, accounts: &[$crate::account_info::AccountInfo], instruction_data: &[u8], mut syscalls: Box<dyn $crate::syscalls::Syscalls>) -> ProgramResult {
            unsafe {
                let default_syscalls = $crate::syscalls::SYSCALLS.swap(&mut syscalls, std::sync::atomic::Ordering::Relaxed);

                let res = $process_instruction( // Allow no generics in fn declaration
                    std::mem::transmute::<&Pubkey, &'static Pubkey>(program_id),
                    std::mem::transmute::<&[AccountInfo], &'static [AccountInfo<'static>]>(accounts),
                    std::mem::transmute::<&[u8], &'static [u8]>(instruction_data)
                );

                let _ = $crate::syscalls::SYSCALLS.swap(default_syscalls, std::sync::atomic::Ordering::Relaxed);
                std::mem::forget(syscalls);

                res
            }
        }
    };
}
