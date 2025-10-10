use crate::{
    bps_mul, constants::SOLANA_DECIMALS, helpers::log_value, load, require,
    states::global_config::GlobalConfig, AmmError,
};
use bytemuck::{Pod, Zeroable};
use {
    pinocchio::{
        account_info::AccountInfo,
        instruction::Seed,
        msg,
        program_error::ProgramError,
        pubkey::{find_program_address, pubkey_eq, Pubkey},
        seeds,
        sysvars::{clock::Clock, Sysvar},
    },
    pinocchio_token_2022::{state::TokenAccount, ID as TOKEN_PROGRAM_ID},
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct BondingCurve {
    pub complete: u8,
    pub bump: u8,
    pub _padding: [u8; 6],

    pub mint: Pubkey,
    pub creator: Pubkey,

    pub initial_real_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,
    pub starting_slot: u64,
}

#[derive(Debug, Clone)]
pub struct BuyResult {
    pub token_amount: u64,
    pub sol_amount: u64,
}

#[derive(Debug, Clone)]
pub struct SellResult {
    pub token_amount: u64,
    pub sol_amount: u64,
}

impl BondingCurve {
    pub const CURVE_SIZE: usize = core::mem::size_of::<Self>();
    pub const SEED_PREFIX: &[u8] = b"bonding_curve";
    pub const SOL_ESCROW_SEED_PREFIX: &[u8] = b"sol_escrow";
    pub const MINT_SEED_PREFIX: &[u8] = b"curve_mint";

    pub fn check_id(curve_account: &AccountInfo, mint: Pubkey) -> Result<(), ProgramError> {
        let curve_seeds: &[&[u8]] = &[BondingCurve::SEED_PREFIX, mint.as_ref()];

        let (expected_curve_pda, _) = find_program_address(curve_seeds, &crate::ID);

        require(
            pubkey_eq(curve_account.key(), &expected_curve_pda),
            ProgramError::IncorrectProgramId,
        )?;

        Ok(())
    }

    pub fn get_signer_seeds<'a>(mint: &'a Pubkey) -> [Seed<'a>; 2] {
        seeds!(Self::SEED_PREFIX, mint.as_ref())
    }

    pub fn init(
        bump: u8,
        configs: GlobalConfig,
        curve_account: &AccountInfo,
        creator_key: &Pubkey,
        mint: &Pubkey,
    ) -> Result<(), ProgramError> {
        let curve_data = load::<BondingCurve>(curve_account)?;

        let slot = (Clock::get()?).slot;

        curve_data._padding = [0u8; 6];
        curve_data.bump = bump;
        curve_data.complete = 0;
        curve_data.starting_slot = slot;
        curve_data.creator = *creator_key;
        curve_data.initial_real_token_reserves = configs.initial_real_token_reserves;
        curve_data.mint = *mint;
        curve_data.real_sol_reserves = 0;
        curve_data.real_token_reserves = configs.initial_real_token_reserves;
        curve_data.virtual_sol_reserves = configs.initial_virtual_sol_reserves;
        curve_data.virtual_token_reserves = configs.initial_virtual_token_reserves;
        curve_data.token_total_supply = configs.token_total_supply;

        Ok(())
    }

    pub fn is_started(&self) -> bool {
        let slot = (Clock::get().unwrap()).slot;

        self.starting_slot > slot
    }

    pub fn calculate_fee(&self, amount: u64) -> Result<u64, ProgramError> {
        let start_slot = self.starting_slot;
        let current_slot = (Clock::get().unwrap()).slot;

        let slots_passed = current_slot - start_slot;

        let mut sol_fee: u64 = 0;

        if slots_passed < 150 {
            msg!("Phase 1: 99% fees between slot 0 - 150");
            sol_fee = bps_mul(9900, amount, 10_000).unwrap();
        } else if (150..250).contains(&slots_passed) {
            msg!("Phase 2: Linear decrease between 150 - 250");

            // Calculate the minimum fee bps (at slot 250) scaled by 100_000 for precision
            let fee_bps = (-8_300_000_i64)
                .checked_mul(slots_passed as i64)
                .ok_or(ProgramError::ArithmeticOverflow)?
                .checked_add(2_162_600_000)
                .ok_or(ProgramError::ArithmeticOverflow)?
                .checked_div(100_000)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            sol_fee = bps_mul(fee_bps as u64, amount, 10_000).unwrap();
        } else if slots_passed > 250 {
            msg!("Phase 3: 1% fees after 250");
            sol_fee = bps_mul(100, amount, 10_000).unwrap();
        }
        Ok(sol_fee)
    }

    pub fn apply_sell(&mut self, token_amount: u64, decimals: u8) -> Option<SellResult> {
        log_value("apply_sell: token_amount:", token_amount as u128);

        let sol_amount = self.get_sol_for_sell_tokens(token_amount, decimals)?;

        // Adjusting token reserve values
        // New Virtual Token Reserves
        let new_virtual_token_reserves =
            (self.virtual_token_reserves as u128).checked_add(token_amount.into())?;

        log_value(
            "apply_sell: new_virtual_token_reserves:",
            new_virtual_token_reserves,
        );
        // New Real Token Reserves
        let new_real_token_reserves =
            (self.real_token_reserves as u128).checked_add(token_amount.into())?;

        log_value(
            "apply_sell: new_real_token_reserves",
            new_real_token_reserves,
        );

        // Adjusting sol reserve values
        // New Virtual Sol Reserves
        let new_virtual_sol_reserves =
            (self.virtual_sol_reserves as u128).checked_sub(sol_amount.into())?;

        log_value(
            "apply_sell: new_virtual_sol_reserves",
            new_virtual_sol_reserves,
        );

        // New Real Sol Reserves
        let new_real_sol_reserves = self.real_sol_reserves.checked_sub(sol_amount)?;

        log_value(
            "apply_sell: new_real_sol_reserves:",
            new_real_sol_reserves.into(),
        );

        self.virtual_token_reserves = new_virtual_token_reserves.try_into().ok()?;
        self.real_token_reserves = new_real_token_reserves.try_into().ok()?;
        self.virtual_sol_reserves = new_virtual_sol_reserves.try_into().ok()?;
        self.real_sol_reserves = new_real_sol_reserves;

        Some(SellResult {
            token_amount,
            sol_amount,
        })
    }

    pub fn apply_buy(&mut self, mut sol_amount: u64, decimals: u8) -> Option<BuyResult> {
        let mut token_amount = self.get_tokens_for_buy_sol(sol_amount, decimals)?;

        log_value("ApplyBuy: sol_amount:", sol_amount.into());
        log_value("ApplyBuy: token_amount", token_amount.into());

        if token_amount >= self.real_token_reserves {
            msg!("Bonding curve completed");

            log_value("real_token_reserves::", self.real_token_reserves.into());

            // Last Buy
            token_amount = self.real_token_reserves;

            // Temporarily store the current state
            let current_virtual_token_reserves = self.virtual_token_reserves;
            let current_virtual_sol_reserves = self.virtual_sol_reserves;

            // Update self with the new token amount
            self.virtual_token_reserves = (current_virtual_token_reserves as u128)
                .checked_sub(token_amount as u128)?
                .try_into()
                .ok()?;
            self.virtual_sol_reserves = 115_005_359_056; // Total raise amount at end

            let recomputed_sol_amount = self.get_sol_for_sell_tokens(token_amount, decimals)?;

            log_value(
                "ApplyBuy: recomputed_sol_amount:",
                recomputed_sol_amount.into(),
            );
            sol_amount = recomputed_sol_amount;

            // Restore the state with the recomputed sol_amount
            self.virtual_token_reserves = current_virtual_token_reserves;
            self.virtual_sol_reserves = current_virtual_sol_reserves;

            // Set complete to true
            self.complete = 1;
        }

        // Adjusting token reserve values
        // New Virtual Token Reserves
        let new_virtual_token_reserves =
            (self.virtual_token_reserves as u128).checked_sub(token_amount as u128)?;

        log_value(
            "ApplyBuy: new_virtual_token_reserves:",
            new_virtual_token_reserves,
        );

        // New Real Token Reserves
        let new_real_token_reserves =
            (self.real_token_reserves as u128).checked_sub(token_amount as u128)?;

        log_value(
            "ApplyBuy: new_real_token_reserves:",
            new_real_token_reserves,
        );

        // Adjusting sol reserve values
        // New Virtual Sol Reserves
        let new_virtual_sol_reserves =
            (self.virtual_sol_reserves as u128).checked_add(sol_amount as u128)?;

        log_value(
            "ApplyBuy: new_virtual_sol_reserves:",
            new_virtual_sol_reserves,
        );

        // New Real Sol Reserves
        let new_real_sol_reserves =
            (self.real_sol_reserves as u128).checked_add(sol_amount as u128)?;

        log_value("ApplyBuy: new_real_sol_reserves:", new_real_sol_reserves);

        self.virtual_token_reserves = new_virtual_token_reserves.try_into().ok()?;
        self.real_token_reserves = new_real_token_reserves.try_into().ok()?;
        self.virtual_sol_reserves = new_virtual_sol_reserves.try_into().ok()?;
        self.real_sol_reserves = new_real_sol_reserves.try_into().ok()?;

        Some(BuyResult {
            token_amount,
            sol_amount,
        })
    }

    pub fn get_sol_for_sell_tokens(&self, token_amount: u64, decimals: u8) -> Option<u64> {
        let mint_decimals = 10u128.pow(decimals as u32);

        // Convert to common decimal basis (using 9 decimals as base)
        let current_sol = self.virtual_sol_reserves as u128;
        let current_tokens = (self.virtual_token_reserves as u128)
            .checked_mul(SOLANA_DECIMALS as u128)? // Scale tokens up to 9 decimals
            .checked_div(mint_decimals)?; // From 6 decimals

        // Calculate new reserves using constant product formula
        let new_tokens = current_tokens.checked_add(
            (token_amount as u128)
                .checked_mul(SOLANA_DECIMALS as u128)? // Scale input tokens to 9 decimals
                .checked_div(mint_decimals)?, // From 6 decimals
        )?;

        let new_sol = (current_sol.checked_mul(current_tokens)?).checked_div(new_tokens)?;

        let sol_out = current_sol.checked_sub(new_sol)?;

        log_value("GetSolForSellTokens: sol_out:", sol_out);

        <u128 as TryInto<u64>>::try_into(sol_out).ok()
    }

    pub fn get_tokens_for_buy_sol(&self, sol_amount: u64, decimals: u8) -> Option<u64> {
        let mint_decimals = 10u128.pow(decimals as u32);

        // Convert to common decimal basis (using 9 decimals as base)
        let current_sol: u128 = self.virtual_sol_reserves as u128;
        // Scaling to SOL's decimal point
        let current_tokens = (self.virtual_token_reserves as u128)
            .checked_mul(SOLANA_DECIMALS as u128)?
            .checked_div(mint_decimals)?;

        // Calculate new reserves using constant product formula
        let new_sol: u128 = current_sol.checked_add(sol_amount as u128)?;
        let k_value_before = current_sol.checked_mul(current_tokens)?;
        let new_tokens = k_value_before.checked_div(new_sol)?;

        let tokens_out: u128 = current_tokens.checked_sub(new_tokens)?;

        // Convert back to mint decimal places for tokens
        let tokens_out = tokens_out
            .checked_mul(mint_decimals)? // Convert to mint decimals
            .checked_div(SOLANA_DECIMALS as u128)?; // From 9 decimals

        log_value("GetTokensForBuySol: tokens_out:", tokens_out);
        <u128 as TryInto<u64>>::try_into(tokens_out).ok()
    }

    pub fn invariant(&self, accounts: &[AccountInfo]) -> Result<(), ProgramError> {
        if let [curve_mint_ata, curve_sol_escrow] = accounts {
            require(
                pubkey_eq(curve_mint_ata.owner(), &TOKEN_PROGRAM_ID),
                ProgramError::IncorrectAuthority,
            )?;

            let token_account_info = TokenAccount::from_account_info(curve_mint_ata)
                .map_err(|_| ProgramError::InvalidAccountData)?;

            if curve_sol_escrow.lamports() < self.real_sol_reserves {
                msg!("Invariant failed: real_sol_reserves != bonding_curve_pool_lamports");
                return Err(AmmError::InvariantFailed.into());
            }

            let tkn_balance_minus_liquidity = token_account_info
                .amount()
                .checked_sub(
                    self.token_total_supply
                        .checked_sub(self.initial_real_token_reserves)
                        .ok_or(ProgramError::ArithmeticOverflow)?,
                )
                .ok_or(ProgramError::ArithmeticOverflow)?;

            if tkn_balance_minus_liquidity != self.real_token_reserves {
                msg!("Invariant failed: real_token_reserves != tkn_balance");
                return Err(AmmError::InvariantFailed.into());
            }

            if self.complete == 1 && self.real_token_reserves != 0 {
                msg!("Invariant failed: bonding curve marked as complete but real_token_reserves != 0");
                return Err(AmmError::InvariantFailed.into());
            }

            if self.complete == 0 && !token_account_info.is_frozen() {
                msg!("Active BondingCurve TokenAccount must always be frozen at the end");
                return Err(AmmError::InvariantFailed.into());
            }

            Ok(())
        } else {
            Err(ProgramError::NotEnoughAccountKeys)
        }
    }
}
