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

pub struct RefundAccounts<'a> {
    pub maker: &'a mut AccountView,
    pub escrow: &'a mut AccountView,
    pub mint_a: &'a AccountView,
    pub vault: &'a AccountView,
    pub maker_ata_a: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a mut [AccountView]> for RefundAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [maker, escrow, mint_a, vault, maker_ata_a, system_program, token_program, _] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        //Basic checks
        if !maker.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        //Sanity Checks
        let mint_a_data = Mint::from_account_view(mint_a)?;

        if !mint_a_data.is_initialized() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            maker,
            escrow,
            mint_a,
            vault,
            maker_ata_a,
            system_program,
            token_program,
        })
    }
}

pub struct Refund<'a> {
    accounts: RefundAccounts<'a>,
}

impl<'a> TryFrom<&'a mut [AccountView]> for Refund<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let accounts = RefundAccounts::try_from(accounts)?;

        //Check if escrow data is valid
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

        drop(escrow_data);

        Ok(Self { accounts })
    }
}

impl<'a> Refund<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

    pub fn process(&mut self) -> ProgramResult {
        CreateIdempotentAta {
            account: self.accounts.maker_ata_a,
            funding_account: self.accounts.maker,
            mint: self.accounts.mint_a,
            wallet: self.accounts.maker,
            system_program: self.accounts.system_program,
            token_program: self.accounts.token_program,
        }
        .invoke()?;

        {
            let maker_ata_a_data = TokenAccount::from_account_view(self.accounts.maker_ata_a)?;

            if !maker_ata_a_data.is_initialized() {
                return Err(ProgramError::InvalidAccountData);
            }

            if maker_ata_a_data.owner() != self.accounts.maker.address() {
                return Err(ProgramError::InvalidAccountData);
            }

            if maker_ata_a_data.mint() != self.accounts.mint_a.address() {
                return Err(ProgramError::InvalidAccountData);
            }
        }

        let escrow_data = self.accounts.escrow.try_borrow()?;
        let escrow = Escrow::load(escrow_data.as_ref())?;

        let seed_binding = escrow.seed.to_le_bytes();
        let bump_binding = escrow.bump;

        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(self.accounts.maker.address().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump_binding),
        ];

        let signer = Signer::from(&escrow_seeds);
        let amount = TokenAccount::from_account_view(self.accounts.vault)?.amount();

        drop(escrow_data); //drop the reference so that escrow can be used during CPI

        //Transfer from vault to maker
        TokenTransfer::new(
            self.accounts.vault,
            self.accounts.maker_ata_a,
            self.accounts.escrow,
            amount,
        )
        .invoke_signed(&[signer.clone()])?;

        //Close empty vault
        CloseAccount::new(
            self.accounts.vault,
            self.accounts.maker,
            self.accounts.escrow,
        )
        .invoke_signed(&[signer.clone()])?;

        //Close escrow PDA account
        let escrow_lamports = self.accounts.escrow.lamports();

        self.accounts.maker.set_lamports(
            self.accounts
                .maker
                .lamports()
                .checked_add(escrow_lamports)
                .ok_or(ProgramError::ArithmeticOverflow)?,
        );

        self.accounts.escrow.set_lamports(0);
        self.accounts.escrow.close()?;

        Ok(())
    }
}
