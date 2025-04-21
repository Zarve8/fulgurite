use crate::{account, burn, mint, mint_account, token_account, transfer_spl};
use solana_program::pubkey::Pubkey;
use crate::suit::*;

#[test]
fn test_account_init() {
    let acc1 = account!();
    acc1.expect_balance(0);
    acc1.expect_owner(&SYSTEM_PROGRAM_ID);

    let acc2 = account!(1000);
    acc2.expect_balance(1000);
    acc2.expect_owner(&SYSTEM_PROGRAM_ID);

    let acc3 = account!(ASSOCIATED_PROGRAM_ID.clone(), 1234);
    acc3.expect_balance(1234);
    assert!(acc3.pubkey.eq(&ASSOCIATED_PROGRAM_ID));

    let acc4 = account!(ASSOCIATED_PROGRAM_ID.clone(), &SYSVAR_PROGRAM_ID, vec![1, 2, 3, 4]);
    acc4.expect_balance(rent_exempt_for_size(4));
}

#[test]
fn test_program_init() {}

#[test]
fn spl_token_init() {
    let owner1 = account!();
    let mut mint1 = mint_account!(0, owner1);
    let mut mint2 = mint_account!(Pubkey::new_rand(), 9, owner1);

    let mut account1 = token_account!(mint1, owner1);
    let mut account2 = token_account!(Pubkey::new_rand(), mint2, owner1);

    mint!(mint1, account1, 100);
    burn!(mint1, account1, 50);
    transfer_spl!(account1, account2, 25);
}
