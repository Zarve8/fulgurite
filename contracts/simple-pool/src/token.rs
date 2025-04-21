use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    rent::Rent,
    sysvar::Sysvar,
    system_instruction
};
use spl_associated_token_account::get_associated_token_address;
use crate::error::PoolError;
use crate::pool::Pool;


pub fn get_vault_account_address(program_id: &Pubkey, mint_id: &Pubkey) -> Pubkey {
    let (key, _bump) = Pubkey::find_program_address(&[
        "vault".as_bytes(),
        program_id.as_ref(),
        mint_id.as_ref(),
    ], program_id);

    key
}


pub(crate) fn find_or_create_vault<'a>(
    program_id: &Pubkey,
    payer: &'a AccountInfo<'a>,
    pool_ai: &'a AccountInfo<'a>,
    mint_ai: &'a AccountInfo<'a>,
    vault_ai: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
    token_program: &'a AccountInfo<'a>,
    sysvar: &'a AccountInfo<'a>,
) -> ProgramResult {
    let (key, bump) = Pubkey::find_program_address(&[
        "vault".as_bytes(),
        program_id.as_ref(),
        mint_ai.key.as_ref(),
    ], program_id);

    if !key.eq(vault_ai.key) {
        return Err(PoolError::InvalidVaultAccountPDA.into());
    }
    if vault_ai.data_len() > 0 {
        msg!("Vault ai for mint {} already exists", mint_ai.key.to_string());
        return Ok(());
    }

    let seeds: &[&[&[u8]]] = &[&[
        "vault".as_bytes(),
        program_id.as_ref(),
        mint_ai.key.as_ref(),
        &[bump]]];

    let idx = system_instruction::create_account(
        payer.key,
        vault_ai.key,
        Rent::get()?.minimum_balance(165),
        165,
        &token_program.key,
    );
    invoke_signed(
        &idx,
        &[payer.clone(), vault_ai.clone(), system_program.clone()],
        seeds,
    )?;

    let idx = &Instruction {
        accounts: vec![
            AccountMeta::new(*vault_ai.key, false),
            AccountMeta::new_readonly(*mint_ai.key, false),
            AccountMeta::new_readonly(*pool_ai.key, false),
            AccountMeta::new_readonly(*sysvar.key, false),
        ],
        data: Vec::from([1]),
        program_id: *token_program.key,
    };
    invoke(
        &idx,
        &[mint_ai.clone(), vault_ai.clone(), sysvar.clone(), token_program.clone(), pool_ai.clone()],
    )
}


pub(crate) fn find_or_create_associated_account<'a>(
    payer: &'a AccountInfo<'a>,
    mint_ai: &'a AccountInfo<'a>,
    account_ai: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
    token_program: &'a AccountInfo<'a>,
    associated_program: &'a AccountInfo<'a>,
    sysvar: &'a AccountInfo<'a>,
) -> ProgramResult {
    if !get_associated_token_address(payer.key, mint_ai.key).eq(account_ai.key) {
        return Err(PoolError::InvalidAssociatedAccountPDA.into());
    }
    if account_ai.data_len() > 0 {
        msg!("Associated account already exist");
        return Ok(());
    }
    let idx = Instruction {
        program_id: *associated_program.key,
        accounts: vec![
            AccountMeta::new(*payer.key, true),
            AccountMeta::new(*account_ai.key, false),
            AccountMeta::new_readonly(*payer.key, false),
            AccountMeta::new_readonly(*mint_ai.key, false),
            AccountMeta::new_readonly(*system_program.key, false),
            AccountMeta::new_readonly(*token_program.key, false),
            AccountMeta::new_readonly(*sysvar.key, false),
        ],
        data: Vec::from([0]),
    };
    invoke(
        &idx,
        &[payer.clone(), account_ai.clone(), mint_ai.clone(), system_program.clone(),
            token_program.clone(), sysvar.clone(), associated_program.clone()],
    )
}


pub(crate) fn take_token<'a>(
    amount: u64,
    payer: &'a AccountInfo<'a>,
    from_account: &'a AccountInfo<'a>,
    to_vault: &'a AccountInfo<'a>,
    token_program: &'a AccountInfo<'a>,
) -> ProgramResult {
    if amount == 0 { return Ok(()); }
    let idx = spl_token::instruction::transfer(
        &token_program.key,
        &from_account.key,
        &to_vault.key,
        &payer.key,
        &[],
        amount,
    )?;

    invoke(
        &idx,
        &[payer.clone(), to_vault.clone(), token_program.clone(), from_account.clone()],
    )
}


pub(crate) fn give_token<'a>(
    amount: u64,
    pool: &Pool,
    pool_ai: &'a AccountInfo<'a>,
    from_vault: &'a AccountInfo<'a>,
    to_account: &'a AccountInfo<'a>,
    token_program: &'a AccountInfo<'a>,
) -> ProgramResult {
    if amount == 0 { return Ok(()); }

    let idx = spl_token::instruction::transfer(
        &token_program.key,
        &from_vault.key,
        &to_account.key,
        &pool_ai.key,
        &[],
        amount,
    )?;

    pool.invoke_signed(pool_ai, &idx, &[token_program.clone(), from_vault.clone(), to_account.clone(), pool_ai.clone()])
}



