<p align="center">
 <img alt="pinocchio-zk-elgamal-proof" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-zk-elgamal-proof</code>
</h3>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-zk-elgamal-proof"><img src="https://img.shields.io/crates/v/pinocchio-zk-elgamal-proof?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-zk-elgamal-proof"><img src="https://img.shields.io/docsrs/pinocchio-zk-elgamal-proof?logo=docsdotrs" /></a>
</p>

## Overview

This crate contains [`pinocchio`](https://crates.io/crates/pinocchio) helpers to perform cross-program invocations (CPIs) for [ZK ElGamal Proof](https://github.com/solana-program/zk-elgamal-proof) program instructions.

Each instruction defines a `struct` with the accounts and parameters required. Once all values are set, you can call directly `invoke` or `invoke_signed` to perform the CPI.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Getting Started

From your project folder:

```bash
cargo add pinocchio-zk-elgamal-proof
```

This will add the `pinocchio-zk-elgamal-proof` dependency to your `Cargo.toml` file.

## Examples

Verify a public key validity with a context state and a proof data array:

```rust
// This example assumes that instruction receives writable `context_state_account` account
// and `context_state_authority` account.
VerifyPubkeyValidity {
    context_state_info: Some(ContextStateInfo {
        context_state_account,
        context_state_authority,
    }),
    proof: Proof::Data(proof_data),
}
.invoke()?;
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
