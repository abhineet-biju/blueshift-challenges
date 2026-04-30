#![no_std]
use pinocchio::{
    entrypoint, error::ProgramError, nostd_panic_handler, AccountView, Address, ProgramResult,
};

entrypoint!(process_instruction);

nostd_panic_handler!();

pub mod instructions;
pub mod state;
pub use instructions::*;

pub const ID: Address = Address::from_str_const("22222222222222222222222222222222222222222222");

fn process_instruction(
    _program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((Make::DISCRIMINATOR, data)) => Make::try_from((accounts, data))?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
