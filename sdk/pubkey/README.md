<p align="center">
 <img alt="pinocchio-pubkey" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-pubkey</code>
</h3>
<p align="center">
 Companion <code>Pubkey</code> helpers for <a href="https://github.com/anza-xyz/pinocchio"><code>pinocchio</code></a>.
</p>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-pubkey"><img src="https://img.shields.io/crates/v/pinocchio-pubkey?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-pubkey"><img src="https://img.shields.io/docsrs/pinocchio-pubkey?logo=docsdotrs" /></a>
</p>

## Overview

This crate provides two convenience macros to resolve `Address`es at compile time:

* `address!`: takes an address value as a base58 `&str` and generates its correspondent `Address` (byte array)
* `declare_id!`: takes an address value as a base58 `&str` (usually representing a program address) and generates an `ID` constant, `check_id()` and `id()` helpers

It also defines a `from_str` helper that takes a `&str` and returns the correspondent `Address` value.

## Examples

Creating an `Address` constant value from a static `&str`:
```rust
use pinocchio::Address;

pub const AUTHORITY: Address = pinocchio_pubkey::address!("7qtAvP4CJuSKauWHtHZJt9wmQRgvcFeUcU3xKrFzxKf1");
```

Declaring the program address of a program (usually on your `lib.rs`):
```rust
pinocchio_pubkey::declare_id!("Ping111111111111111111111111111111111111111");
```

Creating an `Address` from a `&str`:
```rust
let address = String::from("7qtAvP4CJuSKauWHtHZJt9wmQRgvcFeUcU3xKrFzxKf1");
let owner = pinocchio_pubkey::from_str(&address);
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
