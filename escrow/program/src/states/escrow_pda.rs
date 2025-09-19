use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::pubkey::Pubkey;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct EscrowPda {
    pub creator: Pubkey,
    pub amount: u64,    // The amount of mint_a the creator is gonna put in
    pub mint_a: Pubkey, // The token the creator of the escrow put in the escrow pda
    pub mint_b: Pubkey, // The token the receiver send to the creator to release mint_a
    pub receive: u64,   // The amount of mint_b the creator wants in return for giving mint_a
}
impl EscrowPda {
    pub const SERIALIZED_SIZE: usize = 32 + 8 + 32 + 32 + 8;
}
