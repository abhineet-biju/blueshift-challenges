use crate::{
    error::ProtocolError,
    instruction_sysvar::{
        current_instruction_index, instruction_count, load_instruction, INSTRUCTIONS_SYSVAR_ID,
    },
    REPAY_DISCRIMINATOR,
};
use quasar_lang::prelude::*;
use quasar_spl::{
    AssociatedTokenProgram, InterfaceAccount, Mint, Token, TokenCpi, TokenInterface,
};

#[derive(Accounts)]
pub struct Borrow<'info> {
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

impl<'info> Borrow<'info> {
    #[inline(always)]
    pub fn borrow(&self, amount: u64, bumps: &BorrowBumps) -> Result<(), ProgramError> {
        let ix_data = unsafe { self.instructions.to_account_view().borrow_unchecked() };

        let current_index = current_instruction_index(ix_data).ok_or(ProtocolError::InvalidIx)?;
        require_eq!(current_index, 0, ProtocolError::InvalidIx);

        let len = instruction_count(ix_data).ok_or(ProtocolError::MissingRepayIx)?;
        let last_index = len.checked_sub(1).ok_or(ProtocolError::MissingRepayIx)?;
        let repay_ix = load_instruction(ix_data, last_index).ok_or(ProtocolError::MissingRepayIx)?;

        require_keys_eq!(repay_ix.program_id, crate::ID, ProtocolError::InvalidProgram);
        require!(
            repay_ix.data.starts_with(&REPAY_DISCRIMINATOR),
            ProtocolError::InvalidIx
        );

        let borrower_ata = repay_ix
            .account_pubkey(3)
            .ok_or(ProtocolError::InvalidBorrowerAta)?;
        require_keys_eq!(
            borrower_ata,
            *self.borrower_ata.address(),
            ProtocolError::InvalidBorrowerAta
        );

        let protocol_ata = repay_ix
            .account_pubkey(4)
            .ok_or(ProtocolError::InvalidProtocolAta)?;
        require_keys_eq!(
            protocol_ata,
            *self.protocol_ata.address(),
            ProtocolError::InvalidProtocolAta
        );

        require!(amount > 0, ProtocolError::InvalidAmount);

        let signer_seeds = bumps.protocol_seeds();

        self.token_program
            .transfer_checked(
                self.protocol_ata,
                self.mint,
                self.borrower_ata,
                self.protocol,
                amount,
                self.mint.decimals(),
            )
            .invoke_signed(&signer_seeds)
    }
}
