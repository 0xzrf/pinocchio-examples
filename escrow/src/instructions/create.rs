use crate::{
    helper::require,
    states::{CreateEscrow, EscrowPda},
};
use pinocchio::pubkey::try_find_program_address;
use {
    pinocchio::{
        account_info::AccountInfo,
        program_error::ProgramError,
        program_error::ProgramError::NotEnoughAccountKeys,
        pubkey::{pubkey_eq, Pubkey},
        sysvars::{rent::Rent, Sysvar},
        ProgramResult,
    },
    pinocchio_system::instructions::CreateAccount,
    pinocchio_token::{instructions::Transfer, state::TokenAccount},
};

pub fn create_escrow(
    program_id: Pubkey,
    accounts: &[AccountInfo],
    data: CreateEscrow,
) -> ProgramResult {
    validate(program_id, accounts)?;

    if let [creator, mint_a, mint_b, creator_mint_ata, escrow_pda, vault] = accounts {
        let escrow_seeds = EscrowPda::get_signer_seeds(creator.key(), mint_a.key());

        let (expected_pda, _) = try_find_program_address(&escrow_seeds, &program_id).unwrap();

        require(
            pubkey_eq(&expected_pda, escrow_pda.key()),
            ProgramError::IncorrectProgramId,
        )?;

        CreateAccount {
            from: creator,
            lamports: (Rent::get()?).minimum_balance(EscrowPda::ESCROW_SIZE),
            owner: &program_id,
            space: EscrowPda::ESCROW_SIZE as u64,
            to: escrow_pda,
        }
        .invoke()?;

        let mut escrow_data = EscrowPda::load(escrow_pda)?;

        // #[in]
        escrow_data.init(
            creator.key(),
            mint_a.key(),
            mint_b.key(),
            data.send_amount,
            data.recv_amount,
        );

        Transfer {
            amount: data.send_amount,
            authority: creator,
            from: creator_mint_ata,
            to: vault,
        }
        .invoke()?;
    } else {
        return Err(NotEnoughAccountKeys);
    };

    Ok(())
}

pub fn validate(program_id: Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    if let [creator, _, _, creator_mint_ata, escrow_pda, vault] = accounts {
        require(creator.is_signer(), ProgramError::MissingRequiredSignature)?;

        require(
            TokenAccount::from_account_info(vault).unwrap().owner() == escrow_pda.key(),
            ProgramError::InvalidAccountOwner,
        )?;

        require(
            !escrow_pda.data_is_empty(),
            ProgramError::AccountAlreadyInitialized,
        )?;

        require(
            creator_mint_ata.data_is_empty(),
            ProgramError::UninitializedAccount,
        )?;

        require(
            // escrow_pda.owner() == &program_id,
            pubkey_eq(escrow_pda.owner(), &program_id),
            ProgramError::InvalidAccountOwner,
        )?;
    } else {
        return Err(NotEnoughAccountKeys);
    };

    Ok(())
}
