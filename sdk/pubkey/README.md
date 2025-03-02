# <img width="250" alt="pinocchio-pubkey" src="https://github.com/user-attachments/assets/de950d77-1f02-4d52-a2a8-fbf4029aa2dc"/>

Companion `Pubkey` helpers for [`pinocchio`](https://github.com/febo/pinocchio).

This crate provides two convenience macros to resolve `Pubkey`s at compile time:

* `pubkey!`: takes a pubkey value as a base58 `&str` and generates its correspondent `Pubkey` (byte array)
* `declare_id!`: takes a pubkey value as a base58 `&str` (usually representing a program address) and generates an `ID` constant, `check_id()` and `id()` helpers

It also defines a `from_str` helper that takes a `&str` and returns the correspondent `Pubkey` value.

## Examples

Creating a `Pubkey` constant value from a static `&str`:
```rust
use pinocchio::pubkey::Pubkey;

pub const AUTHORITY: Pubkey = pinocchio_pubkey::pubkey!("7qtAvP4CJuSKauWHtHZJt9wmQRgvcFeUcU3xKrFzxKf1");
```

Declaring the program address of a program (usually on your `lib.rs`):
```rust
pinocchio_pubkey::declare_id!("Ping111111111111111111111111111111111111111");
```

Creating a `Pubkey` from a `&str`:
```rust
let address = String::from("7qtAvP4CJuSKauWHtHZJt9wmQRgvcFeUcU3xKrFzxKf1");
let owner = pinocchio_pubkey::from_str(&address);
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
