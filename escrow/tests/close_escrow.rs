mod helpers;

#[cfg(test)]
pub mod close_escrow_tests {
    use crate::helpers::{close::get_close_configs, get_mollusk, structs::ReturnVal};
    use mollusk_svm::result::Check;
    use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

    #[test]
    pub fn test_close() {
        let program_id = Pubkey::new_from_array(escrow::ID);
        let mollusk = get_mollusk(&program_id);

        let ReturnVal {
            account_meta,
            account_infos,
            ix_data,
        } = get_close_configs(&mollusk, &program_id);

        let ix = Instruction::new_with_bytes(program_id, &ix_data, account_meta);

        let checks = [Check::success()];

        let _ = mollusk.process_and_validate_instruction(&ix, &account_infos, &checks);
    }
}
