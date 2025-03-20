<p align="center">
 <img alt="pinocchio-log-macro" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-log-macro</code>
</h3>
<p align="center">
 Companion <code>log!</code> macro for <a href="https://crates.io/crates/pinocchio-log"><code>pinocchio-log</code></a>.
</p>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-log-macro"><img src="https://img.shields.io/crates/v/pinocchio-log-macro?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-log-macro"><img src="https://img.shields.io/docsrs/pinocchio-log-macro?logo=docsdotrs" /></a>
</p>

## Overview

The macro automates the creation of a `Logger` object to log a message. It support a subset of the [`format!`](https://doc.rust-lang.org/std/fmt/) syntax. The macro parses the format string at compile time and generates the calls to a `Logger` object to generate the corresponding formatted message.

## Usage

The macro works very similar to `solana-program` [`msg!`](https://docs.rs/solana-program/latest/solana_program/macro.msg.html) macro.

To output a simple message (static `&str`):
```rust
use pinocchio_log::log

log!("a simple log");
```

To output a formatted message:
```rust
use pinocchio_log::log

let amount = 1_000_000_000;
log!("transfer amount: {}", amount);
```

Since a `Logger` size is statically determined, messages are limited to `200` length by default. When logging larger messages, it is possible to increase the logger buffer size:
```rust
use pinocchio_log::log

let very_long_message = "...";
log!(500, "message: {}", very_long_message);
```

It is possible to include a precision formatting for numeric values:
```rust
use pinocchio_log::log

let lamports = 1_000_000_000;
log!("transfer amount (SOL: {:.9}", lamports);
```

For `&str` types, it is possible to specify a maximum length and a truncation strategy:
```rust
use pinocchio_log::log

let program_name = "pinocchio-program";
// log message: "...program"
log!("{:<.10}", program_name); 
// log message: "pinocchio-..."
log!("{:>.10}", program_name); 
```

## License

The code is licensed under the [Apache License Version 2.0](LICENSE)
