# Anchor Escrow

An SPL token escrow program built with Anchor. Enables trustless peer-to-peer token swaps between two parties.

## Instructions

### `make(id: u64, amount: u64, receive: u64)`

The maker creates an escrow by depositing `amount` of token A into a vault PDA, specifying they want `receive` amount of token B in return. The escrow config is stored as a PDA seeded by `["escrow", maker_pubkey, id]`.

### `take()`

The taker fulfills the escrow by sending the requested token B amount to the maker, then withdrawing the deposited token A from the vault. The vault is closed and rent is returned.

### `refund()`

The maker can cancel the escrow and reclaim their deposited tokens. The vault is closed and rent is returned.

## Architecture

- `EscrowConfig` account stores: id, maker, mint_a, mint_b, receive_amount, and PDA bump
- Vault is an associated token account owned by the escrow PDA
- Uses `token_interface` for SPL Token and Token-2022 compatibility
- Custom discriminators on all instructions (`[0]`, `[1]`, `[2]`)
