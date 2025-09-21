use pinocchio::{program_error::ProgramError, ProgramResult};

/// Checks a boolean expression and returns an error if the expression is false.
///
/// # Arguments
///
/// * `exp` - The boolean expression to evaluate.
/// * `err` - The `ProgramError` to return if `exp` is false.
///
/// # Returns
///
/// * `Ok(())` if `exp` is true.
/// * `Err(ProgramError)` if `exp` is false.
///
/// # Example
///
/// ```
/// require(some_condition, ProgramError::InvalidArgument)?;
/// ```
pub fn require(exp: bool, err: ProgramError) -> ProgramResult {
    if !exp {
        return Err(err);
    }
    Ok(())
}
