use pinocchio::{
    AccountView, Address,
    account::{Ref, RefMut},
    error::ProgramError,
};

#[repr(C, packed)]
pub struct Config {
    state: u8,
    seed: [u8; 8],
    authority: Address,
    mint_x: Address,
    mint_y: Address,
    fee: [u8; 2],
    config_bump: [u8; 1],
}

#[repr(u8)]
pub enum AmmState {
    Uninitialized = 0u8,
    Initialized = 1u8,
    Disabled = 2u8,
    WithdrawOnly = 3u8,
}
impl Config {
    pub const LEN: usize = size_of::<Self>();

    // Pointer casting functions
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { &*(bytes.as_ptr() as *const Config) }
    }
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut Config) }
    }
}

//Reading helpers
impl Config {
    #[inline(always)]
    pub fn load(account: &AccountView) -> Result<Ref<'_, Self>, ProgramError> {
        if account.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if account.owner() != &crate::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Ref::map(account.try_borrow()?, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }
    #[inline(always)]
    pub unsafe fn load_unchecked(account: &AccountView) -> Result<&Self, ProgramError> {
        if account.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if account.owner() != &crate::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        unsafe { Ok(Self::from_bytes_unchecked(account.borrow_unchecked())) }
    }

    // Getter methods for safe field access
    #[inline(always)]
    pub fn state(&self) -> u8 {
        self.state
    }

    #[inline(always)]
    pub fn seed(&self) -> u64 {
        u64::from_le_bytes(self.seed)
    }

    #[inline(always)]
    pub fn authority(&self) -> &Address {
        &self.authority
    }

    #[inline(always)]
    pub fn mint_x(&self) -> &Address {
        &self.mint_x
    }

    #[inline(always)]
    pub fn mint_y(&self) -> &Address {
        &self.mint_y
    }

    #[inline(always)]
    pub fn fee(&self) -> u16 {
        u16::from_le_bytes(self.fee)
    }

    #[inline(always)]
    pub fn config_bump(&self) -> [u8; 1] {
        self.config_bump
    }
}

//Writing helpers
impl Config {
    #[inline(always)]
    pub fn load_mut(account: &mut AccountView) -> Result<RefMut<'_, Self>, ProgramError> {
        if account.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if account.owner() != &crate::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(RefMut::map(account.try_borrow_mut()?, |data| unsafe {
            Self::from_bytes_unchecked_mut(data)
        }))
    }

    pub fn load_mut_unchecked(account: &mut AccountView) -> Result<&mut Self, ProgramError> {
        if account.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if account.owner() != &crate::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        unsafe {
            Ok(Self::from_bytes_unchecked_mut(
                account.borrow_unchecked_mut(),
            ))
        }
    }
    #[inline(always)]
    pub fn set_state(&mut self, state: u8) -> Result<(), ProgramError> {
        if state.ge(&(AmmState::WithdrawOnly as u8)) {
            return Err(ProgramError::InvalidAccountData);
        }
        self.state = state as u8;
        Ok(())
    }

    #[inline(always)]
    pub fn set_fee(&mut self, fee: u16) -> Result<(), ProgramError> {
        if fee.ge(&10_000) {
            return Err(ProgramError::InvalidAccountData);
        }
        self.fee = fee.to_le_bytes();
        Ok(())
    }

    pub fn set_authority(&mut self, authority: Address) -> Result<(), ProgramError> {
        self.authority = authority;
        Ok(())
    }

    pub fn set_seed(&mut self, seed: u64) -> Result<(), ProgramError> {
        self.seed = seed.to_le_bytes();
        Ok(())
    }

    pub fn set_mint_x(&mut self, mint_x: Address) -> Result<(), ProgramError> {
        self.mint_x = mint_x;
        Ok(())
    }

    pub fn set_mint_y(&mut self, mint_y: Address) -> Result<(), ProgramError> {
        self.mint_y = mint_y;
        Ok(())
    }

    pub fn set_config_bump(&mut self, config_bump: [u8; 1]) -> Result<(), ProgramError> {
        self.config_bump = config_bump;
        Ok(())
    }

    #[inline(always)]
    pub fn set_inner(
        &mut self,
        seed: u64,
        authority: Address,
        mint_x: Address,
        mint_y: Address,
        fee: u16,
        config_bump: [u8; 1],
    ) -> Result<(), ProgramError> {
        self.set_state(AmmState::Initialized as u8)?;
        self.set_seed(seed)?;
        self.set_authority(authority)?;
        self.set_mint_x(mint_x)?;
        self.set_mint_y(mint_y)?;
        self.set_fee(fee)?;
        self.set_config_bump(config_bump)?;
        Ok(())
    }

    #[inline(always)]
    pub fn has_authority(&self) -> Option<Address> {
        let bytes = self.authority().as_array();
        let chunks: &[u64; 4] = unsafe { &*(bytes.as_ptr() as *const [u64; 4]) };
        if chunks.iter().any(|&x| x != 0) {
            Some(self.authority)
        } else {
            None
        }
    }
}
