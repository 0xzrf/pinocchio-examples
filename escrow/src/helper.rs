use pinocchio::{program_error::ProgramError, ProgramResult};

/// Checks a boolean expression and returns an error if the expression is false.
///
/// # Arguments
///
/// * `condition` - The boolean expression to evaluate.
/// * `err` - The `ProgramError` to return if `exp` is false.
///
/// # Returns
///
/// * `Ok(())` if `exp` is true.
/// * `Err(ProgramError)` if `exp` is false.
pub fn require(condition: bool, err: ProgramError) -> ProgramResult {
    if !condition {
        return Err(err);
    }
    Ok(())
}
