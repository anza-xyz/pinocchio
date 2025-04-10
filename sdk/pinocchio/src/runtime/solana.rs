use super::Runtime;

pub struct SolanaRuntime;

impl Runtime for SolanaRuntime {
    fn sol_log(message: &str) {
        unsafe {
            crate::syscalls::sol_log_(message.as_ptr(), message.len() as u64);
        }
    }

    fn sol_log_64(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) {
        unsafe {
            crate::syscalls::sol_log_64_(arg1, arg2, arg3, arg4, arg5);
        }
    }

    fn sol_log_data(data: &[&[u8]]) {
        unsafe {
            crate::syscalls::sol_log_data(data as *const _ as *const u8, data.len() as u64);
        };
    }

    fn sol_log_compute_units() {
        unsafe {
            crate::syscalls::sol_log_compute_units_();
        }
    }
}
