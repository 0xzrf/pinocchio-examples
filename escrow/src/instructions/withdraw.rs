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

pub fn withdraw(program_id: Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let escrow_account = validate(&program_id, accounts)?;

    if let [taker, _taker_mint_b_ata, taker_mint_a_ata, maker_b_ata, escrow, escrow_vault, _token_program] =
        accounts
    {
        let bump = [escrow_account.bump];
        let seed = [
            Seed::from(EscrowPda::ESCROW_PREFIX.as_bytes()),
            Seed::from(escrow_account.creator.as_ref()),
            Seed::from(escrow_account.mint_a.as_ref()),
            Seed::from(&bump),
        ];
        let seeds = Signer::from(&seed);
        Transfer {
            amount: escrow_account.amount,
            authority: escrow,
            from: escrow_vault,
            to: taker_mint_a_ata,
        }
        .invoke_signed(&[seeds.clone()])?;

        Transfer {
            amount: escrow_account.receive,
            authority: taker,
            from: _taker_mint_b_ata,
            to: maker_b_ata,
        }
        .invoke()?;

        CloseAccount {
            account: escrow_vault,
            authority: escrow,
            destination: taker,
        }
        .invoke_signed(&[seeds])?;

        // Close the escrow account
        unsafe {
            *taker.borrow_mut_lamports_unchecked() += *escrow.borrow_lamports_unchecked();
            *escrow.borrow_mut_lamports_unchecked() = 0
        };

        Ok(())
    } else {
        Err(ProgramError::NotEnoughAccountKeys) // Never gonna reach here
    }
}

fn validate(program_id: &Pubkey, accounts: &[AccountInfo]) -> Result<EscrowPda, ProgramError> {
    if let [taker, taker_mint_b_ata, taker_mint_a_ata, mint_a, mint_b, escrow, escrow_vault, _] =
        accounts
    {
        let taker_b_info = TokenAccount::from_account_info(taker_mint_b_ata)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let taker_a_info = TokenAccount::from_account_info(taker_mint_a_ata)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let vault_info = TokenAccount::from_account_info(escrow_vault)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        require(taker.is_signer(), ProgramError::MissingRequiredSignature)?;
        require(!escrow.data_is_empty(), ProgramError::UninitializedAccount)?;

        let escrow_account = EscrowPda::load(escrow)?;

        require(
            taker_a_info.is_initialized(),
            ProgramError::UninitializedAccount,
        )?;

        require(
            pubkey_eq(taker_a_info.mint(), &escrow_account.mint_a),
            EscrowErrors::InvalidMint.into(),
        )?;
        require(
            pubkey_eq(mint_a.key(), &escrow_account.mint_a),
            EscrowErrors::InvalidMint.into(),
        )?;
        require(
            pubkey_eq(mint_b.key(), &escrow_account.mint_b),
            EscrowErrors::InvalidMint.into(),
        )?;
        require(
            pubkey_eq(taker_b_info.mint(), &escrow_account.mint_b),
            EscrowErrors::InvalidMint.into(),
        )?;

        require(
            taker_a_info.amount() >= escrow_account.receive,
            EscrowErrors::InvalidBalance.into(),
        )?;

        require(
            vault_info.is_initialized(),
            ProgramError::UninitializedAccount,
        )?;

        let expected_escrow = pinocchio::pubkey::create_program_address(
            &[
                EscrowPda::ESCROW_PREFIX.as_bytes(),
                escrow_account.creator.as_ref(),
                escrow_account.mint_a.as_ref(),
                &[escrow_account.bump],
            ],
            program_id,
        )?;

        require(
            pubkey_eq(&expected_escrow, escrow.key()),
            EscrowErrors::InvalidEscrow.into(),
        )?;

        Ok(escrow_account)
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}
