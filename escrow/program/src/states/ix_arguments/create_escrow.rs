use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::pubkey::Pubkey;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct CreateEscrow {
    pub recv_amount: u64,
    pub send_amount: u64,
}
