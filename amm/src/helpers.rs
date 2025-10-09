use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, log::sol_log_64, msg, program_error::ProgramError};

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

pub fn load<T>(account: &AccountInfo) -> Result<&mut T, ProgramError>
where
    T: Pod + Zeroable,
{
    let data = unsafe { account.borrow_mut_data_unchecked() };

    bytemuck::try_from_bytes_mut::<T>(data).map_err(|_| ProgramError::InvalidAccountData)
}

pub fn load_read_only<T>(account: &AccountInfo) -> Result<&T, ProgramError>
where
    T: Pod + Zeroable,
{
    let data = unsafe { account.borrow_data_unchecked() };

    bytemuck::try_from_bytes::<T>(data).map_err(|_| ProgramError::InvalidAccountData)
}

pub fn log_value(context: &str, value: u128) {
    msg!(context);
    sol_log_64(value as u64, 0, 0, 0, 0);
}
