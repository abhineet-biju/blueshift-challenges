#![no_std]
use pinocchio::{
    AccountView, Address, ProgramResult, entrypoint, error::ProgramError, nostd_panic_handler,
};

entrypoint!(process_instruction);
nostd_panic_handler!();
pub mod instructions;
pub mod state;
pub use instructions::*;
pub use state::*;

pub const ID: Address = Address::from_str_const("22222222222222222222222222222222222222222222");

fn process_instruction(
    _program_id: &AccountView,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {}
}
