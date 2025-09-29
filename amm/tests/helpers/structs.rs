use solana_sdk::{account::Account, message::AccountMeta, pubkey::Pubkey};

pub struct ReturnVal {
    pub account_meta: Vec<AccountMeta>,
    pub account_infos: Vec<(Pubkey, Account)>,
    pub ix_data: Vec<u8>,
}

pub struct SystemConfig {
    pub system_config: (Pubkey, Account),
    pub token_config: (Pubkey, Account),
    pub associated_program_config: (Pubkey, Account),
}
