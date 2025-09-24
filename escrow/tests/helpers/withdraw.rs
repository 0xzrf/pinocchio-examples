use super::{
    common::{get_mint_configs, get_mollusk, get_program_configs, LAMPORTS_PER_SOL},
    structs::{ReturnVal, SystemConfig},
};
use escrow::states::EscrowPda;
use solana_sdk::{
    account::{Account, WritableAccount},
    pubkey::Pubkey,
};
use spl_token::{solana_program::program_option::COption, state::Mint};

pub fn withdraw_configs() -> ReturnVal {
    let program_id = Pubkey::new_from_array(escrow::ID);
    let mollusk = get_mollusk(Pubkey::new_from_array(escrow::ID));

    let SystemConfig {
        system_config,
        token_config,
    } = get_program_configs();

    let (system_program, system_program_account) = system_config;
    let (token_program, token_program_account) = token_config;

    let taker = Pubkey::new_unique();
    let taker_account = Account::new(10 * LAMPORTS_PER_SOL, 0, &system_program);

    let mint_a = Pubkey::new_from_array([0x02; 32]);
    let creator = Pubkey::new_unique();
    let signer_seeds = [
        EscrowPda::ESCROW_PREFIX.as_bytes(),
        creator.as_ref(),
        mint_a.as_ref(),
    ];

    let (escrow_pda, _) =
        solana_sdk::pubkey::Pubkey::find_program_address(&signer_seeds, &program_id);

    let mint_a_config = Mint {
        decimals: 6,
        freeze_authority: COption::None,
        mint_authority: COption::None,
        supply: 100_000,
        is_initialized: true,
    };

    let (mint_a, mint_a_account) = get_mint_configs(None, &mollusk, mint_a_config);

    let mint_b_config = Mint {
        decimals: 6,
        freeze_authority: COption::None,
        mint_authority: COption::None,
        supply: 100_000,
        is_initialized: true,
    };

    let (mint_b, mint_b_account) = get_mint_configs(None, &mollusk, mint_b_config);

    todo!()
}
