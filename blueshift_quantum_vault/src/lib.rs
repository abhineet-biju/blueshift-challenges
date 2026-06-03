#![no_std]
use pinocchio::{entrypoint, error::ProgramError, AccountView, Address, ProgramResult};

entrypoint!(process_instruction);

pub mod instructions;
pub use instructions::*;

pub const ID: Address = Address::from_str_const("22222222222222222222222222222222222222222222");

fn process_instruction(
    _program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((OpenVault::DISCTIMINATOR, data)) => OpenVault::try_from((accounts, data))?.process(),
        Some((SplitVault::DISCRIMINATOR, data)) => {
            SplitVault::try_from((accounts, data))?.process()
        }
        Some((CloseVault::DISCRIMINATOR, data)) => {
            CloseVault::try_from((accounts, data))?.process()
        }
        _ => return Err(ProgramError::InvalidInstructionData),
    }
}
