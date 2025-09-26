use pinocchio::program_error::ProgramError;

/// Errors out if the condition isn't true
///
/// NOTE: The program has too many InvalidInstructionData
pub fn require(condition: bool, err: ProgramError) -> Result<(), ProgramError> {
    if !condition {
        return Err(err);
    }
    Ok(())
}
