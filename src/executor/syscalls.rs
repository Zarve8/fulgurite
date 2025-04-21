use std::collections::HashMap;
use std::mem::forget;
use std::sync::atomic::AtomicPtr;
use solana_program::{
    pubkey::Pubkey,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::Instruction,
    rent::Rent,
    clock::Clock,
    syscalls::Syscalls,
    program_error::ProgramError
};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use crate::runtime::Scope;


pub const MAX_RETURN_DATA: usize = 1024;


impl Syscalls for Scope {
    fn get_processed_sibling_instruction(&mut self, index: usize) -> Option<Instruction> {
        todo!()
    }

    fn get_stack_height(&mut self) -> usize {
        self.receipt.call_stack.len() + 1
    }

    fn sol_log(&mut self, message: &str) {
        self.receipt.push_msg(format!("Program logged: \"{message}\""));
    }

    fn sol_log_64(&mut self, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) {
        self.sol_log(&format!(
            "{arg1:#x}, {arg2:#x}, {arg3:#x}, {arg4:#x}, {arg5:#x}"
        ));
    }

    fn sol_log_data(&mut self, data: &[&[u8]]) {
        let mut s = "Program data:".to_string();
        let mut v = Vec::new();
        for bytes in data.iter() {
            s.push(' ');
            s.push_str(&BASE64_STANDARD.encode(*bytes));
            v.push(Vec::from(*bytes));
        }
        self.receipt.log_datas.push((self.receipt.active_program(), v));
        self.receipt.push_msg(s);
    }

    fn sol_log_compute_units(&mut self) {
        self.receipt.push_msg("Compute unist mot available".to_string());
    }

    fn invoke_signed_unchecked(&mut self, instruction: &Instruction, account_infos: &[AccountInfo], signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        let instruction = unsafe { std::mem::transmute::<&Instruction, &'static Instruction>(instruction) };
        let account_infos = unsafe { std::mem::transmute::<&[AccountInfo], &'static [AccountInfo<'static>]>(account_infos) };
        let inline_program: bool = instruction.program_id.eq(&solana_program::system_program::ID);

        let mut signed: Option<Pubkey> = None;
        if signers_seeds.len() > 0 {
            let generated_key = Pubkey::create_program_address(
                signers_seeds[0],
                &self.get_program_id()
            );
            if generated_key.is_err() {
                self.receipt.push_msg(format!("Invalid Seeds"));
                return Err(ProgramError::InvalidSeeds);
            }

            signed = Some(generated_key.unwrap());
        }

        let account_refs: HashMap<&Pubkey, &AccountInfo> = account_infos.iter()
            .map(|info| (info.key, info))
            .collect();

        let mut accounts: Vec<AccountInfo> = Vec::with_capacity(instruction.accounts.len());
        for account_meta in instruction.accounts.iter() {
            let account_ref = account_refs.get(&account_meta.pubkey);
            if account_ref.is_none() {
                self.receipt.push_msg(format!("Missing Account {}", account_meta.pubkey.to_string()));
                return Err(ProgramError::NotEnoughAccountKeys);
            }

            let account_ref = *account_ref.unwrap();

            if (!account_ref.is_signer) & (account_meta.is_signer) {
                if !signed.unwrap().eq(&account_meta.pubkey) {
                    self.receipt.push_msg(format!("Signer Privilege escalated for {}", account_meta.pubkey.to_string()));
                    return Err(ProgramError::MissingRequiredSignature);
                }
            }

            if (!account_ref.is_writable) & account_meta.is_writable {
                self.receipt.push_msg(format!("Writable Privilege escalated for {}", account_meta.pubkey.to_string()));
                return Err(ProgramError::InvalidInstructionData);
            }

            accounts.push(self.replicate_info(
                account_ref,
                account_meta,
                !inline_program
            ));
        }

        if !account_refs.contains_key(&instruction.program_id) {
            self.receipt.push_msg(format!("Missing Program {}", instruction.program_id.to_string()));
            return Err(ProgramError::InvalidInstructionData);
        }
        if self.receipt.call_stack.len() > 3 {
            self.receipt.push_msg("Call Stack depth exceeded".to_string());
            return Err(ProgramError::Custom(0x0));
        }
        self.receipt.call_stack.push(instruction.program_id.clone());
        self.receipt.log_program_invoked(&instruction.program_id);
        self.receipt.return_data = None;


        println!("\n==== Before ====");
        for meta in self.metas.iter() {
            println!("Meta {}", meta.1.to_string());
        }
        println!("++ Old Accounts ++");
        for acc in accounts.iter() {
            println!("Account {:?}", acc);
        }
        println!("++ New Accounts ++");
        for acc in account_infos.iter() {
            println!("Account {:?}", acc);
        }
        println!("\n");

        let res = unsafe {
            self.get_package(&instruction.program_id)
                .execute(
            unsafe { std::mem::transmute::<&[AccountInfo], &'static [AccountInfo<'static>]>(accounts.as_slice()) },
                instruction.data.as_slice(),
                &instruction.program_id,
                self.clone()
            )
        };

