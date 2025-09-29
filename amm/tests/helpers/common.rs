use super::structs::SystemConfig;
use mollusk_svm::Mollusk;
use solana_sdk::{
    account::{Account, WritableAccount},
    pubkey::Pubkey,
};
use spl_associated_token_account::solana_program::pubkey::Pubkey as aPubkey;
use spl_token::{
    solana_program::{program_option::COption, program_pack::Pack, pubkey::Pubkey as sPubkey},
    state::{Account as ATA, Mint},
};
use spl_token_2022::ID as token_program;

pub fn get_mollusk(program_id: &Pubkey) -> Mollusk {
    let mut mollusk = Mollusk::new(program_id, "target/deploy/amm");

    mollusk.add_program(
        &Pubkey::new_from_array(*token_program.as_array()),
        "tests/elf_files/token_2022",
        &mollusk_svm::program::loader_keys::LOADER_V3,
    );
    mollusk.add_program(
        &Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
        "tests/elf_files/associated_token_program",
        &mollusk_svm::program::loader_keys::LOADER_V3,
    );

    mollusk
}

pub fn find_deterministic_pubkey(id: &str) -> Pubkey {
    assert!(id.len() <= 32, "id too long, must be <= 32");

    let mut seed_array = [b'0'; 32];
    let id_bytes = id.as_bytes();

    seed_array[..id_bytes.len()].copy_from_slice(id_bytes);

    Pubkey::new_from_array(seed_array)
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
pub fn get_mint_accounts(
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
pub fn get_ata_accounts(
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

pub fn to_associated_pubkey(pubkey: &Pubkey) -> aPubkey {
    aPubkey::new_from_array(*pubkey.as_array())
}

pub fn to_spl_pubkey(pubkey: &Pubkey) -> sPubkey {
    sPubkey::new_from_array(*pubkey.as_array())
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
