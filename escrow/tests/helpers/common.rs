use super::structs::SystemConfig;
use mollusk_svm::Mollusk;
use solana_sdk::pubkey::Pubkey;
use spl_token::ID as token_program;

pub fn get_mollusk(program_id: Pubkey) -> Mollusk {
    let mut mollusk = Mollusk::new(&program_id, "target/deploy/escrow");

    mollusk.add_program(
        &Pubkey::new_from_array(*token_program.as_array()),
        "tests/program_bytes/spl_token",
        &mollusk_svm::program::loader_keys::LOADER_V3,
    );

    mollusk
}

pub fn get_program_configs() -> SystemConfig {
    let system_config = mollusk_svm::program::keyed_account_for_system_program();
    let token_config = (
        Pubkey::new_from_array(*token_program.as_array()),
        mollusk_svm::program::create_program_account_loader_v3(&Pubkey::new_from_array(
            *token_program.as_array(),
        )),
    );

    SystemConfig {
        system_config,
        token_config,
    }
}

pub const LAMPORTS_PER_SOL: u64 = 10u64.pow(9);
