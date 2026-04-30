use crate::state::Escrow;
use core::mem::size_of;
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};
use pinocchio_associated_token_account::instructions::Create as CreateAssociatedTokenAccount;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{
    instructions::Transfer as TokenTransfer,
    state::{Account as TokenAccount, Mint},
};

pub struct MakeAccounts<'a> {
    pub maker: &'a AccountView,
    pub escrow: &'a mut AccountView,
    pub mint_a: &'a AccountView,
    pub mint_b: &'a AccountView,
    pub maker_ata_a: &'a AccountView,
    pub vault: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a mut [AccountView]> for MakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [maker, escrow, mint_a, mint_b, maker_ata_a, vault, system_program, token_program, _] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        //Basic checks
        if !maker.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mint_a_data = Mint::from_account_view(mint_a)?;
        let mint_b_data = Mint::from_account_view(mint_b)?;

        if !mint_a_data.is_initialized() || !mint_b_data.is_initialized() {
            return Err(ProgramError::InvalidAccountData);
        }

        let maker_ata_a_data = TokenAccount::from_account_view(maker_ata_a)?;

        if !maker_ata_a_data.is_initialized() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if maker_ata_a_data.mint() != mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        if maker_ata_a_data.owner() != maker.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            maker,
            escrow,
            mint_a,
            mint_b,
            maker_ata_a,
            vault,
            system_program,
            token_program,
        })
    }
}

pub struct MakeInstructionData {
    pub seed: u64,
    pub receive: u64,
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for MakeInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u64>() * 3 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let seed = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let receive = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let amount = u64::from_le_bytes(data[16..24].try_into().unwrap());

        //Basic checks
        if amount == 0 || receive == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            seed,
            receive,
            amount,
        })
    }
}

pub struct Make<'a> {
    pub accounts: MakeAccounts<'a>,
    pub instruction_data: MakeInstructionData,
    pub bump: u8,
}

impl<'a> TryFrom<(&'a mut [AccountView], &'a [u8])> for Make<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a mut [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let accounts = MakeAccounts::try_from(accounts)?;
        let instruction_data = MakeInstructionData::try_from(data)?;

        let (_, bump) = Address::derive_program_address(
            &[
                b"escrow",
                accounts.maker.address().as_ref(),
                instruction_data.seed.to_le_bytes().as_ref(),
            ],
            &crate::ID,
        )
        .ok_or(ProgramError::InvalidSeeds)?;

        let seed_binding = instruction_data.seed.to_le_bytes();
        let bump_binding = [bump];

        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(accounts.maker.address().as_ref()),
            Seed::from(seed_binding.as_ref()),
            Seed::from(bump_binding.as_ref()),
        ];

        let signer = Signer::from(&escrow_seeds);

        CreateAccount::with_minimum_balance(
            accounts.maker,
            accounts.escrow,
            Escrow::LEN as u64,
            &crate::ID,
            None,
        )?
        .invoke_signed(&[signer])?;

        CreateAssociatedTokenAccount {
            funding_account: accounts.maker,
            account: accounts.vault,
            wallet: accounts.escrow,
            mint: accounts.mint_a,
            system_program: accounts.system_program,
            token_program: accounts.token_program,
        }
        .invoke()?;

        Ok(Self {
            accounts,
            instruction_data,
            bump,
        })
    }
}

impl<'a> Make<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&mut self) -> ProgramResult {
        let mut escrow_data = self.accounts.escrow.try_borrow_mut()?;
        let escrow = Escrow::load_mut(escrow_data.as_mut())?;

        escrow.set_inner(
            self.instruction_data.seed,
            self.accounts.maker.address(),
            self.accounts.mint_a.address(),
            self.accounts.mint_b.address(),
            self.instruction_data.receive,
            [self.bump],
        );

        //Transfer tokens to vault
        TokenTransfer::new(
            self.accounts.maker_ata_a,
            self.accounts.vault,
            self.accounts.maker,
            self.instruction_data.amount,
        )
        .invoke()?;

        Ok(())
    }
}
