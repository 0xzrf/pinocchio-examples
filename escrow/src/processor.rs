use crate::{
    instructions::{create_escrow, process_close, process_withdraw},
    states::CreateEscrow,
};
use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum EscrowInstructions {
    CreateEscrow(CreateEscrow),
    Withdraw,
    Close,
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: &[u8],
) -> ProgramResult {
    // Deserialize the data and check if the descriminator is a valid instruction type
    let ix = EscrowInstructions::try_from_slice(instruction)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match ix {
        EscrowInstructions::CreateEscrow(data) => create_escrow(*program_id, accounts, data)?,
        EscrowInstructions::Withdraw => process_withdraw(*program_id, accounts)?,
        EscrowInstructions::Close => process_close(*program_id, accounts)?,
    }
    Ok(())
}
