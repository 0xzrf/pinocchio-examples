use crate::{errors::EscrowErrors, helper::require, states::EscrowPda};
use {
    pinocchio::{
        account_info::AccountInfo,
        instruction::{Seed, Signer},
        program_error::ProgramError,
        pubkey::{pubkey_eq, Pubkey},
        ProgramResult,
    },
    pinocchio_token::{
        instructions::{CloseAccount, Transfer},
        state::TokenAccount,
    },
};

pub fn close(program_id: Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let escrow_data = validate(&program_id, accounts)?;

    if let [creator, creator_mint_ata, escrow, escrow_vault, _token_program] = accounts {
        let bump = [escrow_data.bump];
        let seed = [
            Seed::from(EscrowPda::ESCROW_PREFIX.as_bytes()),
            Seed::from(escrow_data.creator.as_ref()),
            Seed::from(escrow_data.mint_a.as_ref()),
            Seed::from(&bump),
        ];
        let seeds = Signer::from(&seed);

        Transfer {
            amount: TokenAccount::from_account_info(escrow_vault)
                .map_err(|_| ProgramError::InvalidAccountData)?
                .amount(),
            authority: escrow,
            from: escrow_vault,
            to: creator_mint_ata,
        }
        .invoke_signed(&[seeds.clone()])?;

        CloseAccount {
            account: escrow_vault,
            authority: escrow,
            destination: creator,
        }
        .invoke_signed(&[seeds])?;

        // Close the escrow account
        unsafe {
            *creator.borrow_mut_lamports_unchecked() += *escrow.borrow_lamports_unchecked();
            *escrow.borrow_mut_lamports_unchecked() = 0
        };

        Ok(())
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}

pub fn validate(program_id: &Pubkey, accounts: &[AccountInfo]) -> Result<EscrowPda, ProgramError> {
    if let [creator, creator_mint_ata, escrow, escrow_vault, _] = accounts {
        require(creator.is_signer(), ProgramError::MissingRequiredSignature)?;

        require(!escrow.data_is_empty(), ProgramError::UninitializedAccount)?;
        let escrow_data = EscrowPda::load(escrow)?;

        require(
            pubkey_eq(&escrow_data.creator, creator.key()),
            ProgramError::IllegalOwner,
        )?;

        require(
            TokenAccount::from_account_info(creator_mint_ata)
                .map_err(|_| ProgramError::InvalidAccountData)?
                .mint()
                == &escrow_data.mint_a,
            EscrowErrors::InvalidMint.into(),
        )?;

        require(
            TokenAccount::from_account_info(escrow_vault)
                .map_err(|_| ProgramError::InvalidAccountData)?
                .mint()
                == &escrow_data.mint_a,
            EscrowErrors::InvalidMint.into(),
        )?;

        let expected_escrow = pinocchio::pubkey::create_program_address(
            &[
                EscrowPda::ESCROW_PREFIX.as_bytes(),
                escrow_data.creator.as_ref(),
                escrow_data.mint_a.as_ref(),
                &[escrow_data.bump],
            ],
            program_id,
        )?;

        require(
            pubkey_eq(&expected_escrow, escrow.key()),
            EscrowErrors::InvalidEscrow.into(),
        )?;

        Ok(escrow_data)
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}
