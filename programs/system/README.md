<p align="center">
 <img alt="pinocchio-system" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-system</code>
</h3>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-system"><img src="https://img.shields.io/crates/v/pinocchio-system?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-system"><img src="https://img.shields.io/docsrs/pinocchio-system?logo=docsdotrs" /></a>
</p>

## Overview

This crate contains [`pinocchio`](https://crates.io/crates/pinocchio) helpers to perform cross-program invocations (CPIs) for System program instructions.

Each instruction defines a `struct` with the accounts and parameters required. Once all values are set, you can call directly `invoke` or `invoke_signed` to perform the CPI.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Examples

Creating a new account:
```rust
// This example assumes that the instruction receives a writable signer `payer_info`
// and `new_account_info` accounts.
CreateAccount {
    from: payer_info,
    to: new_account_info,
    lamports: 1_000_000_000, // 1 SOL
    space: 200,              // 200 bytes
    owner: &spl_token::ID,
}.invoke()?;
```

Performing a transfer of lamports:
```rust
// This example assumes that the instruction receives a writable signer `payer_info`
// account and a writable `recipient_info` account.
Transfer {
    from: payer_info,
    to: recipient_info,
    lamports: 500_000_000, // 0.5 SOL
}.invoke()?;
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
