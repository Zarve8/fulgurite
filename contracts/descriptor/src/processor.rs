use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg, pubkey::Pubkey,
    program_error::ProgramError,
    system_instruction,
    program::invoke
};
use crate::counter::Counter;
use crate::instruction::DescriptorInstruction;
use borsh::BorshDeserialize;
use solana_program::log::{sol_log, sol_log_64, sol_log_data};


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

        let instruction = DescriptorInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            DescriptorInstruction::ReadToLog => {
                let counter_ai = next_account_info(accounts_iter)?;
                let counter = Counter::load(counter_ai)?;
                counter.log();
                Ok(())
            },
            DescriptorInstruction::WriteData {value} => {
                let counter_ai = next_account_info(accounts_iter)?;
                let mut counter = Counter::load(counter_ai)?;
                assert(counter_ai.is_writable, "Counter not writable")?;
                counter.value = value;
                counter.save(counter_ai)?;
                Ok(())
            },
            DescriptorInstruction::CreateAccount => {
                let counter_ai = next_account_info(accounts_iter)?;
                let system_program = next_account_info(accounts_iter)?;
                assert(counter_ai.is_signer, "Counter not a signer")?;
                assert(counter_ai.is_writable, "Counter not writable")?;
                let counter = Counter::new();
                counter.create(program_id, payer, counter_ai, system_program)?;
                msg!("Ai {:?}", counter_ai);
                assert(counter_ai.owner.eq(program_id), "Counter owner program not assigned")?;
                assert(counter_ai.data_len() == counter.size(), "Counter data not allocated")
            },
            DescriptorInstruction::CreateAccountPDA => {
                let counter_ai = next_account_info(accounts_iter)?;
                let system_program = next_account_info(accounts_iter)?;
                let counter = Counter::new();
                assert(counter_ai.is_writable, "Counter not writable")?;
                counter.create_pda(program_id, payer, counter_ai, system_program)?;
                assert(counter_ai.owner.eq(program_id), "Counter owner program not assigned")?;
                assert(counter_ai.data_len() == counter.size(), "Counter data not allocated")
            },
            DescriptorInstruction::VerifySigner => {
                assert(next_account_info(accounts_iter)?.is_signer, "Signer not signed")
            },
            DescriptorInstruction::TransferSol {amount} => {
                let recipient_ai = next_account_info(accounts_iter)?;
                let system_program = next_account_info(accounts_iter)?;
                assert(payer.lamports() >= amount, "Not enough Sol")?;
                let balance_before_from = payer.lamports();
                let balance_before_to = recipient_ai.lamports();

                let idx = system_instruction::transfer(
                    payer.key,
                    recipient_ai.key,
                    amount,
                );
                invoke(
                    &idx,
                    &[payer.clone(), recipient_ai.clone(), system_program.clone()])?;

                assert(payer.lamports() == (balance_before_from - amount), "Lamprots not charged from payer")?;
                assert(recipient_ai.lamports() == (balance_before_to + amount), "Lamports not transfered to recipient")
            }
            _ => {msg!("Instruction not implemented"); Ok(())}
        }
    }
}