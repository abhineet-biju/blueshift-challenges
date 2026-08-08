use pinocchio::{
    AccountView, Address, ProgramResult, entrypoint, error::ProgramError, nostd_panic_handler,
};

entrypoint!(process_instruction);
pub mod instructions;
pub mod state;
pub use instructions::*;
pub use state::*;
