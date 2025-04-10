use std::{
    format,
    string::String,
    sync::{LazyLock, Mutex},
    vec::Vec,
};

use super::Runtime;

pub static MOCK_RUNTIME: LazyLock<Mutex<MockRuntime>> =
    LazyLock::new(|| Mutex::new(MockRuntime::init()));

pub struct MockRuntime {
    logs: Vec<String>,
    compute_units: u64,
}

impl MockRuntime {
    pub fn init() -> Self {
        MockRuntime {
            logs: Vec::new(),
            compute_units: 0,
        }
    }
}

impl Runtime for MockRuntime {
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
}
