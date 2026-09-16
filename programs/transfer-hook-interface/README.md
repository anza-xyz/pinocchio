<p align="center">
 <img alt="pinocchio-transfer-hook-interface" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" width="127" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-transfer-hook-interface</code>
</h3>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-transfer-hook-interface"><img src="https://img.shields.io/crates/v/pinocchio-transfer-hook-interface?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-transfer-hook-interface"><img src="https://img.shields.io/docsrs/pinocchio-transfer-hook-interface?logo=docsdotrs" /></a>
</p>

## Overview

The pinocchio-transfer-hook-interface library provides onchain helpers for resolving the additional accounts required for executing a transfer-hook instruction.

Each instruction defines a `struct` with the accounts and parameters required. Once all values are set, you can call directly `invoke` or `invoke_signed` to perform the CPI.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## License

The code is licensed under the [Apache License Version 2.0](../../LICENSE)
