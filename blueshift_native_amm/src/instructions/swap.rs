use constant_product_curve::{ConstantProduct, LiquidityPair};
use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock},
};
use pinocchio_token::{
    TokenProgram,
    instructions::{Batch, IntoBatch, Transfer, batch::BatchState},
    state::Account as TokenAccount,
};

use crate::{AmmState, Config, ID};

pub struct SwapAccounts<'a> {
    pub user: &'a AccountView,
    pub user_x_ata: &'a mut AccountView,
    pub user_y_ata: &'a mut AccountView,
    pub vault_x: &'a mut AccountView,
    pub vault_y: &'a mut AccountView,
    pub config: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a mut [AccountView]> for SwapAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [
            user,
            user_x_ata,
            user_y_ata,
            vault_x,
            vault_y,
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
            user_x_ata,
            user_y_ata,
            vault_x,
            vault_y,
            config,
            token_program,
        })
    }
}

pub struct SwapInstructionData {
    pub is_x: u8,
    pub amount: u64,
    pub min_receive: u64,
    pub expiration: i64,
}

impl TryFrom<&[u8]> for SwapInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        const DATA_LEN: usize = size_of::<SwapInstructionData>();

        if data.len() != DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        let is_x = u8::from_le_bytes(
            data[0..1]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let amount = u64::from_le_bytes(
            data[1..9]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let min_receive = u64::from_le_bytes(
            data[9..17]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let expiration = i64::from_le_bytes(
            data[17..25]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        if !matches!(is_x, 0 | 1) || amount == 0 || min_receive == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            is_x,
            amount,
            min_receive,
            expiration,
        })
    }
}

pub struct Swap<'a> {
    pub accounts: SwapAccounts<'a>,
    pub instruction_data: SwapInstructionData,
}

impl<'a> TryFrom<(&'a mut [AccountView], &[u8])> for Swap<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a mut [AccountView], &[u8])) -> Result<Self, Self::Error> {
        let accounts = SwapAccounts::try_from(accounts)?;
        let instruction_data = SwapInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a> Swap<'a> {
    pub const DISCRIMINATOR: &'a u8 = &3;

    pub fn process(&self) -> ProgramResult {
        //Check expiration
        let clock = Clock::get()?;

        if clock.unix_timestamp > self.instruction_data.expiration {
            return Err(ProgramError::InvalidInstructionData);
        }

        let config = Config::load(self.accounts.config)?;

        if config.state() != AmmState::Initialized as u8 {
            return Err(ProgramError::InvalidInstructionData);
        }

        //Check whether token accounts are valid
        let (expected_vault_x, _) = Address::derive_program_address(
            &[
                self.accounts.config.address().as_array(),
                self.accounts.token_program.address().as_array(),
                config.mint_x().as_array(),
            ],
            &pinocchio_associated_token_account::ID,
        )
        .ok_or(ProgramError::InvalidInstructionData)?;

        let (expected_vault_y, _) = Address::derive_program_address(
            &[
                self.accounts.config.address().as_array(),
                self.accounts.token_program.address().as_array(),
                config.mint_y().as_array(),
            ],
            &pinocchio_associated_token_account::ID,
        )
        .ok_or(ProgramError::InvalidAccountData)?;

        if !(self.accounts.vault_x.address() == &expected_vault_x
            && self.accounts.vault_y.address() == &expected_vault_y)
        {
            return Err(ProgramError::InvalidAccountData);
        }

        // Deserialize the token accounts
        let vault_x = TokenAccount::from_account_view(self.accounts.vault_x)?;
        let vault_y = TokenAccount::from_account_view(self.accounts.vault_y)?;

        // Swap Calculations
        let mut curve = ConstantProduct::init(
            vault_x.amount(),
            vault_y.amount(),
            vault_x.amount(),
            config.fee(),
            None,
        )
        .map_err(|_| ProgramError::Custom(1))?;

        let p = match self.instruction_data.is_x {
            1 => LiquidityPair::X,
            0 => LiquidityPair::Y,
            _ => return Err(ProgramError::InvalidInstructionData),
        };

        let swap_result = curve
            .swap(
                p,
                self.instruction_data.amount,
                self.instruction_data.min_receive,
            )
            .map_err(|_| ProgramError::Custom(1))?;

        // Check for correct values
        if swap_result.deposit == 0 || swap_result.withdraw == 0 {
            return Err(ProgramError::InvalidArgument);
        }

        let seed_bytes = config.seed().to_le_bytes();
        let config_bump = config.config_bump();

        //Recreate config seeds
        let config_seeds = [
            Seed::from(b"config"),
            Seed::from(&seed_bytes),
            Seed::from(config.mint_x().as_array()),
            Seed::from(config.mint_y().as_array()),
            Seed::from(&config_bump),
        ];

        let config_signer = Signer::from(&config_seeds);

        //Perform batched CPI
        const MAX_ACCOUNTS_LEN: usize = Transfer::MAX_ACCOUNTS_LEN * 2;
        const DATA_LEN: usize = Batch::header_data_len(2) + Transfer::DATA_LEN * 2;

        let mut batch_state = BatchState::<TokenProgram>::new(MAX_ACCOUNTS_LEN, DATA_LEN);
        let mut batch = batch_state.as_batch()?;

        if self.instruction_data.is_x == 1 {
            Transfer::new(
                self.accounts.user_x_ata,
                self.accounts.vault_x,
                self.accounts.user,
                swap_result.deposit,
            )
            .into_batch(&mut batch)?;

            Transfer::new(
                self.accounts.vault_y,
                self.accounts.user_y_ata,
                self.accounts.config,
                swap_result.withdraw,
            )
            .into_batch(&mut batch)?;
        }

        if self.instruction_data.is_x == 0 {
            Transfer::new(
                self.accounts.user_y_ata,
                self.accounts.vault_y,
                self.accounts.user,
                swap_result.deposit,
            )
            .into_batch(&mut batch)?;

            Transfer::new(
                self.accounts.vault_x,
                self.accounts.user_x_ata,
                self.accounts.config,
                swap_result.withdraw,
            )
            .into_batch(&mut batch)?;
        }

        batch.invoke_signed(&[config_signer])?;

        Ok(())
    }
}
