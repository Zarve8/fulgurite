#[macro_export]
macro_rules! program {
    ($name:expr) => { $crate::runtime::Program::new($name, $crate::solana_program::pubkey::Pubkey::new_rand()) };
    ($name:expr, $key:expr) => { $crate::runtime::Program::new($name, $key) }
}

#[macro_export]
macro_rules! account {
    () => { // Empty Account
        $crate::runtime::Account::new(
            $crate::solana_program::pubkey::Pubkey::new_rand(),
            0,
            &$crate::solana_program::system_program::ID,
            Vec::<u8>::from([])
        )
    };
    ($lamports:expr) => { // Empty Account with lamports
        $crate::runtime::Account::new(
            $crate::solana_program::pubkey::Pubkey::new_rand(),
            $lamports,
            &$crate::solana_program::system_program::ID,
            Vec::<u8>::from([])
        )
    };
    ($key:expr, $lamports:expr) => { // Empty Account with specific key
        $crate::runtime::Account::new(
            $key,
            $lamports,
            &$crate::solana_program::system_program::ID,
            Vec::<u8>::from([])
        )
    };
    ($key:expr, $owner:expr, $data:expr) => { // Account with data
        $crate::runtime::Account::new(
            $key,
            $crate::suit::utils::rent_exempt_for_size($data.len()),
            $owner,
            $data
        )
    };
}
