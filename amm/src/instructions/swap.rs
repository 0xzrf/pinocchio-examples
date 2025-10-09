use crate::{
    require,
    states::{bonding_curve::BondingCurve, global_config::GlobalConfig},
    AmmError,
};
use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    log::sol_log as msg,
    program_error::ProgramError,
    pubkey::{find_program_address, pubkey_eq, Pubkey},
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};

use pinocchio_token_2022::state::TokenAccount;

pub fn process_swap(program_id: Pubkey, accounts: &[AccountInfo], ix_data: &[u8]) -> ProgramResult {
    todo!()
}

pub fn validate(
    program_id: Pubkey,
    accounts: &[AccountInfo],
) -> Result<BondingCurve, ProgramError> {
    if let [buyer, buyer_sol_ata, buyer_mint_ata, mint_a, mint_b, curve_pda, curve_sol_escrow, curve_mint_ata, _, _] =
        accounts
    {
        let buyer_sol_info = TokenAccount::from_account_info(buyer_sol_ata)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let buyer_mint_info = TokenAccount::from_account_info(buyer_mint_ata)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let curve_sol_info = TokenAccount::from_account_info(curve_sol_escrow)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let curve_mint_info = TokenAccount::from_account_info(curve_mint_ata)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        require(buyer.is_signer(), ProgramError::MissingRequiredSignature)?;
        require(
            curve_pda.data_len() == BondingCurve::CURVE_SIZE,
            ProgramError::InvalidAccountData,
        )?;

        let curve_data = BondingCurve::load(curve_pda)?;

        require(curve_data.complete == 0, AmmError::CurveComplete.into())?;
        require(curve_data.is_started(), AmmError::CurveNotStarted.into())?;

        require(
            pubkey_eq(mint_b.key(), &curve_data.mint),
            ProgramError::IncorrectProgramId,
        )?;

        require(
            pubkey_eq(buyer_mint_info.mint(), mint_b.key())
                && pubkey_eq(curve_mint_info.mint(), mint_b.key()),
            ProgramError::IncorrectProgramId,
        )?;

        let wsol_address = b"So11111111111111111111111111111111111111112";
        // require(
        //     pubkey_eq(buyer_sol_info.mint(), wsol_address)
        // )?;

        Ok(*curve_data)
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}
