use super::structs::SystemConfig;
use borsh::BorshSerialize;
use escrow::processor::EscrowInstructions;
use mollusk_svm::Mollusk;
use solana_sdk::{
    account::{Account, WritableAccount},
    pubkey::Pubkey,
};
use spl_token::{
    solana_program::{program_option::COption, program_pack::Pack, pubkey::Pubkey as sPubkey},
    state::{Account as ATA, Mint},
    ID as token_program,
};

pub fn get_mollusk(program_id: &Pubkey) -> Mollusk {
    let mut mollusk = Mollusk::new(program_id, "target/deploy/escrow");

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

/// Creates a new mint with cusotmizable data
///
/// Arguments:
/// - `seed`: opional seeds to make the address deterministic
/// - `mollusk`: Mollusk program to get the minimum balance
/// - `mint_data`: Data to store inside the mint
pub fn get_mint_configs(
    seed: Option<[u8; 32]>,
    mollusk: &Mollusk,
    mint_data: spl_token::state::Mint,
) -> (Pubkey, Account) {
    let mint = if let Some(mint_seeds) = seed {
        Pubkey::new_from_array(mint_seeds)
    } else {
        Pubkey::new_unique()
    };

    let mut mint_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(Mint::LEN),
        Mint::LEN,
        &Pubkey::new_from_array(*token_program.as_array()),
    );

    spl_token::solana_program::program_pack::Pack::pack(
        mint_data,
        mint_account.data_as_mut_slice(),
    )
    .unwrap();

    (mint, mint_account)
}

/// Creates a new ata with cusotmizable data
///
/// Arguments:
/// - `seed`: opional seeds to make the address deterministic
/// - `mollusk`: Mollusk program to get the minimum balance
/// - `mint_data`: Data to store inside the ata
pub fn get_ata_configs(
    seed: Option<[u8; 32]>,
    mollusk: &Mollusk,
    ata_data: spl_token::state::Account,
) -> (Pubkey, Account) {
    let ata = if let Some(mint_seeds) = seed {
        Pubkey::new_from_array(mint_seeds)
    } else {
        Pubkey::new_unique()
    };

    let mut ata_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(ATA::LEN),
        ATA::LEN,
        &Pubkey::new_from_array(*token_program.as_array()),
    );

    spl_token::solana_program::program_pack::Pack::pack(ata_data, ata_account.data_as_mut_slice())
        .unwrap();

    (ata, ata_account)
}

pub fn get_ix_data(ix_data: EscrowInstructions) -> Vec<u8> {
    let mut writer = Vec::new();

    ix_data
        .serialize(&mut writer)
        .expect("Unable to serialize the ix_data");

    writer
}

pub fn get_mint_config(supply: u64) -> Mint {
    Mint {
        decimals: 6,
        freeze_authority: COption::None,
        mint_authority: COption::None,
        is_initialized: true,
        supply,
    }
}

pub fn get_ata_config(amount: u64, mint: sPubkey, owner: sPubkey) -> ATA {
    ATA {
        amount,
        close_authority: COption::None,
        delegate: COption::None,
        delegated_amount: 0,
        is_native: COption::None,
        mint,
        owner,
        state: spl_token::state::AccountState::Initialized,
    }
}

pub const LAMPORTS_PER_SOL: u64 = 10u64.pow(9);
