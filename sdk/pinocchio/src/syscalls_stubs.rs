//! Custom syscall to use when pinocchio is built for non-SBF targets with `syscalls-stubs` feature
//! flag. Requires "std".

#![cfg(not(target_os = "solana"))]

use {
    crate::{
        account_info::AccountInfo,
        instruction::{Instruction, Signer},
        program::ReturnData,
        program_error::ProgramError,
        pubkey::Pubkey,
        ProgramResult, SUCCESS,
    },
    std::{
        boxed::Box,
        sync::{LazyLock, RwLock},
    },
};

#[allow(clippy::incompatible_msrv)]
static SYSCALL_STUBS: LazyLock<RwLock<Box<dyn SyscallStubs>>> =
    LazyLock::new(|| RwLock::new(Box::new(DefaultSyscallStubs {})));

pub fn set_syscall_stubs(syscall_stubs: Box<dyn SyscallStubs>) -> Box<dyn SyscallStubs> {
    std::mem::replace(&mut SYSCALL_STUBS.write().unwrap(), syscall_stubs)
}

pub trait SyscallStubs: Sync + Send {
    fn sol_log(&self, _message: &str) {}

    fn sol_log_64(&self, _arg1: u64, _arg2: u64, _arg3: u64, _arg4: u64, _arg5: u64) {}

    fn sol_log_data(&self, _data: &[&[u8]]) {}

    fn sol_log_pubkey(&self, _pubkey: &Pubkey) {}

    fn sol_try_find_program_address(
        &self,
        _seeds: &[&[u8]],
        _program_id: &Pubkey,
    ) -> Option<(Pubkey, u8)> {
        None
    }

    fn sol_create_program_address(
        &self,
        _seeds: &[&[u8]],
        _program_id: &Pubkey,
    ) -> Result<Pubkey, ProgramError> {
        Err(ProgramError::UnsupportedSysvar)
    }

    fn sol_create_with_seed(
        &self,
        _base: &Pubkey,
        _seed: &[u8],
        _program_id: &Pubkey,
    ) -> Result<Pubkey, ProgramError> {
        Err(ProgramError::UnsupportedSysvar)
    }

    fn sol_invoke_signed(
        &self,
        _instruction: &Instruction,
        _account_infos: &[&AccountInfo],
        _signers_seeds: &[Signer],
    ) -> ProgramResult {
        Err(ProgramError::UnsupportedSysvar)
    }

    fn sol_set_return_data(&self, _data: &[u8]) {}

    fn sol_get_return_data(&self) -> Option<ReturnData> {
        None
    }

    fn sol_get_clock_sysvar(&self, _var_addr: *mut u8) -> u64 {
        !SUCCESS
    }

    fn sol_get_fees_sysvar(&self, _var_addr: *mut u8) -> u64 {
        !SUCCESS
    }

    fn sol_get_rent_sysvar(&self, _var_addr: *mut u8) -> u64 {
        !SUCCESS
    }

    fn sol_memcpy(&self, _dst: &mut [u8], _src: &[u8], _n: usize) {}

    fn sol_memmove(&self, _dst: *mut u8, _src: *const u8, _n: usize) {}

    fn sol_memcmp(&self, _s1: &[u8], _s2: &[u8], _n: usize, _result: &mut i32) {}

    fn sol_memset(&self, _s: &mut [u8], _c: u8, _n: usize) {}
}

struct DefaultSyscallStubs {}
impl SyscallStubs for DefaultSyscallStubs {}

macro_rules! define_stub {
    (fn $name:ident($($arg:ident: $typ:ty),*) -> $ret:ty) => {
        pub(crate) fn $name($($arg: $typ),*) -> $ret {
            SYSCALL_STUBS.read().unwrap().$name($($arg),*)
        }
    };

    (fn $name:ident($($arg:ident: $typ:ty),*)) => {
		    define_stub!(fn $name($($arg: $typ),*) -> ());
    };
}

define_stub!(fn sol_log(message: &str));
define_stub!(fn sol_log_64(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64));
define_stub!(fn sol_log_data(data: &[&[u8]]));
define_stub!(fn sol_log_pubkey(pubkey: &Pubkey));
define_stub!(fn sol_try_find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> Option<(Pubkey, u8)>);
define_stub!(fn sol_create_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> Result<Pubkey, ProgramError>);
define_stub!(fn sol_create_with_seed(base: &Pubkey, seed: &[u8], program_id: &Pubkey) -> Result<Pubkey, ProgramError>);
define_stub!(fn sol_invoke_signed(instruction: &Instruction, account_infos: &[&AccountInfo], signers_seeds: &[Signer]) -> ProgramResult);
define_stub!(fn sol_set_return_data(data: &[u8]));
define_stub!(fn sol_get_return_data() -> Option<ReturnData>);
define_stub!(fn sol_get_clock_sysvar(var_addr: *mut u8) -> u64);
define_stub!(fn sol_get_fees_sysvar(var_addr: *mut u8) -> u64);
define_stub!(fn sol_get_rent_sysvar(var_addr: *mut u8) -> u64);
define_stub!(fn sol_memcpy(dst: &mut [u8], src: &[u8], n: usize));
define_stub!(fn sol_memmove(dst: *mut u8, src: *const u8, n: usize));
define_stub!(fn sol_memcmp(s1: &[u8], s2: &[u8], n: usize, result: &mut i32));
define_stub!(fn sol_memset(s: &mut [u8], c: u8, n: usize));
