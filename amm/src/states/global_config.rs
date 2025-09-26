use crate::{constants::*, require, AmmError};
use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GlobalConfig {
    pub inittialized: u8,
    pub bump: u8,
    pub mint_decimals: u8,
    pub _padding: [u8; 5],

    pub admin: Pubkey,
    pub fee_received: Pubkey,

    // initial values for bonding curve
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub token_total_supply: u64,
}

impl GlobalConfig {
    const GLOBAL_PEFIX: &[u8; 13] = b"global_config";
    const SIZE: usize = U8_LEN * 3 + PUBKEY_LEN * 2 + U64_LEN * 4;

    pub fn update_global(
        &self,
        params: GlobalSettingsInput,
        escrow_program: &AccountInfo,
    ) -> Result<(), ProgramError> {
        let mut escrow_data = GlobalConfig::load(escrow_program)?;
        let GlobalSettingsInput {
            mint_decimals,
            fee_receiver,
            admin,
            initial_virtual_token_reserves,
            initial_virtual_sol_reserves,
            initial_real_token_reserves,
            token_total_supply,
            _padding: _,
        } = params;

        escrow_data.admin = admin;
        escrow_data.fee_received = fee_receiver;
        escrow_data.mint_decimals = mint_decimals;
        escrow_data.initial_real_token_reserves = initial_real_token_reserves;
        escrow_data.initial_virtual_sol_reserves = initial_virtual_sol_reserves;
        escrow_data.initial_virtual_token_reserves = initial_virtual_token_reserves;
        escrow_data.token_total_supply = token_total_supply;

        Ok(())
    }

    pub fn load(escrow_program: &AccountInfo) -> Result<Self, ProgramError> {
        let data = unsafe { escrow_program.borrow_mut_data_unchecked() };

        let escrow_data = bytemuck::try_from_bytes::<GlobalConfig>(&data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(*escrow_data)
    }

    pub fn validate_settings(params: &GlobalSettingsInput) -> Result<(), ProgramError> {
        require(
            params.mint_decimals <= 9,
            ProgramError::InvalidInstructionData,
        )?;

        require(
            params.token_total_supply <= u64::MAX / 2,
            ProgramError::InvalidInstructionData,
        )?;

        require(
            params.initial_virtual_token_reserves > 0,
            ProgramError::InvalidInstructionData,
        )?;
        require(
            params.initial_virtual_sol_reserves > 0,
            ProgramError::InvalidInstructionData,
        )?;
        require(
            params.initial_real_token_reserves > 0,
            ProgramError::InvalidInstructionData,
        )?;

        require(
            params.token_total_supply > params.initial_real_token_reserves,
            ProgramError::InvalidInstructionData,
        )?;

        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GlobalSettingsInput {
    pub mint_decimals: u8,
    pub _padding: [u8; 7],

    pub fee_receiver: Pubkey,
    pub admin: Pubkey,

    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub token_total_supply: u64,
}
