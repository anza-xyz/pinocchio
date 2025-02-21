# <img width="229" alt="pinocchio-token" src="https://github.com/user-attachments/assets/12b0dc2a-94fb-4866-8e6a-60ac74e13b4f"/>

This crate contains [`pinocchio`](https://crates.io/crates/pinocchio) helpers to perform cross-program invocations (CPIs) for SPL Token 2022 instructions.

Each instruction defines an `struct` with the accounts and parameters required. Once all values are set, you can call directly `invoke` or `invoke_signed` to perform the CPI.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Examples

Initializing a mint account:
```rust
// This example assumes that the instruction receives a writable `mint`
// account; `authority` is a `Pubkey`.
InitializeMint {
    mint,
    rent_sysvar,
    decimals: 9,
    mint_authority: authority,
    freeze_authority: Some(authority),
}.invoke()?;
```

Performing a transfer of tokens:
```rust
// This example assumes that the instruction receives writable `from` and `to`
// accounts, and a signer `authority` account.
Transfer {
    from,
    to,
    authority,
    amount: 10,
}.invoke()?;
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
