use crate::Repay;
use pinocchio::cpi::{Seed, Signer};
use pinocchio::sysvars::instructions::{Instructions, INSTRUCTIONS_ID};
use pinocchio::{error::ProgramError, AccountView, ProgramResult};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer as TokenTransfer;

use crate::{get_token_amount, LoanData, ID};

pub struct LoanAccounts<'a> {
    pub borrower: &'a AccountView,
    pub protocol: &'a AccountView,
    pub loan: &'a mut AccountView,
    pub instruction_sysvar: &'a AccountView,
    pub token_accounts: &'a [AccountView],
}

impl<'a> TryFrom<&'a mut [AccountView]> for LoanAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [borrower, protocol, loan, instruction_sysvar, _token_program, _system_program, token_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Basic checks
        if instruction_sysvar.address() != &INSTRUCTIONS_ID {
            return Err(ProgramError::UnsupportedSysvar);
        }

        if token_accounts.len() % 2 != 0 || token_accounts.len() == 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        if loan.try_borrow()?.len().ne(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            borrower,
            protocol,
            loan,
            instruction_sysvar,
            token_accounts,
        })
    }
}

pub struct LoanInstructionData<'a> {
    pub bump: [u8; 1],
    pub fee: u16,
    pub amounts: &'a [u64],
}

impl<'a> TryFrom<&'a [u8]> for LoanInstructionData<'a> {
    type Error = ProgramError;
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        // Get bump
        let (bump, data) = data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        // Get fee
        let (fee, data) = data
            .split_at_checked(size_of::<u16>())
            .ok_or(ProgramError::InvalidInstructionData)?;

        // Verify remaining data is valid
        if data.len() % size_of::<u64>() != 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Get amounts
        let amounts: &[u64] = unsafe {
            core::slice::from_raw_parts(data.as_ptr() as *const u64, data.len() / size_of::<u64>())
        };

        Ok(Self {
            bump: [*bump],
            fee: u16::from_le_bytes(
                fee.try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            ),
            amounts,
        })
    }
}

pub struct Loan<'a> {
    pub accounts: LoanAccounts<'a>,
    pub instruction_data: LoanInstructionData<'a>,
}

impl<'a> TryFrom<(&'a mut [AccountView], &'a [u8])> for Loan<'a> {
    type Error = ProgramError;
    fn try_from((accounts, data): (&'a mut [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let accounts = LoanAccounts::try_from(accounts)?;
        let instruction_data = LoanInstructionData::try_from(data)?;

        if instruction_data.amounts.len() != accounts.token_accounts.len() / 2 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a> Loan<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&mut self) -> ProgramResult {
        let fee = self.instruction_data.fee.to_le_bytes();

        let signer_seeds = [
            Seed::from(b"protocol"),
            Seed::from(&fee),
            Seed::from(&self.instruction_data.bump),
        ];

        let signer = Signer::from(&signer_seeds);

        // Open the LoanData account and create
        // a mutable slice to push the Loan struct to it
        let size = size_of::<LoanData>() * self.instruction_data.amounts.len();

        CreateAccount::with_minimum_balance(
            self.accounts.borrower,
            self.accounts.loan,
            size as u64,
            &ID,
            None,
        )?
        .invoke()?;

        let mut loan_data = self.accounts.loan.try_borrow_mut()?;
        let loan_entries = unsafe {
            core::slice::from_raw_parts_mut(
                loan_data.as_mut_ptr() as *mut LoanData,
                self.instruction_data.amounts.len(),
            )
        };

        // Populate loan_entries
        for (i, amount) in self.instruction_data.amounts.iter().enumerate() {
            let protocol_token_account = &self.accounts.token_accounts[i * 2];
            let borrower_token_account = &self.accounts.token_accounts[i * 2 + 1];

            // Get balance of protocol account and populate LoanData
            let balance = get_token_amount(&protocol_token_account)?;
            let balance_with_fee = balance
                .checked_add(
                    amount
                        .checked_mul(self.instruction_data.fee as u64)
                        .and_then(|x| x.checked_div(10_000))
                        .ok_or(ProgramError::InvalidInstructionData)?,
                )
                .ok_or(ProgramError::InvalidInstructionData)?;

            // Push loan data to the loan account
            loan_entries[i] = LoanData {
                protocol_token_account: protocol_token_account.address().to_bytes(),
                balance: balance_with_fee,
            };

            // Transfer tokens from protocol to borrower
            TokenTransfer::new(
                protocol_token_account,
                borrower_token_account,
                self.accounts.protocol,
                *amount,
            )
            .invoke_signed(&[signer.clone()])?;
        }

        drop(loan_data); //drop to clear mutable reference

        // Introspecting Repay instruction
        let instruction_sysvar =
            unsafe { Instructions::new_unchecked(self.accounts.instruction_sysvar.try_borrow()?) };

        let num_instructions = instruction_sysvar.num_instructions();

        let last_instruction =
            instruction_sysvar.load_instruction_at(num_instructions as usize - 1)?;

        if last_instruction.get_program_id() != &ID {
            return Err(ProgramError::InvalidInstructionData);
        }

        if last_instruction
            .get_instruction_data()
            .first()
            .ok_or(ProgramError::InvalidInstructionData)?
            != Repay::DISCRIMINATOR
        {
            return Err(ProgramError::InvalidInstructionData);
        }

        if &last_instruction.get_instruction_account_at(1)?.key != self.accounts.loan.address() {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(())
    }
}
