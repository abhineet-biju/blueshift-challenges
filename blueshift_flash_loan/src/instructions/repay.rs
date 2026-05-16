use pinocchio::{error::ProgramError, AccountView, ProgramResult};

use crate::{get_token_amount, LoanData};

pub struct RepayAccounts<'a> {
    pub borrower: &'a mut AccountView,
    pub loan: &'a mut AccountView,
    pub token_accounts: &'a [AccountView],
}

impl<'a> TryFrom<&'a mut [AccountView]> for RepayAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [borrower, loan, token_accounts @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self {
            borrower,
            loan,
            token_accounts,
        })
    }
}

// No data required for this instruction

pub struct Repay<'a> {
    pub accounts: RepayAccounts<'a>,
}

impl<'a> TryFrom<&'a mut [AccountView]> for Repay<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let accounts = RepayAccounts::try_from(accounts)?;
        Ok(Self { accounts })
    }
}

impl<'a> Repay<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        let loan_data = self.accounts.loan.try_borrow()?;
        let loan_num = loan_data.len() / size_of::<LoanData>();
        let loan_entries =
            unsafe { core::slice::from_raw_parts(loan_data.as_ptr() as *const LoanData, loan_num) };

        if loan_num.ne(&self.accounts.token_accounts.len()) {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Process each pair of token accounts
        // (protocol, borrower) with corresponding amounts
        for (i, token_account) in self.accounts.token_accounts.iter().enumerate() {
            // validate that the protocol ATA is the same as what's stored in loan
            if token_account.address().to_bytes() != loan_entries[i].protocol_token_account {
                return Err(ProgramError::InvalidAccountData);
            }

            // check if loan is repaid
            let balance = get_token_amount(token_account)?;

            if balance < loan_entries[i].balance {
                return Err(ProgramError::InvalidAccountData);
            }
        }

        // close loan account and retrieve lamports
        drop(loan_data); // freeing up references

        let refund_amt = self.accounts.loan.lamports();

        self.accounts.borrower.set_lamports(
            self.accounts
                .borrower
                .lamports()
                .checked_add(refund_amt)
                .ok_or(ProgramError::ArithmeticOverflow)?,
        );

        self.accounts.loan.set_lamports(0);
        self.accounts.loan.close()?;

        Ok(())
    }
}
