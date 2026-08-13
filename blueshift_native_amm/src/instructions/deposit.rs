use constant_product_curve::ConstantProduct;
use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock},
};
use pinocchio_token::{
    instructions::{Batch, BatchState, IntoBatch, MintTo, Transfer},
    state::{Account as TokenAccount, Mint},
};

use crate::{AmmState, Config};

pub struct DepositAccounts<'a> {
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

impl<'a> TryFrom<&'a mut [AccountView]> for DepositAccounts<'a> {
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

pub struct DepositInstructionData {
    pub amount: u64,
    pub max_x: u64,
    pub max_y: u64,
    pub expiration: i64,
}

impl<'a> TryFrom<&'a [u8]> for DepositInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        const DATA_LEN: usize = size_of::<DepositInstructionData>();

        if data.len() != DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount = u64::from_le_bytes(
            data[..8]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let max_x = u64::from_le_bytes(
            data[8..16]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let max_y = u64::from_le_bytes(
            data[16..24]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let expiration = i64::from_le_bytes(
            data[24..32]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        if amount <= 0 || max_x <= 0 || max_y <= 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            amount,
            max_x,
            max_y,
            expiration,
        })
    }
}

pub struct Deposit<'a> {
    accounts: DepositAccounts<'a>,
    instruction_data: DepositInstructionData,
}

impl<'a> TryFrom<(&'a mut [AccountView], &'a [u8])> for Deposit<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a mut [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(accounts)?;
        let instruction_data = DepositInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        let clock = Clock::get()?;

        //Validate user as signer
        if !self.accounts.user.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        //Check expiration
        if clock.unix_timestamp > self.instruction_data.expiration {
            return Err(ProgramError::InvalidInstructionData);
        }

        let config = unsafe { Config::load_unchecked(self.accounts.config)? };

        //Validate vault accounts
        let (vault_x, _) = Address::derive_program_address(
            &[
                self.accounts.config.address().as_array(),
                self.accounts.token_program.address().as_array(),
                config.mint_x().as_array(),
            ],
            &pinocchio_associated_token_account::ID,
        )
        .ok_or(ProgramError::InvalidInstructionData)?;

        if &vault_x != self.accounts.vault_x.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let (vault_y, _) = Address::derive_program_address(
            &[
                self.accounts.config.address().as_array(),
                self.accounts.token_program.address().as_array(),
                config.mint_y().as_array(),
            ],
            &pinocchio_associated_token_account::ID,
        )
        .ok_or(ProgramError::InvalidInstructionData)?;

        if &vault_y != self.accounts.vault_y.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        //Validate config
        let expected_config = Address::derive_address_const(
            &[
                b"config",
                &config.seed().to_le_bytes(),
                config.mint_x().as_ref(),
                config.mint_y().as_ref(),
            ],
            Some(config.config_bump()[0]),
            &crate::ID,
        );

        if &expected_config != self.accounts.config.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        //Checking whether AMM is initialized
        if config.state() != AmmState::Initialized as u8 {
            return Err(ProgramError::InvalidAccountData);
        }

        //Validate mint_lp
        let (mint_lp, _) = Address::derive_program_address(
            &[b"mint_lp", self.accounts.config.address().as_array()],
            &crate::ID,
        )
        .ok_or(ProgramError::InvalidInstructionData)?;

        if &mint_lp != self.accounts.mint_lp.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        //Deserialize the token accounts
        let mint_lp = unsafe { Mint::from_account_view_unchecked(self.accounts.mint_lp)? };
        let vault_x = unsafe { TokenAccount::from_account_view_unchecked(self.accounts.vault_x)? };
        let vault_y = unsafe { TokenAccount::from_account_view_unchecked(self.accounts.vault_y)? };

        //Grab amounts to deposit
        let (x, y) = match mint_lp.supply() == 0 && vault_x.amount() == 0 && vault_y.amount() == 0 {
            true => (self.instruction_data.max_x, self.instruction_data.max_y),
            false => {
                let amounts = ConstantProduct::xy_deposit_amounts_from_l(
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

        if x == 0 || y == 0 {
            return Err(ProgramError::InvalidArgument);
        }

        //Check for slippage
        if !(x <= self.instruction_data.max_x && y <= self.instruction_data.max_y) {
            return Err(ProgramError::InvalidArgument);
        }

        //Total number of accounts for the batch instruction
        const MAX_ACCOUNTS_LEN: usize = Transfer::MAX_ACCOUNTS_LEN * 2 + MintTo::MAX_ACCOUNTS_LEN;
        //Total length of instruction data for the batch instruction
        const MAX_DATA_LEN: usize =
            Batch::header_data_len(3) + Transfer::DATA_LEN * 2 + MintTo::DATA_LEN;

        let mut batch_state = BatchState::new(MAX_ACCOUNTS_LEN, MAX_DATA_LEN);
        let mut batch = batch_state.as_batch()?;

        Transfer::new(
            self.accounts.user_x_ata,
            self.accounts.vault_x,
            self.accounts.user,
            x,
        )
        .into_batch(&mut batch)?;
        Transfer::new(
            self.accounts.user_y_ata,
            self.accounts.vault_y,
            self.accounts.user,
            y,
        )
        .into_batch(&mut batch)?;
        MintTo::new(
            self.accounts.mint_lp,
            self.accounts.user_lp_ata,
            self.accounts.config,
            self.instruction_data.amount,
        )
        .into_batch(&mut batch)?;

        let seed_binding = config.seed().to_le_bytes();
        let bump_binding = config.config_bump();
        let config_seeds = [
            Seed::from(b"config"),
            Seed::from(&seed_binding),
            Seed::from(config.mint_x().as_array()),
            Seed::from(config.mint_y().as_array()),
            Seed::from(&bump_binding),
        ];
        let signer = Signer::from(&config_seeds);

        batch.invoke_signed(&[signer])?;

        Ok(())
    }
}
