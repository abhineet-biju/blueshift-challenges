# Blueshift Challenge Solutions

Completed solutions for the Solana development challenges on [Blueshift](https://learn.blueshift.gg).

Status: all challenge implementations represented in this repository are complete. The repository also includes an Anchor 0.32.1 port of the flash loan challenge for comparison.

## Completed challenges

### Anchor

| Challenge | Framework | Description |
|-----------|-----------|-------------|
| [Anchor Vault](./blueshift_anchor_vault) | Anchor 0.32.1 | SOL vault with PDA-based deposits and withdrawals |
| [Anchor Escrow](./blueshift_anchor_escrow) | Anchor 0.32.1 | SPL token escrow with make, take, and refund flows |
| [Anchor Flash Loan](./blueshift_anchor_flash_loan) | Anchor 1.0.0-rc.5 | Flash loan protocol enforced with instruction introspection |
| [Anchor Flash Loan (0.32.1)](./blueshift_anchor_flash_loan_0321) | Anchor 0.32.1 | Port of the flash loan to Anchor 0.32.1 with [comparison analysis](./blueshift_anchor_flash_loan_0321/COMPARISON.md) |

### Quasar

| Challenge | Framework | Description |
|-----------|-----------|-------------|
| [Quasar Flash Loan](./blueshift_quasar_flash_loan) | Quasar | Flash loan protocol reimplemented in Quasar with full test coverage |

### Pinocchio

| Challenge | Framework | Description |
|-----------|-----------|-------------|
| [Pinocchio Vault](./blueshift_vault) | Pinocchio 0.10.2 | SOL vault built with the Pinocchio no_std framework |
| [Pinocchio Secp256r1 Vault](./blueshift_secp256r1_vault) | Pinocchio 0.11.1 | SOL vault gated by secp256r1 signature verification, with [migration notes](./blueshift_secp256r1_vault/secp256r1_crate_update.md) for the helper crate update |
| [Pinocchio Escrow](./blueshift_escrow) | Pinocchio 0.11.1 | SPL token escrow with make, take, and refund instructions |
| [Pinocchio Flash Loan](./blueshift_flash_loan) | Pinocchio 0.11.1 | Flash loan protocol with loan and repay instructions |
| [Pinocchio Quantum Vault](./blueshift_quantum_vault) | Pinocchio 0.11.1 | SOL vault with open, split, and close flows authorized by Winternitz signatures |
| [Pinocchio AMM](./blueshift_native_amm) | Pinocchio 0.11.2 | Constant-product AMM with initialize, deposit, withdraw, and swap instructions |

### Assembly (sbpf)

| Challenge | Framework | Description |
|-----------|-----------|-------------|
| [Assembly Memo](./blueshift_assembly_memo) | Assembly (sbpf) | Solana assembly challenge program for memo behavior |
| [Assembly Slippage](./blueshift_assembly_slippage) | Assembly (sbpf) | Solana assembly challenge program for slippage behavior |
| [Assembly Timeout](./blueshift_assembly_timeout) | Assembly (sbpf) | Solana assembly challenge program for timeout behavior |

## Program IDs

All challenge programs use `22222222222222222222222222222222222222222222` as required by the Blueshift platform. Depending on the framework, the address is declared with `declare_id!` or a program ID constant. The platform substitutes the deployed address during verification.

## Building

Each Anchor challenge can be built with:

```bash
anchor build
```

The Quasar challenge uses:

```bash
quasar build
```

Pinocchio challenges use:

```bash
cargo build-sbf
```

Assembly challenges include prebuilt deploy artifacts and can be validated with:

```bash
cargo test
```

## Tech stack

- **Solana:** execution environment for every challenge program
- **Anchor:** program framework, using versions 0.32.1 and 1.0.0-rc.5
- **Quasar:** lightweight framework used for the Quasar flash loan
- **Pinocchio:** low-level framework used for the native Rust programs
- **sbpf:** tooling used for the assembly challenges
- **Mollusk SVM:** test harness used for assembly validation
- **Rust:** program implementation language
- **TypeScript:** integration test language for the Anchor projects
