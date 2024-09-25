# <img width="250" alt="pinocchio-macro" src="https://github.com/user-attachments/assets/d6c34b56-81ed-4ea1-a0d5-6d9eebe6e95b"/>

Companion `Pubkey` helpers [`pinocchio`](https://github.com/febo/pinocchio).

This crate provides two convenience macros to resolve `Pubkey`s at compile time:

* `pubkey!`: takes a pubkey value as a base58 `&str` and generates its correpondent `Pubkey` (byte array)
* `declare_id!`: takes a pubkey value as a base58 `&str` (usually representing a program address) and generates an `ID` constant, `check_id()` and `id()` helpers

It also defines a `decode` helper that takes a `&str` and returns the correspondent `Pubkey` value.

## Examples

Creating a `Pubkey` constant value from a `&str`:
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
let owner = pinocchio_pubkey::decode("7qtAvP4CJuSKauWHtHZJt9wmQRgvcFeUcU3xKrFzxKf1");
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
