#![no_std]

pub use invoker::*;

pub mod callback;
pub mod instructions;
pub mod invoke_parts;
pub mod invoker;

pinocchio_pubkey::declare_id!("11111111111111111111111111111111");
