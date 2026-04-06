# Anchor Flash Loan (0.32.1 Port)

A one-to-one port of the [Anchor Flash Loan](../blueshift_anchor_flash_loan) from Anchor 1.0.0-rc.5 to Anchor 0.32.1. The business logic is identical; only framework-level API differences were adapted.

## Key Differences from 1.0.0-rc.5

- `solana-instructions-sysvar` pinned to `2.2.2` (vs `3.0.0`)
- CPI context uses `token_program.to_account_info()` instead of `token_program.key()`
- Instruction sysvar `AccountInfo` passed directly to `load_current_index_checked`

## Measured Impact

| Metric | 1.0.0-rc.5 | 0.32.1 | Delta |
|--------|-----------|--------|-------|
| Binary size | 170,560 B | 248,240 B | +45.5% |
| CU estimate | 16,311 | 24,323 | +49.1% |

See [COMPARISON.md](./COMPARISON.md) for the full analysis.
