use crate::{
    instructions::{init_global_config::init_global, AmmInstructions},
    require,
};
use pinocchio::pubkey::pubkey_eq;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    ix_data: &[u8],
) -> ProgramResult {
    require(
        pubkey_eq(program_id, &crate::ID),
        ProgramError::IncorrectProgramId,
    )?;

    let (disc, ix) = ix_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match AmmInstructions::try_from(disc)? {
        AmmInstructions::CreateBondingCurve => {}
        AmmInstructions::CreateGlobal => init_global(program_id, accounts, ix)?,
        AmmInstructions::Swap => {}
        AmmInstructions::UpdateGlobal => {}
    }

    Ok(())
}
