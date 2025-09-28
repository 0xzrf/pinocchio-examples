use crate::{
    require,
    states::{
        bonding_curve::{BondingCurve, CreateCurveArgs},
        global_config::GlobalConfig,
    },
};
use {
    pinocchio::{
        account_info::AccountInfo,
        instruction::Signer,
        program_error::ProgramError,
        pubkey::{find_program_address, pubkey_eq, Pubkey},
        sysvars::{rent::Rent, Sysvar},
        ProgramResult,
    },
    pinocchio_associated_token_account::instructions::Create as CreateAta,
    pinocchio_system::instructions::CreateAccount,
    pinocchio_token_2022::{
        instructions::{
            AuthorityType, FreezeAccount, InitializeMint2, MintToChecked, SetAuthority,
        },
        state::Mint,
    },
};

pub fn process_init_bonding_curve(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    ix_data: &[u8],
) -> ProgramResult {
    validate(program_id, accounts)?;

    if let [creator, config_pda, curve_pda, mint, curve_mint_ata, curve_sol_escrow, system_program, token2022_program] =
        accounts
    {
        let seeds: &[&[u8]] = &[GlobalConfig::GLOBAL_PEFIX];

        let (global_config_pda, _) = find_program_address(seeds, program_id);

        require(
            pubkey_eq(&global_config_pda, config_pda.key()),
            ProgramError::IncorrectProgramId,
        )?;

        require(
            !config_pda.data_is_empty(),
            ProgramError::UninitializedAccount,
        )?;

        let config_data = GlobalConfig::load(config_pda)?;

        require(
            config_data.inittialized.eq(&1),
            ProgramError::UninitializedAccount,
        )?;

        let curve_seeds: &[&[u8]] = &[BondingCurve::SEED_PREFIX, mint.key().as_ref()];

        let (expected_curve_pda, curve_bump) = find_program_address(curve_seeds, program_id);

        require(
            pubkey_eq(curve_pda.key(), &expected_curve_pda),
            ProgramError::IncorrectProgramId,
        )?;

        CreateAccount {
            from: creator,
            to: curve_pda,
            lamports: (Rent::get()?).minimum_balance(BondingCurve::CURVE_SIZE),
            space: BondingCurve::CURVE_SIZE as u64,
            owner: program_id,
        }
        .invoke()?;

        BondingCurve::init(
            curve_bump,
            *config_data,
            curve_pda,
            creator.key(),
            mint.key(),
        )?;

        require(
            ix_data.len() == CreateCurveArgs::LEN,
            ProgramError::InvalidInstructionData,
        )?;

        let mut aligned_ix = [0u8; CreateCurveArgs::LEN];

        aligned_ix.copy_from_slice(ix_data);

        let args = bytemuck::try_from_bytes::<CreateCurveArgs>(ix_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        init_mint(
            args,
            creator,
            mint,
            token2022_program,
            curve_pda,
            config_data.mint_decimals,
        )?;

        CreateAta {
            funding_account: creator,
            account: curve_mint_ata,
            wallet: curve_pda,
            mint,
            system_program,
            token_program: token2022_program,
        }
        .invoke()?;

        let seeds = BondingCurve::get_signer_seeds(mint.key());

        let signer_seeds = Signer::from(&seeds);

        MintToChecked {
            mint,
            account: curve_mint_ata,
            amount: config_data.token_total_supply,
            decimals: config_data.mint_decimals,
            mint_authority: curve_pda,
            token_program: token2022_program.key(),
        }
        .invoke_signed(&[signer_seeds])?;

        let accounts: &[AccountInfo] = &[
            *creator,
            *curve_mint_ata,
            *curve_pda,
            *mint,
            *system_program,
            *token2022_program,
        ];

        mint_and_revoke_authorities(
            accounts,
            config_data.token_total_supply,
            config_data.mint_decimals,
        )?;

        Ok(())
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}

pub fn validate(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    if let [creator, config_pda, curve_pda, mint, curve_mint_ata, curve_sol_escrow, _, _] = accounts
    {
        require(creator.is_signer(), ProgramError::MissingRequiredSignature)?;
        require(mint.is_writable(), ProgramError::MissingRequiredSignature)?;
        require(
            curve_pda.is_signer(),
            ProgramError::MissingRequiredSignature,
        )?;

        require(
            curve_pda.data_is_empty()
                || mint.data_is_empty()
                || curve_mint_ata.data_is_empty()
                || curve_sol_escrow.data_is_empty(),
            ProgramError::AccountAlreadyInitialized,
        )?;

        let sol_escrow_seeds: &[&[u8]] =
            &[BondingCurve::SOL_ESCROW_SEED_PREFIX, mint.key().as_ref()];

        let (expected_sol_escrow, _) = find_program_address(sol_escrow_seeds, program_id);

        require(
            pubkey_eq(&expected_sol_escrow, curve_sol_escrow.key()),
            ProgramError::IncorrectProgramId,
        )?;

        Ok(())
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}

pub fn init_mint(
    args: &CreateCurveArgs,
    creator: &AccountInfo,
    mint: &AccountInfo,
    token2022_program: &AccountInfo,
    curve_pda: &AccountInfo,
    decimals: u8,
) -> Result<(), ProgramError> {
    let rent = Rent::get()?;
    // Create the account for the Mint
    CreateAccount {
        from: creator,
        to: mint,
        owner: token2022_program.key(),
        lamports: rent.minimum_balance(Mint::BASE_LEN),
        space: Mint::BASE_LEN as u64,
    }
    .invoke()?;

    InitializeMint2 {
        decimals,
        freeze_authority: Some(curve_pda.key()),
        mint,
        mint_authority: curve_pda.key(),
        token_program: token2022_program.key(),
    }
    .invoke()?;

    Ok(())
}

pub fn mint_and_revoke_authorities(
    accounts: &[AccountInfo],
    total_supply: u64,
    mint_decimals: u8,
) -> Result<(), ProgramError> {
    if let [creator, curve_mint_ata, curve_pda, mint, system_program, token_program] = accounts {
        CreateAta {
            funding_account: creator,
            account: curve_mint_ata,
            wallet: curve_pda,
            mint,
            system_program,
            token_program,
        }
        .invoke()?;

        let seeds = BondingCurve::get_signer_seeds(mint.key());

        let signer_seeds = Signer::from(&seeds);

        MintToChecked {
            mint,
            account: curve_mint_ata,
            amount: total_supply,
            decimals: mint_decimals,
            mint_authority: curve_pda,
            token_program: token_program.key(),
        }
        .invoke_signed(&[signer_seeds.clone()])?;

        // Setting mint authroity to none to avoid rug-pulls
        SetAuthority {
            account: mint,
            authority: curve_pda,
            authority_type: AuthorityType::MintTokens,
            new_authority: None,
            token_program: token_program.key(),
        }
        .invoke_signed(&[signer_seeds.clone()])?;

        FreezeAccount {
            account: curve_mint_ata,
            freeze_authority: curve_pda,
            mint,
            token_program: token_program.key(),
        }
        .invoke_signed(&[signer_seeds])?;

        Ok(())
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}
