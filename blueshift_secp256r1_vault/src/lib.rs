#![no_std]

use pinocchio::{
    entrypoint, error::ProgramError, nostd_panic_handler, AccountView, Address, ProgramResult,
};

entrypoint!(process_instruction);

nostd_panic_handler!();

pub mod instructions;
use instructions::*;

pub const ID: Address = Address::from_str_const("22222222222222222222222222222222222222222222");

fn process_instruction(
    _program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts: &[AccountView] = accounts;
    match instruction_data.split_first() {
        Some((Deposit::DISCRIMINATOR, data)) => Deposit::try_from((accounts, data))?.process(),
        Some((Withdraw::DISCRIMINATOR, data)) => Withdraw::try_from((accounts, data))?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
