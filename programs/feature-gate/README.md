<p align="center">
  <code>pinocchio-feature-gate</code>
</p>

## Overview

This crate contains [`pinocchio`](https://crates.io/crates/pinocchio)-compatible types and helpers for the [SPL Feature Gate program](https://github.com/solana-program/feature-gate), the on-chain program introduced by [SIMD-0089](https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0089-programify-feature-gate-program.md) that allows core contributors to revoke pending feature activations.

It provides:

- **Program and account IDs** for the Feature Gate program, the incinerator, and the System program.
- **`Feature`** — a 9-byte `#[repr(C)]` zero-copy type binary-compatible with the bincode encoding of `Option<u64>` used by the runtime.
- **`RevokePendingActivation`** — a CPI instruction builder for the program's sole instruction, used to revoke a feature activation that is pending but not yet activated by the runtime.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Layout of a feature account

| Offset | Size | Field                                                            |
|:------:|:----:|:-----------------------------------------------------------------|
| `0`    | `1`  | `Option` tag — `0` for `None` (pending), `1` for `Some` (activated) |
| `1`    | `8`  | Activation slot (little-endian `u64`, valid when tag is `1`)      |

## Examples

Reading a feature account's activation status from a program:

```rust
use pinocchio_feature_gate::state::Feature;

let feature = Feature::from_account_view(feature_account)?;

if let Some(slot) = feature.activated_at() {
    // Feature has been activated by the runtime at `slot`.
} else {
    // Feature is pending activation.
}
```

Revoking a pending feature activation via CPI:

```rust
use pinocchio_feature_gate::instructions::RevokePendingActivation;

RevokePendingActivation {
    feature,
    incinerator,
    system_program,
}
.invoke()?;
```

The `feature` account must be a signer and writable, and its lamports
will be burned to the [incinerator](https://explorer.solana.com/address/1nc1nerator11111111111111111111111111111111)
at the end of the current block.

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
