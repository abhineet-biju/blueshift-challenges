#![allow(ambiguous_glob_reexports)]
pub mod error;
pub mod instructions;

use anchor_lang::prelude::*;

pub use instructions::*;

declare_id!("22222222222222222222222222222222222222222222");

#[program]
pub mod blueshift_anchor_flash_loan {
    use super::*;

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        borrow::handler(ctx, amount)
    }
    pub fn repay(ctx: Context<Repay>) -> Result<()> {
        repay::handler(ctx)
    }
}
