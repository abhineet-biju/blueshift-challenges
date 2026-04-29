use core::mem::size_of;
use pinocchio::{error::ProgramError, Address};

#[repr(C)]
pub struct Escrow {
    pub seed: u64,
    pub maker: Address,
    pub mint_a: Address,
    pub mint_b: Address,
    pub receive: Address,
    pub bump: [u8; 1],
}

impl Escrow {
    pub const LEN: usize = size_of<u64>() 
        + size_of<Address>() 
        + size_of<Address>() 
        + size_of<Address>() 
        + size_of<u64>()
        + size_of<[u8;1]()

    #[inline(always)]
    pub fn load_mut(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Escrow::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
    }

}
