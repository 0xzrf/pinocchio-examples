use crate::{
    load, load_read_only, require,
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
    pinocchio_token_2022::state::{AccountState, TokenAccount},
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
    msg!("AMM INSTRUCTION: SWAP");
    let (curve_data, swap_params) = validate(program_id, accounts, ix_data)?;

    if let [buyer, buyer_sol_ata, buyer_mint_ata, mint_a, mint_b, config, curve_pda, curve_sol_escrow, curve_mint_ata, fee_receiver, _system_program, _token_program] =
        accounts
    {
        Ok(())
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}

pub fn validate(
    program_id: Pubkey,
    accounts: &[AccountInfo],
    ix_data: &[u8],
) -> Result<(BondingCurve, SwapParams), ProgramError> {
    if let [buyer, buyer_sol_ata, buyer_mint_ata, mint_a, mint_b, config, curve_pda, curve_sol_escrow, curve_mint_ata, fee_receiver, _, _] =
        accounts
    {
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

        let global_config = load_read_only::<GlobalConfig>(config)?;

        require(
            pubkey_eq(fee_receiver.key(), &global_config.fee_receiver),
            ProgramError::IncorrectProgramId,
        )?;

        require(
            pubkey_eq(mint_b.key(), &curve_data.mint),
            ProgramError::IncorrectProgramId,
        )?;
        let buyer_sol_info = TokenAccount::from_account_info(buyer_sol_ata)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let buyer_mint_info = TokenAccount::from_account_info(buyer_mint_ata)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let curve_sol_info = TokenAccount::from_account_info(curve_sol_escrow)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let curve_mint_info = TokenAccount::from_account_info(curve_mint_ata)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        require(
            pubkey_eq(buyer_mint_info.mint(), mint_b.key())
                && pubkey_eq(curve_mint_info.mint(), mint_b.key()),
            ProgramError::IncorrectProgramId,
        )?;

        require(
            pubkey_eq(buyer_mint_info.owner(), buyer.key())
                && pubkey_eq(buyer_sol_info.owner(), buyer.key())
                && pubkey_eq(curve_mint_info.owner(), curve_pda.key())
                && pubkey_eq(curve_sol_escrow.owner(), curve_pda.key()),
            ProgramError::IncorrectAuthority,
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

        require(
            ix_params.base_in == 1 && ix_params.base_in == 0,
            ProgramError::InvalidInstructionData,
        )?;

        Ok((*curve_data, *ix_params))
    } else {
        Err(ProgramError::NotEnoughAccountKeys)
    }
}
