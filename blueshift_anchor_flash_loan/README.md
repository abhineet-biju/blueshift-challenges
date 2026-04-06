# Anchor Flash Loan

A flash loan protocol built with Anchor 1.0.0-rc.5. Allows borrowing tokens within a single transaction, enforced atomically via instruction introspection.

## Instructions

### `borrow(amount: u64)`

Transfers tokens from the protocol's ATA to the borrower. Uses Solana's instructions sysvar to verify:
- The borrow instruction is first in the transaction (index 0)
- The last instruction in the transaction is a valid `repay` call to this program
- The repay instruction references the same borrower and protocol ATAs

### `repay()`

Reads the borrow amount from the first instruction's data, calculates a 5% fee (500 basis points), and transfers `amount + fee` back to the protocol.

## Architecture

- Protocol authority is a PDA derived from `["protocol"]`
- Instruction introspection via `solana-instructions-sysvar` ensures atomic borrow+repay
- Uses `token_interface` for SPL Token and Token-2022 compatibility
- 11 custom error codes for comprehensive validation

## See Also

- [Anchor 0.32.1 port](../blueshift_anchor_flash_loan_0321) with binary size and CU comparison
- [Quasar implementation](../blueshift_quasar_flash_loan) with full test coverage
