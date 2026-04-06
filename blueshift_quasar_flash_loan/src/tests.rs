extern crate std;

use std::vec;
use std::vec::Vec;

use quasar_svm::{
    token::{
        create_keyed_associated_token_account_with_program, create_keyed_mint_account,
        create_keyed_system_account, Mint,
    },
    Account, AccountMeta, Instruction, ProgramError, Pubkey, QuasarSvm,
};
use solana_instruction::{BorrowedAccountMeta, BorrowedInstruction};
use solana_instructions_sysvar::construct_instructions_data;

const INITIAL_PROTOCOL_LIQUIDITY: u64 = 5_000_000;
const INITIAL_BORROWER_BALANCE: u64 = 100_000;
const BORROW_AMOUNT: u64 = 1_000_000;
const EXPECTED_FEE: u64 = (BORROW_AMOUNT * 500) / 10_000;

fn instructions_sysvar_pubkey() -> Pubkey {
    Pubkey::new_from_array(solana_instructions_sysvar::ID.to_bytes())
}

fn setup() -> QuasarSvm {
    let elf = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/target/deploy/blueshift_quasar_flash_loan.so"
    ))
    .unwrap();

    QuasarSvm::new()
        .with_program(&Pubkey::from(crate::ID), &elf)
        .with_token_program()
        .with_associated_token_program()
}

fn token_amount(account: &Account) -> u64 {
    let amount_bytes: [u8; 8] = account.data[64..72].try_into().unwrap();
    u64::from_le_bytes(amount_bytes)
}

fn instructions_sysvar_account(instructions: &[Instruction]) -> Account {
    let borrowed = instructions
        .iter()
        .map(|ix| BorrowedInstruction {
            program_id: &ix.program_id,
            accounts: ix
                .accounts
                .iter()
                .map(|meta| BorrowedAccountMeta {
                    pubkey: &meta.pubkey,
                    is_signer: meta.is_signer,
                    is_writable: meta.is_writable,
                })
                .collect(),
            data: &ix.data,
        })
        .collect::<Vec<_>>();

    Account {
        address: instructions_sysvar_pubkey(),
        lamports: 0,
        data: construct_instructions_data(&borrowed),
        owner: quasar_svm::solana_sdk_ids::sysvar::ID,
        executable: false,
    }
}

