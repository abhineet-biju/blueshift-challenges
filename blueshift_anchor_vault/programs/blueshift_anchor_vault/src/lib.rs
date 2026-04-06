use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("22222222222222222222222222222222222222222222");

#[derive(Accounts)]
pub struct VaultAction<'info> {
    #[account()]
    user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump
        )]
    vault: SystemAccount<'info>,

    system_program: Program<'info, System>,
}
#[program]
pub mod blueshift_anchor_vault {

    use super::*;

    pub fn deposit(ctx: Context<VaultAction>, amount: u64) -> Result<()> {
        require_eq!(
            ctx.accounts.vault.lamports(),
            0,
            VaultError::VaultAlreadyExists
        );
        require_gt!(
            amount,
            Rent::get()?.minimum_balance(0),
            VaultError::InvalidAmount
        );

        let cpi_accounts = Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), cpi_accounts);

        transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<VaultAction>) -> Result<()> {
        require_neq!(ctx.accounts.vault.lamports(), 0, VaultError::InvalidAmount);

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.user.to_account_info(),
        };

        let user_key = ctx.accounts.user.key();
        let signer_seeds: &[&[u8]] = &[b"vault", user_key.as_ref(), &[ctx.bumps.vault]];
        let signer = &[signer_seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            cpi_accounts,
            signer,
        );

        transfer(cpi_ctx, ctx.accounts.vault.lamports())?;
        Ok(())
    }
}

#[error_code]
pub enum VaultError {
    #[msg("Vault already exists")]
    VaultAlreadyExists,

    #[msg("Invalid amount entered")]
    InvalidAmount,
}
