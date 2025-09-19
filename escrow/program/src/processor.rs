use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum EscrowInstructions {
    CreateEscrow,
    Withdraw,
    Close,
}

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction: &[u8],
) -> ProgramResult {
    // Deserialize the data and check if the descriminator is a valid instruction type
    let ix = EscrowInstructions::try_from_slice(instruction)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match ix {
        EscrowInstructions::CreateEscrow => {}
        EscrowInstructions::Withdraw => {}
        EscrowInstructions::Close => {}
    }

    Ok(())
}
