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

pub fn bps_mul(bps: u64, value: u64, divisor: u64) -> Option<u64> {
    bps_mul_raw(bps, value, divisor).unwrap().try_into().ok()
}

pub fn bps_mul_raw(bps: u64, value: u64, divisor: u64) -> Option<u128> {
    (value as u128)
        .checked_mul(bps as u128)?
        .checked_div(divisor as u128)
}
