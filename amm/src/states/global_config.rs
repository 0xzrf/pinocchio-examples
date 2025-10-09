use crate::{load, require};
use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{find_program_address, pubkey_eq, Pubkey},
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct GlobalConfig {
    pub inittialized: u8,
    pub mint_decimals: u8,
    pub _padding: [u8; 6],

    pub admin: Pubkey,
    pub fee_receiver: Pubkey,

    // initial values for bonding curve
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub token_total_supply: u64,
}

impl GlobalConfig {
    pub const GLOBAL_PEFIX: &[u8; 13] = b"global_config";
    pub const SIZE: usize = core::mem::size_of::<GlobalConfig>();

    pub fn update_global(
        params: GlobalSettingsInput,
        global_account: &AccountInfo,
    ) -> Result<(), ProgramError> {
        let global_data = load::<GlobalConfig>(global_account)?;
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

        global_data.admin = admin;
        global_data.fee_receiver = fee_receiver;
        global_data.mint_decimals = mint_decimals;
        global_data.initial_real_token_reserves = initial_real_token_reserves;
        global_data.initial_virtual_sol_reserves = initial_virtual_sol_reserves;
        global_data.initial_virtual_token_reserves = initial_virtual_token_reserves;
        global_data.token_total_supply = token_total_supply;
        global_data.inittialized = 1;
        global_data._padding = [0u8; 6];

        Ok(())
    }

    pub fn check_id(global_account: &AccountInfo) -> Result<(), ProgramError> {
        let global_seeds: &[&[u8]] = &[GlobalConfig::GLOBAL_PEFIX];

        let (expected_global_config, _) = find_program_address(global_seeds, &crate::ID);

        require(
            pubkey_eq(global_account.key(), &expected_global_config),
            ProgramError::IncorrectProgramId,
        )?;
        Ok(())
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
