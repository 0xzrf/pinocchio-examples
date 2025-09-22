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
