use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};
use pinocchio_associated_token_account::instructions::CreateIdempotent as CreateIdempotentAta;
use pinocchio_token::{
    instructions::{CloseAccount, Transfer as TokenTransfer},
    state::{Account as TokenAccount, Mint},
};

use crate::state::Escrow;

pub struct TakeAccounts<'a> {
    pub taker: &'a mut AccountView,
    pub maker: &'a AccountView,
    pub escrow: &'a mut AccountView,
    pub mint_a: &'a AccountView,
    pub mint_b: &'a AccountView,
    pub vault: &'a AccountView,
    pub taker_ata_a: &'a AccountView,
    pub taker_ata_b: &'a AccountView,
    pub maker_ata_b: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a mut [AccountView]> for TakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [taker, maker, escrow, mint_a, mint_b, vault, taker_ata_a, taker_ata_b, maker_ata_b, system_program, token_program, _] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        //Basic Checks
        if !taker.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mint_a_data = Mint::from_account_view(mint_a)?;
        let mint_b_data = Mint::from_account_view(mint_b)?;

        if !mint_a_data.is_initialized() || !mint_b_data.is_initialized() {
            return Err(ProgramError::InvalidAccountData);
        }

        let taker_ata_b_data = TokenAccount::from_account_view(taker_ata_b)?;

        if !taker_ata_b_data.is_initialized() {
            return Err(ProgramError::InvalidAccountData);
        }

        if taker_ata_b_data.mint() != mint_b.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        if taker_ata_b_data.owner() != taker.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            taker,
            maker,
            escrow,
            mint_a,
            mint_b,
            vault,
            taker_ata_a,
            taker_ata_b,
            maker_ata_b,
            system_program,
            token_program,
        })
    }
}

pub struct Take<'a> {
    accounts: TakeAccounts<'a>,
}

impl<'a> TryFrom<&'a mut [AccountView]> for Take<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let accounts = TakeAccounts::try_from(accounts)?;

        //Check if escrow account is valid
        {
            let escrow_data = accounts.escrow.try_borrow()?;
            let escrow = Escrow::load(escrow_data.as_ref())?;

            let escrow_address = Address::derive_address(
                &[
                    b"escrow",
                    accounts.maker.address().as_ref(),
                    escrow.seed.to_le_bytes().as_ref(),
                ],
                Some(escrow.bump[0]),
                &crate::ID,
            );

            if &escrow_address != accounts.escrow.address() {
                return Err(ProgramError::InvalidAccountOwner);
            }
        }

        Ok(Self { accounts })
    }
}

impl<'a> Take<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        //Initialize necessary accounts
        CreateIdempotentAta {
            account: self.accounts.maker_ata_b,
            funding_account: self.accounts.taker,
            mint: self.accounts.mint_b,
            wallet: self.accounts.maker,
            system_program: self.accounts.system_program,
            token_program: self.accounts.token_program,
        }
        .invoke()?;

        CreateIdempotentAta {
            account: self.accounts.taker_ata_a,
            funding_account: self.accounts.taker,
            mint: self.accounts.mint_a,
            wallet: self.accounts.taker,
            system_program: self.accounts.system_program,
            token_program: self.accounts.token_program,
        }
        .invoke()?;

        {
            let taker_ata_a_data = TokenAccount::from_account_view(self.accounts.taker_ata_a)?;
            let maker_ata_b_data = TokenAccount::from_account_view(self.accounts.maker_ata_b)?;

            if !taker_ata_a_data.is_initialized() || !maker_ata_b_data.is_initialized() {
                return Err(ProgramError::InvalidAccountData);
            }

            if taker_ata_a_data.mint() != self.accounts.mint_a.address() {
                return Err(ProgramError::InvalidAccountData);
            }

            if maker_ata_b_data.mint() != self.accounts.mint_b.address() {
                return Err(ProgramError::InvalidAccountData);
            }

            if taker_ata_a_data.owner() != self.accounts.taker.address() {
                return Err(ProgramError::InvalidAccountData);
            }

            if maker_ata_b_data.owner() != self.accounts.maker.address() {
                return Err(ProgramError::InvalidAccountData);
            }
        }

        let escrow_data = self.accounts.escrow.try_borrow_mut()?;
        let escrow = Escrow::load(escrow_data.as_ref())?;

        let seed_binding = escrow.seed.to_le_bytes();
        let bump_binding = escrow.bump;
        let receive_binding = escrow.receive;

        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(self.accounts.maker.address().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump_binding),
        ];

        let signer = Signer::from(&escrow_seeds);
        let amount = TokenAccount::from_account_view(self.accounts.vault)?.amount();

        drop(escrow_data); //drop the mutable reference so that escrow can be used during CPI

        //Transfer from vault to taker
        TokenTransfer::new(
            self.accounts.vault,
            self.accounts.taker_ata_a,
            self.accounts.escrow,
            amount,
        )
        .invoke_signed(&[signer.clone()])?;

        //Transfer from taker to maker
        TokenTransfer::new(
            self.accounts.taker_ata_b,
            self.accounts.maker_ata_b,
            self.accounts.taker,
            receive_binding,
        )
        .invoke()?;

        //Close empty vault
        CloseAccount::new(
            self.accounts.vault,
            self.accounts.maker,
            self.accounts.escrow,
        )
        .invoke_signed(&[signer.clone()])?;

        //Close escrow account
        let escrow_lamports = self.accounts.escrow.lamports();

        self.accounts.taker.set_lamports(
            self.accounts
                .taker
                .lamports()
                .checked_add(escrow_lamports)
                .ok_or(ProgramError::ArithmeticOverflow)?,
        );

        self.accounts.escrow.set_lamports(0);
        self.accounts.escrow.close()?;

        Ok(())
    }
}
