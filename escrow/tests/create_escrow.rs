mod helpers;

#[cfg(test)]
pub mod create_escrow_tests {
    use crate::helpers::{create::*, get_mollusk, structs::ReturnVal};
    use mollusk_svm::result::Check;
    use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

    #[test]
    pub fn test_validatioin_fails() {
        let program_id = Pubkey::new_from_array(escrow::ID);
        let mollusk = get_mollusk(&program_id);

        let ReturnVal {
            account_infos,
            account_meta,
            ix_data,
        } = get_create_config(10, 10, &mollusk);

        let ix = Instruction::new_with_bytes(program_id, &ix_data, account_meta.clone());

        let checks = vec![
            Check::all_rent_exempt(),
            Check::success(),
            Check::account(&account_meta[4].pubkey) // Escrow pda
                .owner(&program_id)
                .build(),
        ];

        let _result = mollusk.process_and_validate_instruction(&ix, &account_infos, &checks);
    }
}
