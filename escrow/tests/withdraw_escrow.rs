mod helpers;

#[cfg(test)]
pub mod tests {
    use crate::helpers::{get_mollusk, structs::ReturnVal, withdraw::withdraw_configs};
    use mollusk_svm::result::Check;
    use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

    #[test]
    pub fn test_withdraw() {
        let program_id = Pubkey::new_from_array(escrow::ID);
        let mollusk = get_mollusk(&program_id);

        let ReturnVal {
            account_infos,
            account_meta,
            ix_data,
        } = withdraw_configs(&mollusk);

        let checks = [Check::success()];

        let ix = Instruction::new_with_bytes(program_id, &ix_data, account_meta);

        let _ = mollusk.process_and_validate_instruction(&ix, &account_infos, &checks);
    }
}
