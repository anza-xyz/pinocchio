use {
    pinocchio::{log::sol_log, program_error::ProgramError},
    solana_program_option::COption,
};

pub type Result<T> = core::result::Result<T, ProgramError>;

pub fn from_c_option<T>(data: COption<T>) -> Option<T> {
    if data.is_some() {
        Some(data.unwrap())
    } else {
        None
    }
}

pub fn show<T: core::fmt::Debug>(label: &str, data: T) {
    sol_log(&format!("✅ {}: {:?}", label, data));
}
