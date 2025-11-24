<p align="center">
 <img alt="pinocchio-memo" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-memo</code>
</h3>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-memo"><img src="https://img.shields.io/crates/v/pinocchio-memo?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-memo"><img src="https://img.shields.io/docsrs/pinocchio-memo?logo=docsdotrs" /></a>
</p>

## Overview

This crate contains [`pinocchio`](https://crates.io/crates/pinocchio) helpers to perform cross-program invocations (CPIs) for [SPL Memo](https://github.com/solana-program/memo) program instructions.

Each instruction defines a `struct` with the accounts and parameters required. Once all values are set, you can call directly `invoke` or `invoke_signed` to perform the CPI.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Getting Started

From your project folder:

```bash
cargo add pinocchio-memo
```

This will add the `pinocchio-memo` dependency to your `Cargo.toml` file.

## Examples

Creating a memo:
```rust
// Both accounts should be signers
Memo {
    signers: &[&accounts[0], &accounts[1]],
    memo: "hello",
}
.invoke()?;
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
