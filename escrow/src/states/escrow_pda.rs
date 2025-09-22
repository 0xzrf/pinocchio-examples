use crate::errors::EscrowErrors;
use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

/// `EscrowPda` represents the state of an escrow account in the program.
///
/// This struct stores all the necessary information for an escrow transaction,
/// including the creator's public key, the token mints involved, and the amounts
/// to be exchanged. It is serialized and stored in the escrow PDA (Program Derived Address)
/// account on-chain.
///
/// Fields:
/// - `creator`: The public key of the user who created the escrow.
/// - `amount`: The amount of `mint_a` tokens the creator is depositing into escrow.
/// - `mint_a`: The token mint address for the asset being offered by the creator.
/// - `mint_b`: The token mint address for the asset the creator expects in return.
/// - `receive`: The amount of `mint_b` tokens the creator expects to receive in exchange.
///
/// The struct also provides utility methods for PDA seed generation, loading from account data,
/// and initialization.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct EscrowPda {
    pub creator: Pubkey,
    pub amount: u64,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
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
