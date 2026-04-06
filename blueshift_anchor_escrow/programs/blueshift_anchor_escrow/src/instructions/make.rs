use crate::errors::EscrowError;
use crate::state::EscrowConfig;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        init,
        space = 1 + EscrowConfig::INIT_SPACE,
        payer = maker,
        seeds = [b"escrow", maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump,
        )]
    pub escrow_config: Account<'info, EscrowConfig>,

    #[account(mint::token_program = token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
        )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow_config,
        associated_token::token_program = token_program
        )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    fn populate_escrow(&mut self, id: u64, receive_amount: u64, bump: u8) -> Result<()> {
        self.escrow_config.set_inner(EscrowConfig {
            id,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            receive_amount: receive_amount,
            bump: bump,
        });

        Ok(())
    }

    fn deposit_token(&mut self, amount: u64) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.maker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

        transfer_checked(cpi_ctx, amount, self.mint_a.decimals)?;

        Ok(())
    }
}

pub fn handler(ctx: Context<Make>, id: u64, amount: u64, receive: u64) -> Result<()> {
    require_gt!(amount, 0, EscrowError::InvalidAmount);
    require_gt!(receive, 0, EscrowError::InvalidAmount);

    ctx.accounts
        .populate_escrow(id, receive, ctx.bumps.escrow_config)?;

    ctx.accounts.deposit_token(amount)?;

    Ok(())
}
