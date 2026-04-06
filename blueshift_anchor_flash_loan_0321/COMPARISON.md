# Anchor Flash Loan Comparison

This folder is a one-to-one port of `../blueshift_anchor_flash_loan` to Anchor `0.32.1`.

## What stayed the same

- The flash-loan flow is unchanged: `borrow` must be the first instruction, `repay` must be the last instruction, and repayment includes a 5% fee.
- The account model is unchanged: borrower signer, protocol PDA, borrower ATA, protocol ATA, instructions sysvar, token interface, ATA program, and system program.
- The error enum and the borrow/repay business logic are unchanged aside from compatibility fixes required by Anchor `0.32.1`.

## What changed

- The workspace folder and internal program name were suffixed with `_0321` so this version can live beside the original without naming collisions.
- Rust dependencies were downgraded from Anchor `1.0.0-rc.5` / `anchor-spl 1.0.0-rc.5` to `anchor-lang 0.32.1` / `anchor-spl 0.32.1`.
- The TypeScript package was changed from `@anchor-lang/core` to `@coral-xyz/anchor`, which is the matching client package for Anchor `0.32.1`.
- `solana-instructions-sysvar` was pinned to `2.2.2` instead of `3.0.0` so its Solana crate versions match Anchor `0.32.1`.
- CPI context creation now passes `token_program.to_account_info()` instead of `token_program.key()`, which matches the older `CpiContext::new` and `CpiContext::new_with_signer` signatures in Anchor `0.32.1`.
- The instruction introspection call now passes the instructions sysvar `AccountInfo` directly to `load_current_index_checked`.
- This folder includes a `.anchorversion` file with `0.32.1` to make the intended Anchor CLI version explicit for AVM-based workflows.

## Practical takeaway

The program logic ports cleanly from Anchor `1.0.0-rc.5` to `0.32.1`, but the surrounding API surface is not identical. The main differences are:

- package names on the TypeScript side,
- Solana crate-version compatibility around the instructions sysvar, and
- CPI helper signatures expecting `AccountInfo` rather than a program pubkey.

## Validation

- `cargo test` passes in this `0.32.1` workspace.
- `anchor build` and `anchor test` both pass in this workspace after switching AVM to `0.32.1`.
- For comparison, the original `1.0.0-rc.5` workspace was rebuilt with `anchor build --ignore-keys` because its keypair and `declare_id!` value currently differ.

## Measured output

- Binary size:
  - `blueshift_anchor_flash_loan.so` (`1.0.0-rc.5`) = `170,560` bytes
  - `blueshift_anchor_flash_loan_0321.so` (`0.32.1`) = `248,240` bytes
- Quasar static CU profile:
  - `blueshift_anchor_flash_loan.so` (`1.0.0-rc.5`) = `16,311 CU`
  - `blueshift_anchor_flash_loan_0321.so` (`0.32.1`) = `24,323 CU`

## Interpretation

- The `0.32.1` binary is about `77,680` bytes larger, roughly a `45.5%` increase in ELF size.
- The `0.32.1` Quasar estimate is `8,012 CU` higher, roughly a `49.1%` increase versus the `1.0.0-rc.5` build.
- Quasar's `profile` command is a static analysis of the sBPF binary's call graph, so these numbers are best treated as a relative compiler/runtime footprint comparison, not a final on-chain transaction simulation result.
