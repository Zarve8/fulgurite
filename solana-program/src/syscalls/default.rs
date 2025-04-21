use std::sync::atomic::AtomicPtr;
use crate::account_info::AccountInfo;
use crate::clock::Clock;
use crate::entrypoint::ProgramResult;
use crate::instruction::Instruction;
use crate::pubkey::Pubkey;
use crate::rent::Rent;
use crate::syscalls::Syscalls;


pub struct DefaultSyscalls {}

impl DefaultSyscalls {
    pub fn new() -> Self {
        Self {}
    }

    pub fn new_ref() -> *mut Box<dyn Syscalls> {
        let mut syscalls_box: Box<dyn Syscalls> = Box::new(Self::new());
        let syscalls_ptr = &mut syscalls_box as *mut Box<dyn Syscalls>;
        std::mem::forget(syscalls_box); // Owned by lazy_static
        syscalls_ptr
    }
}

impl Syscalls for DefaultSyscalls {
    fn get_processed_sibling_instruction(&mut self, index: usize) -> Option<Instruction> {
        panic!("Syscalls was not provided")
    }

    fn get_stack_height(&mut self) -> usize {
        panic!("Syscalls was not provided")
    }

    fn sol_log(&mut self, message: &str) {
        panic!("Syscalls was not provided")
    }

    fn sol_log_64(&mut self, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) {
        panic!("Syscalls was not provided")
    }

    fn sol_log_data(&mut self, data: &[&[u8]]) {
        panic!("Syscalls was not provided")
    }

    fn sol_log_compute_units(&mut self) {
        panic!("Syscalls was not provided")
    }

    fn invoke_signed_unchecked(&mut self, instruction: &Instruction, account_infos: &[AccountInfo], signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        panic!("Syscalls was not provided")
    }

    fn set_return_data(&mut self, data: &[u8]) {
        panic!("Syscalls was not provided")
    }

    fn get_return_data(&mut self) -> Option<(Pubkey, Vec<u8>)> {
        panic!("Syscalls was not provided")
    }

    fn get_clock(&mut self) -> Clock {
        panic!("Syscalls was not provided")
    }

    fn get_rent(&mut self) -> Rent {
        panic!("Syscalls was not provided")
    }

    fn set_owner(&mut self, to: &Pubkey, owner: &Pubkey) {
        panic!("Syscalls was not provided")
    }

    fn set_data(&mut self, info: &AccountInfo, data: Vec<u8>) {
        panic!("Syscalls was not provided")
    }

    fn set_lamports(&mut self, to: &Pubkey, amount: u64) {
        panic!("Syscalls was not provided")
    }

    fn get_program_id(&self) -> Pubkey {
        panic!("Syscalls was not provided")
    }

    fn finalize_system_invoke<'a>(&mut self, accounts: &'a [AccountInfo<'a>]) -> ProgramResult  {
        panic!("Syscalls was not provided")
    }

    fn rent_exempt_for_size(&mut self, size: usize) -> u64 {
        panic!("Syscalls was not provided")
    }

    fn get_data_ptr(&mut self, key: &Pubkey) -> AtomicPtr<Vec<u8>> {
        panic!("Syscalls was not provided")
    }
}