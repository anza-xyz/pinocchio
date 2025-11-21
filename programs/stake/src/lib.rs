#![no_std]

pub mod instructions;
pub mod state;

pinocchio_pubkey::declare_id!("Stake11111111111111111111111111111111111111");

use core::mem::MaybeUninit;
use pinocchio::{account_info::AccountInfo, instruction::AccountMeta};

const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();
const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();

#[inline(always)]
fn write_bytes(destination: &mut [MaybeUninit<u8>], source: &[u8]) {
    for (d, s) in destination.iter_mut().zip(source.iter()) {
        d.write(*s);
    }
}
