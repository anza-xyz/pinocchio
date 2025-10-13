<h1 align="center">
  <code>pinocchio</code>
</h1>
<p align="center">
  <img width="400" alt="Limestone" src="https://github.com/user-attachments/assets/3a1894b4-403f-4c35-90aa-548e7672fe90" />
</p>
<p align="center">
  Create Solana programs with no dependencies attached.
</p>

<p align="center">
  <a href="https://github.com/anza-xyz/pinocchio/actions/workflows/main.yml"><img src="https://img.shields.io/github/actions/workflow/status/anza-xyz/pinocchio/main.yml?logo=GitHub" /></a>
  <a href="https://crates.io/crates/pinocchio"><img src="https://img.shields.io/crates/v/pinocchio?logo=rust" /></a>
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

Pinocchio is a zero-dependency library to create Solana programs in Rust. It takes advantage of the way SVM loaders serialize the program input parameters into a byte array that is then passed to the program's entrypoint to define zero-copy types to read the input. Since the communication between a program and SVM loader &mdash; either at the first time the program is called or when one program invokes the instructions of another program &mdash; is done via a byte array, a program can define its own types. This completely eliminates the dependency on the `solana-program` crate, which in turn mitigates dependency issues by having a crate specifically designed to create on-chain programs.

As a result, Pinocchio can be used as a replacement for [`solana-program`](https://crates.io/crates/solana-program) to write on-chain programs, which are optimized in terms of both compute units consumption and binary size.

The library defines:
* program entrypoint
* core data types
* logging macros
* `syscall` functions
* access to system accounts (`sysvars`)
* cross-program invocation

## Features

* Zero dependencies and `no_std` crate
* Efficient `entrypoint!` macro – no copies or allocations
* Improved CU consumption of cross-program invocations

## Getting started

From your project folder:

```bash
cargo add pinocchio
```

This will add `pinocchio` as a dependency to your project.

## Defining the program entrypoint

A Solana program needs to define an entrypoint, which will be called by the runtime to begin the program execution. The `entrypoint!` macro emits the common boilerplate to set up the program entrypoint. The macro will also set up [global allocator](https://doc.rust-lang.org/stable/core/alloc/trait.GlobalAlloc.html) and [custom panic hook](https://github.com/anza-xyz/rust/blob/2830febbc59d44bdd7ad2c3b81731f1d08b96eba/library/std/src/sys/pal/sbf/mod.rs#L49) using the [default_allocator!](https://docs.rs/pinocchio/latest/pinocchio/macro.default_allocator.html) and [default_panic_handler!](https://docs.rs/pinocchio/latest/pinocchio/macro.default_panic_handler.html) macros.

The [`entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.entrypoint.html) is a convenience macro that invokes three other macros to set all symbols required for a program execution:

* [`program_entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.program_entrypoint.html): declares the program entrypoint
* [`default_allocator!`](https://docs.rs/pinocchio/latest/pinocchio/macro.default_allocator.html): declares the default (bump) global allocator
* [`default_panic_handler!`](https://docs.rs/pinocchio/latest/pinocchio/macro.default_panic_handler.html): declares the default panic handler

If all dependencies are `no_std`, you should append [`nostd_panic_handler!`](https://docs.rs/pinocchio/latest/pinocchio/macro.nostd_panic_handler.html) to declare a rust runtime panic handler. There's no need to do this if any dependency is `std` since rust compiler will emit std panic handler.

To use the `entrypoint!` macro, use the following in your entrypoint definition:
```rust
use pinocchio::{
  account::AccountView,
  entrypoint,
  ProgramResult,
  Address
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

`pinocchio` also offers variations of the program entrypoint (`lazy_program_allocator`) and global allocator (`no_allocator`). In order to use these, the program needs to specify the program entrypoint, global allocator and panic handler individually. The `entrypoint!` macro is equivalent to writing:
```rust
program_entrypoint!(process_instruction);
default_allocator!();
default_panic_handler!();
```
Any of these macros can be replaced by other implementations and `pinocchio` offers a couple of variants for this.

📌 [`lazy_program_entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.lazy_program_entrypoint.html)

The `entrypoint!` macro looks similar to the "standard" one found in `solana-program`. It parses the whole input and provides the `program_id`, `accounts` and `instruction_data` separately. This consumes compute units before the program begins its execution. In some cases, it is beneficial for a program to have more control when the input parsing is happening, even whether the parsing is needed or not &mdash; this is the purpose of the [`lazy_program_entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.lazy_program_entrypoint.html) macro. This macro only wraps the program input and provides methods to parse the input on demand.

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
  account::AccountView,
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

## Crate feature: `alloc`

The `alloc` feature is enabled by default and it uses the [`alloc`](https://doc.rust-lang.org/alloc/) crate. This provides access to dynamic memory allocation in combination with the `default_allocator`, e.g., required to use `String` and `Vec` in a program. Helpers that need to allocate memory, such as fetching `SlotHashes` sysvar data, are also available.

When no allocation is needed or desired, the feature can be disabled:

```
pinocchio = { version = "0.10.0", default-features = false }
```

## Advance entrypoint configuration

The symbols emitted by the entrypoint macros &mdash; program entrypoint, global allocator and default panic handler &mdash; can only be defined once globally. If the program crate is also intended to be used as a library, it is common practice to define a Cargo [feature](https://doc.rust-lang.org/cargo/reference/features.html) in your program crate to conditionally enable the module that includes the `entrypoint!` macro invocation. The convention is to name the feature `bpf-entrypoint`.

```rust
#[cfg(feature = "bpf-entrypoint")]
mod entrypoint {
  use pinocchio::{
    account::AccountView,
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

## License

The code is licensed under the [Apache License Version 2.0](LICENSE)

The library in this repository is based/includes code from:
* [`nitrate`](https://github.com/nifty-oss/nitrate)
* [`solana-nostd-entrypoint`](https://github.com/cavemanloverboy/solana-nostd-entrypoint/tree/main)
* [`solana-sdk`](https://github.com/anza-xyz/solana-sdk)
