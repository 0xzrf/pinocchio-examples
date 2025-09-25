use crate::helpers::{
    common::{get_ata_accounts, get_mint_accounts, get_program_configs, LAMPORTS_PER_SOL},
    structs::{ReturnVal, SystemConfig},
};
use borsh::BorshSerialize;
use escrow::{
    processor::EscrowInstructions,
    states::{CreateEscrow, EscrowPda},
};
use mollusk_svm::Mollusk;
use solana_sdk::{account::Account, message::AccountMeta, pubkey::Pubkey};
use spl_token::{
    solana_program::{program_option::COption, program_pack::Pack, pubkey::Pubkey as sPubkey},
    state::Mint,
};

/// Get the raw ix data for create ix
///
/// Fields:
/// - `recv_amount`: The amount the to recv when someone is closing the pda
/// - `send_amount`: The amount of mint_b to be sent
///
pub fn get_create_raw_ix_data(send: u64, recv: u64) -> Vec<u8> {
    let ix_data = EscrowInstructions::CreateEscrow(CreateEscrow {
        recv_amount: recv * 10u64.pow(6),
        send_amount: send * 10u64.pow(6),
    });

    let mut writer = Vec::new();

    ix_data
        .serialize(&mut writer)
        .expect("Unable to serialize the ix_data");

    writer
}

#[allow(unused)]
/// Get the configs, like acocunt meta and vec
pub fn get_create_config(send: u64, recv: u64, mollusk: &Mollusk) -> ReturnVal {
    let SystemConfig {
        system_config,
        token_config,
    } = get_program_configs();

    let (system_program, system_program_account) = system_config;
    let (token_program, token_program_account) = token_config;

    let creator = Pubkey::new_unique();

    let creator_account = Account::new(10 * LAMPORTS_PER_SOL, 0, &system_program);

    let mint_a_config = Mint {
        decimals: 6,
        freeze_authority: COption::None,
        mint_authority: COption::None,
        supply: 100_000,
        is_initialized: true,
    };

    let (mint_a, mint_a_account) = get_mint_accounts(Some([0x01; 32]), mollusk, mint_a_config);

    let mint_b_config = Mint {
        decimals: 6,
        freeze_authority: COption::None,
        mint_authority: COption::None,
        supply: 100_000,
        is_initialized: true,
    };

    let (mint_b, mint_b_account) = get_mint_accounts(Some([0x02; 32]), mollusk, mint_b_config);

    let signer_seeds = [
        EscrowPda::ESCROW_PREFIX.as_bytes(),
        creator.as_ref(),
        mint_a.as_ref(),
    ];

    let (escrow_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &signer_seeds,
        &Pubkey::new_from_array(escrow::ID),
    );

    let escrow_account = Account::new(0, 0, &system_program);

    let creator_mint_ata = Pubkey::new_from_array([0x04; 32]);

    let creator_ata_config = spl_token::state::Account {
        amount: 90_000 * 10u64.pow(6),
        mint: sPubkey::new_from_array(*mint_a.as_array()), // TODO: Could cause issue
        close_authority: COption::None,
        owner: sPubkey::new_from_array(*creator.as_array()), // TODO: Could cause issue
        delegate: COption::None,
        delegated_amount: 0,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::None,
    };
    let (creator_mint_ata, creator_ata_account) =
        get_ata_accounts(Some([0x04; 32]), mollusk, creator_ata_config);

    let escrow_mint_ata = Pubkey::new_from_array([0x04; 32]);

    let mut vault_account = Account::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );

    let vault_config = spl_token::state::Account {
        amount: 90_000,
        mint: sPubkey::new_from_array(*mint_a.as_array()), // TODO: Could cause issue
        close_authority: COption::None,
        owner: sPubkey::new_from_array(*escrow_pda.as_array()), // TODO: Could cause issue
        delegate: COption::None,
        delegated_amount: 0,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::None,
    };
    let (_, vault) = get_ata_accounts(Some([0x04; 32]), mollusk, vault_config);

    let ix_data = get_create_raw_ix_data(send, recv);

    ReturnVal {
        account_infos: vec![
            (creator, creator_account),
            (mint_a, mint_a_account),
            (mint_b, mint_b_account),
            (creator_mint_ata, creator_ata_account),
            (escrow_pda, escrow_account),
            (escrow_mint_ata, vault_account),
            (system_program, system_program_account),
            (token_program, token_program_account),
        ],
        account_meta: vec![
            AccountMeta::new(creator, true),
            AccountMeta::new(mint_a, false),
            AccountMeta::new(mint_b, false),
            AccountMeta::new(creator_mint_ata, false),
            AccountMeta::new(escrow_pda, true),
            AccountMeta::new(escrow_mint_ata, false),
            AccountMeta::new(system_program, false),
            AccountMeta::new(token_program, false),
        ],
        ix_data,
    }
}
