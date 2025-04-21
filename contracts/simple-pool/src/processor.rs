use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg, pubkey::Pubkey,
    program_error::ProgramError,
};
use crate::instruction::{PoolInstruction};
use borsh::{BorshDeserialize};
use crate::error::PoolError;
use crate::pool::Pool;
use crate::token::{find_or_create_associated_account, find_or_create_vault, give_token, take_token};


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
        let payer = next_account_info(accounts_iter)?;
        assert(payer.is_signer, "Payer is not signed")?;
        assert(payer.lamports() >= 5000000, "Payer not funded")?;

        let instruction = PoolInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            PoolInstruction::Create => {
                msg!("Create new pool instruction");
                // Programs not checked, only for test purpose
                let system_program = next_account_info(accounts_iter)?;
                let token_program = next_account_info(accounts_iter)?;
                let sysvar = next_account_info(accounts_iter)?;

                let pool_ai = next_account_info(accounts_iter)?;
                if pool_ai.data_len() > 0 {
                    return Err(PoolError::PoolAlreadyCreated.into());
                }

                let mint_left = next_account_info(accounts_iter)?;
                let mint_right = next_account_info(accounts_iter)?;

                let vault_left = next_account_info(accounts_iter)?;
                find_or_create_vault(program_id, payer, pool_ai, mint_left, vault_left, system_program, token_program, sysvar)?;
                let vault_right = next_account_info(accounts_iter)?;
                find_or_create_vault(program_id, payer, pool_ai, mint_right, vault_right, system_program, token_program, sysvar)?;

                let mut pool = Pool::new(
                    payer.key.clone(),
                    mint_left.key.clone(),
                    mint_right.key.clone(),
                    vault_left.key.clone(),
                    vault_right.key.clone());

                pool.create_pda(program_id, payer, pool_ai, system_program)?;
                pool.save(pool_ai)
            }
            PoolInstruction::ProvideLiquidity { mut amount_left, mut amount_right } => {
                msg!("Provide liquidity instruction");
                let token_program = next_account_info(accounts_iter)?;

                let pool_ai = next_account_info(accounts_iter)?;
                let mut pool = Pool::load(pool_ai)?;
                pool.display();
                if !pool.owner.eq(payer.key) {
                    return Err(PoolError::NotAPoolOwner.into());
                }

                let mint_left = next_account_info(accounts_iter)?;
                let mint_right = next_account_info(accounts_iter)?;
                let vault_left = next_account_info(accounts_iter)?;
                let vault_right = next_account_info(accounts_iter)?;
                pool.validate(mint_left, mint_right, vault_left, vault_right)?;

                let associated_left = next_account_info(accounts_iter)?;
                let associated_right = next_account_info(accounts_iter)?;

                if pool.amount_left > 0 {
                    // R(L + dl) = L(R + dr)
                    amount_right = (pool.amount_right * amount_left) / pool.amount_left;
                    msg!("Using relation for liquidity {} / {}", amount_left, amount_right);
                } else {
                    msg!("Initial liquidity for pool {} / {}", amount_left, amount_right);
                }

                take_token(amount_left, payer, associated_left, vault_left, token_program)?;
                take_token(amount_right, payer,  associated_right, vault_right, token_program)?;

                pool.amount_left += amount_left;
                pool.amount_right += amount_right;
                pool.save(pool_ai)
            }
            PoolInstruction::Exchange { mut amount_left, mut amount_right } => {
                msg!("Provide exchange instruction");
                if amount_left == 0 && amount_right == 0 {
                    return Err(PoolError::ZeroAmountNotAllowed.into());
                }
                if amount_left > 0 && amount_right > 0 {
                    return Err(PoolError::OnlyOneWayExchangeAllowed.into());
                }

                let system_program = next_account_info(accounts_iter)?;
                let token_program = next_account_info(accounts_iter)?;
                let associated_program = next_account_info(accounts_iter)?;
                let sysvar = next_account_info(accounts_iter)?;

                let pool_ai = next_account_info(accounts_iter)?;
                let mut pool = Pool::load(pool_ai)?;

                let mint_left = next_account_info(accounts_iter)?;
                let mint_right = next_account_info(accounts_iter)?;
                let vault_left = next_account_info(accounts_iter)?;
                let vault_right = next_account_info(accounts_iter)?;
                pool.validate(mint_left, mint_right, vault_left, vault_right)?;

                let associated_left = next_account_info(accounts_iter)?;
                if amount_right > 0 {
                    find_or_create_associated_account(
                        payer,
                        mint_left,
                        associated_left,
                        system_program,
                        token_program,
                        associated_program,
                        sysvar)?;
                }
                let associated_right = next_account_info(accounts_iter)?;
                if amount_left > 0 {
                    find_or_create_associated_account(
                        payer,
                        mint_right,
                        associated_right,
                        system_program,
                        token_program,
                        associated_program,
                        sysvar)?;
                }

                if amount_left > 0 { // Left to right
                    // (L + dl)(R - dr) = C = RL
                    amount_right = (amount_left * pool.amount_right) / (pool.amount_left + amount_left);
                    msg!("Exchange {} -> {}", amount_left, amount_right);
                    if pool.amount_right <= amount_right {
                        return Err(PoolError::NotEnoughLiquidity.into());
                    }

                    take_token(amount_left, payer,  associated_left, vault_left, token_program)?;
                    give_token(amount_right, &pool, pool_ai, vault_right, associated_right, token_program)?;

                    pool.amount_left += amount_left;
                    pool.amount_right -= amount_right;
                    msg!("Left in pool {} / {}", pool.amount_left, pool.amount_right);
                }
                else { // Right to left
                    // (L - dl)(R + dr) = C = RL
                    amount_left= (amount_right * pool.amount_left) / (pool.amount_right + amount_right);
                    msg!("Exchange {} <- {}", amount_left, amount_right);
                    if pool.amount_left <= amount_left {
                        return Err(PoolError::NotEnoughLiquidity.into());
                    }

                    take_token(amount_right, payer,  associated_right, vault_right, token_program)?;
                    give_token(amount_right, &pool, pool_ai,  vault_left, associated_left, token_program)?;

                    pool.amount_left -= amount_left;
                    pool.amount_right += amount_right;
                    msg!("Left in pool {} / {}", pool.amount_left, pool.amount_right);
                }

                pool.save(pool_ai)
            }
            _ => {
                msg!("Instruction not implemented");
                Ok(())
            }
        }
    }
}
