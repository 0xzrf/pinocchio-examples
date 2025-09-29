mod helpers;

#[cfg(test)]
pub mod init_curve_tests {
    use super::*;
    use amm::ID;
    use helpers::{
        get_mollusk, ix_configs::init_bonding_curve_configs::get_init_bonding_curve_configs,
    };
    use solana_sdk::pubkey::Pubkey;

    #[test]
    pub fn test_init_curve_runs_sucessfully() {
        let program_id = Pubkey::new_from_array(ID);
        let mollusk = get_mollusk(&program_id);
        get_init_bonding_curve_configs(&mollusk, &program_id);
        assert!(true);
    }
}
