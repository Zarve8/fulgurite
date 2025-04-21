use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg, pubkey::Pubkey,
    program_error::ProgramError,
    system_instruction,
    instruction::{Instruction, AccountMeta},
    program::invoke,
    log::{sol_log, sol_log_data}
};
use crate::instruction::ViewerInstruction;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program::invoke_signed;
use descriptor_contract::{
    instruction::DescriptorInstruction,
    counter:: Counter
};
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;


fn assert(condition: bool, msg: &str) -> ProgramResult {
    if !condition {
        msg!("Error: {}", msg);
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}

pub struct Processor {}

impl Processor {
    pub fn process_instruction<'g>(program_id: &Pubkey, accounts: &'g [AccountInfo<'g>], instruction_data: &[u8]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer= next_account_info(accounts_iter)?;
        assert(payer.is_signer, "Payer is not signed")?;
        assert(payer.lamports() >= 5000000, "Payer not funded")?;

        let instruction = ViewerInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            ViewerInstruction::Log => {
                sol_log("Simple log");
                Ok(())
            },
            ViewerInstruction::LogData => {
                sol_log_data(&[&[1, 2, 3, 4]]);
                Ok(())
            },
            ViewerInstruction::CallAndRead => {
                let counter_ai = next_account_info(accounts_iter)?;
                let descriptor_program = next_account_info(accounts_iter)?;

                let mut bytes: Vec<u8> = Vec::new();
                DescriptorInstruction::WriteData {value: 99}.serialize(&mut bytes)
                    .map_err(|_| ProgramError::BorshIoError("Failed to serialize".to_string()))?;

                invoke(
                    &Instruction::new_with_bytes(
                        *descriptor_program.key,
                        bytes.as_slice(),
                        vec![
                            AccountMeta::new(*payer.key, true),
                            AccountMeta::new(*counter_ai.key, false)
                        ]
                    ),
                    &[
                        payer.clone(),
                        counter_ai.clone(),
                        descriptor_program.clone()
                    ]
                )?;

                let counter = Counter::load(counter_ai)?;
                assert(counter.value == 99, "Counter not updated")
            },
            ViewerInstruction::PDASignature => {
                let pda_account_ai = next_account_info(accounts_iter)?;
                let descriptor_program = next_account_info(accounts_iter)?;

                let (_key, bump) = Pubkey::find_program_address(&[
                    "viewer".as_bytes(),
                    &payer.key.to_bytes(),
                    &program_id.to_bytes(),
                ], program_id);

                let seeds: &[&[&[u8]]] = &[&[
                    "viewer".as_bytes(),
                    &payer.key.to_bytes(),
                    &program_id.to_bytes(),
                    &[bump]]];

                let mut bytes: Vec<u8> = Vec::new();
                DescriptorInstruction::VerifySigner.serialize(&mut bytes)
                    .map_err(|_| ProgramError::BorshIoError("Failed to serialize".to_string()))?;

                invoke_signed(
                    &Instruction::new_with_bytes(
                        *descriptor_program.key,
                        bytes.as_slice(),
                        vec![
                            AccountMeta::new(*payer.key, true),
                            AccountMeta::new_readonly(*pda_account_ai.key, true)
                        ]
                    ),
                    &[payer.clone(), pda_account_ai.clone(), descriptor_program.clone()],
                    seeds
                )
            },
            ViewerInstruction::ReallocAccount { new_size } => {
                let account_ai = next_account_info(accounts_iter)?;
                let system_program = next_account_info(accounts_iter)?;
                assert(account_ai.is_writable, "Account not writable")?;

                let idx = system_instruction::transfer(
                    payer.key,
                    account_ai.key,
                    Rent::get()?.minimum_balance(
                        new_size.saturating_sub(account_ai.data_len())
                    ),
                );
                invoke(
                    &idx,
                    &[payer.clone(), account_ai.clone(), system_program.clone()])?;
                account_ai.realloc(new_size, false)?;

                assert(account_ai.data_len() == new_size, "Account not realloced")
            }
            _ => {msg!("Instruction not implemented"); Ok(())}
        }
    }
}