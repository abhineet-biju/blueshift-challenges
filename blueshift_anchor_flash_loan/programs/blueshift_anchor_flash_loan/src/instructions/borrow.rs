use crate::error::ProtocolError;
use crate::instruction;
use crate::ID;
use anchor_lang::prelude::*;
use solana_instructions_sysvar::{
    load_current_index_checked, load_instruction_at_checked, ID as INSTRUCTIONS_SYSVAR_ID,
};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Borrow<'info> {
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

pub fn handler(ctx: Context<Borrow>, borrow_amount: u64) -> Result<()> {
    // instrcution introspection
    let ixs = ctx.accounts.instructions.to_account_info();

    let current_index = load_current_index_checked(&ctx.accounts.instructions)?;
    require_eq!(current_index, 0, ProtocolError::InvalidIx);

    let instructions_sysvar = ixs.try_borrow_data()?;
    let len = u16::from_le_bytes(instructions_sysvar[0..2].try_into().unwrap());
    if let Ok(repay_ix) = load_instruction_at_checked(len as usize - 1, &ixs) {
        require_keys_eq!(repay_ix.program_id, ID, ProtocolError::InvalidProgram);
        require!(
            repay_ix.data[0..8].eq(instruction::Repay::DISCRIMINATOR),
            ProtocolError::InvalidIx
        );
        require_keys_eq!(
            repay_ix
                .accounts
                .get(3)
                .ok_or(ProtocolError::InvalidBorrowerAta)?
                .pubkey,
            ctx.accounts.borrower_ata.key(),
            ProtocolError::InvalidBorrowerAta
        );
        require_keys_eq!(
            repay_ix
                .accounts
                .get(4)
                .ok_or(ProtocolError::InvalidProtocolAta)?
                .pubkey,
            ctx.accounts.protocol_ata.key(),
            ProtocolError::InvalidProtocolAta
        );
    } else {
        return Err(ProtocolError::MissingRepayIx.into());
    }

    require_gt!(borrow_amount, 0, ProtocolError::InvalidAmount);

    let signer_seeds: &[&[u8]] = &[b"protocol".as_ref(), &[ctx.bumps.protocol]];

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.protocol_ata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.borrower_ata.to_account_info(),
                authority: ctx.accounts.protocol.to_account_info(),
            },
            &[signer_seeds],
        ),
        borrow_amount,
        ctx.accounts.mint.decimals,
    )?;

    Ok(())
}
