mod helpers;

#[cfg(test)]
pub mod init_curve_tests {
    use super::*;
    use amm::{states::bonding_curve::BondingCurve, ID};
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

        let bonding_curve_account = account_infos[2].0;
        let mint_account = account_infos[3].0;
        let curve_seeds: &[&[u8]] = &[BondingCurve::SEED_PREFIX, mint_account.as_ref()];
        let (_, curve_bump) = Pubkey::find_program_address(curve_seeds, &program_id);

        let expected_curve_data = BondingCurve {
            initial_real_token_reserves: 793_100_000_000_000,
            token_total_supply: 1_000_000_000_000_000,
            virtual_sol_reserves: 30000000000,
            virtual_token_reserves: 1_073_000_000_000_000,
            real_sol_reserves: 0,
            _padding: [0u8; 6],
            bump: curve_bump,
            mint: *mint_account.as_array(),
            complete: 0,
            creator: *account_infos[0].0.as_array(),
            real_token_reserves: 793_100_000_000_000,
            starting_slot: mollusk.sysvars.clock.slot,
        };

        let expected_data_bytes = bytemuck::bytes_of(&expected_curve_data);

        let checks = [
            Check::success(),
            Check::all_rent_exempt(),
            Check::account(&bonding_curve_account)
                .space(BondingCurve::CURVE_SIZE)
                .lamports(
                    mollusk
                        .sysvars
                        .rent
                        .minimum_balance(BondingCurve::CURVE_SIZE),
                )
                .owner(&program_id)
                .data(expected_data_bytes)
                .build(),
        ];

        let _ = mollusk.process_and_validate_instruction(&ix, &account_infos, &checks);
    }
}
