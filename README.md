<h1 align="center">
  <code>pinocchio</code>
</h1>
<p align="center">
  <img width="400" alt="Limestone" src="https://github.com/user-attachments/assets/3a1894b4-403f-4c35-90aa-548e7672fe90" />
</p>
<p align="center">
  Create Solana programs with no external dependencies attached.
</p>

<p align="center">
  <a href="https://github.com/anza-xyz/pinocchio/actions/workflows/main.yml"><img src="https://img.shields.io/github/actions/workflow/status/anza-xyz/pinocchio/main.yml?logo=GitHub" /></a>
  <a href="https://crates.io/crates/pinocchio"><img src="https://img.shields.io/crates/v/pinocchio?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio"><img src="https://img.shields.io/docsrs/pinocchio?logo=docsdotrs" /></a>
</p>

<p align="right">
<i>I've got no dependencies</i><br />
<i>To hold me down</i><br />
<i>To make me fret</i><br />
<i>Or make me frown</i><br />
<i>I had dependencies</i><br />
<i>But now I'm free</i><br />
<i>There are no dependencies on me</i>
</p>

## Overview

Pinocchio is a *no external* dependencies library to create Solana programs in Rust. The only dependencies are types from the Solana SDK specifically designed for on-chain programs. This mitigates dependency issues and offers an efficient zero-copy library to write programs, optimized in terms of both compute units consumption and binary size.

## Features

* `no_std` crate
* only dependencies to Solana SDK types
* efficient `program_entrypoint!` macro with no copies or allocations
* lightweight `lazy_program_entrypoint` providing more control over how the input is parsed

## Getting started

From your project folder:

```bash
cargo add pinocchio
```

This will add `pinocchio` as a dependency to your project.

## Defining the program entrypoint

