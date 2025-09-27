pub mod helpers;

#[cfg(test)]
pub mod init_global_tests {
    use super::*;
    use amm::{
        states::global_config::{GlobalConfig, GlobalSettingsInput},
        ID,
    };
    use helpers::{
        get_mollusk, ix_configs::init_global_configs::get_init_global_configs, ReturnVal,
    };
    use mollusk_svm::result::Check;
    use solana_sdk::{
        account::Account, instruction::Instruction, message::AccountMeta, pubkey::Pubkey,
    };

    #[test]
    pub fn test_init_global_works() {
        let program_id = Pubkey::new_from_array(ID);

        let mollusk = get_mollusk(&program_id);

        let ReturnVal {
            account_meta,
            account_infos,
            ix_data,
        } = get_init_global_configs(&program_id);
        let seeds: &[&[u8]] = &[GlobalConfig::GLOBAL_PEFIX];

        let (_, bump) = Pubkey::find_program_address(seeds, &program_id);

        let ix_args = GlobalConfig {
            mint_decimals: 6,
            _padding: [0; 5],
            inittialized: 1,
            bump,
            admin: Pubkey::new_from_array([0x2; 32]).to_bytes(),
            fee_receiver: Pubkey::new_from_array([0x1; 32]).to_bytes(),

            initial_real_token_reserves: 793_100_000_000_000,
            initial_virtual_token_reserves: 1_073_000_000_000_000,
            initial_virtual_sol_reserves: 30000000000,
            token_total_supply: 1_000_000_000_000_000,
        };

        let global_config_account = &account_meta[1].pubkey.clone();
        let expected_global_data = bytemuck::bytes_of(&ix_args);

        let ix = Instruction::new_with_bytes(program_id, &ix_data, account_meta);

        let checks = [
            Check::success(),
            Check::all_rent_exempt(),
            Check::account(global_config_account)
                .space(GlobalConfig::SIZE)
                .data(expected_global_data)
                .lamports(mollusk.sysvars.rent.minimum_balance(GlobalConfig::SIZE))
                .owner(&program_id)
                .rent_exempt()
                .build(),
        ];

        mollusk.process_and_validate_instruction(&ix, &account_infos, &checks);
    }

    #[test]
    pub fn test_fails_if_admin_is_not_signer() {
        let program_id = Pubkey::new_from_array(ID);

        let mollusk = get_mollusk(&program_id);

        let ReturnVal {
            mut account_meta,
            account_infos,
            ix_data,
        } = get_init_global_configs(&program_id);

        account_meta[0] = AccountMeta::new(account_meta[0].pubkey, false);

        let ix = Instruction::new_with_bytes(program_id, &ix_data, account_meta);

        mollusk.process_and_validate_instruction(
            &ix,
            &account_infos,
            &[Check::err(
                solana_sdk::program_error::ProgramError::MissingRequiredSignature,
            )],
        );
    }

    #[test]
    pub fn test_fails_if_global_not_empty() {
        let program_id = Pubkey::new_from_array(ID);

        let mollusk = get_mollusk(&program_id);

        let ReturnVal {
            account_meta,
            mut account_infos,
            ix_data,
        } = get_init_global_configs(&program_id);

        let psudo_global_account = Account::new(0, 100, &account_infos[2].0);

        account_infos[1] = (account_infos[1].0, psudo_global_account);

        let ix = Instruction::new_with_bytes(program_id, &ix_data, account_meta);

        mollusk.process_and_validate_instruction(
            &ix,
            &account_infos,
            &[Check::err(
                solana_sdk::program_error::ProgramError::AccountAlreadyInitialized,
            )],
        );
    }

    #[test]
    pub fn test_fails_wrong_global_config() {
        let program_id = Pubkey::new_from_array(ID);

        let mollusk = get_mollusk(&program_id);

        let ReturnVal {
            mut account_meta,
            mut account_infos,
            ix_data,
        } = get_init_global_configs(&program_id);

        let psudo_global = Pubkey::new_unique();
        account_infos[1] = (psudo_global, account_infos[1].1.clone());
        account_meta[1] = AccountMeta::new(psudo_global, true);

        let ix = Instruction::new_with_bytes(program_id, &ix_data, account_meta);

        mollusk.process_and_validate_instruction(
            &ix,
            &account_infos,
            &[Check::err(
                solana_sdk::program_error::ProgramError::IncorrectProgramId,
            )],
        );
    }

    #[test]
    pub fn test_fails_wrong_account_len() {
        let program_id = Pubkey::new_from_array(ID);

        let mollusk = get_mollusk(&program_id);

        let ReturnVal {
            mut account_meta,
            mut account_infos,
            ix_data,
        } = get_init_global_configs(&program_id);

        let new_pubkey = Pubkey::new_unique();
        let new_pubkey_account = Account::new(0, 0, &Pubkey::new_unique());
        account_meta.push(AccountMeta::new(new_pubkey, false));
        account_infos.push((new_pubkey, new_pubkey_account));

        let ix = Instruction::new_with_bytes(program_id, &ix_data, account_meta);

        mollusk.process_and_validate_instruction(
            &ix,
            &account_infos,
            &[Check::err(
                solana_sdk::program_error::ProgramError::NotEnoughAccountKeys,
            )],
        );
    }
}
