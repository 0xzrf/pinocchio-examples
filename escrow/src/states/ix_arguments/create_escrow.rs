use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct CreateEscrow {
    pub recv_amount: u64,
    pub send_amount: u64,
}
