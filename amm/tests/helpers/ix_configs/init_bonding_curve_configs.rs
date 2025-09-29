use crate::helpers::{
    find_deterministic_pubkey, get_program_configs, to_associated_pubkey, ReturnVal, SystemConfig,
};
use amm::states::{bonding_curve::BondingCurve, global_config::GlobalConfig};
use mollusk_svm::Mollusk;
use {
    solana_sdk::{
        account::Account, message::AccountMeta, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
    },
    spl_associated_token_account::get_associated_token_address_with_program_id,
};

pub fn get_init_bonding_curve_configs(mollusk: &Mollusk, program_id: &Pubkey) -> ReturnVal {
    let SystemConfig {
        system_config: (system_program, system_program_account),
        token_config: (token_program, token_program_account),
    } = get_program_configs();

    let creator = find_deterministic_pubkey("creator");

    let creator_account = Account::new(10 * LAMPORTS_PER_SOL, 0, &system_program);

    let global_seeds: &[&[u8]] = &[GlobalConfig::GLOBAL_PEFIX];

    let (global_config, bump) = Pubkey::find_program_address(global_seeds, &program_id);

    let mut global_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(GlobalConfig::SIZE),
        GlobalConfig::SIZE,
        &program_id,
    );

    let global_field = GlobalConfig {
        mint_decimals: 6,
        _padding: [0; 6],
        inittialized: 1,
        admin: Pubkey::new_from_array([0x2; 32]).to_bytes(),
        fee_receiver: Pubkey::new_from_array([0x1; 32]).to_bytes(),

        initial_real_token_reserves: 793_100_000_000_000,
        initial_virtual_token_reserves: 1_073_000_000_000_000,
        initial_virtual_sol_reserves: 30000000000,
        token_total_supply: 1_000_000_000_000_000,
    };

    global_account.data = bytemuck::bytes_of(&global_field).to_vec();

    let mint = find_deterministic_pubkey("mint");

    let mint_account = Account::new(0, 0, &system_program);

    let curve_seeds: &[&[u8]] = &[BondingCurve::SEED_PREFIX, mint.as_ref()];
    let (curve_pda, _) = Pubkey::find_program_address(curve_seeds, &program_id);

    let curve_account = Account::new(0, 0, &system_program);

    let curve_mint_ata = get_associated_token_address_with_program_id(
        &to_associated_pubkey(&curve_pda),
        &to_associated_pubkey(&mint),
        &to_associated_pubkey(&token_program),
    );

    let curve_mint_ata = Pubkey::new_from_array(*curve_mint_ata.as_array());

    let curve_mint_ata_account = Account::new(0, 0, &system_program);

    let sol_escrow_seeds: &[&[u8]] = &[BondingCurve::SOL_ESCROW_SEED_PREFIX, mint.as_ref()];

    let (sol_escrow_pda, _) = Pubkey::find_program_address(sol_escrow_seeds, &program_id);

    let sol_escrow_account = Account::new(0, 0, &system_program);

    let ix_data = vec![2];

    ReturnVal {
        account_infos: vec![
            (creator, creator_account),
            (global_config, global_account),
            (curve_pda, curve_account),
            (mint, mint_account),
            (curve_mint_ata, curve_mint_ata_account),
            (sol_escrow_pda, sol_escrow_account),
            (system_program, system_program_account),
            (token_program, token_program_account),
        ],
        account_meta: vec![
            AccountMeta::new(creator, true),
            AccountMeta::new(global_config, false),
            AccountMeta::new(curve_pda, true),
            AccountMeta::new(mint, true),
            AccountMeta::new(curve_mint_ata, true),
            AccountMeta::new(sol_escrow_pda, false),
            AccountMeta::new(system_program, false),
            AccountMeta::new(token_program, false),
        ],
        ix_data,
    }
}
