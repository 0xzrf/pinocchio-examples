use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

pub fn init_global(program_id: &Pubkey, accounts: &[AccountInfo], ix_data: &[u8]) -> ProgramResult {
    Ok(())
}

pub fn validate(program_id: &Pubkey, accounts: &[AccountInfo], ix_data: &[u8]) -> ProgramResult {
    if let [admin] = accounts {
    } else {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    Ok(())
}
