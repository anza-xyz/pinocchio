use std::{
    boxed::Box,
    collections::HashMap,
    format,
    string::String,
    sync::{LazyLock, Mutex},
    vec::Vec,
};

use crate::{
    account_info::AccountInfo,
    instruction::{Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use super::Runtime;

pub static MOCK_RUNTIME: LazyLock<Mutex<MockRuntime>> =
    LazyLock::new(|| Mutex::new(MockRuntime::init()));

pub struct MockAccount {
    pub key: Pubkey,
    pub lamports: u64,
    pub data: MockData,
}

pub enum MockData {
    Bytes(Vec<u8>),
    Program(Box<dyn Fn(&Pubkey, &[AccountInfo], &[u8]) -> ProgramResult + Send>),
}

pub struct MockRuntime {
    accounts: HashMap<Pubkey, MockAccount>,
    logs: Vec<String>,
    compute_units: u64,
}

impl MockRuntime {
    pub fn init() -> Self {
        MockRuntime {
            accounts: HashMap::new(),
            logs: Vec::new(),
            compute_units: 0,
        }
    }

    pub fn add_program(&mut self, program: MockAccount) {
        let MockData::Program(_) = program.data else {
            panic!("invalid MockAccount")
        };
        self.accounts.insert(program.key, program);
    }

    pub fn invoke<const ACCOUNTS: usize>(
        &self,
        instruction: &Instruction,
        account_infos: &[&AccountInfo; ACCOUNTS],
    ) {
        let program = self
            .accounts
            .get(instruction.program_id)
            .expect("program not found");

        let MockData::Program(ref program) = program.data else {
            panic!("invalid program id")
        };

        let accounts: Vec<_> = account_infos.iter().map(|acc| (*acc).clone()).collect();

        program(
            instruction.program_id,
            accounts.as_slice(),
            instruction.data,
        )
        .expect("program failed to execute");
    }
}

impl Runtime for MockRuntime {
    ////////////////////////////////////////////////////////////////////////////
    // LOG SYS CALLS
    ////////////////////////////////////////////////////////////////////////////

    fn sol_log(message: &str) {
        MOCK_RUNTIME.lock().unwrap().logs.push(message.into());
    }

    fn sol_log_64(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) {
        let mut rt = MOCK_RUNTIME.lock().unwrap();
        rt.logs.push(format!(
            "Program log: {:x} {:x} {:x} {:x} {:x}",
            arg1, arg2, arg3, arg4, arg5
        ));
    }

    fn sol_log_data(data: &[&[u8]]) {
        MOCK_RUNTIME
            .lock()
            .unwrap()
            .logs
            .push(format!("data: {:?}", data));
    }

    fn sol_log_compute_units() {
        let mut rt = MOCK_RUNTIME.lock().unwrap();
        let cu = rt.compute_units;
        rt.logs.push(format!("cu: {}", cu));
    }

    ////////////////////////////////////////////////////////////////////////////
    // MEM SYS CALLS
    ////////////////////////////////////////////////////////////////////////////

    unsafe fn sol_memcpy(_dst: &mut [u8], _src: &[u8], _n: usize) {
        unimplemented!()
    }

    unsafe fn sol_memmove(_dst: *mut u8, _src: *mut u8, _n: usize) {
        unimplemented!()
    }

    unsafe fn sol_memcmp(_s1: &[u8], _s2: &[u8], _n: usize) -> i32 {
        unimplemented!()
    }

    unsafe fn sol_memset(_s: &mut [u8], _c: u8, _n: usize) {
        unimplemented!()
    }

    ////////////////////////////////////////////////////////////////////////////
    // CPI CALLS
    ////////////////////////////////////////////////////////////////////////////

    fn invoke_signed<const ACCOUNTS: usize>(
        instruction: &Instruction,
        account_infos: &[&AccountInfo; ACCOUNTS],
        _signers_seeds: &[Signer],
    ) -> ProgramResult {
        if instruction.accounts.len() < ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        for index in 0..ACCOUNTS {
            let account_info = account_infos[index];
            let account_meta = &instruction.accounts[index];

            if account_info.key() != account_meta.pubkey {
                return Err(ProgramError::InvalidArgument);
            }

            if account_meta.is_writable {
                account_info.check_borrow_mut_data()?;
                account_info.check_borrow_mut_lamports()?;
            } else {
                account_info.check_borrow_data()?;
                account_info.check_borrow_lamports()?;
            }
        }

        MOCK_RUNTIME
            .lock()
            .unwrap()
            .invoke(instruction, account_infos);

        Ok(())
    }

    unsafe fn invoke_signed_access_unchecked<const ACCOUNTS: usize>(
        instruction: &Instruction,
        account_infos: &[&AccountInfo; ACCOUNTS],
        _signers_seeds: &[Signer],
    ) -> ProgramResult {
        if instruction.accounts.len() < ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        for index in 0..ACCOUNTS {
            let account_info = account_infos[index];
            let account_meta = &instruction.accounts[index];
            if account_info.key() != account_meta.pubkey {
                return Err(ProgramError::InvalidArgument);
            }
        }

        MOCK_RUNTIME
            .lock()
            .unwrap()
            .invoke(instruction, account_infos);

        Ok(())
    }
}
