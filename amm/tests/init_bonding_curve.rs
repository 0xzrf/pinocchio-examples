mod helpers;

#[cfg(test)]
pub mod init_curve_tests {
    use super::*;
    use amm::ID;
    use helpers::{
        get_mollusk, ix_configs::init_bonding_curve_configs::get_init_bonding_curve_configs,
        ReturnVal,
    };
    use mollusk_svm::result::Check;
    use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

    #[test]
    pub fn test_init_curve_runs_sucessfully() {
        let program_id = Pubkey::new_from_array(ID);
        let mollusk = get_mollusk(&program_id);
        let ReturnVal {
            account_infos,
            account_meta,
            ix_data,
        } = get_init_bonding_curve_configs(&mollusk, &program_id);

        let ix = Instruction::new_with_bytes(program_id, &ix_data, account_meta);

        let checks = [Check::success()];

        let _ = mollusk.process_and_validate_instruction(&ix, &account_infos, &checks);
    }
}
