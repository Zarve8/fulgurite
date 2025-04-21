pub mod default;

use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicPtr;
use std::sync::Mutex;
use {
    std::sync::{Arc, RwLock},
    crate::syscalls::default::DefaultSyscalls,
};
use solana_program::clock::Clock;
use crate::account_info::AccountInfo;
use crate::entrypoint::ProgramResult;
use crate::instruction::Instruction;
use crate::pubkey::Pubkey;
use crate::rent::Rent;


lazy_static::lazy_static! {
    pub static ref SYSCALLS: AtomicPtr<Box<dyn Syscalls>> = AtomicPtr::new(DefaultSyscalls::new_ref());
}

#[macro_export]
macro_rules! syscalls {
    () => {
        unsafe { $crate::syscalls::SYSCALLS.load(std::sync::atomic::Ordering::Relaxed).as_mut().unwrap() }
    }
}

#[macro_export]
macro_rules! not_supported {
    () => {
        panic!("Not supported by Fulgurite")
    };
    ($method:expr) => {
        panic!("Method \"{}\" isn't supported by Fulgurite", $method)
    };
}


pub trait Syscalls: Sync + Send {
    fn get_processed_sibling_instruction(&mut self, index: usize) -> Option<Instruction>;
    fn get_stack_height(&mut self) -> usize;
    fn sol_log(&mut self, message: &str);
    fn sol_log_64(&mut self, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64);
    fn sol_log_data(&mut self, data: &[&[u8]]);
    fn sol_log_compute_units(&mut self);
    fn invoke_signed_unchecked(&mut self, instruction: &Instruction, account_infos: &[AccountInfo], signers_seeds: &[&[&[u8]]]) -> ProgramResult;
    fn set_return_data(&mut self, data: &[u8]);
    fn get_return_data(&mut self) -> Option<(Pubkey, Vec<u8>)>;
    fn get_clock(&mut self) -> Clock;
    fn get_rent(&mut self) -> Rent;

    fn set_owner(&mut self, to: &Pubkey, owner: &Pubkey);
    fn set_data(&mut self, info: &AccountInfo, data: Vec<u8>);
    fn set_lamports(&mut self, to: &Pubkey, amount: u64);

    fn get_program_id(&self) -> Pubkey;
    fn finalize_system_invoke<'a>(&mut self, accounts: &'a [AccountInfo<'a>]) -> ProgramResult;
    fn rent_exempt_for_size(&mut self, size: usize) -> u64;
    fn get_data_ptr(&mut self, key: &Pubkey) -> AtomicPtr<Vec<u8>>;
}









