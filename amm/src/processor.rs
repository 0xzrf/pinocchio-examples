use crate::instructions::AmmInstructions;
use crate::require;
use pinocchio::pubkey::pubkey_eq;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

pub fn process_instruction(
    pubkey: &Pubkey,
    accounts: &[AccountInfo],
    ix_data: &[u8],
) -> ProgramResult {
    require(
        pubkey_eq(pubkey, &crate::ID),
        ProgramError::IncorrectProgramId,
    )?;

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
