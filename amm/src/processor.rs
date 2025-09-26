use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::instructions::AmmInstructions;

pub fn process_instruction(
    pubkey: &Pubkey,
    accounts: &[AccountInfo],
    ix_data: &[u8],
) -> ProgramResult {
    let (disc, ix) = ix_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match AmmInstructions::try_from(disc)? {
        AmmInstructions::CreateBondingCurve => {}
        AmmInstructions::CreateGlobal => {}
        AmmInstructions::Swap => {}
        AmmInstructions::UpdateGlobal => {}
    }

    Ok(())
}
