use core::mem::MaybeUninit;

use pinocchio::{error::ProgramError, AccountView, ProgramResult};
use solana_winternitz::signature::WinternitzSignature;

pub struct SplitVaultAccounts<'a> {
    pub vault: &'a mut AccountView,
    pub split: &'a mut AccountView,
    pub refund: &'a mut AccountView,
}

impl<'a> TryFrom<&'a mut [AccountView]> for SplitVaultAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [vault, split, refund] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self {
            vault,
            split,
            refund,
        })
    }
}

pub struct SplitVaultInstructionData {
    pub signature: WinternitzSignature,
    pub amount: [u8; 8],
    pub bump: [u8; 1],
}

impl<'a> TryFrom<&'a [u8]> for SplitVaultInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != core::mem::size_of::<SplitVaultInstructionData>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut signature_array = MaybeUninit::<[u8; 896]>::uninit();
        unsafe {
            core::ptr::copy_nonoverlapping(
                data[0..896].as_ptr(),
                signature_array.as_mut_ptr() as *mut u8,
                896,
            );
        } // this method is preffered over safe versions to prevent zero-init costs

        Ok(Self {
            signature: WinternitzSignature::from(unsafe { signature_array.assume_init() }),
            amount: data[896..904]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
            bump: data[904..905]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        })
    }
}

pub struct SplitVault<'a> {
    pub accounts: SplitVaultAccounts<'a>,
    pub instruction_data: SplitVaultInstructionData,
}

impl<'a> TryFrom<(&'a mut [AccountView], &'a [u8])> for SplitVault<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a mut [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let accounts = SplitVaultAccounts::try_from(accounts)?;
        let instruction_data = SplitVaultInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a> SplitVault<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        // Assemble Split message
        let mut message = [0u8; 72]; // amount + split pub key + refund pub key
        message[0..8].clone_from_slice(&self.instruction_data.amount);
        message[8..40].clone_from_slice(self.accounts.split.address().as_ref());
        message[40..72].clone_from_slice(self.accounts.refund.address().as_ref());

        // Recover pubkey hash from signature
        let hash = self
            .instruction_data
            .signature
            .recover_pubkey(&message)
            .merklize();

        // PDA equivalence check
        if solana_nostd_sha256::hashv(&[
            hash.as_ref(),
            self.instruction_data.bump.as_ref(),
            crate::ID.as_ref(),
            b"ProgramDerivedAddress",
        ])
        .ne(self.accounts.vault.address().as_ref())
        {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Close vault
        // Add amount to the split acc
        self.accounts.split.set_lamports(
            self.accounts
                .split
                .lamports()
                .checked_add(u64::from_le_bytes(self.instruction_data.amount))
                .ok_or(ProgramError::ArithmeticOverflow)?,
        );

        // Refund remaining amount to refund account
        self.accounts.refund.set_lamports(
            self.accounts
                .refund
                .lamports()
                .checked_add(
                    self.accounts
                        .vault
                        .lamports()
                        .checked_sub(u64::from_le_bytes(self.instruction_data.amount))
                        .ok_or(ProgramError::ArithmeticOverflow)?,
                )
                .ok_or(ProgramError::ArithmeticOverflow)?,
        );

        self.accounts.vault.close();

        Ok(())
    }
}
