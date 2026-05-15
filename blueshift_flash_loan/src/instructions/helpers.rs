use pinocchio::{error::ProgramError, AccountView};

#[repr(C, packed)]
pub struct LoanData {
    pub protocol_token_account: [u8; 32],
    pub balance: u64,
}

pub fn get_token_amount(account: &AccountView) -> Result<u64, ProgramError> {
    if !account.owned_by(&pinocchio_token::ID) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if account.data_len().ne(&pinocchio_token::state::Account::LEN) {
        return Err(ProgramError::InvalidAccountData);
    }

    let data = account.try_borrow()?;

    if data.len() != pinocchio_token::state::Account::LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(u64::from_le_bytes(data[64..72].try_into().unwrap()))
}
