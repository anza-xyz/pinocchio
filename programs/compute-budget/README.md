<p align="center">
  <img alt="pinocchio-compute-budget" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" height="100"/>
</p>
<h3 align="center">
  <code>pinocchio-compute-budget</code>
</h3>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-compute-budget"><img src="https://img.shields.io/crates/v/pinocchio-compute-budget?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-compute-budget"><img src="https://img.shields.io/docsrs/pinocchio-compute-budget?logo=docsdotrs" /></a>
</p>

## Overview

This crate contains [`pinocchio`](https://crates.io/crates/pinocchio) helpers to perform cross-program invocations (CPIs) for Compute Budget program instructions.

The Compute Budget program allows you to optimize transaction costs and execution by controlling compute unit limits, setting priority fees, and requesting additional heap memory.

Each instruction defines a `struct` with the required parameters. Once all values are set, you can call `invoke()` to perform the CPI.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Instructions

### SetComputeUnitLimit

Set the maximum compute units for your transaction. Default is 1,400,000 CU. Setting a lower limit reduces fees when you know your program uses less.

```rust
use pinocchio_compute_budget::SetComputeUnitLimit;

// Limit transaction to 50,000 compute units
SetComputeUnitLimit {
    units: 50_000,
}.invoke()?;
```

### SetComputeUnitPrice

Set the price in micro-lamports per compute unit. This is the **priority fee** mechanism - higher prices lead to faster confirmation.

The total priority fee is calculated as: `priority_fee = compute_unit_limit * compute_unit_price`

```rust
use pinocchio_compute_budget::SetComputeUnitPrice;

// Set priority fee to 10,000 micro-lamports per CU
// With 50,000 CU limit above, this is 50,000 * 10,000 = 500,000,000 micro-lamports
// = 0.5 lamports priority fee
SetComputeUnitPrice {
    micro_lamports: 10_000,
}.invoke()?;
```

**Common price ranges:**
- `0` - No priority (default, slowest)
- `1-1,000` - Low priority
- `1,000-10,000` - Medium priority  
- `10,000-100,000` - High priority
- `100,000+` - Very high priority (network congestion)

### RequestHeapFrame

Request additional heap memory beyond the default 32 KB, up to 256 KB total.

```rust
use pinocchio_compute_budget::RequestHeapFrame;

// Request an additional 32 KB heap frame (64 KB total)
RequestHeapFrame {
    bytes: 32 * 1024,
}.invoke()?;
```

**Requirements:**
- Must be a multiple of 8 KB (8,192 bytes)
- Maximum total heap is 256 KB

## Complete Example

Here's how to combine compute budget instructions in a program:

```rust
use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    msg,
    ProgramResult,
    pubkey::Pubkey,
};
use pinocchio_compute_budget::{SetComputeUnitLimit, SetComputeUnitPrice};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Set compute limit to save on fees
    SetComputeUnitLimit {
        units: 50_000,
    }.invoke()?;

    // Set priority fee for fast confirmation
    SetComputeUnitPrice {
        micro_lamports: 10_000,
    }.invoke()?;

    // Your program logic here...
    msg!("Processing with optimized compute budget!");

    Ok(())
}
```

## Why Use Compute Budget Instructions?

1. **Save on Fees**: Set lower compute unit limits when you know your program uses less
2. **Faster Confirmation**: Set priority fees to incentivize validators
3. **More Heap**: Request additional heap for programs that need it
4. **Better UX**: Faster transactions = better user experience

## License

The code is licensed under the [Apache License Version 2.0](../../LICENSE)
