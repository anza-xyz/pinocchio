//! The `Mint`, `Account` and `Multisig` readers of `pinocchio-token` and
//! `pinocchio-token-2022` against the `Pack` implementations of
//! `spl-token-interface` and `spl-token-2022-interface`.
//!
//! Accounts are packed by the reference crate and read back by Pinocchio
//! through each of its constructors.

use {
    pinocchio_spl_reference_tests::TestAccount,
    solana_address::Address,
    solana_program_error::ProgramError,
    solana_program_option::COption,
    solana_program_pack::{IsInitialized, Pack},
};

const AMOUNTS: [u64; 4] = [0, 1, 0x0102_0304_0506_0708, u64::MAX];

/// A present key that is all zeroes must still read as present: only the
/// option tag decides.
const KEYS: [Option<[u8; 32]>; 3] = [None, Some([0xa7; 32]), Some([0; 32])];

/// Offsets of single-byte fields and of the first byte of four-byte option
/// tags in the SPL layouts.
const MINT_AUTHORITY_TAG: usize = 0;
const MINT_IS_INITIALIZED: usize = 45;
const FREEZE_AUTHORITY_TAG: usize = 46;
const DELEGATE_TAG: usize = 72;
const ACCOUNT_STATE: usize = 108;
const IS_NATIVE_TAG: usize = 109;
const CLOSE_AUTHORITY_TAG: usize = 129;
const MULTISIG_IS_INITIALIZED: usize = 2;

fn coption<K: From<[u8; 32]>>(key: Option<[u8; 32]>) -> COption<K> {
    key.map(K::from).into()
}

fn bytes<K: AsRef<[u8]>>(key: Option<&K>) -> Option<[u8; 32]> {
    key.map(|key| key.as_ref().try_into().unwrap())
}

fn packed<T: Pack>(value: T) -> Vec<u8> {
    let mut data = vec![0; T::LEN];
    T::pack(value, &mut data).unwrap();
    data
}

fn with_byte(mut data: Vec<u8>, offset: usize, byte: u8) -> Vec<u8> {
    data[offset] = byte;
    data
}

fn token_state(account: &pinocchio_token::state::Account) -> u8 {
    account.state().unwrap() as u8
}

fn token_2022_state(account: &pinocchio_token_2022::state::Account) -> u8 {
    account.state().unwrap() as u8
}

