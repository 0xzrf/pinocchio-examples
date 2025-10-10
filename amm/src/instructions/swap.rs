use crate::{
    load, load_read_only, log_value, require,
    states::{
        bonding_curve::{BondingCurve, BuyResult, SellResult},
        global_config::GlobalConfig,
    },
    AmmError,
};
use bytemuck::{Pod, Zeroable};
use {
    pinocchio::{
        account_info::AccountInfo,
        instruction::Signer,
        msg,
        program_error::ProgramError,
        pubkey::{pubkey_eq, Pubkey},
        sysvars::{rent::Rent, Sysvar},
        ProgramResult,
    },
    pinocchio_system::instructions::Transfer as SendSol,
    pinocchio_token_2022::{
        instructions::{FreezeAccount, ThawAccount, TransferChecked},
        state::{AccountState, Mint, TokenAccount},
        ID as TOKEN_PROGRMA_ID,
    },
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SwapParams {
    pub base_in: u8,
    pub padding: [u8; 7],
    pub exact_in_amount: u64,
    pub min_out_amount: u64,
}

pub fn process_swap(accounts: &[AccountInfo], ix_data: &[u8]) -> ProgramResult {
    msg!("AMM INSTRUCTION: SWAP");
    let (mut curve_data, swap_params) = validate(accounts, ix_data)?;

    let SwapParams {
        base_in,
        padding: _,
        exact_in_amount,
        min_out_amount,
    } = swap_params;

    if let [buyer, buyer_mint_ata, _mint_a, mint_b, _config, curve_pda, curve_sol_escrow, curve_mint_ata, fee_receiver, _system_program, _token_program] =
        accounts
    {
        let signer_seeds = BondingCurve::get_signer_seeds(mint_b.key());
        let signer = Signer::from(&signer_seeds);
        ThawAccount {
            account: curve_mint_ata,
            freeze_authority: curve_pda,
            mint: mint_b,
            token_program: &TOKEN_PROGRMA_ID,
        }
        .invoke_signed(&[signer.clone()])?;

        let sol_amount: u64;
        let fee_lamports: u64;

        let mint_info =
            Mint::from_account_info(mint_b).map_err(|_| ProgramError::InvalidAccountData)?;
        if base_in == 1 {
            // Sell Tokens
            let buyer_mint_info = TokenAccount::from_account_info(buyer_mint_ata)
                .map_err(|_| ProgramError::InvalidAccountData)?;

            require(
                buyer_mint_info.state() == AccountState::Initialized,
                ProgramError::UninitializedAccount,
            )?;

            require(
                buyer_mint_info.amount() >= swap_params.exact_in_amount,
                ProgramError::InsufficientFunds,
            )?;

            let curve_account_data = load::<BondingCurve>(curve_pda)?;

            let sell_result = curve_account_data
                .apply_sell(exact_in_amount, mint_info.decimals())
                .ok_or(AmmError::CouldNotSell)?;

            sol_amount = sell_result.sol_amount;
            fee_lamports = curve_account_data.calculate_fee(sol_amount)?;

            log_value("Fee in SOL:", fee_lamports.into());

            let sell_accounts = &[
                *buyer,
                *buyer_mint_ata,
                *curve_pda,
                *curve_mint_ata,
                *curve_sol_escrow,
                *mint_b,
                *fee_receiver,
            ];
            complete_sell(
                sell_accounts,
                sell_result,
                min_out_amount,
                fee_lamports,
                mint_info.decimals(),
                signer,
            )?;
        } else {
            // Buy tokens
            let fee_lamports = curve_data.calculate_fee(swap_params.exact_in_amount)?;
            let buy_amount_applied = swap_params.exact_in_amount - fee_lamports;

            let buy_result = curve_data
                .apply_buy(buy_amount_applied, mint_info.decimals())
                .ok_or(AmmError::CouldNotBuy)?;

            let buy_acconts = &[
                *buyer,
                *buyer_mint_ata,
                *curve_pda,
                *curve_mint_ata,
                *curve_sol_escrow,
                *mint_b,
                *fee_receiver,
            ];

            complete_buy(
                buy_acconts,
                buy_result,
                min_out_amount,
                fee_lamports,
                mint_info.decimals(),
                signer,
            )?;
        }

        let invariant_accounts = &[*curve_mint_ata, *curve_sol_escrow];
        curve_data.invariant(invariant_accounts)?;

        Ok(())
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}

pub fn complete_sell(
    accounts: &[AccountInfo],
    sell_result: SellResult,
    min_out_amount: u64,
    fee_lamports: u64,
    decimals: u8,
    seeds: Signer,
) -> ProgramResult {
    if let [buyer, buyer_mint_ata, curve_pda, curve_mint_ata, curve_sol_ata, mint, fee_receiver] =
        accounts
    {
        require(
            sell_result.sol_amount >= min_out_amount,
            AmmError::SlippageExceeded.into(),
        )?;

        TransferChecked {
            amount: sell_result.token_amount,
            authority: buyer,
            decimals,
            from: buyer_mint_ata,
            to: curve_mint_ata,
            mint,
            token_program: &TOKEN_PROGRMA_ID,
        }
        .invoke()?;

        FreezeAccount {
            account: curve_mint_ata,
            freeze_authority: curve_pda,
            mint,
            token_program: &TOKEN_PROGRMA_ID,
        }
        .invoke_signed(&[seeds.clone()])?;

        // Sending SOL from buyer to the curve_sol_escrow
        SendSol {
            from: curve_sol_ata,
            lamports: sell_result.sol_amount,
            to: buyer,
        }
        .invoke_signed(&[seeds])?;

        // Send Fee to the fee_receiver
        SendSol {
            from: buyer,
            lamports: fee_lamports,
            to: fee_receiver,
        }
        .invoke()?;

        Ok(())
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}

pub fn complete_buy(
    accounts: &[AccountInfo],
    buy_result: BuyResult,
    min_out_amount: u64,
    fee_lamports: u64,
    decimals: u8,
    seeds: Signer,
) -> ProgramResult {
    if let [buyer, buyer_mint_ata, curve_pda, curve_mint_ata, curve_sol_ata, mint, fee_receiver] =
        accounts
    {
        require(
            buy_result.token_amount >= min_out_amount,
            AmmError::SlippageExceeded.into(),
        )?;

        TransferChecked {
            amount: buy_result.token_amount,
            authority: curve_pda,
            decimals,
            from: curve_mint_ata,
            to: buyer_mint_ata,
            mint,
            token_program: &TOKEN_PROGRMA_ID,
        }
        .invoke_signed(&[seeds.clone()])?;

        FreezeAccount {
            account: curve_mint_ata,
            freeze_authority: curve_pda,
            mint,
            token_program: &TOKEN_PROGRMA_ID,
        }
        .invoke_signed(&[seeds])?;

        // Sending SOL from buyer to the curve_sol_escrow
        SendSol {
            from: buyer,
            lamports: buy_result.sol_amount,
            to: curve_sol_ata,
        }
        .invoke()?;

        // Send Fee to the fee_receiver
        SendSol {
            from: buyer,
            lamports: fee_lamports,
            to: fee_receiver,
        }
        .invoke()?;

        Ok(())
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}

pub fn validate(
    accounts: &[AccountInfo],
    ix_data: &[u8],
) -> Result<(BondingCurve, SwapParams), ProgramError> {
    if let [buyer, buyer_mint_ata, _mint_a, mint_b, config, curve_pda, curve_sol_escrow, curve_mint_ata, fee_receiver, _, _] =
        accounts
    {
        require(buyer.is_signer(), ProgramError::MissingRequiredSignature)?;

        require(
            curve_pda.data_len() == BondingCurve::CURVE_SIZE,
            ProgramError::InvalidAccountData,
        )?;

        BondingCurve::check_id(curve_pda, *mint_b.key())?;

        GlobalConfig::check_id(config)?;

        let curve_data = load::<BondingCurve>(curve_pda)?;

        require(curve_data.complete == 0, AmmError::CurveComplete.into())?;
        require(curve_data.is_started(), AmmError::CurveNotStarted.into())?;

        let global_config = load_read_only::<GlobalConfig>(config)?;

        require(
            pubkey_eq(fee_receiver.key(), &global_config.fee_receiver),
            ProgramError::IncorrectProgramId,
        )?;

        require(
            pubkey_eq(mint_b.key(), &curve_data.mint),
            ProgramError::IncorrectProgramId,
        )?;
        let buyer_mint_info = TokenAccount::from_account_info(buyer_mint_ata)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let curve_mint_info = TokenAccount::from_account_info(curve_mint_ata)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        require(
            pubkey_eq(buyer_mint_info.mint(), mint_b.key())
                && pubkey_eq(curve_mint_info.mint(), mint_b.key()),
            ProgramError::IncorrectProgramId,
        )?;

        require(
            pubkey_eq(buyer_mint_info.owner(), buyer.key())
                && pubkey_eq(curve_mint_info.owner(), curve_pda.key())
                && pubkey_eq(curve_sol_escrow.owner(), curve_pda.key()),
            ProgramError::IncorrectAuthority,
        )?;

        // TODO: check if mint_a is wsol address

        let ix_params = bytemuck::try_from_bytes::<SwapParams>(ix_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        let required_lamports = (Rent::get()?).minimum_balance(0);

        require(
            buyer.lamports()
                >= required_lamports
                    .checked_add(ix_params.exact_in_amount)
                    .unwrap(),
            ProgramError::InsufficientFunds,
        )?;
        require(
            ix_params.exact_in_amount > 0,
            ProgramError::InvalidInstructionData,
        )?;

        require(
            ix_params.base_in == 1 && ix_params.base_in == 0,
            ProgramError::InvalidInstructionData,
        )?;

        Ok((*curve_data, *ix_params))
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}
