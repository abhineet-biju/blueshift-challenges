# pinocchio-secp256r1-instruction: Migration to Pinocchio 0.11

## Problem

`pinocchio-secp256r1-instruction` v0.1.2 depends on `pinocchio ^0.8.0`. Pinocchio has since released v0.11, which introduced several breaking API changes. Because Rust treats types from different crate versions as distinct, projects using pinocchio 0.11 cannot use the secp256r1 crate — the `ProgramError` types are incompatible, causing `?` operator failures across crate boundaries.

### Concrete error example

```rust
// In a pinocchio 0.11 program:
let signer = secp256r1_ix.get_signer(0)?;
// ERROR: '?' couldn't convert the error to 'ProgramError'
// The secp256r1 crate returns pinocchio 0.8's ProgramError,
// but the program expects pinocchio 0.11's ProgramError.
```

## What changed between Pinocchio 0.8 and 0.11

### 1. `ProgramError` moved modules

```rust
// 0.8
use pinocchio::program_error::ProgramError;

// 0.11
use pinocchio::error::ProgramError;
```

### 2. `Pubkey` replaced by `Address`

`Pubkey` was a type alias for `[u8; 32]`. `Address` is a newtype struct wrapping `[u8; 32]`.

```rust
// 0.8
pub type Pubkey = [u8; 32];

// 0.11
pub struct Address(pub(crate) [u8; 32]);
```

This means you can no longer directly assign a byte array literal to an `Address`:

```rust
// 0.8 — works because Pubkey is just [u8; 32]
pub const SECP256R1_PROGRAM_ID: Pubkey = [0x06, 0x92, ...];

// 0.11 — must use the constructor
pub const SECP256R1_PROGRAM_ID: Address = Address::new_from_array([0x06, 0x92, ...]);
```

### 3. `AccountInfo` replaced by `AccountView`

```rust
// 0.8
use pinocchio::account_info::AccountInfo;

// 0.11
use pinocchio::AccountView;
```

### 4. `IntrospectedInstruction::get_program_id()` return type

```rust
// 0.8
pub fn get_program_id(&self) -> &Pubkey   // returns &[u8; 32]

// 0.11
pub fn get_program_id(&self) -> &Address  // returns &Address (newtype)
```

## Changes required in the secp256r1 crate

The crate is a single file (`src/lib.rs`, ~410 lines, half tests). The internal parsing logic is all raw byte/pointer operations and does not need changes. The migration is purely at the type and import boundaries.

### 1. `Cargo.toml` — bump dependency

```toml
# Before
[dependencies.pinocchio]
version = "^0.8.0"

# After
[dependencies.pinocchio]
version = "^0.11.0"
```

### 2. Update imports

```rust
// Before
use pinocchio::{
    program_error::ProgramError, pubkey::Pubkey, sysvars::instructions::IntrospectedInstruction,
};

// After
use pinocchio::{
    error::ProgramError, address::Address, sysvars::instructions::IntrospectedInstruction,
};
```

### 3. Update `SECP256R1_PROGRAM_ID` constant

```rust
// Before
pub const SECP256R1_PROGRAM_ID: Pubkey = [
    0x06, 0x92, 0x0d, 0xec, 0x2f, 0xea, 0x71, 0xb5,
    0xb7, 0x23, 0x81, 0x4d, 0x74, 0x2d, 0xa9, 0x03,
    0x1c, 0x83, 0xe7, 0x5f, 0xdb, 0x79, 0x5d, 0x56,
    0x8e, 0x75, 0x47, 0x80, 0x20, 0x00, 0x00, 0x00,
];

// After
pub const SECP256R1_PROGRAM_ID: Address = Address::new_from_array([
    0x06, 0x92, 0x0d, 0xec, 0x2f, 0xea, 0x71, 0xb5,
    0xb7, 0x23, 0x81, 0x4d, 0x74, 0x2d, 0xa9, 0x03,
    0x1c, 0x83, 0xe7, 0x5f, 0xdb, 0x79, 0x5d, 0x56,
    0x8e, 0x75, 0x47, 0x80, 0x20, 0x00, 0x00, 0x00,
]);
```

### 4. Update `TryFrom<&IntrospectedInstruction>` program ID comparison

```rust
// Before — Pubkey is [u8; 32], get_program_id() returns &Pubkey
if SECP256R1_PROGRAM_ID.ne(ix.get_program_id()) {
    return Err(ProgramError::IncorrectProgramId);
}

// After — both sides are Address, comparison should work via PartialEq.
// If ne() doesn't resolve, use:
if &SECP256R1_PROGRAM_ID != ix.get_program_id() {
    return Err(ProgramError::IncorrectProgramId);
}
```

### 5. Tests

The existing tests only exercise `TryFrom<&[u8]>`, `get_signer`, `get_signature`, and `get_message_data` — all of which operate on raw `&[u8]` internally. They should not require changes beyond the updated import paths (if they reference `ProgramError` or `Pubkey` directly).

## Summary

| Area | Before (0.8) | After (0.11) |
|---|---|---|
| Error type path | `pinocchio::program_error::ProgramError` | `pinocchio::error::ProgramError` |
| Key type | `pubkey::Pubkey` (`[u8; 32]`) | `address::Address` (newtype struct) |
| Account type | `account_info::AccountInfo` | `AccountView` |
| Program ID constant | Raw byte array literal | `Address::new_from_array([...])` |
| `get_program_id()` return | `&Pubkey` / `&[u8; 32]` | `&Address` |

The actual byte-level parsing logic (signature offsets, pubkey extraction, message data slicing) is untouched — only the pinocchio API surface types change.
