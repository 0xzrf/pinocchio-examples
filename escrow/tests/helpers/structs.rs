use solana_sdk::account::Account;
use solana_sdk::message::AccountMeta;
use solana_sdk::pubkey::Pubkey;

pub struct ReturnVal {
    pub account_meta: Vec<AccountMeta>,
    pub account_infos: Vec<(Pubkey, Account)>,
}
