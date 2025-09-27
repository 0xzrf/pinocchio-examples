use pinocchio::program_error::ProgramError;
pub mod init_global_config;

#[repr(u8)]
pub enum AmmInstructions {
    CreateGlobal,
    UpdateGlobal,
    CreateBondingCurve,
    Swap,
}

impl TryFrom<&u8> for AmmInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(AmmInstructions::CreateGlobal),
            1 => Ok(AmmInstructions::UpdateGlobal),
            2 => Ok(AmmInstructions::CreateBondingCurve),
            3 => Ok(AmmInstructions::Swap),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
