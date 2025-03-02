# <img width="250" alt="pinocchio-system" src="https://github.com/user-attachments/assets/6a775333-c3a1-4623-aa7a-afdc8c492594"/>

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
