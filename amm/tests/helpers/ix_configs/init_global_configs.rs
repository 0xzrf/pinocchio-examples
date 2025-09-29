use crate::helpers::{get_program_configs, ReturnVal, SystemConfig};
use amm::states::global_config::{GlobalConfig, GlobalSettingsInput};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::{account::Account, message::AccountMeta, pubkey::Pubkey};

pub fn get_init_global_configs(program_id: &Pubkey) -> ReturnVal {
    let admin = Pubkey::new_unique();

    let SystemConfig {
        system_config: (system_address, system_account),
        token_config: _,
        associated_program_config: _,
    } = get_program_configs();

    let admin_account = Account::new(10 * LAMPORTS_PER_SOL, 0, &system_address);
    let seeds: &[&[u8]] = &[GlobalConfig::GLOBAL_PEFIX];

    let (global_pubkey, _) = Pubkey::find_program_address(seeds, program_id);

    let global_account = Account::new(0, 0, &system_address);

    let ix_args = GlobalSettingsInput {
        mint_decimals: 6,
        _padding: [0; 7],

        admin: Pubkey::new_from_array([0x2; 32]).to_bytes(),
        fee_receiver: Pubkey::new_from_array([0x1; 32]).to_bytes(),

        initial_real_token_reserves: 793_100_000_000_000,
        initial_virtual_token_reserves: 1_073_000_000_000_000,
        initial_virtual_sol_reserves: 30000000000,
        token_total_supply: 1_000_000_000_000_000,
    };

    // instruction discriminator = 0
    let mut ix_data = vec![0];

    // Serialize the instruction data
    ix_data.extend_from_slice(bytemuck::bytes_of(&ix_args));

    ReturnVal {
        account_infos: vec![
            (admin, admin_account),
            (global_pubkey, global_account),
            (system_address, system_account),
        ],
        account_meta: vec![
            AccountMeta::new(admin, true),
            AccountMeta::new(global_pubkey, true),
            AccountMeta::new(system_address, false),
        ],
        ix_data,
    }
}
