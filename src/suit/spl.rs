use solana_program::{
    program_option::COption,
    pubkey::Pubkey
};
pub use spl_token::state::Account as SPLAccount;
pub use spl_token::state::Mint as SPLMint;
pub use spl_token::state::AccountState as SPLAccountState;


#[macro_export]
macro_rules! mint_account {
    ($decimals:expr, $owner:expr) => {
        $crate::suit::TypedAccount::new_packed(
            $crate::solana_program::pubkey::Pubkey::new_rand(),
            &crate::suit::program_ids::SPL_PROGRAM_ID,
            $crate::suit::spl::new_mint_struct($owner.pubkey(), 0)
        )
    };
    ($key:expr, $decimals:expr, $owner:expr) => {
        $crate::suit::TypedAccount::new_packed(
            $key,
            &crate::suit::program_ids::SPL_PROGRAM_ID,
            $crate::suit::spl::new_mint_struct($owner.pubkey(), $decimals)
        )
    };
}


#[macro_export]
macro_rules! token_account {
    ($mint:expr, $owner:expr) => {
        $crate::suit::TypedAccount::new_packed(
            spl_associated_token_account::get_associated_token_address($owner.pubkey(), $mint.pubkey()),
            &crate::suit::program_ids::SPL_PROGRAM_ID,
            $crate::suit::spl::new_token_account_struct($mint.pubkey(), $owner.pubkey())
        )
    };
    ($key:expr, $mint:expr, $owner:expr) => {
        $crate::suit::TypedAccount::new_packed(
            $key,
            &crate::suit::program_ids::SPL_PROGRAM_ID,
            $crate::suit::spl::new_token_account_struct($mint.pubkey(), $owner.pubkey())
        )
    };
}


#[macro_export]
macro_rules! mint{
    ($mint:expr, $account:expr, $amount:expr) => {
        $mint.supply += $amount;
        $account.amount += $amount;
    }
}


#[macro_export]
macro_rules! burn{
    ($mint:expr, $account:expr, $amount:expr) => {
        if $account.amount < $amount {
            $mint.supply -= $account.amount;
            $account.amount = 0;
        }
        else {
            $mint.supply -= $amount;
            $account.amount -= $amount;
        }
    }
}


#[macro_export]
macro_rules! transfer_spl {
    ($from:expr, $to:expr, $amount:expr) => {
        if $from.amount < $amount {
            $from.amount = 0;
            $to.amount += $amount;
        }
        else {
            $from.amount -= $amount;
            $to.amount += $amount;
        }
    }
}


pub fn new_mint_struct(owner: &Pubkey, decimals: u8) -> SPLMint {
    SPLMint {
        mint_authority: COption::Some(owner.clone()),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: COption::None
    }
}

pub fn new_token_account_struct(mint_key: &Pubkey, owner: &Pubkey) -> SPLAccount {
    SPLAccount {
        mint: mint_key.clone(),
        owner: owner.clone(),
        amount: 0,
        delegate: COption::None,
        state: SPLAccountState::Initialized,
        is_native: COption::None, //TODO research
        delegated_amount: 0,
        close_authority: COption::None
    }
}
