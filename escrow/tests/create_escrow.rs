mod helpers;

#[cfg(test)]
pub mod create_escrow_tests {
    use crate::helpers::{create::*, get_mollusk};
    use solana_sdk::pubkey::Pubkey;

    #[test]
    pub fn test_validatioin_fails() {
        let program_id = Pubkey::new_from_array(escrow::ID);
        let mollusk = get_mollusk(program_id);
    }
}
