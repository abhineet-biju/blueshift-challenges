# Blueshift Challenge Solutions

Completed challenge solutions from [blueshift.gg](https://blueshift.gg) - a Solana smart contract development platform.

## Challenges

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

### Assembly (sbpf)

| Challenge | Framework | Description |
|-----------|-----------|-------------|
| [Assembly Memo](./blueshift_assembly_memo) | Assembly (sbpf) | Solana assembly challenge program for memo behavior |
| [Assembly Slippage](./blueshift_assembly_slippage) | Assembly (sbpf) | Solana assembly challenge program for slippage behavior |
| [Assembly Timeout](./blueshift_assembly_timeout) | Assembly (sbpf) | Solana assembly challenge program for timeout behavior |

## Program IDs

All programs use `declare_id!("22222222222222222222222222222222222222222222")` as required by the Blueshift challenge platform. The platform substitutes the actual program ID during build and verification.

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

## Tech Stack

- **Solana** - L1 blockchain
- **Anchor** - Solana smart contract framework (versions 0.32.1 and 1.0.0-rc.5)
- **Quasar** - Lightweight Solana program framework by Blueshift
- **Pinocchio** - no_std, zero-copy Solana program framework
- **sbpf** - Blueshift tooling for Solana assembly challenge programs
- **Mollusk SVM** - Program test harness used for assembly challenge validation
- **Rust** - Program language
- **TypeScript** - Test language (Anchor projects)
