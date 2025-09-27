use crate::{
    require,
    states::global_config::{GlobalConfig, GlobalSettingsInput},
};
use {
    pinocchio::{
        account_info::AccountInfo,
        log::sol_log,
        program_error::ProgramError,
        pubkey::{find_program_address, pubkey_eq, Pubkey},
        sysvars::{rent::Rent, Sysvar},
        ProgramResult,
    },
    pinocchio_system::instructions::CreateAccount,
};

pub fn init_global(program_id: &Pubkey, accounts: &[AccountInfo], ix_data: &[u8]) -> ProgramResult {
    sol_log("AMM Instruction: INIT_GLOBAL");
    if let [admin, global_config, _] = accounts {
        require(admin.is_signer(), ProgramError::MissingRequiredSignature)?;

        require(
            global_config.data_is_empty(),
            ProgramError::AccountAlreadyInitialized,
        )?;

        let seeds: &[&[u8]] = &[GlobalConfig::GLOBAL_PEFIX];

        let (global_config_pda, bump) = find_program_address(seeds, program_id);

        require(
            pubkey_eq(&global_config_pda, global_config.key()),
            ProgramError::IncorrectProgramId,
        )?;
        require(
            ix_data.len() == GlobalConfig::SIZE,
            ProgramError::InvalidInstructionData,
        )?;

        sol_log("Validation successful");

        CreateAccount {
            from: admin,
            lamports: (Rent::get()?).minimum_balance(GlobalConfig::SIZE),
            space: GlobalConfig::SIZE as u64,
            owner: program_id,
            to: global_config,
        }
        .invoke()?;

        let mut aligned_ix_buf = [0u8; GlobalConfig::SIZE]; // putting raw ix_data will fail since it started at index 1 of the original instruction_data, so this new allocation is required

        aligned_ix_buf.copy_from_slice(ix_data);

        let params = bytemuck::try_from_bytes::<GlobalSettingsInput>(&aligned_ix_buf)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        GlobalConfig::validate_settings(params)?;

        GlobalConfig::update_global(*params, global_config, bump)?;
    } else {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    Ok(())
}
