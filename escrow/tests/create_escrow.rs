mod helpers;

#[cfg(test)]
pub mod create_escrow_tests {
    use crate::helpers::{create::*, get_mollusk, structs::ReturnVal};
    use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

    #[test]
    pub fn test_validatioin_fails() {
        let program_id = Pubkey::new_from_array(escrow::ID);
        let mollusk = get_mollusk(program_id);

        let ReturnVal {
            account_infos,
            account_meta,
        } = get_create_ix_account_infos();
        let ix_data = get_create_raw_ix_data(10, 10);

        let ix = Instruction::new_with_bytes(program_id, &ix_data, account_meta);

        let result = mollusk.process_instruction(&ix, &account_infos);
    }
}
