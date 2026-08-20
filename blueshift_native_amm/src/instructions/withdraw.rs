use constant_product_curve::ConstantProduct;
use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock},
};
use pinocchio_token::{
    TokenProgram,
    instructions::{Batch, Burn, IntoBatch, Transfer, batch::BatchState},
    state::{Account as TokenAccount, Mint},
};

use crate::{Config, ID};

pub struct WithdrawAccounts<'a> {
    pub user: &'a AccountView,
    pub mint_lp: &'a mut AccountView,
    pub vault_x: &'a mut AccountView,
    pub vault_y: &'a mut AccountView,
    pub user_x_ata: &'a mut AccountView,
    pub user_y_ata: &'a mut AccountView,
    pub user_lp_ata: &'a mut AccountView,
    pub config: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a mut [AccountView]> for WithdrawAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [
            user,
            mint_lp,
            vault_x,
            vault_y,
            user_x_ata,
            user_y_ata,
            user_lp_ata,
            config,
            token_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if token_program.address() != &pinocchio_token::ID {
            return Err(ProgramError::InvalidAccountData);
        }

        if !config.owned_by(&ID) {
            return Err(ProgramError::InvalidAccountData);
        }
        if vault_x.address() == vault_y.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self {
            user,
            mint_lp,
            vault_x,
            vault_y,
            user_x_ata,
            user_y_ata,
            user_lp_ata,
            config,
            token_program,
        })
    }
}

pub struct WithdrawInstructionData {
    pub amount: u64,
    pub min_x: u64,
    pub min_y: u64,
    pub expiration: i64,
}

impl TryFrom<&[u8]> for WithdrawInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        const DATA_LEN: usize = size_of::<WithdrawInstructionData>();

        if data.len() != DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount = u64::from_le_bytes(
            data[0..8]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        let min_x = u64::from_le_bytes(
            data[8..16]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        let min_y = u64::from_le_bytes(
            data[16..24]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        let expiration = i64::from_le_bytes(
            data[24..32]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        //Basic arithmetic checks
        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        //Expiration check
        let clock = Clock::get()?;
        if clock.unix_timestamp > expiration {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            amount,
            min_x,
            min_y,
            expiration,
        })
    }
}

pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>,
    pub instruction_data: WithdrawInstructionData,
}

impl<'a> TryFrom<(&'a mut [AccountView], &[u8])> for Withdraw<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a mut [AccountView], &[u8])) -> Result<Self, Self::Error> {
        let accounts = WithdrawAccounts::try_from(accounts)?;
        let instruction_data = WithdrawInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a> Withdraw<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

    pub fn process(&self) -> ProgramResult {
        let mint_lp = unsafe { Mint::from_account_view_unchecked(self.accounts.mint_lp)? };
        let vault_x = unsafe { TokenAccount::from_account_view_unchecked(self.accounts.vault_x)? };
        let vault_y = unsafe { TokenAccount::from_account_view_unchecked(self.accounts.vault_y)? };

        //Load config to recreate PDA signer seeds
        let config = Config::load(self.accounts.config)?;

        //Check if provided mint_lp is valid
        let (expected_mint_lp, _) = Address::derive_program_address(
            &[b"mint_lp", self.accounts.config.address().as_array()],
            &crate::ID,
        )
        .ok_or(ProgramError::InvalidInstructionData)?;

        if &expected_mint_lp != self.accounts.mint_lp.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let (x, y) = match mint_lp.supply() == self.instruction_data.amount {
            true => (vault_x.amount(), vault_y.amount()),
            false => {
                let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
                    vault_x.amount(),
                    vault_y.amount(),
                    mint_lp.supply(),
                    self.instruction_data.amount,
                    6,
                )
                .map_err(|_| ProgramError::InvalidArgument)?;

                (amounts.x, amounts.y)
            }
        };

        // Check for slippage
        if !(x >= self.instruction_data.min_x && y >= self.instruction_data.min_y) {
            return Err(ProgramError::InvalidArgument);
        }

        const MAX_ACCOUNTS_LEN: usize = Burn::MAX_ACCOUNTS_LEN + Transfer::MAX_ACCOUNTS_LEN * 2;
        const MAX_DATA_LEN: usize =
            Batch::header_data_len(3) + Burn::DATA_LEN + Transfer::DATA_LEN * 2;

        let mut batch_state = BatchState::<TokenProgram>::new(MAX_ACCOUNTS_LEN, MAX_DATA_LEN);
        let mut batch = batch_state.as_batch()?;

        //Since user owns the LP token account, they authorize the token burn
        Burn::new(
            self.accounts.user_lp_ata,
            self.accounts.mint_lp,
            self.accounts.user,
            self.instruction_data.amount,
        )
        .into_batch(&mut batch)?;

        //The config PDA owns both vaults, so it authorizes the token transfer
        Transfer::new(
            self.accounts.vault_x,
            self.accounts.user_x_ata,
            self.accounts.config,
            x,
        )
        .into_batch(&mut batch)?;

        Transfer::new(
            self.accounts.vault_y,
            self.accounts.user_y_ata,
            self.accounts.config,
            y,
        )
        .into_batch(&mut batch)?;

        //Reconstruct config PDA seeds for signer
        let seed_bytes = config.seed().to_le_bytes();
        let bump_bytes = config.config_bump();

        let config_seeds = [
            Seed::from(b"config"),
            Seed::from(&seed_bytes),
            Seed::from(config.mint_x().as_array()),
            Seed::from(config.mint_y().as_array()),
            Seed::from(&bump_bytes),
        ];

        let config_signer = Signer::from(&config_seeds);

        // The user's original transaction signature remains available.
        // invoke_signed additionally grants signer privilege to the config PDA.
        batch.invoke_signed(&[config_signer])?;

        Ok(())
    }
}
