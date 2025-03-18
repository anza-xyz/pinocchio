<p align="center">
 <img alt="pinocchio-associated-token-account" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-associated-token-account</code>
</h3>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-associated-token-account"><img src="https://img.shields.io/crates/v/pinocchio-associated-token-account?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-associated-token-account"><img src="https://img.shields.io/docsrs/pinocchio-associated-token-account?logo=docsdotrs" /></a>
</p>

## Overview

This crate contains [`pinocchio`](https://crates.io/crates/pinocchio) helpers to perform cross-program invocations (CPIs) for SPL Associated Token Account program instructions.

Each instruction defines a `struct` with the accounts and parameters required. Once all values are set, you can call directly `invoke` or `invoke_signed` to perform the CPI.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Getting Started

From your project folder:

```bash
cargo add pinocchio-associated-token-account
```

This will add the `pinocchio-associated-token-account` dependency to your `Cargo.toml` file.

## Examples

Creating an associated token account:
```rust
// Those examples assume that each instruction receives writable and signer `funding_account` account,
// writable `account` account, and `wallet`, `mint`, `system_program`, `token_program` accounts.
Create {
    funding_account,
    account,
    wallet,
    mint,
    system_program,
    token_program,
}.invoke()?;

CreateIdempotent {
    funding_account,
    account,
    wallet,
    mint,
    system_program,
    token_program,
}.invoke()?;
```

Recovering Nested
```rust
// This example assumes that instruction receives writable and signer `wallet` account,
// writable `account` and `destination_account`, and `mint`, `owner_account`, `owner_mint`,
// `token_program` accounts.
RecoverNested {
    account,
    mint,
    destination_account,
    owner_account,
    owner_mint,
    wallet,
    token_program,
}.invoke()?;
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
