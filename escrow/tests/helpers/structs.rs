use solana_sdk::account::Account;
use solana_sdk::message::AccountMeta;
use solana_sdk::pubkey::Pubkey;

pub struct ReturnVal {
    pub account_meta: Vec<AccountMeta>,
    pub account_infos: Vec<(Pubkey, Account)>,
    pub ix_data: Vec<u8>,
}

pub struct SystemConfig {
    pub system_config: (Pubkey, Account),
    pub token_config: (Pubkey, Account),
}