fn borrow_ix(
    borrower: Pubkey,
    protocol: Pubkey,
    mint: Pubkey,
    borrower_ata: Pubkey,
    protocol_ata: Pubkey,
    instructions_sysvar: Pubkey,
    amount: u64,
) -> Instruction {
    let mut data = crate::BORROW_DISCRIMINATOR.to_vec();
    data.extend_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id: Pubkey::from(crate::ID),
        accounts: vec![
            AccountMeta::new(borrower, true),
            AccountMeta::new(protocol, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(borrower_ata, false),
            AccountMeta::new(protocol_ata, false),
            AccountMeta::new_readonly(instructions_sysvar, false),
            AccountMeta::new_readonly(quasar_svm::SPL_TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(quasar_svm::SPL_ASSOCIATED_TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(quasar_svm::system_program::ID, false),
        ],
        data,
    }
}

fn repay_ix(
    borrower: Pubkey,
    protocol: Pubkey,
    mint: Pubkey,
    borrower_ata: Pubkey,
    protocol_ata: Pubkey,
    instructions_sysvar: Pubkey,
) -> Instruction {
    Instruction {
        program_id: Pubkey::from(crate::ID),
        accounts: vec![
            AccountMeta::new(borrower, true),
            AccountMeta::new(protocol, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(borrower_ata, false),
            AccountMeta::new(protocol_ata, false),
            AccountMeta::new_readonly(instructions_sysvar, false),
            AccountMeta::new_readonly(quasar_svm::SPL_TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(quasar_svm::SPL_ASSOCIATED_TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(quasar_svm::system_program::ID, false),
        ],
        data: crate::REPAY_DISCRIMINATOR.to_vec(),
    }
}

fn protocol_fixture() -> (Pubkey, Account) {
    let program_id = Pubkey::from(crate::ID);
    let (protocol, _) = Pubkey::find_program_address(&[b"protocol"], &program_id);
    (protocol, create_keyed_system_account(&protocol, 1_000_000_000))
}

#[test]
fn test_flash_loan_chain_round_trips_principal_plus_fee() {
    let mut svm = setup();

    let borrower = Pubkey::new_unique();
    let borrower_account = create_keyed_system_account(&borrower, 10_000_000_000);

    let (protocol, protocol_account) = protocol_fixture();
    let mint = Pubkey::new_unique();
    let mint_account = create_keyed_mint_account(
        &mint,
        &Mint {
            mint_authority: Some(borrower).into(),
            supply: INITIAL_PROTOCOL_LIQUIDITY + INITIAL_BORROWER_BALANCE,
            decimals: 6,
            is_initialized: true,
            freeze_authority: None.into(),
        },
    );

    let borrower_ata_account = create_keyed_associated_token_account_with_program(
        &borrower,
        &mint,
        INITIAL_BORROWER_BALANCE,
        &quasar_svm::SPL_TOKEN_PROGRAM_ID,
    );
    let borrower_ata = borrower_ata_account.address;

    let protocol_ata_account = create_keyed_associated_token_account_with_program(
        &protocol,
        &mint,
        INITIAL_PROTOCOL_LIQUIDITY,
        &quasar_svm::SPL_TOKEN_PROGRAM_ID,
    );
    let protocol_ata = protocol_ata_account.address;

    let instructions_sysvar = instructions_sysvar_pubkey();
    let borrow = borrow_ix(
        borrower,
        protocol,
        mint,
        borrower_ata,
        protocol_ata,
        instructions_sysvar,
        BORROW_AMOUNT,
    );
    let repay = repay_ix(
        borrower,
        protocol,
        mint,
        borrower_ata,
        protocol_ata,
        instructions_sysvar,
    );
    let instructions_account = instructions_sysvar_account(&[borrow.clone(), repay.clone()]);

    let result = svm.process_instruction_chain(
        &[borrow, repay],
        &[
            borrower_account,
            protocol_account,
            mint_account,
            borrower_ata_account,
            protocol_ata_account,
            instructions_account,
        ],
    );

    result.assert_success();

    let borrower_ata_after = result.account(&borrower_ata).unwrap();
    let protocol_ata_after = result.account(&protocol_ata).unwrap();

    assert_eq!(
        token_amount(borrower_ata_after),
        INITIAL_BORROWER_BALANCE - EXPECTED_FEE
    );
    assert_eq!(
        token_amount(protocol_ata_after),
        INITIAL_PROTOCOL_LIQUIDITY + EXPECTED_FEE
    );
}

#[test]
fn test_borrow_requires_terminal_repay_instruction() {
    let mut svm = setup();

    let borrower = Pubkey::new_unique();
    let borrower_account = create_keyed_system_account(&borrower, 10_000_000_000);

    let (protocol, protocol_account) = protocol_fixture();
    let mint = Pubkey::new_unique();
    let mint_account = create_keyed_mint_account(
        &mint,
        &Mint {
            mint_authority: Some(borrower).into(),
            supply: INITIAL_PROTOCOL_LIQUIDITY + INITIAL_BORROWER_BALANCE,
            decimals: 6,
            is_initialized: true,
            freeze_authority: None.into(),
        },
    );

    let borrower_ata_account = create_keyed_associated_token_account_with_program(
        &borrower,
        &mint,
        INITIAL_BORROWER_BALANCE,
        &quasar_svm::SPL_TOKEN_PROGRAM_ID,
    );
    let borrower_ata = borrower_ata_account.address;

    let protocol_ata_account = create_keyed_associated_token_account_with_program(
        &protocol,
        &mint,
        INITIAL_PROTOCOL_LIQUIDITY,
        &quasar_svm::SPL_TOKEN_PROGRAM_ID,
    );
    let protocol_ata = protocol_ata_account.address;

    let instructions_sysvar = instructions_sysvar_pubkey();
    let borrow = borrow_ix(
        borrower,
        protocol,
        mint,
        borrower_ata,
        protocol_ata,
        instructions_sysvar,
        BORROW_AMOUNT,
    );
    let instructions_account = instructions_sysvar_account(std::slice::from_ref(&borrow));

    let result = svm.process_instruction(
        &borrow,
        &[
            borrower_account,
            protocol_account,
            mint_account,
            borrower_ata_account,
            protocol_ata_account,
            instructions_account,
        ],
    );

    result.assert_error(ProgramError::Custom(crate::error::ProtocolError::InvalidIx as u32));
}
