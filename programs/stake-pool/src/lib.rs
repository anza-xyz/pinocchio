#![no_std]

pub mod instructions;
pub mod state;

pinocchio_pubkey::declare_id!("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy");

use core::mem::MaybeUninit;

const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

#[inline(always)]
fn write_bytes(destination: &mut [MaybeUninit<u8>], source: &[u8]) {
    for (d, s) in destination.iter_mut().zip(source.iter()) {
        d.write(*s);
    }
}
