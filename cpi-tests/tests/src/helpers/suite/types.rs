use {
    solana_keypair::Keypair,
    solana_program_option::COption,
    std::fmt::Debug,
    strum::IntoEnumIterator,
    strum_macros::{Display, EnumIter, IntoStaticStr},
};

const SOL_AMOUNT_DEFAULT: u64 = 1_000;
const SOL_AMOUNT_INCREASED: u64 = 100_000;

const TOKEN_AMOUNT_DEFAULT: u64 = 1_000_000_000;
const TOKEN_AMOUNT_INCREASED: u64 = 100_000_000_000;

const DECIMALS_COIN_SOL: u8 = 9;
const DECIMALS_TOKEN_DEFAULT: u8 = 6;
const DECIMALS_TOKEN_WBTC: u8 = 8;

const KEYPAIR_ADMIN: &str =
    "3SKiuW2cbAJH8KDAuhB5cdJnAGU8Y9a95gRWMFB6zPy8XH45HTNebRALhL1EqPv2QkBytb8iTu577TcmLutkzC9g";
const PUBKEY_ADMIN: &str = "Fk5wpZL2pV8AYkMKnEo5TAJ1p88FmUxBbKsZLwpiqWqQ";

const KEYPAIR_ALICE: &str =
    "4TwYiTAG6eHLznaSGZinmQGSFxKxxmx7DHwKcbs5WkasMmLPP5fv1BYKJjsfmR47KFzmA2gs5DHtsZnR8YvMCinB";
const PUBKEY_ALICE: &str = "68ZZmGRDn5971SDrj5Ldj6MUJTeRUdSV1NQUuzsaQ4N3";

const KEYPAIR_BOB: &str =
    "zsbe2oRXt1K3gRNCurjZFTVzQtYqJjhyPAQMk4VsLWe3QoU3pMGZDVVRvmgZXgLtXvAsC9kGi4ShpYpjrQbtaf5";
const PUBKEY_BOB: &str = "FPS369ZvUkQTdsU8pzmypafNnNghiDHi8G6gDCvux5SB";

const KEYPAIR_USDC: &str =
    "5gi185z4U57MEkJJTzweNrJQJQftaQH2onL8ZGRyXKWA3zspJWyQfF1J8ZRV7zd3D8aZyZxtaw8MsPZpMLMGh6L2";
const PUBKEY_USDC: &str = "8XdLEJXrM3yYfFg5EpMqcCKmXXSQeBKfTjvNP619LbE2";

const KEYPAIR_PYTH: &str =
    "5fbcPBxRADG5oxsK3K7PtM5A2CXFSErQ7bWoTXA1qeZsngyFYzWKUm4R7pBtD9fazVA9FgFC4h4WschCTQ7xjeJG";
const PUBKEY_PYTH: &str = "HBtzyBH14hR6t5UfYT3ptL6d1pMnVCep2RY8vUgHmaRA";

const KEYPAIR_WBTC: &str =
    "2RyN2wrHo8fDrvqULn61ThcSeMyBE3eQ35ADxk5bvjkMrtZRKZwYNRQgxS33UkTrw3udySYMeoJxapbLbyz3aDiZ";
const PUBKEY_WBTC: &str = "An6eCPnnsspFAy5bUrgnNkU4hkedv9ZDRUJazUTG1ewb";

const PUBKEY_WSOL: &str = "So11111111111111111111111111111111111111112";

/// for extensions
pub trait SolPubkey {
    fn pubkey(&self) -> solana_pubkey::Pubkey;
}

/// for actual tests
pub trait PinPubkey {
    fn pubkey(&self) -> pinocchio::pubkey::Pubkey;
}

#[derive(Debug, Clone, Copy, Display, IntoStaticStr, EnumIter, PartialEq, Eq)]
pub enum AppUser {
    Admin,
    Alice,
    Bob,
}

impl SolPubkey for AppUser {
    fn pubkey(&self) -> solana_pubkey::Pubkey {
        solana_pubkey::Pubkey::from_str_const(self.get_pubkey_str())
    }
}

impl PinPubkey for AppUser {
    fn pubkey(&self) -> pinocchio::pubkey::Pubkey {
        pinocchio_pubkey::from_str(self.get_pubkey_str())
    }
}

impl AppUser {
    fn get_pubkey_str(&self) -> &str {
        match self {
            Self::Admin => PUBKEY_ADMIN,
            Self::Alice => PUBKEY_ALICE,
            Self::Bob => PUBKEY_BOB,
        }
    }

    pub fn keypair(&self) -> Keypair {
        let base58_string = match self {
            Self::Admin => KEYPAIR_ADMIN,
            Self::Alice => KEYPAIR_ALICE,
            Self::Bob => KEYPAIR_BOB,
        };

        Keypair::from_base58_string(base58_string)
    }

    pub fn list() {
        for item in Self::iter() {
            println!("{:#?}: {:#?}", item, SolPubkey::pubkey(&item));
        }
        println!();
    }

