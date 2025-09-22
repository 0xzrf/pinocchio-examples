use borsh::BorshSerialize;
use escrow::{
    processor::EscrowInstructions,
    states::{CreateEscrow, EscrowPda},
};
use mollusk_svm::Mollusk;
use solana_sdk::{
    account::{Account, WritableAccount},
    pubkey::Pubkey,
};
use spl_token::{
    solana_program::{program_option::COption, program_pack::Pack},
    state::Mint,
};

const LAMPORTS_PER_SOL: u64 = 10u64.pow(9);
const SPL_TOKEN_ID: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub fn get_raw_ix_data(send: u64, recv: u64) -> Vec<u8> {
    let ix_data = EscrowInstructions::CreateEscrow(CreateEscrow {
        recv_amount: recv,
        send_amount: send,
    });

    let mut writer = Vec::new();

    ix_data
        .serialize(&mut writer)
        .expect("Unable to serialize the ix_data");

    writer
}

pub fn get_account_infos() -> [Account; 6] {
    let (mollusk, program_id) = get_mollusk();
    let (system_program, _) = mollusk_svm::program::keyed_account_for_system_program();

    let creator = Pubkey::new_unique();

    let creator_account = Account::new(10 * LAMPORTS_PER_SOL, 0, &system_program);

    let mint_a = Pubkey::new_from_array([0x02; 32]);

    let signer_seeds = [
        EscrowPda::ESCROW_PREFIX.as_bytes(),
        creator.as_ref(),
        mint_a.as_ref(),
    ];

    let (escrow_pda, _) =
        solana_sdk::pubkey::Pubkey::find_program_address(&signer_seeds, &program_id);

    let mut mint_a_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(Mint::LEN),
        Mint::LEN,
        &SPL_TOKEN_ID,
    );

    spl_token::solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            decimals: 6,
            mint_authority: COption::None,
            supply: 100_000,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_a_account.data_as_mut_slice(),
    )
    .unwrap();

    let mint_b = Pubkey::new_from_array([0x03; 32]);

    let mut mint_b_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(Mint::LEN),
        Mint::LEN,
        &SPL_TOKEN_ID,
    );

    spl_token::solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            decimals: 6,
            mint_authority: COption::None,
            supply: 100_000,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_b_account.data_as_mut_slice(),
    )
    .unwrap();

    todo!()
}

pub fn get_mollusk() -> (Mollusk, Pubkey) {
    let program_id = Pubkey::new_unique();

    let mut mollusk = Mollusk::new(&program_id, "target/deploy/escrow");

    mollusk.add_program(
        &SPL_TOKEN_ID,
        "tests/program_bytes/spl_token",
        &mollusk_svm::program::loader_keys::LOADER_V3,
    );

    (mollusk, program_id)
}
