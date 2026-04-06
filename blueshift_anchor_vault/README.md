# Anchor Vault

A simple SOL vault program built with Anchor. Users can deposit SOL into a PDA-based vault and withdraw it later.

## Instructions

### `deposit(amount: u64)`

Transfers SOL from the user to a vault PDA derived from `["vault", user_pubkey]`. Validates that the vault is empty (first deposit only) and that the amount exceeds rent-exemption minimum.

### `withdraw()`

Withdraws the entire vault balance back to the user. Uses PDA signer seeds for authorization via CPI.

## Architecture

- Single `VaultAction` account context shared by both instructions
- Vault is a `SystemAccount` PDA - no custom state, just holds lamports
- Custom `VaultError` enum for validation errors