    pub fn get_initial_asset_amount(&self, asset: impl Into<AppAsset>) -> u64 {
        match self {
            Self::Admin => match asset.into() {
                AppAsset::Coin(_) => SOL_AMOUNT_INCREASED,
                AppAsset::Token(_) => TOKEN_AMOUNT_INCREASED,
            },
            _ => match asset.into() {
                AppAsset::Coin(_) => SOL_AMOUNT_DEFAULT,
                AppAsset::Token(_) => TOKEN_AMOUNT_DEFAULT,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Display, IntoStaticStr, EnumIter, PartialEq, Eq)]
pub enum AppCoin {
    SOL,
}

#[derive(Debug, Clone, Copy, Display, IntoStaticStr, EnumIter, PartialEq, Eq)]
pub enum AppToken {
    USDC,
    PYTH,
    WBTC,
    WSOL,
}

impl SolPubkey for AppToken {
    fn pubkey(&self) -> solana_pubkey::Pubkey {
        solana_pubkey::Pubkey::from_str_const(self.get_pubkey_str())
    }
}

impl PinPubkey for AppToken {
    fn pubkey(&self) -> pinocchio::pubkey::Pubkey {
        pinocchio_pubkey::from_str(self.get_pubkey_str())
    }
}

impl AppToken {
    fn get_pubkey_str(&self) -> &str {
        match self {
            Self::USDC => PUBKEY_USDC,
            Self::PYTH => PUBKEY_PYTH,
            Self::WBTC => PUBKEY_WBTC,
            Self::WSOL => PUBKEY_WSOL,
        }
    }

    pub fn keypair(&self) -> Keypair {
        let base58_string = match self {
            Self::USDC => KEYPAIR_USDC,
            Self::PYTH => KEYPAIR_PYTH,
            Self::WBTC => KEYPAIR_WBTC,
            Self::WSOL => panic!("WSOL doesn't have keypair!"),
        };

        Keypair::from_base58_string(base58_string)
    }

    pub fn list() {
        for item in Self::iter() {
            println!("{:#?}: {:#?}", item, SolPubkey::pubkey(&item));
        }
        println!();
    }
}

pub trait GetDecimals {
    fn get_decimals(&self) -> u8;
}

impl GetDecimals for AppAsset {
    fn get_decimals(&self) -> u8 {
        match self {
            Self::Coin(project_coin) => project_coin.get_decimals(),
            Self::Token(project_token) => project_token.get_decimals(),
        }
    }
}

impl GetDecimals for AppCoin {
    fn get_decimals(&self) -> u8 {
        match self {
            Self::SOL => DECIMALS_COIN_SOL,
        }
    }
}

impl GetDecimals for AppToken {
    fn get_decimals(&self) -> u8 {
        match self {
            Self::USDC => DECIMALS_TOKEN_DEFAULT,
            Self::PYTH => DECIMALS_TOKEN_DEFAULT,
            Self::WBTC => DECIMALS_TOKEN_WBTC,
            Self::WSOL => DECIMALS_COIN_SOL,
        }
    }
}

#[derive(Debug, Clone, Copy, Display)]
pub enum AppAsset {
    Coin(AppCoin),
    Token(AppToken),
}

impl From<AppCoin> for AppAsset {
    fn from(project_coin: AppCoin) -> Self {
        Self::Coin(project_coin)
    }
}

impl From<AppToken> for AppAsset {
    fn from(project_token: AppToken) -> Self {
        Self::Token(project_token)
    }
}

pub type Result<T> = std::result::Result<T, pinocchio::program_error::ProgramError>;
pub type TestResult<T> = std::result::Result<T, TestError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestError {
    pub info: String,
    pub index: Option<u32>,
}

impl TestError {
    pub fn from_unknown(message: impl ToString) -> Self {
        Self {
            info: message.to_string(),
            index: None,
        }
    }

    pub fn from_raw_error(error: impl Debug) -> Self {
        Self {
            info: format!("{:?}", error),
            index: None,
        }
    }

    /// Parses custom program error from a vector of log strings
    /// Returns the error code as u32 if found, otherwise returns None
    pub fn parse_custom_program_error(logs: &[String]) -> Option<u32> {
        const ERROR_PREFIX: &str = "failed: custom program error: ";

        for log in logs {
            if let Some(error_start) = log.find(ERROR_PREFIX) {
                let error_part = &log[error_start + ERROR_PREFIX.len()..];

                // Find the hex value (should start with "0x")
                if error_part.starts_with("0x") {
                    let hex_str = &error_part[2..]; // Remove "0x" prefix

                    // Parse hex string to u32
                    if let Ok(error_code) = u32::from_str_radix(hex_str, 16) {
                        return Some(error_code);
                    }
                }
            }
        }

        None
    }

    pub fn parse_program_error(logs: &[String]) -> Option<&str> {
        const ERROR_PREFIX: &str = "failed: ";

        for log in logs {
            if let Some(error_start) = log.find(ERROR_PREFIX) {
                let error_part = &log[error_start + ERROR_PREFIX.len()..];

                return Some(error_part);
            }
        }

        None
    }
}

pub fn sol_to_pin_pubkey(sol_pubkey: &solana_pubkey::Pubkey) -> pinocchio::pubkey::Pubkey {
    pinocchio::pubkey::Pubkey::from(sol_pubkey.to_bytes())
}

pub fn pin_to_sol_pubkey(pin_pubkey: &pinocchio::pubkey::Pubkey) -> solana_pubkey::Pubkey {
    solana_pubkey::Pubkey::new_from_array(*pin_pubkey)
}

pub fn pin_pubkey_to_addr(pubkey: &pinocchio::pubkey::Pubkey) -> solana_address::Address {
    solana_address::Address::new_from_array(*pubkey)
}

pub fn addr_to_sol_pubkey(addr: &solana_address::Address) -> solana_pubkey::Pubkey {
    addr.to_bytes().into()
}

pub fn to_c_option<T>(data: Option<T>) -> COption<T> {
    match data {
        Some(x) => COption::Some(x),
        None => COption::None,
    }
}
