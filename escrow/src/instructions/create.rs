use crate::{
    helper::require,
    states::{CreateEscrow, EscrowPda},
};
use {
    pinocchio::{
        account_info::AccountInfo,
        log::sol_log,
        program_error::ProgramError,
        program_error::ProgramError::NotEnoughAccountKeys,
        pubkey::{pubkey_eq, try_find_program_address, Pubkey},
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
    sol_log("Escrow: CreateEscrow");

    validate(accounts)?;

    if let [creator, mint_a, mint_b, creator_mint_ata, escrow_pda, vault, _system_program, _token_program] =
        accounts
    {
        let escrow_seeds = EscrowPda::get_signer_seeds(creator.key(), mint_a.key());

        let (expected_pda, bump) = try_find_program_address(&escrow_seeds, &program_id).unwrap();

        require(
            pubkey_eq(&expected_pda, escrow_pda.key()),
            ProgramError::IncorrectProgramId,
        )?;

        sol_log("Creating PDA");
        CreateAccount {
            from: creator,
            lamports: (Rent::get()?).minimum_balance(EscrowPda::ESCROW_SIZE),
            owner: &program_id,
            space: EscrowPda::ESCROW_SIZE as u64,
            to: escrow_pda,
        }
        .invoke()?;

        sol_log("pda created");
        let mut escrow_data = EscrowPda::load(escrow_pda)?;

        escrow_data.init(
            creator.key(),
            mint_a.key(),
            mint_b.key(),
            data.send_amount,
            data.recv_amount,
            bump,
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

pub fn validate(accounts: &[AccountInfo]) -> ProgramResult {
    if let [creator, mint_a, _, _creator_mint_ata, escrow_pda, vault, _, _] = accounts {
        require(creator.is_signer(), ProgramError::MissingRequiredSignature)?;

        require(
            TokenAccount::from_account_info(vault).unwrap().mint() == mint_a.key(),
            ProgramError::InvalidAccountOwner,
        )?;

        require(
            escrow_pda.data_is_empty(),
            ProgramError::AccountAlreadyInitialized,
        )?;
        sol_log("Validation complete");
    } else {
        return Err(NotEnoughAccountKeys);
    };

    Ok(())
}
