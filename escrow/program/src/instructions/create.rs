use crate::{
    constants::system_program_pubkey,
    errors::EscrowErrors,
    helper::require,
    states::{CreateEscrow, EscrowPda},
};
use {
    pinocchio::{
        account_info::AccountInfo,
        program_error::ProgramError,
        program_error::ProgramError::NotEnoughAccountKeys,
        pubkey::Pubkey,
        sysvars::{rent::Rent, Sysvar},
        ProgramResult,
    },
    pinocchio_associated_token_account::check_id as check_associated_address,
    pinocchio_system::instructions::CreateAccountWithSeed,
    pinocchio_token::check_id as check_token_address,
};

pub fn create_escrow(
    program_id: Pubkey,
    accounts: &[AccountInfo],
    data: CreateEscrow,
) -> ProgramResult {
    validate(program_id, accounts)?;

    if let [creator, mint_a, mint_b, create_mint_ata, escrow_pda, escrow_pda_mint_ata, system_program, token_program, associated_token_program] =
        accounts
    {
        let escrow_data = EscrowPda {
            amount: data.send_amount,
            creator: *creator.key(),
            mint_a: *mint_a.key(),
            mint_b: *mint_b.key(),
            receive: data.recv_amount,
        };

        let account_span = EscrowPda::SERIALIZED_SIZE;

        let lamports = (Rent::get()?).minimum_balance(account_span);

        // let seeds = "escrow_pda" + creator.key();

        let create_pda = CreateAccountWithSeed {
            base: Some(escrow_pda),
            from: creator,
            lamports,
            owner: &program_id,
            seed: "escrow_pda",
            to: escrow_pda,
            space: account_span as u64,
        };
    } else {
        return Err(NotEnoughAccountKeys);
    };

    Ok(())
}

pub fn validate(program_id: Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    if let [creator, mint_a, mint_b, creator_mint_ata, escrow_pda, escrow_pda_mint_ata, system_program, token_program, associated_token_program] =
        accounts
    {
        require(creator.is_signer(), EscrowErrors::NotRentExempt.into())?;
        require(
            creator.is_writable() && escrow_pda_mint_ata.is_writable(),
            ProgramError::Immutable,
        )?;

        require(
            check_token_address(token_program.key())
                && system_program.key() == &system_program_pubkey
                && check_associated_address(associated_token_program.key()),
            ProgramError::IncorrectProgramId,
        )?;

        require(
            escrow_pda.owner() == &program_id,
            ProgramError::InvalidAccountOwner,
        )?;
    } else {
        return Err(NotEnoughAccountKeys);
    };

    Ok(())
}
