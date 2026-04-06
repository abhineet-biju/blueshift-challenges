use crate::error::ProtocolError;
use anchor_lang::prelude::*;
use solana_instructions_sysvar::{
    load_instruction_at_checked, ID as INSTRUCTIONS_SYSVAR_ID,
};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Repay<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    #[account(
        mut,
        seeds = [b"protocol".as_ref()],
        bump
        )]
    pub protocol: SystemAccount<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = borrower,
        associated_token::mint = mint,
        associated_token::authority = borrower,
        associated_token::token_program = token_program
        )]
    pub borrower_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = protocol
        )]
    pub protocol_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: validated by address constraint against the instructions sysvar
    #[account(address = INSTRUCTIONS_SYSVAR_ID)]
    instructions: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Repay>) -> Result<()> {
    let ixs = ctx.accounts.instructions.to_account_info();

    let mut amount_borrowed: u64;
    if let Ok(borrow_ix) = load_instruction_at_checked(0, &ixs) {
        let mut borrowed_data: [u8; 8] = [0u8; 8];
        borrowed_data.copy_from_slice(&borrow_ix.data[8..16]);
        amount_borrowed = u64::from_le_bytes(borrowed_data);
    } else {
        return Err(ProtocolError::MissingBorrowIx.into());
    }

    let fee = (amount_borrowed as u128)
        .checked_mul(500)
        .unwrap()
        .checked_div(10_000)
        .ok_or(ProtocolError::Overflow)? as u64;

    amount_borrowed = amount_borrowed
        .checked_add(fee)
        .ok_or(ProtocolError::Overflow)?;

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.borrower_ata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.protocol_ata.to_account_info(),
                authority: ctx.accounts.borrower.to_account_info(),
            },
        ),
        amount_borrowed,
        ctx.accounts.mint.decimals,
    )?;

    Ok(())
}
