use std::mem::forget;
use borsh::BorshSerialize;
use crate::{executor::package::program_utils::{
    limited_deserialize,
}};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::InstructionError,
    pubkey::Pubkey,
    syscalls::Syscalls,
    system_instruction::{MAX_PERMITTED_DATA_LENGTH, SystemError, SystemInstruction},
    system_program,
    program_error::ProgramError
};
use crate::executor::Package;
use crate::executor::package::program_utils::convert_instruction_error;


pub struct SystemProgramPackage {}

impl Package for SystemProgramPackage {
    fn execute<'e>(&self, accounts: &'e [AccountInfo<'e>], instruction_data: &'e [u8], _program_id: &'e Pubkey, mut syscalls: Box<dyn Syscalls>) -> ProgramResult {
        let res = Self::processor(accounts, instruction_data, &mut syscalls)
        .map_err(|err| convert_instruction_error(err));
        forget(syscalls); // Owned by test scope
        res
    }
}

impl SystemProgramPackage {
    fn processor<'e>(accounts: &'e [AccountInfo<'e>], instruction_data: &'e [u8], mut syscalls: &mut Box<dyn Syscalls>) -> Result<(), InstructionError> {
        let instruction: SystemInstruction = limited_deserialize(instruction_data)?;

        match instruction {
            SystemInstruction::CreateAccount {
                lamports,
                space,
                owner,
            } => {

                Self::check_number_of_instruction_accounts(&accounts, 2)?;
                Self::create_account(&accounts[0], &accounts[1], lamports, space, &owner, syscalls)
            },
            SystemInstruction::Assign { owner } => {
                Self::check_number_of_instruction_accounts(&accounts, 1)?;
                Self::assign(&accounts[0], &owner, syscalls)
            },
            SystemInstruction::Transfer { lamports } => {
                Self::check_number_of_instruction_accounts(&accounts, 2)?;
                Self::transfer(&accounts[0], &accounts[1], lamports, syscalls)
            },
            SystemInstruction::Allocate { space } => {
                Self::check_number_of_instruction_accounts(&accounts, 1)?;
                Self::allocate(&accounts[0], space, syscalls)
            },
            _ => {
                syscalls.sol_log(&format!("Unimplemented instruction {:?}", instruction));
                return Err(InstructionError::GenericError)
            }
        }
    }

    fn allocate<'a>(info: &'a AccountInfo<'a>, space: u64, syscalls: &mut Box<dyn Syscalls>) -> Result<(), InstructionError> {
        if !info.is_signer {
            syscalls.sol_log(&format!("Allocate: 'to' account {:?} must sign", info.key));
            return Err(InstructionError::MissingRequiredSignature);
        }

        if !info.data_is_empty() || !system_program::check_id(info.owner) {
            syscalls.sol_log(&format!("Allocate: account {:?} already in use", info.key));
            return Err(SystemError::AccountAlreadyInUse.into());
        }

        if space > MAX_PERMITTED_DATA_LENGTH {
            syscalls.sol_log(&format!(
                "Allocate: requested {}, max allowed {}",
                space,
                MAX_PERMITTED_DATA_LENGTH
            ));
            return Err(SystemError::InvalidAccountDataLength.into());
        }

        // info.realloc(space as usize, true).unwrap();
        syscalls.set_data(info, vec![0; space as usize]);
        Ok(())
    }

    fn assign<'a>(info: &'a AccountInfo<'a>, owner: &Pubkey, syscalls: &mut Box<dyn Syscalls>) -> Result<(), InstructionError> {
        if info.owner.eq(owner) {
            return Ok(());
        }

        if !info.is_signer {
            syscalls.sol_log(&format!("Assign: account {:?} must sign", info.key));
            return Err(InstructionError::MissingRequiredSignature);
        }

        syscalls.set_owner(&info.key, owner);
        Ok(())
    }

    fn create_account<'a>(from: &'a AccountInfo<'a>, to: &'a AccountInfo<'a>, lamports: u64, space: u64, owner: &Pubkey, syscalls: &mut Box<dyn Syscalls>) -> Result<(), InstructionError> {
        if to.lamports() > 0 {
            syscalls.sol_log(&format!(
                "Create Account: account {:?} already in use",
                to.key
            ));
            return Err(SystemError::AccountAlreadyInUse.into());
        }

        Self::allocate(to, space, syscalls)?;
        Self::assign(to, owner, syscalls)?;
        Self::transfer(from, to, lamports, syscalls)
    }

    fn transfer<'a>(from: &'a AccountInfo<'a>, to: &'a AccountInfo<'a>, lamports: u64, syscalls: &mut Box<dyn Syscalls>) -> Result<(), InstructionError> {
        if !from.is_signer {
            syscalls.sol_log(&format!("Transfer: `from` account {:?} must sign", from.key));
            return Err(InstructionError::MissingRequiredSignature);
        }

        if from.executable {
            return Err(InstructionError::ExecutableLamportChange.into());
        }
        if !from.data_is_empty() {
            syscalls.sol_log(&format!("Transfer: `from` must not carry data"));
            return Err(InstructionError::InvalidArgument.into());
        }
        if !from.is_writable {
            return Err(InstructionError::ReadonlyLamportChange.into());
        }

        if lamports > from.lamports() {
            syscalls.sol_log(&format!("Transfer: insufficient lamports {}, need {}",
            from.lamports(),
            lamports));
            return Err(SystemError::ResultWithNegativeLamports.into());
        }

        syscalls.set_lamports(&from.key, from.lamports() - lamports);
        syscalls.set_lamports(&to.key, to.lamports() + lamports);

        Ok(())
    }

    fn check_number_of_instruction_accounts<'e>(accounts: &[AccountInfo<'e>], count: usize) -> Result<(), InstructionError> {
        if accounts.len() < count {
            return Err(InstructionError::NotEnoughAccountKeys);
        }
        Ok(())
    }
}