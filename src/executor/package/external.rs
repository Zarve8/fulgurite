use std::ffi::{OsStr, OsString};
use crate::executor::Package;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::{ProgramResult, ProcessInstruction},
    pubkey::Pubkey,
    syscalls::Syscalls,
};
use libloading::{Library, library_filename, Symbol};
use std::path::Path;
use resolve_path::PathResolveExt;
use std::ops::Deref;


pub struct ExternalPackage {
    lib: Library,
}

impl ExternalPackage {
    pub fn new(package_name: &str) -> Self {
        let path_str = format!("target/debug/lib{}.so", package_name.replace("-", "_"));
        let path_cow = path_str.resolve();
        let path: &Path = path_cow.deref();
        Self::new_from_path(
            path.as_os_str()
        )
    }

    pub fn new_from_path(path: &OsStr) -> Self  {
        let lib = unsafe {
            Library::new(path.clone())
        }.expect(&format!("Missing Program {:?}", path));

        let _: Symbol<ProcessInstruction> = unsafe {
            lib.get(b"entrypoint")
        }.expect(&format!("Not a Program {:?}", path));

        Self { lib }
    }
}

impl Package for ExternalPackage {
    fn execute<'e>(&self, accounts: &'e [AccountInfo<'e>], instruction_data: &'e [u8], program_id: &'e Pubkey, syscalls: Box<dyn Syscalls>) -> ProgramResult {
        let mut entrypoint: Symbol<ProcessInstruction> = unsafe { self.lib.get(b"entrypoint").unwrap() };
        entrypoint(program_id, accounts, instruction_data, syscalls)
    }
}