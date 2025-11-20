use {
    pinocchio::{log::sol_log, program_error::ProgramError, pubkey::Pubkey},
    solana_program_option::COption,
    spl_pod::optional_keys::OptionalNonZeroPubkey,
};

pub type Result<T> = core::result::Result<T, ProgramError>;

pub fn from_c_option<T>(data: COption<T>) -> Option<T> {
    if data.is_some() {
        Some(data.unwrap())
    } else {
        None
    }
}

pub fn from_optional_non_zero_pubkey(address: OptionalNonZeroPubkey) -> Option<Pubkey> {
    Option::<solana_address::Address>::from(address).map(|x| x.to_bytes())
}

pub fn show<T: core::fmt::Debug>(label: &str, data: T) {
    sol_log(&format!("✅ {}: {:?}", label, data));
}
