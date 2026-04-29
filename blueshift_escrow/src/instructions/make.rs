use pinocchio::{error::ProgramError, AccountView, Address};
use pinocchio_token::state::{Account as TokenAccount, Mint};

pub struct MakeAccounts<'a> {
    pub maker: &'a AccountView,
    pub escrow: &'a AccountView,
    pub mint_a: &'a AccountView,
    pub mint_b: &'a AccountView,
    pub maker_ata_a: &'a AccountView,
    pub vault: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for MakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
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
        if amount 
    }
}
