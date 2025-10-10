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
    #[error("Curve Completed")]
    CurveComplete,
    #[error("Curve Not started yet")]
    CurveNotStarted,
    #[error("Couldn't buy")]
    CouldNotBuy,
    #[error("Couldn't sell")]
    CouldNotSell,
    #[error("Slippage Exceeded")]
    SlippageExceeded,
    #[error("Invariant Failed")]
    InvariantFailed,
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
            AmmError::CurveComplete => "Validation Error: Curve Already Complete",
            AmmError::CurveNotStarted => "Validation Error: Curve Not started yet",
            AmmError::SlippageExceeded => "Swap Error: Slippage exceeded",
            AmmError::CouldNotBuy => "Swap Error: Couldn't buy tokens",
            AmmError::CouldNotSell => "Swap Error: Couldn't sell tokens",
            AmmError::InvariantFailed => "Swap Error: Invariants failed",
        }
    }
}
