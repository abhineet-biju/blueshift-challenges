use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

pub struct OpenVaultAccounts<'a> {
    pub payer: &'a AccountView,
    pub vault: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for OpenVaultAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [payer, vault, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self { payer, vault })
    }
}

pub struct OpenVaultInstructionData {
    hash: [u8; 32],
    bump: [u8; 1],
}

impl<'a> TryFrom<&'a [u8]> for OpenVaultInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != core::mem::size_of::<OpenVaultInstructionData>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let hash = data[0..32]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        let bump = data[32..33]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self { hash, bump })
    }
}

pub struct OpenVault<'a> {
    pub accounts: OpenVaultAccounts<'a>,
    pub instruction_data: OpenVaultInstructionData,
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for OpenVault<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let accounts = OpenVaultAccounts::try_from(accounts)?;
        let instruction_data = OpenVaultInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a> OpenVault<'a> {
    pub const DISCTIMINATOR: &'a u8 = &0;

    pub fn process(&self) -> ProgramResult {
        let lamports = Rent::get()?.try_minimum_balance(0)?;
        let seeds = [
            Seed::from(&self.instruction_data.hash),
            Seed::from(&self.instruction_data.bump),
        ];
        let signer = [Signer::from(&seeds)];

        CreateAccount {
            from: &self.accounts.payer,
            to: &self.accounts.vault,
            lamports,
            space: 0,
            owner: &crate::ID,
        }
        .invoke_signed(&signer)?;

        Ok(())
    }
}
