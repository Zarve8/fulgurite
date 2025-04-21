pub use crate::clock::Clock;
use crate::{pubkey, syscalls, sysvar::Sysvar};
use crate::account_info::AccountInfo;
use crate::program_error::ProgramError;
use crate::pubkey::Pubkey;
use crate::sysvar::SysvarId;

const CLOCK_ID: Pubkey = pubkey!("SysvarC1ock11111111111111111111111111111111");

impl SysvarId for Clock {
    fn id() -> Pubkey {
        CLOCK_ID.clone()
    }

    fn check_id(pubkey: &Pubkey) -> bool {
        CLOCK_ID.eq(pubkey)
    }
}

impl Sysvar for Clock {
    fn size_of() -> usize {
        0x1337
    }

    fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        if !CLOCK_ID.eq(account_info.key) {
            return Err(ProgramError::UnsupportedSysvar);
        }

        Ok(syscalls!().get_clock())
    }

    fn to_account_info(&self, account_info: &mut AccountInfo) -> Option<()> {
        Some(())
    }


    fn get() -> Result<Self, ProgramError> {
        Ok(syscalls!().get_clock())
    }
}