A Solana program needs to define an entrypoint, which will be called by the runtime to begin the program execution. The `entrypoint!` macro emits the common boilerplate to set up the program entrypoint. The macro will also set up [global allocator](https://doc.rust-lang.org/stable/core/alloc/trait.GlobalAlloc.html) and [custom panic hook](https://github.com/anza-xyz/rust/blob/2830febbc59d44bdd7ad2c3b81731f1d08b96eba/library/std/src/sys/pal/sbf/mod.rs#L49) using the [default_allocator!](https://docs.rs/pinocchio/latest/pinocchio/macro.default_allocator.html) and [default_panic_handler!](https://docs.rs/pinocchio/latest/pinocchio/macro.default_panic_handler.html) macros.

The [`entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.entrypoint.html) is a convenience macro that invokes three other macros to set all components required for a program execution:

* [`program_entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.program_entrypoint.html): declares the program entrypoint
* [`default_allocator!`](https://docs.rs/pinocchio/latest/pinocchio/macro.default_allocator.html): declares the default (bump) global allocator
* [`default_panic_handler!`](https://docs.rs/pinocchio/latest/pinocchio/macro.default_panic_handler.html): declares the default panic handler

When all dependencies are `no_std`, you should use [`nostd_panic_handler!`](https://docs.rs/pinocchio/latest/pinocchio/macro.nostd_panic_handler.html) instead of `default_panic_handler!` to declare a rust runtime panic handler. There's no need to do this when any dependency is `std` since rust compiler will emit a panic handler.

To use the `entrypoint!` macro, use the following in your entrypoint definition:
```rust
use pinocchio::{
  AccountView,
  Address,
  entrypoint,
  ProgramResult
};
use solana_program_log::log;

entrypoint!(process_instruction);

pub fn process_instruction(
  program_id: &Address,
  accounts: &[AccountView],
  instruction_data: &[u8],
) -> ProgramResult {
  log!("Hello from my pinocchio program!");
  Ok(())
}
```

The information from the input is parsed into their own entities:

* `program_id`: the `ID` of the program being called
* `accounts`: the accounts received
* `instruction_data`: data for the instruction

`pinocchio` also offers variations of the program entrypoint (`lazy_program_entrypoint`) and global allocator (`no_allocator`). In order to use these, the program needs to specify the program entrypoint, global allocator and panic handler individually. The `entrypoint!` macro is equivalent to writing:
```rust
program_entrypoint!(process_instruction);
default_allocator!();
default_panic_handler!();
```
Any of these macros can be replaced by alternative implementations.

📌 [`lazy_program_entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.lazy_program_entrypoint.html)

The `entrypoint!` macro looks similar to the "standard" one found in [`solana-program-entrypoint`](https://docs.rs/solana-program-entrypoint/latest/solana_program_entrypoint/macro.entrypoint.html). It parses the whole input and provides the `program_id`, `accounts` and `instruction_data` separately. This consumes compute units before the program begins its execution. In some cases, it is beneficial for a program to have more control when the input parsing is happening, even whether the parsing is needed or not &mdash; this is the purpose of the [`lazy_program_entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.lazy_program_entrypoint.html) macro. This macro only wraps the program input and provides methods to parse the input on demand.

The `lazy_entrypoint` is suitable for programs that have a single or very few instructions, since it requires the program to handle the parsing, which can become complex as the number of instructions increases. For *larger* programs, the [`program_entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.program_entrypoint.html) will likely be easier and more efficient to use.

To use the `lazy_program_entrypoint!` macro, use the following in your entrypoint definition:
```rust
use pinocchio::{
  default_allocator,
  default_panic_handler,
  entrypoint::InstructionContext,
  lazy_program_entrypoint,
  ProgramResult
};

lazy_program_entrypoint!(process_instruction);
default_allocator!();
default_panic_handler!();

pub fn process_instruction(
  mut context: InstructionContext
) -> ProgramResult {
    Ok(())
}
```

The `InstructionContext` provides on-demand access to the information of the input:

* `remaining()`: number of remaining accounts to be parsed
* `next_account()`: parsers the next available account (can be used as many times as accounts remaining)
* `instruction_data()`: parsers the instruction data
* `program_id()`: parsers the program id

> ⚠️ **Note:**
> The `lazy_program_entrypoint!` does not set up a global allocator nor a panic handler. A program should explicitly use one of the provided macros to set them up or include its own implementation.

📌 [`no_allocator!`](https://docs.rs/pinocchio/latest/pinocchio/macro.no_allocator.html)

When writing programs, it can be useful to make sure the program does not attempt to make any allocations. For this cases, `pinocchio` includes a [`no_allocator!`](https://docs.rs/pinocchio/latest/pinocchio/macro.no_allocator.html) macro that sets a global allocator just panics at any attempt to allocate memory.

To use the `no_allocator!` macro, use the following in your entrypoint definition:
```rust
use pinocchio::{
  AccountView,
  default_panic_handler,
  no_allocator,
  program_entrypoint,
  ProgramResult,
  Address
};

program_entrypoint!(process_instruction);
default_panic_handler!();
no_allocator!();

pub fn process_instruction(
  program_id: &Address,
  accounts: &[AccountView],
  instruction_data: &[u8],
) -> ProgramResult {
  Ok(())
}
```
> ⚠️ **Note:**
> The `no_allocator!` macro can also be used in combination with the `lazy_program_entrypoint!`.

Since the `no_allocator!` macro does not allocate memory, the `32kb` memory region reserved for the heap remains unused. To take advantage of this, the `no_allocator!` macro emits an `allocate_unchecked` helper function that allows you to manually reserve memory for a type at compile time.

```rust
/// static allocation:
///    - 0 is the offset when the type will be allocated
///    - `allocate_unchecked` returns a mutable reference to the allocated type
let lamports = allocate_unchecked::<u64>(0);
*lamports = 1_000_000_000;
```

Note that it is the developer's responsibility to ensure that types do not overlap in memory &mdash; the `offset + <size of type>` of different types must not overlap.

## Crate features

### `alloc`

The `alloc` feature is enabled by default and it uses the [`alloc`](https://doc.rust-lang.org/alloc/) crate. This provides access to dynamic memory allocation in combination with the `default_allocator`, e.g., required to use `String` and `Vec` in a program. Helpers that need to allocate memory, such as fetching `SlotHashes` sysvar data, are also available.

When no allocation is needed or desired, the feature can be disabled:

```
pinocchio = { version = "0.10.0", default-features = false }
```

> ⚠️ **Note:**
> The `default_allocator` macro is not available when disabling the `alloc` feature.

### `copy`

The `copy` feature enables the derivation of the `Copy` trait for types. It also enables the `copy` feature
on the `solana-account-view` and `solana-address` re-exports.

### `cpi`

The `cpi` feature enables the cross-program invocation helpers, as well as types to define instructions and signer information.

```
pinocchio = { version = "0.10.0", features = ["cpi"] }
```

## Advanced entrypoint configuration

The components emitted by the entrypoint macros &mdash; program entrypoint, global allocator and default panic handler &mdash; can only be defined once globally. If the program crate is also intended to be used as a library, it is common practice to define a Cargo [feature](https://doc.rust-lang.org/cargo/reference/features.html) in your program crate to conditionally enable the module that includes the `entrypoint!` macro invocation. The convention is to name the feature `bpf-entrypoint`.

```rust
#[cfg(feature = "bpf-entrypoint")]
mod entrypoint {
  use pinocchio::{
    AccountView,
    entrypoint,
    ProgramResult,
    Address
  };

  entrypoint!(process_instruction);

  pub fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
  ) -> ProgramResult {
    Ok(())
  }
}
```
When building the program binary, you must enable the `bpf-entrypoint` feature:
```bash
cargo build-sbf --features bpf-entrypoint
```

## Upstream BPF compatibility

Pinocchio is compatible with upstream BPF target (`target_arch = bpf`). When using syscalls (e.g.,
cross-program invocations), it is necessary to explicitly enable static syscalls in your
program's `Cargo.toml`:
```
[dependencies]
# Enable static syscalls for BPF target
solana-define-syscall = { version = "4.0.1", features = ["unstable-static-syscalls"] }
```

## License

The code is licensed under the [Apache License Version 2.0](LICENSE)
