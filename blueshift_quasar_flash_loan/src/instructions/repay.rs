use crate::{
    error::ProtocolError,
    instruction_sysvar::{load_instruction, INSTRUCTIONS_SYSVAR_ID},
};
use quasar_lang::prelude::*;
use quasar_spl::{
    AssociatedTokenProgram, InterfaceAccount, Mint, Token, TokenCpi, TokenInterface,
};

#[derive(Accounts)]
pub struct Repay<'info> {
    pub borrower: &'info mut Signer,

    #[account(mut, seeds = [b"protocol"], bump)]
    pub protocol: &'info mut SystemAccount,

    pub mint: &'info InterfaceAccount<Mint>,

    #[account(
        init_if_needed,
        payer = borrower,
        associated_token::mint = mint,
        associated_token::authority = borrower,
        associated_token::token_program = token_program
    )]
    pub borrower_ata: &'info mut InterfaceAccount<Token>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = protocol,
        associated_token::token_program = token_program
    )]
    pub protocol_ata: &'info mut InterfaceAccount<Token>,

    #[account(address = INSTRUCTIONS_SYSVAR_ID)]
    pub instructions: &'info UncheckedAccount,

    pub token_program: &'info Interface<TokenInterface>,
    pub associated_token_program: &'info Program<AssociatedTokenProgram>,
    pub system_program: &'info Program<System>,
}

impl<'info> Repay<'info> {
    #[inline(always)]
    pub fn repay(&self) -> Result<(), ProgramError> {
        let ix_data = unsafe { self.instructions.to_account_view().borrow_unchecked() };
        let borrow_ix = load_instruction(ix_data, 0).ok_or(ProtocolError::MissingBorrowIx)?;
        let amount_slice = borrow_ix
            .data
            .get(8..16)
            .ok_or(ProtocolError::MissingBorrowIx)?;

        let mut amount_bytes = [0u8; 8];
        amount_bytes.copy_from_slice(amount_slice);
        let mut amount_borrowed = u64::from_le_bytes(amount_bytes);

        let fee = (amount_borrowed as u128)
            .checked_mul(500)
            .ok_or(ProtocolError::Overflow)?
            .checked_div(10_000)
            .ok_or(ProtocolError::Overflow)? as u64;

        amount_borrowed = amount_borrowed
            .checked_add(fee)
            .ok_or(ProtocolError::Overflow)?;

        self.token_program
            .transfer_checked(
                self.borrower_ata,
                self.mint,
                self.protocol_ata,
                self.borrower,
                amount_borrowed,
                self.mint.decimals(),
            )
            .invoke()
    }
}
