use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;


#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum PoolError {
    #[error("PoolNotExists")]
    PoolNotExists,

    #[error("PoolAlreadyCreated")]
    PoolAlreadyCreated,

    #[error("InvalidMintAccountsOrder")]
    InvalidMintAccountsOrder,

    #[error("ZeroAmountNotAllowed")]
    ZeroAmountNotAllowed,

    #[error("OnlyOneWayExchangeAllowed")]
    OnlyOneWayExchangeAllowed,

    #[error("NotAPoolOwner")]
    NotAPoolOwner,

    #[error("InvalidPoolAccountPDA")]
    InvalidPoolAccountPDA,

    #[error("InvalidVaultAccountPDA")]
    InvalidVaultAccountPDA,

    #[error("InvalidAssociatedAccountPDA")]
    InvalidAssociatedAccountPDA,

    #[error("InvalidVault")]
    InvalidVault,

    #[error("InvalidMint")]
    InvalidMint,

    #[error("NotEnoughLiquidity")]
    NotEnoughLiquidity,
}


impl PrintProgramError for PoolError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<PoolError> for ProgramError {
    fn from(e: PoolError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for PoolError {
    fn type_of() -> &'static str {
        "Pool Error"
    }
}


