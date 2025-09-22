use borsh::{BorshDeserialize, BorshSerialize};

/// `CreateEscrow` represents the arguments sent to the create_escrow ix
///
///
/// Fields:
/// - `recv_amount`: The amount the to recv when someone is closing the pda
/// - `send_amount`: The amount of mint_b to be sent
///
#[derive(BorshDeserialize, BorshSerialize)]
pub struct CreateEscrow {
    pub recv_amount: u64,
    pub send_amount: u64,
}
