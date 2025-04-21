use crate::account_info::AccountInfo;
use crate::syscalls;

/// Print a message to the log.
#[macro_export]
macro_rules! info {
    ($msg:expr) => {
        $crate::log::sol_log($msg)
    };
    ($arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr) => {
        $crate::log::sol_log_64(
            $arg1 as u64,
            $arg2 as u64,
            $arg3 as u64,
            $arg4 as u64,
            $arg5 as u64,
        )
    };
}

#[macro_export]
macro_rules! msg {
    ($msg:expr) => {
        $crate::log::sol_log($msg)
    };
    ($($arg:tt)*) => ($crate::log::sol_log(&format!($($arg)*)));
}

/// Print a string to the log.
#[inline]
pub fn sol_log(message: &str) {
    syscalls!().sol_log(message)
}

/// Print 64-bit values represented as hexadecimal to the log.
#[inline]
pub fn sol_log_64(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) {
    syscalls!().sol_log_64(arg1, arg2, arg3, arg4, arg5)
}

/// Print some slices as base64.
pub fn sol_log_data(data: &[&[u8]]) {
    syscalls!().sol_log_data(data)
}

#[allow(dead_code)]
pub fn sol_log_slice(slice: &[u8]) {
    for (i, s) in slice.iter().enumerate() {
        sol_log_64(0, 0, 0, i as u64, *s as u64);
    }
}

/// Print the hexadecimal representation of the program's input parameters.
///
/// - `accounts` - A slice of [`AccountInfo`].
/// - `data` - The instruction data.
pub fn sol_log_params(accounts: &[AccountInfo], data: &[u8]) {
    for (i, account) in accounts.iter().enumerate() {
        msg!("AccountInfo");
        sol_log_64(0, 0, 0, 0, i as u64);
        msg!("- Is signer");
        sol_log_64(0, 0, 0, 0, account.is_signer as u64);
        msg!("- Key");
        account.key.log();
        msg!("- Lamports");
        sol_log_64(0, 0, 0, 0, account.lamports());
        msg!("- Account data length");
        sol_log_64(0, 0, 0, 0, account.data_len() as u64);
        msg!("- Owner");
        account.owner.log();
    }
    msg!("Instruction data");
    sol_log_slice(data);
}

/// Print the remaining compute units available to the program.
#[inline]
pub fn sol_log_compute_units() {
    syscalls!().sol_log_compute_units()
}
