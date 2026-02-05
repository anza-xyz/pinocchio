# Address Lookup Table // TODO: fancy image insted of H1


This crate contains [`pinocchio`](https://crates.io/crates/pinocchio) helpers to perform cross-program invocations (CPIs) for Address Lookup Table program instructions.

Each instruction defines a `struct` with the accounts and parameters required. Once all values are set, you can call directly `invoke` or `invoke_signed` to perform the CPI.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Getting Started

From your project folder:

```bash
cargo add pinocchio-address-lookup-table
```

This will add the `pinocchio-address-lookup-table` dependency to your `Cargo.toml` file.

## Examples
// TODO: provide examples

## License

The code is licensed under the [Apache License Version 2.0](LICENSE)
