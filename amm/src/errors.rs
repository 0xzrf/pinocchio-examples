use {
    pinocchio::program_error::{ProgramError, ToStr},
    thiserror_no_std::Error,
};

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum AmmError {
    #[error("Couldn't mut borrow account data")]
    BorrowInvalid,
    #[error("Invalid Mint")]
    InvalidMint,
    #[error("Invalid token amount")]
    InvalidBalance,
}

impl From<AmmError> for ProgramError {
    fn from(e: AmmError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl ToStr for AmmError {
    fn to_str<E>(&self) -> &'static str {
        match self {
            AmmError::BorrowInvalid => "Load Error: Couldn't mut borrow account data",
            AmmError::InvalidMint => "Validation Error: Invalid Mint",
            AmmError::InvalidBalance => "Validation Error: Invalid Token amount",
        }
    }
}
