use super::{
    common::{
        get_ata_accounts, get_ata_config, get_ix_data, get_mint_accounts, get_mint_config,
        get_program_configs, LAMPORTS_PER_SOL,
    },
    structs::{ReturnVal, SystemConfig},
};
use borsh::BorshSerialize;
use escrow::{processor::EscrowInstructions, states::EscrowPda};
use mollusk_svm::Mollusk;
use solana_sdk::{account::Account, message::AccountMeta, pubkey::Pubkey};
use spl_token::solana_program::pubkey::Pubkey as sPubkey;

pub fn get_close_configs(mollusk: &Mollusk, program_id: &Pubkey) -> ReturnVal {
    let creator = Pubkey::new_from_array([0x01; 32]);
    let SystemConfig {
        system_config,
        token_config,
    } = get_program_configs();

    let (token_program, token_program_account) = token_config;
    let (system_program, _) = system_config;

    let creator_account = Account::new(
        10u64.checked_mul(LAMPORTS_PER_SOL).unwrap(),
        0,
        &system_program,
    );

    let mint_a_config = get_mint_config(100_000 * 10u64.pow(6));

    let (mint_a, _) = get_mint_accounts(Some([0x02; 32]), mollusk, mint_a_config);
    let mint_a_config = get_mint_config(100_000 * 10u64.pow(6));

    let (mint_b, _) = get_mint_accounts(Some([0x03; 32]), mollusk, mint_a_config);

    let creator_a_config = get_ata_config(
        10_000 * 10u64.pow(6),
        sPubkey::new_from_array(*mint_a.as_array()),
        sPubkey::new_from_array(*creator.as_array()),
    );

    let (creator_a_ata, creator_a_ata_account) =
        get_ata_accounts(Some([0x04; 32]), mollusk, creator_a_config);

    let signer_seeds = [
        EscrowPda::ESCROW_PREFIX.as_bytes(),
        creator.as_ref(),
        mint_a.as_ref(),
    ];

    let (escrow_pda, bump) = Pubkey::find_program_address(&signer_seeds, program_id);

    let mut escrow_pda_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(EscrowPda::ESCROW_SIZE),
        0,
        program_id,
    );

    let escrow_data = EscrowPda {
        creator: *creator.as_array(),
        amount: 20_000u64.checked_mul(10u64.pow(6)).unwrap(), // taker a ata will gain
        bump,
        mint_a: *mint_a.as_array(),
        mint_b: *mint_b.as_array(),
        receive: 10_000u64.checked_mul(10u64.pow(6)).unwrap(), // taker b ata will gain
    };

    escrow_data.serialize(&mut escrow_pda_account.data).unwrap();

    let vault_a_config = get_ata_config(
        20_000 * 10u64.pow(6),
        sPubkey::new_from_array(*mint_a.as_array()),
        sPubkey::new_from_array(*escrow_pda.as_array()),
    );

    let (vault_a_ata, vault_a_ata_account) =
        get_ata_accounts(Some([0x05; 32]), mollusk, vault_a_config);

    let ix_data = get_ix_data(EscrowInstructions::Close);

    ReturnVal {
        account_infos: vec![
            (creator, creator_account),
            (creator_a_ata, creator_a_ata_account),
            (escrow_pda, escrow_pda_account),
            (vault_a_ata, vault_a_ata_account),
            (token_program, token_program_account),
        ],
        account_meta: vec![
            AccountMeta::new(creator, true),
            AccountMeta::new(creator_a_ata, false),
            AccountMeta::new(escrow_pda, true),
            AccountMeta::new(vault_a_ata, false),
            AccountMeta::new(token_program, false),
        ],
        ix_data,
    }
}
