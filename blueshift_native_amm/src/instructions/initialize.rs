use core::mem::MaybeUninit;

use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{instructions::InitializeMint2, state::Mint};

use crate::{Config, ID};
pub struct InitializeAccounts<'a> {
    pub initializer: &'a mut AccountView,
    pub mint_lp: &'a mut AccountView,
    pub config: &'a mut AccountView,
}

impl<'a> TryFrom<&'a mut [AccountView]> for InitializeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(acccounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [initializer, mint_lp, config, ..] = acccounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self {
            initializer,
            mint_lp,
            config,
        })
    }
}

#[repr(C, packed)]
pub struct InitializeInstructionData {
    pub seed: u64,
    pub fee: u16,
    pub mint_x: [u8; 32],
    pub mint_y: [u8; 32],
    pub config_bump: [u8; 1],
    pub lp_bump: [u8; 1],
    pub authority: [u8; 32],
}

impl<'a> TryFrom<&'a [u8]> for InitializeInstructionData {
    type Error = ProgramError;
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        const INITIALIZE_DATA_LEN_WITH_AUTHORITY: usize = size_of::<InitializeInstructionData>();
        const INITIALIZE_DATA_LEN: usize =
            INITIALIZE_DATA_LEN_WITH_AUTHORITY - size_of::<[u8; 32]>();
        match data.len() {
            INITIALIZE_DATA_LEN_WITH_AUTHORITY => {
                Ok(unsafe { (data.as_ptr() as *const Self).read_unaligned() })
            }
            INITIALIZE_DATA_LEN => {
                //If authority's not present, then we initialize it with 0s
                let mut raw: MaybeUninit<[u8; INITIALIZE_DATA_LEN_WITH_AUTHORITY]> =
                    MaybeUninit::uninit();
                let raw_ptr = raw.as_mut_ptr() as *mut u8;

                unsafe {
                    core::ptr::copy_nonoverlapping(data.as_ptr(), raw_ptr, INITIALIZE_DATA_LEN);

                    core::ptr::write_bytes(raw_ptr.add(INITIALIZE_DATA_LEN), 0, 32);

                    Ok((raw.as_ptr() as *mut Self).read_unaligned())
                }
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

pub struct Initialize<'a> {
    pub accounts: InitializeAccounts<'a>,
    pub instruction_data: InitializeInstructionData,
}

impl<'a> TryFrom<(&'a mut [AccountView], &'a [u8])> for Initialize<'a> {
    type Error = ProgramError;
    fn try_from((accounts, data): (&'a mut [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let accounts = InitializeAccounts::try_from(accounts)?;
        let instruction_data = InitializeInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a> Initialize<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&mut self) -> ProgramResult {
        const LEGACY_RENT_EXEMPTION_MULTIPLIER: u64 = 2;

        //Initializing seeds for signing
        let seed_binding = self.instruction_data.seed.to_le_bytes();
        let config_seeds = [
            Seed::from(b"config"),
            Seed::from(&seed_binding),
            Seed::from(&self.instruction_data.mint_x),
            Seed::from(&self.instruction_data.mint_y),
            Seed::from(&self.instruction_data.config_bump),
        ];

        let mint_lp_seeds = [
            Seed::from(b"mint_lp"),
            Seed::from(self.accounts.config.address().as_ref()),
            Seed::from(&self.instruction_data.lp_bump),
        ];

        let config_signer = Signer::from(&config_seeds);
        let mint_lp_signer = Signer::from(&mint_lp_seeds);

        let mut create_config = CreateAccount::with_minimum_balance(
            self.accounts.initializer,
            self.accounts.config,
            Config::LEN as u64,
            &ID,
            None,
        )?;

        create_config.lamports = create_config
            .lamports
            .checked_mul(LEGACY_RENT_EXEMPTION_MULTIPLIER)
            .ok_or(ProgramError::InvalidArgument)?;

        create_config.invoke_signed(&[config_signer])?;

        let mut create_mint_lp = CreateAccount::with_minimum_balance(
            self.accounts.initializer,
            self.accounts.mint_lp,
            Mint::LEN as u64,
            &pinocchio_token::ID,
            None,
        )?;

        create_mint_lp.lamports = create_mint_lp
            .lamports
            .checked_mul(LEGACY_RENT_EXEMPTION_MULTIPLIER)
            .ok_or(ProgramError::InvalidArgument)?;

        create_mint_lp.invoke_signed(&[mint_lp_signer])?;

        InitializeMint2::new(
            self.accounts.mint_lp,
            6,
            self.accounts.config.address(),
            None,
        )
        .invoke()?;

        //Populate config
        let config_data = Config::load_mut_unchecked(self.accounts.config)?;
        config_data.set_inner(
            self.instruction_data.seed,
            Address::from(self.instruction_data.authority),
            Address::from(self.instruction_data.mint_x),
            Address::from(self.instruction_data.mint_y),
            self.instruction_data.fee,
            self.instruction_data.config_bump,
        )?;

        Ok(())
    }
}
