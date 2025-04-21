use simple_pool_contract::{
    instruction::PoolInstruction,
    pool::Pool,
    token::get_vault_account_address
};
use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use crate::{account, mint, mint_account, token_account};
use crate::runtime::Program;
use crate::suit::borshed_wrapper::BorshedWrapper;
use crate::suit::{SPLAccount, SYSTEM_PROGRAM_ID, TypedAccount};
use crate::suit::packed_wrapper::PackedWrapper;


#[test]
fn test_create_pool () {
    let mut pool_program = Program::inline(simple_pool_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut system_program = Program::system_program();
    let mut token_program = Program::token_program();
    let mut sysvar_program = Program::sysvar_program();

    let mut owner = account!(1000000000);
    let mut mint1 = mint_account!(0, owner);
    let mut mint2 = mint_account!(3, owner);

    let mut pool_acc = account!(Pool::find_address(pool_program.pubkey(), owner.pubkey(), mint1.pubkey(), mint2.pubkey()), 0);
    let mut vault1 = account!(get_vault_account_address(pool_program.pubkey(), mint1.pubkey()), 0);
    let mut vault2 = account!(get_vault_account_address(pool_program.pubkey(), mint2.pubkey()), 0);

    let receipt = pool_program.invoke_with_borsh(
      &PoolInstruction::Create,
        vec![
            owner.meta(true, true),
            system_program.meta(),
            token_program.meta(),
            sysvar_program.meta(),
            pool_acc.meta(false, true),
            mint1.meta(false, true),
            mint2.meta(false, true),
            vault1.meta(false, true),
            vault2.meta(false, true)
        ]
    );

    println!("{:?}", receipt);
    receipt.expect_ok();

    let pool: TypedAccount<BorshedWrapper<Pool>> = TypedAccount::from_account_borshed(pool_acc).unwrap();
    assert_eq!(&pool.owner, owner.pubkey());
    assert_eq!(&pool.mint_left, mint1.pubkey());
    assert_eq!(&pool.mint_right, mint2.pubkey());
    assert_eq!(&pool.vault_left, vault1.pubkey());
    assert_eq!(&pool.vault_right, vault2.pubkey());
    assert_eq!(pool.amount_left, 0);
    assert_eq!(pool.amount_right, 0);
}

#[test]
fn test_add_liquidity() {
    let mut pool_program = Program::inline(simple_pool_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut token_program = Program::token_program();

    let mut owner = account!(1000000000);
    let mut mint1 = mint_account!(0, owner);
    let mut mint2 = mint_account!(3, owner);
    let mut token_acc1 = token_account!(mint1, owner);
    let mut token_acc2 = token_account!(mint2, owner);
    mint!(mint1, token_acc1, 1000);
    mint!(mint2, token_acc2, 1000000);

    let mut pool = TypedAccount::new_borshed(
        Pool::find_address(pool_program.pubkey(), owner.pubkey(), mint1.pubkey(), mint2.pubkey()),
        pool_program.pubkey(),
        Pool::new(
            owner.pubkey().clone(),
            mint1.pubkey().clone(),
            mint2.pubkey().clone(),
            SYSTEM_PROGRAM_ID.clone(),
            SYSTEM_PROGRAM_ID.clone(),
        )
    );

    let mut vault1 = token_account!(get_vault_account_address(pool_program.pubkey(), mint1.pubkey()), mint1, pool);
    let mut vault2 = token_account!(get_vault_account_address(pool_program.pubkey(), mint2.pubkey()), mint2, pool);
    pool.vault_left = vault1.pubkey().clone();
    pool.vault_right = vault2.pubkey().clone();
    pool.bump = Pool::find_bump(pool_program.pubkey(), owner.pubkey(), mint1.pubkey(), mint2.pubkey());

    let receipt = pool_program.invoke_with_borsh(
        &PoolInstruction::ProvideLiquidity {amount_left: 1000, amount_right: 1000000},
        vec![
            owner.meta(true, true),
            token_program.meta(),
            pool.meta(false, true),
            mint1.meta(false, true),
            mint2.meta(false, true),
            vault1.meta(false, true),
            vault2.meta(false, true),
            token_acc1.meta(false, true),
            token_acc2.meta(false, true)
        ]
    );

    println!("{:?}", receipt);
    receipt.expect_ok();
    assert_eq!(pool.amount_left, 1000);
    assert_eq!(pool.amount_right, 1000000);
    assert_eq!(vault1.amount, 1000);
    assert_eq!(vault2.amount, 1000000);
    assert_eq!(token_acc1.amount, 0);
    assert_eq!(token_acc2.amount, 0);
}

#[test]
fn test_exchange() {
    let mut pool_program = Program::inline(simple_pool_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut system_program = Program::system_program();
    let mut token_program = Program::token_program();
    let mut associated_program = Program::associated_token_program();
    let mut sysvar_program = Program::sysvar_program();

    let mut owner = account!(1000000000);
    let mut mint1 = mint_account!(0, owner);
    let mut mint2 = mint_account!(3, owner);
    let mut token_acc1 = token_account!(mint1, owner);
    let mut token_acc2 = account!(get_associated_token_address(owner.pubkey(), mint2.pubkey()), 0);
    mint!(mint1, token_acc1, 10);

    let mut pool = TypedAccount::new_borshed(
        Pool::find_address(pool_program.pubkey(), owner.pubkey(), mint1.pubkey(), mint2.pubkey()),
        pool_program.pubkey(),
        Pool::new(
            owner.pubkey().clone(),
            mint1.pubkey().clone(),
            mint2.pubkey().clone(),
            SYSTEM_PROGRAM_ID.clone(),
            SYSTEM_PROGRAM_ID.clone(),
        )
    );

    let mut vault1 = token_account!(get_vault_account_address(pool_program.pubkey(), mint1.pubkey()), mint1, pool);
    let mut vault2 = token_account!(get_vault_account_address(pool_program.pubkey(), mint2.pubkey()), mint2, pool);
    mint!(mint1, vault1, 1000);
    mint!(mint2, vault2, 1000000);

    pool.vault_left = vault1.pubkey().clone();
    pool.vault_right = vault2.pubkey().clone();
    pool.bump = Pool::find_bump(pool_program.pubkey(), owner.pubkey(), mint1.pubkey(), mint2.pubkey());
    pool.amount_left = 1000;
    pool.amount_right = 1000000;

    let receipt = pool_program.invoke_with_borsh(
        &PoolInstruction::Exchange {amount_left: 10, amount_right: 0},
        vec![
            owner.meta(true, true),
            system_program.meta(),
            token_program.meta(),
            associated_program.meta(),
            sysvar_program.meta(),
            pool.meta(false, true),
            mint1.meta(false, true),
            mint2.meta(false, true),
            vault1.meta(false, true),
            vault2.meta(false, true),
            token_acc1.meta(false, true),
            token_acc2.meta(false, true)
        ]
    );

    println!("{:?}", receipt);
    receipt.expect_ok();
    assert_eq!(pool.amount_left, 1010);
    assert_eq!(pool.amount_right, 990100);
    assert_eq!(vault1.amount, 1010);
    assert_eq!(vault2.amount, 990100);
    assert_eq!(token_acc1.amount, 0);

    let token_acc2: TypedAccount<PackedWrapper<SPLAccount>> = TypedAccount::<PackedWrapper<SPLAccount>>::from_account_packed(token_acc2).unwrap();
    assert_eq!(token_acc2.amount, 9900);
}

