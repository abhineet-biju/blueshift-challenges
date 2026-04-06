# Quasar Flash Loan

A flash loan protocol built with the [Quasar framework](https://github.com/blueshift-gg/quasar) - Blueshift's lightweight alternative to Anchor for Solana program development.

## Instructions

### `borrow(amount: u64)`

Transfers tokens from the protocol's ATA to the borrower. Validates via instruction introspection that a matching `repay` instruction is the last instruction in the transaction.

### `repay()`

Reads the borrow amount from the first instruction, applies a 5% fee, and transfers `amount + fee` back to the protocol.

## Architecture

- Built with `quasar-lang` and `quasar-spl` (no_std)
- Custom instruction sysvar parser in `instruction_sysvar.rs` for raw sysvar data deserialization
- Custom discriminators matching the Anchor ABI for cross-framework compatibility
- Protocol PDA derived from `["protocol"]`

## Tests

Two comprehensive SVM-based tests using `quasar-svm`:

- `test_flash_loan_chain_round_trips_principal_plus_fee` - Full borrow+repay cycle validating fee calculation
- `test_borrow_requires_terminal_repay_instruction` - Ensures borrow fails without a trailing repay instruction
