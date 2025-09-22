use {
    pinocchio::program_error::{ProgramError, ToStr},
    thiserror_no_std::Error,
};

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum EscrowErrors {
    #[error("Couldn't mut borrow account data")]
    BorrowInvalid,
    #[error("Invalid Mint")]
    InvalidMint,
    #[error("Invalid escrow provided")]
    InvalidEscrow,
    #[error("Invalid token amount")]
    InvalidBalance,
}

impl From<EscrowErrors> for ProgramError {
    fn from(e: EscrowErrors) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl ToStr for EscrowErrors {
    fn to_str<E>(&self) -> &'static str {
        match self {
            EscrowErrors::BorrowInvalid => "Load Error: Couldn't mut borrow account data",
            EscrowErrors::InvalidMint => "Validation Error: Invalid Mint",
            EscrowErrors::InvalidEscrow => "Validation Error: Invalid escrow provided",
            EscrowErrors::InvalidBalance => "Validation Error: Invalid Token amount",
        }
    }
}
