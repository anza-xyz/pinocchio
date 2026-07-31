<p align="center">
 <img alt="pinocchio-token" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" width="127" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-token</code>
</h3>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-token"><img src="https://img.shields.io/crates/v/pinocchio-token?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-token"><img src="https://img.shields.io/docsrs/pinocchio-token?logo=docsdotrs" /></a>
</p>

## Overview

This crate contains [`pinocchio`](https://crates.io/crates/pinocchio) helpers to perform cross-program invocations (CPIs) for SPL Token instructions.

Each instruction defines a `struct` with the accounts and parameters required. Once all values are set, you can call directly `invoke` or `invoke_signed` to perform the CPI.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Examples

Initializing a mint account:
```rust
use pinocchio_token::instructions::InitializeMint;

// This example assumes that the instruction receives a writable `mint`
// account; `authority` is an `&Address`.
InitializeMint::new(
    mint,            // mint account
    rent_sysvar,     // rent sysvar
    9,               // decimals
    authority,       // mint authority
    Some(authority), // freeze authority
).invoke()?;
```

Performing a transfer of tokens:
```rust
use pinocchio_token::instructions::Transfer;

// This example assumes that the instruction receives writable `from` and `to`
// accounts, and a signer `authority` account.
Transfer::new(
    from,        // from account
    to,          // to account
    authority,   // authority
    10,          // amount
).invoke()?;
```

## Using `Batch` instruction

The `Batch` (instruction discriminator `255`) enables efficient CPI interaction with the Token program.
This is a new instruction that can execute a variable number of Token instructions in a single invocation
of the Token program. Therefore, the base CPI invoke units (currently `1000` CUs) are only consumed once,
instead of for each CPI instruction. This significantly improves the CUs required to perform multiple
Token instructions in a CPI context.

To use a `Batch` instruction, we first create instructions to be executed and instead of invoking them
directly, we add them to a batch.

```rust
use {
  core::mem::MaybeUninit,
  pinocchio_token::instructions::{
    Batch, InitializeAccount, InitializeMint, IntoBatch, MintTo,
  },
};

// Determine the maximum number of accounts and data length required
// for the batch instruction.

const ACCOUNTS_LEN: usize = InitializeMint::ACCOUNTS_LEN
  + InitializeAccount::ACCOUNTS_LEN
  + MintTo::MAX_ACCOUNTS_LEN;

const DATA_LEN: usize = Batch::header_data_len(3)
  + InitializeMint::MAX_DATA_LEN
  + InitializeAccount::DATA_LEN
  + MintTo::DATA_LEN;

// Create uninitialized arrays for the batch instruction.

let mut data = [const { MaybeUninit::uninit() }; DATA_LEN];
let mut instruction_accounts = [const { MaybeUninit::uninit() }; ACCOUNTS_LEN];
let mut accounts = [const { MaybeUninit::uninit() }; ACCOUNTS_LEN];

// Create a new batch instruction with the uninitialized arrays.

let mut batch = Batch::new(&mut data, &mut instruction_accounts, &mut accounts)?;

InitializeMint::new(
  mint_account,
  rent_sysvar,
  9,
  authority,
  Some(authority),
)
.into_batch(&mut batch)?;

InitializeAccount::new(
  token_account,
  mint_account,
  owner,
  rent_sysvar
).into_batch(&mut batch)?;

MintTo::new(
  mint_account,
  token_account,
  authority_account,
  1000
).into_batch(&mut batch)?;

// Invoke the batch instruction to execute all instructions in a single CPI.

batch.invoke()?;
```

## Using different Token programs

The helpers in this crate offer two ways to use a different token program during invoke. Instead of using `invoke` and `invoke_signed`, programs can use:

  - `invoke_with_program` and `invoke_signed_with_program`: these accept an additional `Address`
    parameter and validate that it is equal to either the Token or Token-2022 program address.

  - `invoke_with_unverified_program` and `invoke_signed_with_unverified_program`: these accept
    an additional parameter but do not perform any validation. They can be used to invoke
    the instruction on any compatible custom token program.

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