        println!("==== After ====");
        for meta in self.metas.iter() {
            println!("Meta {}", meta.1.to_string());
        }
        println!("++ Old Accounts ++");
        for acc in accounts.iter() {
            println!("Account {:?}", acc);
        }
        println!("++ New Accounts ++");
        for acc in account_infos.iter() {
            println!("Account {:?}", acc);
        }
        println!("\n");

        if instruction.program_id.eq(&solana_program::system_program::ID) {
            self.finalize_system_invoke(
                unsafe { std::mem::transmute::<&[AccountInfo], &'static [AccountInfo<'static>]>(accounts.as_slice()) }
            )?;
        }

        match &res {
            Ok(_) => {self.receipt.log_program_succeed();}
            Err(err) => {self.receipt.log_program_failed(err.clone());}
        }
        self.receipt.call_stack.pop();

        res
    }

    fn set_return_data(&mut self, data: &[u8]) {
        if data.len() > MAX_RETURN_DATA {
            panic!("Return Data exceeded length");
        }
        self.receipt.return_data = Some((self.receipt.active_program(), Vec::from(data)))
    }

    fn get_return_data(&mut self) -> Option<(Pubkey, Vec<u8>)> {
        self.receipt.return_data.clone()
    }

    fn get_clock(&mut self) -> Clock {
       self.settings.as_clock()
    }

    fn get_rent(&mut self) -> Rent {
        self.settings.as_rent()
    }

    fn set_owner(&mut self, to: &Pubkey, owner: &Pubkey) {
        if !self.metas.contains_key(to) {
            panic!("Undefined Account {}", to.to_string());
        }
        let mut meta = self.metas.get_mut(to).unwrap();
        meta.set_owner(owner);
    }

    fn set_data(&mut self, info: &AccountInfo, data: Vec<u8>) {
        if !self.metas.contains_key(info.key) {
            panic!("Undefined Account {}", info.key.to_string());
        }
        let mut meta = self.metas.get_mut(info.key).unwrap();
        meta.set_data(info, data);
    }

    fn set_lamports(&mut self, to: &Pubkey, amount: u64) {
        if !self.metas.contains_key(to) {
            panic!("Undefined Account {}", to.to_string());
        }
        let mut meta = self.metas.get_mut(to).unwrap();
        meta.set_lamports(amount);
    }

    fn get_program_id(&self) -> Pubkey {
        self.receipt.active_program()
    }

    fn finalize_system_invoke(&mut self, accounts: &[AccountInfo]) -> ProgramResult {
        for acc in accounts.iter() {
            if acc.lamports() < self.rent_exempt_for_size(acc.data_len()) {
                self.receipt.push_msg(format!("Account {} is not rent exempt", acc.key.to_string()));
                return Err(ProgramError::AccountNotRentExempt);
            }
        }

        Ok(())
    }

    fn rent_exempt_for_size(&mut self, size: usize) -> u64 {
        let rent = self.get_rent();
        rent.lamports_per_byte_year * (rent.exemption_threshold as u64) * (size as u64)
    }
    
    fn get_data_ptr(&mut self, key: &Pubkey) -> AtomicPtr<Vec<u8>> {
        if !self.metas.contains_key(key) {
            panic!("Undefined Account {}", key.to_string());
        }
        let mut meta = self.metas.get_mut(key).unwrap();
        meta.get_data_ptr()
    }
}
