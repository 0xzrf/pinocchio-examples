use crate::errors::EscrowErrors;
use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct EscrowPda {
    pub creator: Pubkey,
    pub amount: u64,    // The amount of mint_a the creator is gonna put in
    pub mint_a: Pubkey, // The token the creator of the escrow put in the escrow pda
    pub mint_b: Pubkey, // The token the receiver send to the creator to release mint_a
    pub receive: u64,   // The amount of mint_b the creator wants in return for giving mint_a
}
impl EscrowPda {
    pub const ESCROW_SIZE: usize = 32 + 8 + 32 + 32 + 8;
    pub const ESCROW_PREFIX: &str = "escrow";

    #[inline(always)]
    pub fn get_signer_seeds<'a>(creator: &'a Pubkey, mint_a: &'a Pubkey) -> [&'a [u8]; 3] {
        [
            Self::ESCROW_PREFIX.as_bytes(),
            creator.as_ref(),
            mint_a.as_ref(),
        ]
    }

    #[inline(always)]
    pub fn load(escrow_account: &AccountInfo) -> Result<Self, ProgramError> {
        if escrow_account.can_borrow_mut_data().is_ok() {
            let data = unsafe { escrow_account.borrow_data_unchecked() };

            let escrow_data =
                EscrowPda::try_from_slice(data).map_err(|_| ProgramError::InvalidAccountData)?;

            Ok(escrow_data)
        } else {
            Err(EscrowErrors::BorrowInvalid.into())
        }
    }

    #[inline(always)]
    pub fn init(
        &mut self,
        creator: &Pubkey,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
        amount: u64,
        recieve: u64,
    ) {
        self.creator = *creator;
        self.amount = amount;
        self.mint_a = *mint_a;
        self.mint_b = *mint_b;
        self.receive = recieve;
    }
}
