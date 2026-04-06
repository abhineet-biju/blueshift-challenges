# Blueshift Challenge Solutions

Completed challenge solutions from [blueshift.gg](https://blueshift.gg) - a Solana smart contract development platform.

## Challenges

| Challenge | Framework | Description |
|-----------|-----------|-------------|
| [Anchor Vault](./blueshift_anchor_vault) | Anchor 0.32.1 | SOL vault with PDA-based deposit and withdraw |
| [Anchor Escrow](./blueshift_anchor_escrow) | Anchor 0.32.1 | SPL token escrow with make, take, and refund |
| [Anchor Flash Loan](./blueshift_anchor_flash_loan) | Anchor 1.0.0-rc.5 | Flash loan protocol with instruction introspection |
| [Anchor Flash Loan (0.32.1)](./blueshift_anchor_flash_loan_0321) | Anchor 0.32.1 | Port of the flash loan to Anchor 0.32.1 with [comparison analysis](./blueshift_anchor_flash_loan_0321/COMPARISON.md) |
| [Quasar Flash Loan](./blueshift_quasar_flash_loan) | Quasar | Flash loan reimplemented using the Quasar framework with full test coverage |

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

## Tech Stack

- **Solana** - L1 blockchain
- **Anchor** - Solana smart contract framework (versions 0.32.1 and 1.0.0-rc.5)
- **Quasar** - Lightweight Solana program framework by Blueshift
- **Rust** - Program language
- **TypeScript** - Test language (Anchor projects)
