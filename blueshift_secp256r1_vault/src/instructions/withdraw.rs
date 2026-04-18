use pinocchio::{
    account::Ref,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{
        clock::Clock,
        instructions::{Instructions, IntrospectedInstruction},
        Sysvar,
    },
    AccountView, ProgramResult,
};
use pinocchio_secp256r1_instruction::{Secp256r1Instruction, Secp256r1Pubkey};
use pinocchio_system::instructions::Transfer;

pub struct WithdrawAccounts<'a> {
    pub payer: &'a AccountView,
    pub vault: &'a AccountView,
    pub instructions_sysvar: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for WithdrawAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [payer, vault, instructions_sysvar, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        //Basic checks
        if !payer.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if !vault.owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if vault.lamports().eq(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            payer,
            vault,
            instructions_sysvar,
        })
    }
}

pub struct WithdrawInstructionData {
    pub bump: [u8; 1],
}

impl<'a> TryFrom<&'a [u8]> for WithdrawInstructionData {
    type Error = ProgramError;
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self {
            bump: [*data.first().ok_or(ProgramError::InvalidInstructionData)?],
        })
    }
}

pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>,
    pub instruction_data: WithdrawInstructionData,
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for Withdraw<'a> {
    type Error = ProgramError;
    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let accounts = WithdrawAccounts::try_from(accounts)?;
        let instruction_data = WithdrawInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a> Withdraw<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;
    pub fn process(&mut self) -> ProgramResult {
        //sysvar introspection struct
        let instructions: Instructions<Ref<[u8]>> =
            Instructions::try_from(self.accounts.instructions_sysvar)?;

        let ix: IntrospectedInstruction = instructions.get_instruction_relative(1)?;

        let secp256r1_ix = Secp256r1Instruction::try_from(&ix)?;

        if secp256r1_ix.num_signatures() != 1 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let signer_key: Secp256r1Pubkey = *secp256r1_ix.get_signer(0)?;

        let (payer, expiry) = secp256r1_ix
            .get_message_data(0)?
            .split_at_checked(32)
            .ok_or(ProgramError::InvalidInstructionData)?;

        if self.accounts.payer.address().as_ref().ne(payer) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        //Get current timestamp
        let now = Clock::get()?.unix_timestamp;

        //Get signature expiry timestamp
        let expiry = i64::from_le_bytes(
            expiry
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        if now > expiry {
            return Err(ProgramError::InvalidInstructionData);
        }

        //Invoke CPI
        let seeds = [
            Seed::from(b"vault"),
            Seed::from(signer_key[..1].as_ref()),
            Seed::from(signer_key[1..].as_ref()),
            Seed::from(&self.instruction_data.bump),
        ];

        let signers = [Signer::from(&seeds)];
        Transfer {
            from: self.accounts.vault,
            to: self.accounts.payer,
            lamports: self.accounts.vault.lamports(),
        }
        .invoke_signed(&signers)
    }
}
