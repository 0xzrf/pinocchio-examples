use crate::{bps_mul, states::global_config::GlobalConfig};
use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    instruction::Seed,
    log::sol_log,
    program_error::ProgramError,
    pubkey::Pubkey,
    seeds,
    sysvars::{clock::Clock, Sysvar},
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CreateCurveArgs {
    pub name: [u8; 15],
    pub uri: [u8; 32],
    pub symbol: [u8; 5],
}

impl CreateCurveArgs {
    pub const LEN: usize = core::mem::size_of::<Self>();
}

impl BondingCurve {
    pub const CURVE_SIZE: usize = core::mem::size_of::<Self>();
    pub const SEED_PREFIX: &[u8] = b"bonding_curve";
    pub const SOL_ESCROW_SEED_PREFIX: &[u8] = b"sol_escrow";
    pub const MINT_SEED_PREFIX: &[u8] = b"curve_mint";

    #[inline(always)]
    /// Loads and returns a mutable reference to the [`BondingCurve`] struct stored in the given account.
    ///
    /// # Safety
    ///
    /// This function uses [`AccountInfo::borrow_mut_data_unchecked`] to obtain a mutable reference to the account's data buffer.
    /// This is inherently unsafe because it bypasses Rust's usual borrow checking, and can cause undefined behavior if:
    /// - The same account's data is mutably borrowed more than once in the same scope.
    /// - The account's data is accessed elsewhere while this reference is alive.
    ///
    /// **To avoid undefined behavior, ensure that you do not call this function multiple times on the same account within a single scope.**
    ///
    /// # Errors
    ///
    /// Returns [`ProgramError::InvalidAccountData`] if the account's data cannot be parsed as a [`BondingCurve`] struct.
    ///
    /// # Arguments
    ///
    /// * `curve_account` - The [`AccountInfo`] containing the serialized [`BondingCurve`] data.
    ///
    /// # Example
    ///
    /// ```
    /// let curve_data = BondingCurve::load(&curve_account)?;
    /// ```
    pub fn load(curve_account: &AccountInfo) -> Result<&mut Self, ProgramError> {
        let data = unsafe { curve_account.borrow_mut_data_unchecked() };

        bytemuck::try_from_bytes_mut::<BondingCurve>(data)
            .map_err(|_| ProgramError::InvalidAccountData)
    }

    pub fn get_signer_seeds(mint: &Pubkey) -> [Seed; 2] {
        seeds!(Self::SEED_PREFIX, mint.as_ref())
    }

    pub fn init(
        bump: u8,
        configs: GlobalConfig,
        curve_account: &AccountInfo,
        creator_key: &Pubkey,
        mint: &Pubkey,
    ) -> Result<(), ProgramError> {
        let curve_data = BondingCurve::load(curve_account)?;

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
            sol_log("Phase 1: 99% fees between slot 0 - 150");
            sol_fee = bps_mul(9900, amount, 10_000).unwrap();
        } else if (150..250).contains(&slots_passed) {
            sol_log("Phase 2: Linear decrease between 150 - 250");

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
            sol_log("Phase 3: 1% fees after 250");
            sol_fee = bps_mul(100, amount, 10_000).unwrap();
        }
        Ok(sol_fee)
    }
}
