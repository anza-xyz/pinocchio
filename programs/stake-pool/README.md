<p align="center">
 <img alt="pinocchio-stake-pool" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-stake-pool</code>
</h3>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-stake-pool"><img src="https://img.shields.io/crates/v/pinocchio-stake-pool?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-stake-pool"><img src="https://img.shields.io/docsrs/pinocchio-stake-pool?logo=docsdotrs" /></a>
</p>

## Overview

This crate contains [`pinocchio`](https://crates.io/crates/pinocchio) helpers to perform cross-program invocations (CPIs) for [Stake-Pool](https://github.com/solana-program/stake-pool) program instructions.

Each instruction defines a `struct` with the accounts and parameters required. Once all values are set, you can call directly `invoke` or `invoke_signed` to perform the CPI.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Getting Started

From your project folder:

```bash
cargo add pinocchio-stake-pool
```

This will add the `pinocchio-stake-pool` dependency to your `Cargo.toml` file.

## Examples

Creating a stake pool:
```rust
// Both accounts should be signers
Memo {
    signers: &[&account_infos[0], &account_infos[1]],
    stake: "hello",
}
.invoke()?;
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
