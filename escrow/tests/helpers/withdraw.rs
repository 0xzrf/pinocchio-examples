use super::{
    common::{
        get_ata_config, get_ata_configs, get_ix_data, get_mint_config, get_mint_configs,
        get_program_configs, LAMPORTS_PER_SOL,
    },
    structs::{ReturnVal, SystemConfig},
};
use borsh::BorshSerialize;
use escrow::{processor::EscrowInstructions, states::EscrowPda};
use mollusk_svm::Mollusk;
use solana_sdk::{account::Account, message::AccountMeta, pubkey::Pubkey};
use spl_token::solana_program::pubkey::Pubkey as sPubkey;

pub fn withdraw_configs(mollusk: &Mollusk) -> ReturnVal {
    let program_id = Pubkey::new_from_array(escrow::ID);

    let SystemConfig {
        system_config,
        token_config,
    } = get_program_configs();

    let (system_program, system_program_account) = system_config;
    let (token_program, token_program_account) = token_config;

    let taker = Pubkey::new_unique();
    let taker_account = Account::new(10 * LAMPORTS_PER_SOL, 0, &system_program);

    let mint_a_config = get_mint_config(100_000 * 10u64.pow(6));
    let (mint_a, mint_a_account) = get_mint_configs(None, mollusk, mint_a_config);
    let creator = Pubkey::new_unique();
    let signer_seeds = [
        EscrowPda::ESCROW_PREFIX.as_bytes(),
        creator.as_ref(),
        mint_a.as_ref(),
    ];

    let (escrow_pda, escrow_bump) =
        solana_sdk::pubkey::Pubkey::find_program_address(&signer_seeds, &program_id);

    let mint_b_config = get_mint_config(100_000 * 10u64.pow(6));

    let (mint_b, mint_b_account) = get_mint_configs(None, mollusk, mint_b_config);

    let taker_ata_config = get_ata_config(
        90_000u64.checked_mul(10u64.pow(6)).unwrap(),
        sPubkey::new_from_array(*mint_a.as_array()),
        sPubkey::new_from_array(*taker.as_array()),
    );
    let (taker_a_mint_ata, taker_a_mint_ata_account) = // This ata's balance should increse
        get_ata_configs(Some([0x04; 32]), mollusk, taker_ata_config);
    let taker_b_ata_config = get_ata_config(
        90_000u64.checked_mul(10u64.pow(6)).unwrap(),
        sPubkey::new_from_array(*mint_b.as_array()),
        sPubkey::new_from_array(*taker.as_array()),
    );

    let (taker_b_mint_ata, taker_b_mint_ata_account) = // This will decrese in amount
        get_ata_configs(Some([0x05; 32]), mollusk, taker_b_ata_config);

    let maker_ata_config = get_ata_config(
        90_000u64.checked_mul(10u64.pow(6)).unwrap(),
        sPubkey::new_from_array(*mint_b.as_array()),
        sPubkey::new_from_array(*creator.as_array()),
    );
    let (maker_b_mint_ata, maker_b_mint_ata_account) = // this will increase in amount
        get_ata_configs(Some([0x06; 32]), mollusk, maker_ata_config);

    let vault_ata_config = get_ata_config(
        20_000u64.checked_mul(10u64.pow(6)).unwrap(),
        sPubkey::new_from_array(*mint_a.as_array()),
        sPubkey::new_from_array(*escrow_pda.as_array()),
    );

    let (vault_a_ata, vault_a_ata_account) =
        get_ata_configs(Some([0x07; 32]), mollusk, vault_ata_config);

    let mut escrow_pda_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(EscrowPda::ESCROW_SIZE),
        0,
        &program_id,
    );

    let escrow_data = EscrowPda {
        creator: *creator.as_array(),
        amount: 20_000u64.checked_mul(10u64.pow(6)).unwrap(), // taker a ata will gain
        bump: escrow_bump,
        mint_a: *mint_a.as_array(),
        mint_b: *mint_b.as_array(),
        receive: 10_000u64.checked_mul(10u64.pow(6)).unwrap(), // taker b ata will gain
    };

    escrow_data.serialize(&mut escrow_pda_account.data).unwrap();

    let ix_data = get_ix_data(EscrowInstructions::Withdraw);

    ReturnVal {
        account_infos: vec![
            (taker, taker_account),
            (taker_b_mint_ata, taker_b_mint_ata_account),
            (taker_a_mint_ata, taker_a_mint_ata_account),
            (maker_b_mint_ata, maker_b_mint_ata_account),
            (mint_a, mint_a_account),
            (mint_b, mint_b_account),
            (escrow_pda, escrow_pda_account),
            (vault_a_ata, vault_a_ata_account),
            (token_program, token_program_account),
        ],
        account_meta: vec![
            AccountMeta::new(taker, true),
            AccountMeta::new(taker_b_mint_ata, false),
            AccountMeta::new(taker_a_mint_ata, false),
            AccountMeta::new(maker_b_mint_ata, false),
            AccountMeta::new(mint_a, false),
            AccountMeta::new(mint_b, false),
            AccountMeta::new(escrow_pda, true),
            AccountMeta::new(vault_a_ata, false),
            AccountMeta::new(token_program, false),
        ],
        ix_data,
    }
}
