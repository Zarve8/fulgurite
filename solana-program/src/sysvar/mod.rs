use {
    crate::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
};

pub mod clock;
pub mod rent;


pub trait SysvarId {
    fn id() -> Pubkey;

    fn check_id(pubkey: &Pubkey) -> bool;
}


pub trait Sysvar:
SysvarId + Default + Sized + serde::Serialize + serde::de::DeserializeOwned {
    fn size_of() -> usize;
    fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> ;
    fn to_account_info(&self, account_info: &mut AccountInfo) -> Option<()>;
    fn get() -> Result<Self, ProgramError>;
}

/*
pub trait Sysvar:
SysvarId + Default + Sized + serde::Serialize + serde::de::DeserializeOwned {
    fn size_of() -> usize {
        0
    }

    fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        Err(ProgramError::UnsupportedSysvar)
    }

    fn to_account_info(&self, account_info: &mut AccountInfo) -> Option<()> {
        None
    }

    fn get() -> Result<Self, ProgramError> {
        Err(ProgramError::UnsupportedSysvar)
    }
}

 */