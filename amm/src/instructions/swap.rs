use crate::{
    load, require,
    states::{bonding_curve::BondingCurve, global_config::GlobalConfig},
    AmmError,
};
use bytemuck::{Pod, Zeroable};
use {
    pinocchio::{
        account_info::AccountInfo,
        msg,
        program_error::ProgramError,
        pubkey::{find_program_address, pubkey_eq, Pubkey},
        ProgramResult,
    },
    pinocchio_token_2022::state::TokenAccount,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SwapParams {
    pub base_in: u8,
    pub padding: [u8; 7],
    pub exact_in_amount: u64,
    pub min_out_amount: u64,
}

pub fn process_swap(program_id: Pubkey, accounts: &[AccountInfo], ix_data: &[u8]) -> ProgramResult {
    todo!()
}

pub fn validate(
    program_id: Pubkey,
    accounts: &[AccountInfo],
    ix_data: &[u8],
) -> Result<BondingCurve, ProgramError> {
    if let [buyer, buyer_sol_ata, buyer_mint_ata, mint_a, mint_b, config, curve_pda, curve_sol_escrow, curve_mint_ata, fee_receiver, _, _] =
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

        BondingCurve::check_id(curve_pda, *mint_b.key())?;

        GlobalConfig::check_id(config)?;

        let curve_data = load::<BondingCurve>(curve_pda)?;

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

        // TODO: check if mint_a is wsol address

        let ix_params = bytemuck::try_from_bytes::<SwapParams>(ix_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        require(
            ix_params.exact_in_amount > 0,
            ProgramError::InvalidInstructionData,
        )?;
        require(
            buyer_mint_info.amount() >= ix_params.exact_in_amount,
            ProgramError::InsufficientFunds,
        )?;

        Ok(*curve_data)
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}
