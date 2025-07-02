<p align="center">
 <img alt="pinocchio-token-2022" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-token-2022</code>
</h3>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-token-2022"><img src="https://img.shields.io/crates/v/pinocchio-token-2022?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-token-2022"><img src="https://img.shields.io/docsrs/pinocchio-token-2022?logo=docsdotrs" /></a>
</p>

## Overview

This crate contains [`pinocchio`](https://crates.io/crates/pinocchio) helpers to perform cross-program invocations (CPIs) for SPL Token-2022 instructions.

Each instruction defines a `struct` with the accounts and parameters required. Once all values are set, you can call directly `invoke` or `invoke_signed` to perform the CPI.

Instruction that are common to both SPL Token and SPL Token-2022 programs expect the program address, so they can be used to invoke either token program.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Examples

Initializing a mint account:

```rust
// This example assumes that the instruction receives a writable `mint`
// account; `authority` is a `Pubkey`.
// The SPL Token program is being invoked.
InitializeMint {
    mint,
    rent_sysvar,
    decimals: 9,
    mint_authority: authority,
    freeze_authority: Some(authority),
    token_program: Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
}.invoke()?;
```

Performing a transfer of tokens:

```rust
// This example assumes that the instruction receives writable `from` and `to`
// accounts, and a signer `authority` account.
// The SPL Token-2022 is being invoked.
Transfer {
    from,
    to,
    authority,
    amount: 10,
    token_program: Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")
}.invoke()?;
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
