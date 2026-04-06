#![no_std]

use quasar_lang::prelude::*;

pub mod error;
mod instruction_sysvar;
pub mod instructions;

use instructions::*;

pub const BORROW_DISCRIMINATOR: [u8; 8] = [228, 253, 131, 202, 207, 116, 89, 18];
pub const REPAY_DISCRIMINATOR: [u8; 8] = [234, 103, 67, 82, 208, 234, 219, 166];

declare_id!("22222222222222222222222222222222222222222222");

#[program]
mod blueshift_quasar_flash_loan {
    use super::*;

    #[instruction(discriminator = [228, 253, 131, 202, 207, 116, 89, 18])]
    pub fn borrow(ctx: Ctx<Borrow>, amount: u64) -> Result<(), ProgramError> {
        ctx.accounts.borrow(amount, &ctx.bumps)
    }

    #[instruction(discriminator = [234, 103, 67, 82, 208, 234, 219, 166])]
    pub fn repay(ctx: Ctx<Repay>) -> Result<(), ProgramError> {
        ctx.accounts.repay()
    }
}

#[cfg(test)]
mod tests;