macro_rules! base_state_tests {
    (
        $module:ident,
        pinocchio: $pinocchio:ident,
        other_program: $other:ident,
        reference: $reference:ident,
        len: $LEN:ident,
        state: $state:path $(,)?
    ) => {
        mod $module {
            use {
                super::*,
                $pinocchio::state as pin,
                $reference::state::{self as reference, AccountState},
            };

            /// Runs `check` on what each constructor reads from `data`.
            macro_rules! read {
                ($Type:ident, $data:expr, $check:expr) => {{
                    let data: &[u8] = $data;
                    let account = TestAccount::new($pinocchio::ID, data);
                    let view = account.view();
                    $check(&*pin::$Type::from_account_view(&view).unwrap());
                    $check(unsafe { pin::$Type::from_account_view_unchecked(&view) }.unwrap());
                    $check(unsafe { pin::$Type::from_bytes_unchecked(data) });
                }};
            }

            fn sample_mint() -> Vec<u8> {
                packed(reference::Mint {
                    mint_authority: coption(Some([1; 32])),
                    supply: 7,
                    decimals: 6,
                    is_initialized: true,
                    freeze_authority: coption(Some([2; 32])),
                })
            }

            fn sample_account() -> Vec<u8> {
                packed(reference::Account {
                    mint: [3; 32].into(),
                    owner: [4; 32].into(),
                    amount: 8,
                    delegate: coption(Some([5; 32])),
                    state: AccountState::Initialized,
                    is_native: COption::Some(9),
                    delegated_amount: 10,
                    close_authority: coption(Some([6; 32])),
                })
            }

            #[test]
            fn lengths() {
                assert_eq!(pin::Mint::$LEN, reference::Mint::LEN);
                assert_eq!(pin::Account::$LEN, reference::Account::LEN);
                assert_eq!(pin::Multisig::LEN, reference::Multisig::LEN);
            }

            fn check_mint(mint: &pin::Mint, packed: &reference::Mint) {
                let mint_authority = bytes(Option::from(packed.mint_authority.as_ref()));
                let freeze_authority = bytes(Option::from(packed.freeze_authority.as_ref()));

                assert_eq!(mint.has_mint_authority(), mint_authority.is_some());
                assert_eq!(bytes(mint.mint_authority()), mint_authority);
                if let Some(key) = mint_authority {
                    assert_eq!(mint.mint_authority_unchecked().to_bytes(), key);
                }
                assert_eq!(mint.supply(), packed.supply);
                assert_eq!(mint.decimals(), packed.decimals);
                assert_eq!(mint.is_initialized(), packed.is_initialized);
                assert_eq!(mint.has_freeze_authority(), freeze_authority.is_some());
                assert_eq!(bytes(mint.freeze_authority()), freeze_authority);
                if let Some(key) = freeze_authority {
                    assert_eq!(mint.freeze_authority_unchecked().to_bytes(), key);
                }
            }

            #[test]
            fn mint() {
                for mint_authority in KEYS {
                    for freeze_authority in KEYS {
                        for (i, supply) in AMOUNTS.into_iter().enumerate() {
                            for is_initialized in [true, false] {
                                let mint = reference::Mint {
                                    mint_authority: coption(mint_authority),
                                    supply,
                                    decimals: [0, 6, 9, u8::MAX][i],
                                    is_initialized,
                                    freeze_authority: coption(freeze_authority),
                                };
                                read!(Mint, &packed(mint), |read| check_mint(read, &mint));
                            }
                        }
                    }
                }
            }

            fn check_account(account: &pin::Account, packed: &reference::Account) {
                let delegate = bytes(Option::from(packed.delegate.as_ref()));
                let native_amount = Option::<u64>::from(packed.is_native);
                let close_authority = bytes(Option::from(packed.close_authority.as_ref()));

                assert_eq!(account.mint().to_bytes(), packed.mint.to_bytes());
                assert_eq!(account.owner().to_bytes(), packed.owner.to_bytes());
                assert_eq!(account.amount(), packed.amount);
                assert_eq!(account.has_delegate(), delegate.is_some());
                assert_eq!(bytes(account.delegate()), delegate);
                if let Some(key) = delegate {
                    assert_eq!(account.delegate_unchecked().to_bytes(), key);
                }
                assert_eq!($state(account), packed.state as u8);
                assert_eq!(account.is_initialized(), packed.is_initialized());
                assert_eq!(account.is_frozen(), packed.is_frozen());
                assert_eq!(account.is_native(), packed.is_native());
                assert_eq!(account.native_amount(), native_amount);
                if let Some(amount) = native_amount {
                    assert_eq!(account.native_amount_unchecked(), amount);
                }
                assert_eq!(account.delegated_amount(), packed.delegated_amount);
                assert_eq!(account.has_close_authority(), close_authority.is_some());
                assert_eq!(bytes(account.close_authority()), close_authority);
                if let Some(key) = close_authority {
                    assert_eq!(account.close_authority_unchecked().to_bytes(), key);
                }
            }

            #[test]
            fn account() {
                let states = [
                    AccountState::Uninitialized,
                    AccountState::Initialized,
                    AccountState::Frozen,
                ];

                // The three amounts are rotated through `AMOUNTS` so that
                // each takes every value and no two are ever equal.
                for delegate in KEYS {
                    for close_authority in KEYS {
                        for state in states {
                            for native in [false, true] {
                                for i in 0..AMOUNTS.len() {
                                    let amount = |offset: usize| AMOUNTS[(i + offset) % 4];
                                    let account = reference::Account {
                                        mint: [0x11 + i as u8; 32].into(),
                                        owner: [0x21 + i as u8; 32].into(),
                                        amount: amount(0),
                                        delegate: coption(delegate),
                                        state,
                                        is_native: native.then(|| amount(1)).into(),
                                        delegated_amount: amount(2),
                                        close_authority: coption(close_authority),
                                    };
                                    read!(Account, &packed(account), |read| check_account(
                                        read, &account
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            fn check_multisig(multisig: &pin::Multisig, packed: &reference::Multisig) {
                let n = usize::from(packed.n);
                let signers: Vec<_> = multisig.signers().iter().map(Address::to_bytes).collect();
                let expected: Vec<[u8; 32]> = packed.signers[..n]
                    .iter()
                    .map(|signer| signer.to_bytes())
                    .collect();

                assert_eq!(multisig.required_signers(), packed.m);
                assert_eq!(multisig.signers_len(), n);
                assert_eq!(signers, expected);
                assert_eq!(multisig.is_initialized(), packed.is_initialized);
            }

            #[test]
            fn multisig() {
                // Neither side compares `m` with `n` when reading.
                for m in 1..=11 {
                    for n in 1..=11 {
                        for is_initialized in [true, false] {
                            let multisig = reference::Multisig {
                                m,
                                n,
                                is_initialized,
                                signers: core::array::from_fn(|i| [0x40 + i as u8; 32].into()),
                            };
                            read!(Multisig, &packed(multisig), |read| check_multisig(
                                read, &multisig
                            ));
                        }
                    }
                }
            }

            /// Pinocchio leaves the initialization check to `is_initialized`,
            /// where the reference's `unpack` refuses the account.
            #[test]
            fn uninitialized_accounts_are_readable() {
                assert!(reference::Mint::unpack(&[0; 82]).is_err());
                assert!(reference::Account::unpack(&[0; 165]).is_err());
                assert!(reference::Multisig::unpack(&[0; 355]).is_err());

                read!(Mint, &[0; 82], |mint: &pin::Mint| {
                    assert!(!mint.is_initialized())
                });
                read!(Account, &[0; 165], |account: &pin::Account| {
                    assert!(!account.is_initialized())
                });
                read!(Multisig, &[0; 355], |multisig: &pin::Multisig| {
                    assert!(!multisig.is_initialized())
                });
            }

            #[test]
            fn owner_and_length_are_checked() {
                macro_rules! check {
                    ($Type:ident, $len:expr) => {
                        for (owner, len, error) in [
                            ($other::ID, $len, ProgramError::IncorrectProgramId),
                            ($pinocchio::ID, 0, ProgramError::InvalidAccountData),
                            ($pinocchio::ID, $len - 1, ProgramError::InvalidAccountData),
                            ($pinocchio::ID, $len + 1, ProgramError::InvalidAccountData),
                        ] {
                            let account = TestAccount::new(owner, &vec![0; len]);
                            let view = account.view();
                            let checked = pin::$Type::from_account_view(&view).map(drop);
                            assert_eq!(checked, Err(error.clone()));
                            let unchecked =
                                unsafe { pin::$Type::from_account_view_unchecked(&view) };
                            assert_eq!(unchecked.map(drop), Err(error));
                        }
                    };
                }
                check!(Mint, 82);
                check!(Account, 165);
                check!(Multisig, 355);
            }

            /// The reference refuses option tags other than `[0, 0, 0, 0]`
            /// and `[1, 0, 0, 0]`; Pinocchio only reads the first byte.
            #[test]
            fn option_tags() {
                let tags = [
                    [0, 0, 0, 0],
                    [1, 0, 0, 0],
                    [2, 0, 0, 0],
                    [0, 1, 1, 1],
                    [1, 1, 1, 1],
                ];
                for tag in tags {
                    let present = tag[0] == 1;
                    let with_tag = |mut data: Vec<u8>, offset: usize| {
                        data[offset..offset + 4].copy_from_slice(&tag);
                        data
                    };

                    let data = with_tag(sample_mint(), MINT_AUTHORITY_TAG);
                    read!(Mint, &data, |mint: &pin::Mint| {
                        assert_eq!(mint.has_mint_authority(), present);
                        assert_eq!(mint.mint_authority().is_some(), present);
                    });
                    let data = with_tag(sample_mint(), FREEZE_AUTHORITY_TAG);
                    read!(Mint, &data, |mint: &pin::Mint| {
                        assert_eq!(mint.has_freeze_authority(), present);
                        assert_eq!(mint.freeze_authority().is_some(), present);
                    });
                    let data = with_tag(sample_account(), DELEGATE_TAG);
                    read!(Account, &data, |account: &pin::Account| {
                        assert_eq!(account.has_delegate(), present);
                        assert_eq!(account.delegate().is_some(), present);
                    });
                    let data = with_tag(sample_account(), IS_NATIVE_TAG);
                    read!(Account, &data, |account: &pin::Account| {
                        assert_eq!(account.is_native(), present);
                        assert_eq!(account.native_amount().is_some(), present);
                    });
                    let data = with_tag(sample_account(), CLOSE_AUTHORITY_TAG);
                    read!(Account, &data, |account: &pin::Account| {
                        assert_eq!(account.has_close_authority(), present);
                        assert_eq!(account.close_authority().is_some(), present);
                    });
                }
            }

            /// The reference refuses any `is_initialized` byte other than `0`
            /// or `1`, and any account state byte above `2`. Pinocchio reads
            /// every value as below; these are pinned so that a change is
            /// deliberate.
            #[test]
            fn flag_bytes() {
                for byte in 0..=u8::MAX {
                    let data = with_byte(sample_mint(), MINT_IS_INITIALIZED, byte);
                    read!(Mint, &data, |mint: &pin::Mint| {
                        assert_eq!(mint.is_initialized(), byte == 1)
                    });

                    let data = with_byte(sample_account(), ACCOUNT_STATE, byte);
                    read!(Account, &data, |account: &pin::Account| {
                        assert_eq!(account.is_initialized(), byte != 0);
                        assert_eq!(account.is_frozen(), byte == 2);
                        if byte <= 2 {
                            assert_eq!($state(account), byte);
                        }
                    });

                    let multisig = packed(reference::Multisig {
                        m: 1,
                        n: 1,
                        is_initialized: true,
                        signers: Default::default(),
                    });
                    let data = with_byte(multisig, MULTISIG_IS_INITIALIZED, byte);
                    read!(Multisig, &data, |multisig: &pin::Multisig| {
                        assert_eq!(multisig.is_initialized(), byte != 0)
                    });
                }
            }
        }
    };
}

base_state_tests!(
    token,
    pinocchio: pinocchio_token,
    other_program: pinocchio_token_2022,
    reference: spl_token_interface,
    len: LEN,
    state: token_state,
);

base_state_tests!(
    token_2022,
    pinocchio: pinocchio_token_2022,
    other_program: pinocchio_token,
    reference: spl_token_2022_interface,
    len: BASE_LEN,
    state: token_2022_state,
);

/// Both crates return `InvalidAccountData` for an account state byte the
/// reference refuses, rather than reading it as some state.
#[test]
fn account_state_above_frozen_is_an_error() {
    let mut data = vec![0; pinocchio_token::state::Account::LEN];
    for byte in 0..=u8::MAX {
        data[ACCOUNT_STATE] = byte;
        let reference = spl_token_interface::state::Account::unpack_unchecked(&data).map(drop);
        let reference_2022 =
            spl_token_2022_interface::state::Account::unpack_unchecked(&data).map(drop);
        assert_eq!(reference.is_ok(), byte <= 2);
        assert_eq!(reference_2022.is_ok(), byte <= 2);

        let expected = (byte <= 2)
            .then_some(byte)
            .ok_or(ProgramError::InvalidAccountData);
        let token = unsafe { pinocchio_token::state::Account::from_bytes_unchecked(&data) };
        assert_eq!(token.state().map(|state| state as u8), expected);
        let token_2022 =
            unsafe { pinocchio_token_2022::state::Account::from_bytes_unchecked(&data) };
        assert_eq!(token_2022.state().map(|state| state as u8), expected);
    }
}

/// Token-2022 also accepts a mint or account longer than its base when the
/// account type byte after the base account length matches, except at the
/// multisig length.
#[test]
fn token_2022_extended_lengths() {
    use pinocchio_token_2022::{state as pin, ID};

    const MINT: u8 = 1;
    const ACCOUNT: u8 = 2;

    for (len, account_type, mint_ok, account_ok) in [
        (166, MINT, true, false),
        (166, ACCOUNT, false, true),
        (166, 0, false, false),
        (300, MINT, true, false),
        (300, ACCOUNT, false, true),
        (355, MINT, false, false),
        (355, ACCOUNT, false, false),
    ] {
        let mut data = vec![0; len];
        data[165] = account_type;
        let account = TestAccount::new(ID, &data);
        let view = account.view();

        let expect = |ok: bool| ok.then_some(()).ok_or(ProgramError::InvalidAccountData);
        assert_eq!(
            pin::Mint::from_account_view(&view).map(drop),
            expect(mint_ok)
        );
        assert_eq!(
            unsafe { pin::Mint::from_account_view_unchecked(&view) }.map(drop),
            expect(mint_ok)
        );
        assert_eq!(
            pin::Account::from_account_view(&view).map(drop),
            expect(account_ok)
        );
        assert_eq!(
            unsafe { pin::Account::from_account_view_unchecked(&view) }.map(drop),
            expect(account_ok)
        );
    }
}
