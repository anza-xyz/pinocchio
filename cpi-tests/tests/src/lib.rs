#[cfg(test)]
pub mod cpi_guard;
#[cfg(test)]
pub mod default_account_state;
#[cfg(test)]
pub mod group_member_pointer;
#[cfg(test)]
pub mod group_pointer;
#[cfg(test)]
pub mod initialize_mint;
#[cfg(test)]
pub mod initialize_token_account;
#[cfg(test)]
pub mod memo_transfer;
#[cfg(test)]
pub mod metadata_pointer;
#[cfg(test)]
pub mod permanent_delegate;
#[cfg(test)]
pub mod pausable;
#[cfg(test)]
pub mod scaled_ui_amount;
#[cfg(test)]
pub mod token_group;
#[cfg(test)]
pub mod token_group_member;
#[cfg(test)]
pub mod transfer_hook;
#[cfg(test)]
pub mod interest_bearing_mint;

pub mod helpers {
    pub mod extensions {
        pub mod token_2022 {
            pub mod cpi_guard;
            pub mod default_account_state;
            pub mod group_member_pointer;
            pub mod group_pointer;
            pub mod initialize_mint;
            pub mod initialize_multisig;
            pub mod initialize_token_account;
            pub mod memo_transfer;
            pub mod metadata_pointer;
            pub mod permanent_delegate;
            pub mod pausable;
            pub mod scaled_ui_amount;
            pub mod token_account;
            pub mod token_group;
            pub mod transfer_hook;
            pub mod interest_bearing_mint;
        }
    }

    pub mod suite {
        pub mod core;
        pub mod solana_kite;
        pub mod types;
    }
}
