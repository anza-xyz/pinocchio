<p align="center">
 <img alt="pinocchio-record" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-record</code>
</h3>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-record"><img src="https://img.shields.io/crates/v/pinocchio-record?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-record"><img src="https://img.shields.io/docsrs/pinocchio-record?logo=docsdotrs" /></a>
</p>

## Overview

This crate contains [`pinocchio`](https://crates.io/crates/pinocchio) helpers to perform cross-program invocations (CPIs) for [SPL Record](https://github.com/solana-program/record) program instructions.

Each instruction defines a `struct` with the accounts and parameters required. Once all values are set, you can call directly `invoke` or `invoke_signed` to perform the CPI.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Getting Started

From your project folder:

```bash
cargo add pinocchio-record
```

This will add the `pinocchio-record` dependency to your `Cargo.toml` file.

## Examples

initialize:
```rust
// Those examples assume that each instruction receives writable `account` account and read-only `authority` account.
Initialize {
    account,
    authority,
}.invoke()?;
```

write:
```rust
// This example assumes that the instruction receives writable `account` account and a signer `authority` account.
Write {
    account,
    authority,
    offset: 12,
    data: &[1, 2, 3]
}.invoke()?;
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
