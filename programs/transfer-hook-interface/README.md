<p align="center">
  <code>pinocchio-transfer-hook-interface</code>
</p>

## Overview

This crate contains [`pinocchio`](https://crates.io/crates/pinocchio)-compatible types and helpers for the [SPL Transfer Hook Interface](https://spl.solana.com/transfer-hook-interface).

It provides everything needed to **build** a transfer hook program and to **invoke** one via CPI, without depending on `solana-program` or the SPL crate itself:

- **Discriminator constants** for `Execute`, `InitializeExtraAccountMetaList`, and `UpdateExtraAccountMetaList` instructions (SHA-256 based, matching SPL).
- **`ExtraAccountMeta`** — a 35-byte `#[repr(C)]` type binary-compatible with `spl-tlv-account-resolution::ExtraAccountMeta`.
- **`Seed`** — typed seed components (`Literal`, `InstructionData`, `AccountKey`, `AccountData`) with pack into the 32-byte `address_config` field.
- **`ExtraAccountMetaList`** — helpers to initialize and read the TLV-encoded extra account metas PDA data.
- **`Execute`** — a CPI instruction builder for invoking a hook program's `Execute` handler.
- **PDA derivation** — `get_extra_account_metas_address()` and seed collection helpers.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Examples

Building a transfer hook program — validating the incoming `Execute` instruction:

```rust
use pinocchio_transfer_hook_interface::EXECUTE_DISCRIMINATOR;

// In your program's entrypoint:
let (discriminator, rest) = instruction_data.split_at(8);
if discriminator == EXECUTE_DISCRIMINATOR {
    let amount = u64::from_le_bytes(rest[..8].try_into().unwrap());
    // ... your transfer hook logic
}
```

Initializing extra account metas for your hook:

```rust
use pinocchio_transfer_hook_interface::{
    EXECUTE_DISCRIMINATOR,
    state::{ExtraAccountMeta, ExtraAccountMetaList, Seed},
};

let metas = [
    ExtraAccountMeta::new_with_pubkey(&my_config_pubkey, false, false),
    ExtraAccountMeta::new_with_seeds(
        &[Seed::AccountKey { index: 0 }, Seed::AccountKey { index: 2 }],
        false,
        true,
    )?,
];
let size = ExtraAccountMetaList::size_of(metas.len());
// Write into the PDA's account data:
ExtraAccountMetaList::init(pda_data, &EXECUTE_DISCRIMINATOR, &metas)?;
```

Invoking a hook program via CPI:

```rust
use pinocchio_transfer_hook_interface::instruction::{AdditionalAccount, Execute};

Execute {
    source,
    mint,
    destination,
    authority,
    extra_account_metas_pda,
    additional_accounts: &[
        AdditionalAccount { account: &config, is_signer: false, is_writable: true },
    ],
    program_id: &hook_program_id,
    amount: 1_000_000,
}
.invoke()?;
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
