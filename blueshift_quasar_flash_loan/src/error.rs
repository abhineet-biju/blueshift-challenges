use quasar_lang::prelude::*;

#[error_code]
pub enum ProtocolError {
    InvalidIx = 6000,
    InvalidInstructionIndex,
    InvalidAmount,
    NotEnoughFunds,
    ProgramMismatch,
    InvalidProgram,
    InvalidBorrowerAta,
    InvalidProtocolAta,
    MissingRepayIx,
    MissingBorrowIx,
    Overflow,
}
