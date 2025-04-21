pub use crate::rent::Rent;
use crate::{pubkey, syscalls, sysvar::Sysvar};
use crate::account_info::AccountInfo;
use crate::program_error::ProgramError;
use crate::pubkey::Pubkey;
use crate::sysvar::SysvarId;

const RENT_ID: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");

pub fn id() -> Pubkey {
    RENT_ID.clone()
}

impl SysvarId for Rent {
    fn id() -> Pubkey {
        RENT_ID.clone()
    }

    fn check_id(pubkey: &Pubkey) -> bool {
        RENT_ID.eq(pubkey)
    }
}

impl Sysvar for Rent {
    fn size_of() -> usize {
        0x1337
    }

    fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        if !RENT_ID.eq(account_info.key) {
            return Err(ProgramError::UnsupportedSysvar);
        }

        Ok(syscalls!().get_rent())
    }

    fn to_account_info(&self, account_info: &mut AccountInfo) -> Option<()> {
        Some(())
    }

    fn get() -> Result<Self, ProgramError> {
        Ok(syscalls!().get_rent())
    }
}
