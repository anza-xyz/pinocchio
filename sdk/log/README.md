<p align="center">
 <img alt="pinocchio-log" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-log</code>
</h3>
<p align="center">
 Lightweight log utility for Solana programs.
</p>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-log"><img src="https://img.shields.io/crates/v/pinocchio-log?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-log"><img src="https://img.shields.io/docsrs/pinocchio-log?logo=docsdotrs" /></a>
</p>

## Overview

Currently, logging messages that require formatting are a bit heavy on the CU consumption. There are two aspects when comes to determining the cost of a log message:

1. `base cost`: this is the cost of the log syscall. It will either be the [`syscall_base_cost`](https://github.com/anza-xyz/agave/blob/master/compute-budget/src/compute_budget.rs#L167) (currently `100` CU) or a number of CUs equal to the length of the message, whichever value is higher.

2. `formatting cost`: the compute units required to format the message. This is variable and depends on the number and type of the arguments. Formatting is performed using Rust built-in `format!` routines, which in turn use `format_args!`.

It is known that Rust formatting routines are CPU-intensive for constrained environments. This has been noted on both the `solana-program` [`msg!`](https://docs.rs/solana-program/latest/solana_program/macro.msg.html) documentation and more generally on [rust development](https://github.com/rust-lang/rust/issues/99012).

While the cost related to (1) is *fixed*, in the sense that it does not change with the addition of formatting, it is possible to improve the overall cost of logging a formatted message using a lightweight formatting routine &mdash; this is what this crate does.

This crate defines a lightweight `Logger` type to format log messages and a companion `log!` macro. The logger is a fixed size buffer that can be used to format log messages before sending them to the log output. Any type that implements the `Log` trait can be appended to the logger. Additionally, the logger can the dereferenced to a `&[u8]` slice, which can be used for other purposes &mdash; e.g., it can be used to create `&str` to be stored on an account or return data of programs.

Below is a sample of the improvements observed when formatting log messages, measured in terms of compute units (CU):
| Output message                      | `log!` | `msg!`          | Improvement (%) |
|-------------------------------------|--------|-----------------|-----------------|
| `"Hello world!"`                    | 104    | 104             | -               |
| `"lamports={}"` + `u64`             | 286    | 625 (+339)      | 55%             |
| `"{}"` + `[&str; 2]`                | 119    | 1610 (+1491)    | 93%             |
| `"{}"` + `[u64; 2]`                 | 483    | 1154 (+671)     | 49%             |
| `"lamports={}"` + `i64`             | 299    | 659 (+360)      | 55%             |
| `"{}"` + `[u8; 32]` (address bytes) | 2783   | 8397 (+5614)    | 67%             |
| `"lamports={:.9}"` + `u64`          | 438    | 2656 (+2218)`*` | 84%             |

`*` For `msg!`, the value is logged as a `f64` otherwise the precision formatting is ignored.

> Note: The improvement in CU is accumulative, meaning that if you are logging multiple `u64` values, there will be a 40% improvement per formatted `u64` value.

## Features

* Zero dependencies and `no_std` crate
* Independent of SDK (i.e., works with `pinocchio`, `solana-program` or `anchor`)
* Support for `&str`, unsigned and signed integer types
* `log!` macro to facilitate log message formatting

## Getting Started

From your project folder:
```bash
cargo add pinocchio-log
```

## Usage

The `Logger` can be used directly:
```rust
use pinocchio_log::logger::Logger;

let mut logger = Logger::<100>::default();
logger.append("Hello ");
logger.append("world!");
logger.log();
```

 or via the `log!` macro:
 ```rust
use pinocchio_log::log

let lamports = 1_000_000_000;
log!("transfer amount: {}", lamports);
// Logs the transfer amount in SOL (lamports with 9 decimal digits)
log!("transfer amount (SOL): {:.9}", lamports);
```

Since the formatting routine does not perform additional allocations, the `Logger` type has a fixed size specified on its creation. When using the `log!` macro, it is also possible to specify the size of the logger buffer:

```rust
use pinocchio_log::log

let lamports = 1_000_000_000;
log!(50, "transfer amount: {}", lamports);
```

It is also possible to dereference the `Logger` into a `&[u8]` slice and use the result for other purposes:
```rust
use pinocchio_log::logger::Logger;

let amount = 1_000_000_000;
let mut logger = Logger::<100>::default();
logger.append("Prize ");
logger.append(amount);

let prize_title = core::str::from_utf8(&logger)?;
```

When using the `Logger` directly, it is possible to include a precision formatting for numeric values:
```rust
use pinocchio_log::logger::{Attribute, Logger};

let lamports = 1_000_000_000;
let mut logger = Logger::<100>::default();
logger.append("SOL: ");
logger.append_with_args(amount, &[Argument::Precision(9)]);
logger.log()
```

or a formatting string on the `log!` macro:
```rust
use pinocchio_log::log

let lamports = 1_000_000_000;
log!("transfer amount (SOL: {:.9}", lamports);
```

For `&str` types, it is possible to specify a maximum length and a truncation strategy using one of the `Argument::Truncate*` variants:
```rust
use pinocchio_log::logger::{Attribute, Logger};

let program_name = "pinocchio-program";
let mut logger = Logger::<100>::default();
logger.append_with_args(program_name, &[Argument::TruncateStart(10)]);
// log message: "...program"
logger.log();

let mut logger = Logger::<100>::default();
logger.append_with_args(program_name, &[Argument::TruncateEnd(10)]);
// log message: "pinocchio-..."
logger.log();
```

or a formatting string on the `log!` macro:
```rust
use pinocchio_log::log

let program_name = "pinocchio-program";
// log message: "...program"
log!("{:<.10}", program_name); 
// log message: "pinocchio-..."
log!("{:>.10}", program_name); 
```

## Formatting Options

Formatting options are represented by `Attribute` variants and can be passed to the `Logger` when appending messages using `append_with_args`.

| Variant                | Description                                     | Macro Format     |
| ---------------------- | ----------------------------------------------- | ---------------- |
| `Precision(u8)`        | Number of decimal places to display for numbers`*` | "{.*precision*}" |
| `TruncateEnd(usize)`   | Truncate the output at the end when the specified maximum number of characters (size) is exceeded | "{>.*size*}"     |
| `TruncateStart(usize)` | Truncate the output at the start when the specified maximum number of characters (size) is exceeded | "{<.*size*}"     |

`*` The `Precision` adds a decimal formatting to integer numbers. This is useful to log numeric integer amounts that represent values with decimal precision.

## License

The code is licensed under the [Apache License Version 2.0](LICENSE)
