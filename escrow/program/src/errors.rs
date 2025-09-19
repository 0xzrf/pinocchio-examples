use {
    pinocchio::program_error::{ProgramError, ToStr},
    thiserror_no_std::Error,
};

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum EscrowErrors {
    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
}

impl From<EscrowErrors> for ProgramError {
    fn from(e: EscrowErrors) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl ToStr for EscrowErrors {
    fn to_str<E>(&self) -> &'static str {
        match self {
            EscrowErrors::NotRentExempt => "Error: Lamport balance below rent-exempt threshold",
        }
    }
}
