#![no_std]

pub mod instructions;

extern crate alloc;

pinocchio_pubkey::declare_id!("AddressLookupTab1e1111111111111111111111111");

use core::mem::MaybeUninit;

// TODO: copy-pasted from token program/ Maybe it'll be better to move it under pinochio or even into dedicated crate
const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

#[inline(always)]
fn write_bytes(destination: &mut [MaybeUninit<u8>], source: &[u8]) {
    for (d, s) in destination.iter_mut().zip(source.iter()) {
        d.write(*s);
    }
}
