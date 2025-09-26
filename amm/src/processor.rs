use crate::states::global_config::GlobalSettingsInput;
use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum AmmInstructions {
    CreateGlobal(GlobalSettingsInput),
    UpdateGlobal,
    CreateBondingCurve,
    Swap,
}

pub fn process_instruction(
    pubkey: &Pubkey,
    accounts: &[AccountInfo],
    ix_data: &[u8],
) -> ProgramResult {
    let ix_data = AmmInstructions::try_from_slice(ix_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match ix_data {
        AmmInstructions::CreateBondingCurve => {}
        AmmInstructions::CreateGlobal(data) => {}
        AmmInstructions::Swap => {}
        AmmInstructions::UpdateGlobal => {}
    }

    Ok(())
}
