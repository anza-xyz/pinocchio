//! `StateWithExtensions` and `StateWithExtensionsMut` of
//! `pinocchio-token-2022` against those of `spl-token-2022-interface`.
//!
//! Mints and token accounts with extensions are built by the reference crate
//! the way the Token-2022 program builds them, then read by Pinocchio through
//! each of its constructors and compared with what the reference reads.

use {
    bytemuck::bytes_of_mut,
    core::{mem::size_of, slice},
    pinocchio_spl_reference_tests::TestAccount,
    pinocchio_token_2022::{
        state::{
            try_calculate_account_len, CpiGuardExtension, DefaultAccountStateExtension,
            ExtensionBaseState, ExtensionType, ExtensionValue, GroupMemberPointerExtension,
            GroupPointerExtension, ImmutableOwnerExtension, MemoTransferExtension,
            MetadataPointerExtension, MintCloseAuthorityExtension, NonTransferableAccountExtension,
            NonTransferableExtension, PausableAccountExtension, PausableExtension,
            PermanentDelegateExtension, PermissionedBurnExtension, StateWithExtensions,
            StateWithExtensionsMut, TransferFeeAmountExtension, TransferHookAccountExtension,
            TransferHookExtension, MAX_EXTENSIONS,
        },
        ID,
    },
    solana_address::Address,
    solana_nullable::MaybeNull,
    solana_program_error::ProgramError,
    solana_program_pack::Pack,
    spl_token_2022_interface::{
        error::TokenError,
        extension::{
            confidential_transfer::{ConfidentialTransferAccount, ConfidentialTransferMint},
            confidential_transfer_fee::ConfidentialTransferFeeAmount,
            cpi_guard::CpiGuard,
            default_account_state::DefaultAccountState,
            group_member_pointer::GroupMemberPointer,
            group_pointer::GroupPointer,
            immutable_owner::ImmutableOwner,
            interest_bearing_mint::InterestBearingConfig,
            memo_transfer::MemoTransfer,
            metadata_pointer::MetadataPointer,
            mint_close_authority::MintCloseAuthority,
            non_transferable::{NonTransferable, NonTransferableAccount},
            pausable::{PausableAccount, PausableConfig},
            permanent_delegate::PermanentDelegate,
            permissioned_burn::PermissionedBurnConfig,
            scaled_ui_amount::ScaledUiAmountConfig,
            transfer_fee::{TransferFeeAmount, TransferFeeConfig},
            transfer_hook::{TransferHook, TransferHookAccount},
            BaseState, BaseStateWithExtensions, BaseStateWithExtensionsMut, Extension,
            ExtensionType as RefType, StateWithExtensions as RefState,
            StateWithExtensionsMut as RefStateMut,
        },
        state::{Account, AccountState, Mint, Multisig},
    },
};

/// Index of the account type byte; extension entries start right after it.
const ACCOUNT_TYPE_INDEX: usize = 165;
const TLV_START: usize = ACCOUNT_TYPE_INDEX + 1;

const INVALID: ProgramError = ProgramError::InvalidAccountData;
const NOT_FOUND: ProgramError = ProgramError::Custom(TokenError::ExtensionNotFound as u32);

type Lookups = Vec<(u16, Result<Vec<u8>, ProgramError>)>;

/// Pairs each Pinocchio extension type with the reference type, and says how
/// to copy a reference value into Pinocchio's field by field.
macro_rules! extensions {
    ($($pin:ident = $reference:ident, |$p:ident, $r:ident| $copy:block)*) => {
        fn pinocchio_lookups(state: &mut impl PinocchioState) -> Lookups {
            vec![$(($pin::TYPE as u16, state.lookup::<$pin>()),)*]
        }

        fn reference_lookups<S: BaseState>(state: &impl BaseStateWithExtensions<S>) -> Lookups {
            vec![$((
                u16::from($reference::TYPE),
                state
                    .get_extension::<$reference>()
                    .map(|value| bytemuck::bytes_of(value).to_vec()),
            ),)*]
        }

        /// Copies every extension present in `from` into `to` through
        /// `get_extension_mut`.
        fn copy_extensions<S: BaseState, B: ExtensionBaseState>(
            from: &impl BaseStateWithExtensions<S>,
            to: &mut StateWithExtensionsMut<B>,
        ) {
            $(
                if let Ok($r) = from.get_extension::<$reference>() {
                    let $p = to.get_extension_mut::<$pin>().unwrap();
                    $copy
                }
            )*
        }
    };
}

extensions! {
    MintCloseAuthorityExtension = MintCloseAuthority, |p, r| {
        p.close_authority = r.close_authority
    }
    PermanentDelegateExtension = PermanentDelegate, |p, r| { p.delegate = r.delegate }
    PermissionedBurnExtension = PermissionedBurnConfig, |p, r| { p.authority = r.authority }
    DefaultAccountStateExtension = DefaultAccountState, |p, r| {
        p.set_state(r.state.try_into().unwrap())
    }
    NonTransferableExtension = NonTransferable, |_p, _r| {}
    PausableExtension = PausableConfig, |p, r| {
        p.authority = r.authority;
        p.paused = r.paused;
    }
    TransferHookExtension = TransferHook, |p, r| {
        p.authority = r.authority;
        p.program_id = r.program_id;
    }
    MetadataPointerExtension = MetadataPointer, |p, r| {
        p.authority = r.authority;
        p.metadata_address = r.metadata_address;
    }
    GroupPointerExtension = GroupPointer, |p, r| {
        p.authority = r.authority;
        p.group_address = r.group_address;
    }
    GroupMemberPointerExtension = GroupMemberPointer, |p, r| {
        p.authority = r.authority;
        p.member_address = r.member_address;
    }
    ImmutableOwnerExtension = ImmutableOwner, |_p, _r| {}
    NonTransferableAccountExtension = NonTransferableAccount, |_p, _r| {}
    PausableAccountExtension = PausableAccount, |_p, _r| {}
    TransferFeeAmountExtension = TransferFeeAmount, |p, r| { p.withheld_amount = r.withheld_amount }
    MemoTransferExtension = MemoTransfer, |p, r| {
        p.require_incoming_transfer_memos = r.require_incoming_transfer_memos
    }
    CpiGuardExtension = CpiGuard, |p, r| { p.lock_cpi = r.lock_cpi }
    TransferHookAccountExtension = TransferHookAccount, |p, r| { p.transferring = r.transferring }
}

/// A reference base state and its Pinocchio counterpart.
trait Kind: BaseState + Pack + Copy {
    type Pin: ExtensionBaseState;

    /// Extension types Pinocchio has a value type for.
    const SUPPORTED: &'static [RefType];

    /// Extension types it has none for, which it must still step over and
    /// list.
    const UNSUPPORTED: &'static [RefType];

    /// Further combinations worth building.
    const COMBINATIONS: &'static [&'static [RefType]];

    fn sample(seed: u8) -> Self;
}

impl Kind for Mint {
    type Pin = pinocchio_token_2022::state::Mint;

    const SUPPORTED: &'static [RefType] = &[
        RefType::MintCloseAuthority,
        RefType::PermanentDelegate,
        RefType::PermissionedBurn,
        RefType::DefaultAccountState,
        RefType::NonTransferable,
        RefType::Pausable,
        RefType::TransferHook,
        RefType::MetadataPointer,
        RefType::GroupPointer,
        RefType::GroupMemberPointer,
    ];

    const UNSUPPORTED: &'static [RefType] = &[
        RefType::TransferFeeConfig,
        RefType::InterestBearingConfig,
        RefType::ScaledUiAmount,
        RefType::ConfidentialTransferMint,
    ];

    // These add up to the multisig length, so the reference pads the mint
    // with two more bytes.
    const COMBINATIONS: &'static [&'static [RefType]] = &[&[
        RefType::ConfidentialTransferMint,
        RefType::ScaledUiAmount,
        RefType::InterestBearingConfig,
        RefType::NonTransferable,
    ]];

    fn sample(seed: u8) -> Self {
        Mint {
            mint_authority: (seed != 0).then_some([0x11; 32].into()).into(),
            supply: [0, u64::MAX, 0x0102_0304_0506_0708][usize::from(seed)],
            decimals: 3 * seed,
            is_initialized: true,
            freeze_authority: (seed == 1).then_some([0x22; 32].into()).into(),
        }
    }
}

impl Kind for Account {
    type Pin = pinocchio_token_2022::state::Account;

    const SUPPORTED: &'static [RefType] = &[
        RefType::ImmutableOwner,
        RefType::NonTransferableAccount,
        RefType::PausableAccount,
        RefType::TransferFeeAmount,
        RefType::MemoTransfer,
        RefType::CpiGuard,
        RefType::TransferHookAccount,
    ];

    const UNSUPPORTED: &'static [RefType] = &[
        RefType::ConfidentialTransferAccount,
        RefType::ConfidentialTransferFeeAmount,
    ];

    const COMBINATIONS: &'static [&'static [RefType]] = &[];

    fn sample(seed: u8) -> Self {
        Account {
            mint: [0x30 + seed; 32].into(),
            owner: [0x40 + seed; 32].into(),
            amount: [u64::MAX, 0, 1][usize::from(seed)],
            delegate: (seed != 0).then_some([0x50; 32].into()).into(),
            state: [AccountState::Initialized, AccountState::Frozen][usize::from(seed % 2)],
            is_native: (seed == 1).then_some(2_039_280).into(),
            delegated_amount: [1, 0x0102_0304_0506_0708, 0][usize::from(seed)],
            close_authority: (seed == 2).then_some([0x60; 32].into()).into(),
        }
    }
}

/// Initializes extension `ty` with values that depend on `seed`, and returns
/// the size of its value. Seed 0 leaves every field empty, 1 sets every
/// field, and 2 mixes the two.
fn init_extension<S: BaseState>(state: &mut RefStateMut<S>, ty: RefType, seed: u8) -> usize {
    let key = |slot: u8| -> MaybeNull<Address> {
        let present = seed == 1 || (seed == 2 && slot == 1);
        let key = Address::new_from_array([0x80 + 0x10 * seed + slot; 32]);
        present.then_some(key).try_into().unwrap()
    };
    let flag = seed == 1;

    macro_rules! init {
        ($T:ty) => {{
            state.init_extension::<$T>(true).unwrap();
            size_of::<$T>()
        }};
        ($T:ty, |$value:ident| $set:expr) => {{
            let $value = state.init_extension::<$T>(true).unwrap();
            $set;
            size_of::<$T>()
        }};
    }
    // Pinocchio cannot decode these, so their contents are arbitrary.
    macro_rules! opaque {
        ($T:ty) => {
            init!($T, |value| bytes_of_mut(value).fill(seed))
        };
    }

    match ty {
        RefType::MintCloseAuthority => init!(MintCloseAuthority, |e| e.close_authority = key(0)),
        RefType::PermanentDelegate => init!(PermanentDelegate, |e| e.delegate = key(0)),
        RefType::PermissionedBurn => init!(PermissionedBurnConfig, |e| e.authority = key(0)),
        RefType::DefaultAccountState => init!(DefaultAccountState, |e| e.state = seed),
        RefType::NonTransferable => init!(NonTransferable),
        RefType::Pausable => init!(PausableConfig, |e| {
            e.authority = key(0);
            e.paused = flag.into();
        }),
        RefType::TransferHook => init!(TransferHook, |e| {
            e.authority = key(0);
            e.program_id = key(1);
        }),
        RefType::MetadataPointer => init!(MetadataPointer, |e| {
            e.authority = key(0);
            e.metadata_address = key(1);
        }),
        RefType::GroupPointer => init!(GroupPointer, |e| {
            e.authority = key(0);
            e.group_address = key(1);
        }),
        RefType::GroupMemberPointer => init!(GroupMemberPointer, |e| {
            e.authority = key(0);
            e.member_address = key(1);
        }),
        RefType::ImmutableOwner => init!(ImmutableOwner),
        RefType::NonTransferableAccount => init!(NonTransferableAccount),
        RefType::PausableAccount => init!(PausableAccount),
        RefType::TransferFeeAmount => init!(TransferFeeAmount, |e| {
            e.withheld_amount = [0, u64::MAX, 0x0102_0304_0506_0708][usize::from(seed)].into()
        }),
        RefType::MemoTransfer => init!(MemoTransfer, |e| {
            e.require_incoming_transfer_memos = flag.into()
        }),
        RefType::CpiGuard => init!(CpiGuard, |e| e.lock_cpi = flag.into()),
        RefType::TransferHookAccount => {
            init!(TransferHookAccount, |e| e.transferring = flag.into())
        }
        RefType::TransferFeeConfig => opaque!(TransferFeeConfig),
        RefType::InterestBearingConfig => opaque!(InterestBearingConfig),
        RefType::ScaledUiAmount => opaque!(ScaledUiAmountConfig),
        RefType::ConfidentialTransferMint => opaque!(ConfidentialTransferMint),
        RefType::ConfidentialTransferAccount => opaque!(ConfidentialTransferAccount),
        RefType::ConfidentialTransferFeeAmount => opaque!(ConfidentialTransferFeeAmount),
        _ => unreachable!("{ty:?} is not built by these tests"),
    }
}

/// Builds a mint or token account with the reference crate the way the
/// Token-2022 program does, with `unused` zero bytes after the extensions.
/// Without `unused` the account has no extension area at all.
///
/// Also returns whether the extension area is exactly full.
fn build<K: Kind>(types: &[RefType], unused: Option<usize>, seed: u8) -> (Vec<u8>, bool) {
    let len = match unused {
        None => K::LEN,
        Some(unused) if types.is_empty() => TLV_START + unused,
        Some(unused) => RefType::try_calculate_account_len::<K>(types).unwrap() + unused,
    };
    let mut data = vec![0; len];
    let mut state = RefStateMut::<K>::unpack_uninitialized(&mut data).unwrap();
    let mut end = TLV_START;
    for &ty in types {
        end += 4 + init_extension(&mut state, ty, seed);
    }
    state.base = K::sample(seed);
    state.pack_base();
    if unused.is_some() {
        state.init_account_type().unwrap();
    }
    (data, unused.is_none() || end == len)
}

/// Everything Pinocchio reads from an account.
#[derive(Debug, PartialEq)]
struct Reading {
    base: Vec<u8>,
    types: Result<Vec<u16>, ProgramError>,
    lookups: Lookups,
}

/// The two Pinocchio states, so that the same reads can be made on both.
trait PinocchioState {
    fn base(&self) -> Vec<u8>;
    fn types(&self) -> Result<Vec<u16>, ProgramError>;
    fn lookup<V: ExtensionValue>(&mut self) -> Result<Vec<u8>, ProgramError>;
}

fn bytes<T>(value: &T) -> Vec<u8> {
    // SAFETY: only used on base states and extension values, which are plain
    // bytes without padding.
    unsafe { slice::from_raw_parts(value as *const T as *const u8, size_of::<T>()) }.to_vec()
}

fn types(
    written: Result<usize, ProgramError>,
    out: &[ExtensionType],
) -> Result<Vec<u16>, ProgramError> {
    written.map(|count| out[..count].iter().map(|&ty| ty as u16).collect())
}

impl<B: ExtensionBaseState> PinocchioState for &StateWithExtensions<B> {
    fn base(&self) -> Vec<u8> {
        bytes(&self.base)
    }

    fn types(&self) -> Result<Vec<u16>, ProgramError> {
        let mut out = [ExtensionType::Uninitialized; MAX_EXTENSIONS];
        types(self.write_extension_types(&mut out), &out)
    }

    fn lookup<V: ExtensionValue>(&mut self) -> Result<Vec<u8>, ProgramError> {
        self.get_extension::<V>().map(bytes)
    }
}

impl<B: ExtensionBaseState> PinocchioState for &mut StateWithExtensionsMut<B> {
    fn base(&self) -> Vec<u8> {
        bytes(&self.base)
    }

    fn types(&self) -> Result<Vec<u16>, ProgramError> {
        let mut out = [ExtensionType::Uninitialized; MAX_EXTENSIONS];
        types(self.write_extension_types(&mut out), &out)
    }

    /// Looks the extension up with both `get_extension` and
    /// `get_extension_mut`, which must find the same value.
    fn lookup<V: ExtensionValue>(&mut self) -> Result<Vec<u8>, ProgramError> {
        let found = |value: &V| (value as *const V as usize, bytes(value));
        let shared = self.get_extension::<V>().map(found);
        let unique = self.get_extension_mut::<V>().map(|value| found(value));
        assert_eq!(shared, unique, "get_extension_mut::<{:?}>", V::TYPE);
        shared.map(|(_, value)| value)
    }
}

fn read(mut state: impl PinocchioState) -> Reading {
    Reading {
        base: state.base(),
        types: state.types(),
        lookups: pinocchio_lookups(&mut state),
    }
}

/// Reads `data` through every constructor, which must all agree.
fn read_every_way<B: ExtensionBaseState>(data: &[u8]) -> Result<Reading, ProgramError> {
    let account = TestAccount::new(ID, data);
    let mut view = account.view();
    let mut copy = data.to_vec();

    let readings = [
        StateWithExtensions::<B>::from_bytes(data).map(read),
        StateWithExtensions::<B>::from_account_view(&view).map(|state| read(&*state)),
        unsafe { StateWithExtensions::<B>::from_account_view_unchecked(&view) }.map(read),
        StateWithExtensionsMut::<B>::from_bytes_mut(&mut copy).map(read),
        StateWithExtensionsMut::<B>::from_account_view_mut(&mut view)
            .map(|mut state| read(&mut *state)),
        unsafe { StateWithExtensionsMut::<B>::from_account_view_unchecked_mut(&mut view) }
            .map(read),
    ];
    for reading in &readings[1..] {
        assert_eq!(reading, &readings[0]);
    }
    assert_eq!(copy, data);
    assert_eq!(account.data(), data);

    let [first, ..] = readings;
    first
}

fn reference_reading<K: Kind>(data: &[u8]) -> Result<Reading, ProgramError> {
    let state = RefState::<K>::unpack(data)?;
    let mut base = vec![0; K::LEN];
    K::pack(state.base, &mut base)?;
    Ok(Reading {
        base,
        types: state
            .get_extension_types()
            .map(|types| types.into_iter().map(u16::from).collect()),
        lookups: reference_lookups(&state),
    })
}

/// The accounts compared: no extension area; then each extension alone,
/// all those Pinocchio supports, all of them in both orders, and any further
/// combinations, each followed by several lengths of unused space.
fn corpus<K: Kind>() -> Vec<(Vec<RefType>, Option<usize>)> {
    let all = [K::SUPPORTED, K::UNSUPPORTED].concat();
    let mut lists = vec![vec![], K::SUPPORTED.to_vec(), all.clone()];
    lists.push(all.iter().rev().copied().collect());
    lists.extend(all.iter().map(|&ty| vec![ty]));
    lists.extend(K::COMBINATIONS.iter().map(|types| types.to_vec()));

    let mut corpus = vec![(vec![], None)];
    for types in lists {
        for unused in [0, 1, 5] {
            corpus.push((types.clone(), Some(unused)));
        }
    }
    corpus
}

fn reads_match_the_reference<K: Kind>() {
    for (types, unused) in corpus::<K>() {
        for seed in 0..3 {
            let (data, full) = build::<K>(&types, unused, seed);
            let mut expected = reference_reading::<K>(&data).unwrap();

            for (ty, lookup) in &mut expected.lookups {
                let ty = RefType::try_from(*ty).unwrap();
                assert_eq!(lookup.is_ok(), types.contains(&ty), "{ty:?} in {types:?}");

                // When the extension area is full or missing, the reference
                // reports an absent extension as `InvalidAccountData`, because
                // its lookup shares `get_extension_indices` with initialization,
                // which needs space. Pinocchio reports `ExtensionNotFound`.
                if full && ty.get_account_type() == K::ACCOUNT_TYPE && !types.contains(&ty) {
                    assert_eq!(*lookup, Err(INVALID));
                    *lookup = Err(NOT_FOUND);
                }
            }

            assert_eq!(
                read_every_way::<K::Pin>(&data),
                Ok(expected),
                "{types:?} with {unused:?} unused bytes, seed {seed}",
            );
        }
    }
}

#[test]
fn mint_reads_match_the_reference() {
    reads_match_the_reference::<Mint>();
}

#[test]
fn token_account_reads_match_the_reference() {
    reads_match_the_reference::<Account>();
}

/// Copying the values of one account into another through
/// `get_extension_mut` must write the bytes the reference writes.
fn writes_match_the_reference<K: Kind>() {
    let lists = [K::SUPPORTED.to_vec()]
        .into_iter()
        .chain(K::SUPPORTED.iter().map(|&ty| vec![ty]));

    for types in lists {
        for seed in 0..3 {
            let (data, _) = build::<K>(&types, Some(0), seed);
            let (target, _) = build::<K>(&types, Some(0), (seed + 1) % 3);
            let source = RefState::<K>::unpack(&target).unwrap();

            let mut expected = target.clone();
            expected[..K::LEN].copy_from_slice(&data[..K::LEN]);

            let mut copy = data.clone();
            copy_extensions(
                &source,
                StateWithExtensionsMut::<K::Pin>::from_bytes_mut(&mut copy).unwrap(),
            );
            assert_eq!(copy, expected, "{types:?}, seed {seed}");

            let account = TestAccount::new(ID, &data);
            let mut view = account.view();
            copy_extensions(
                &source,
                &mut StateWithExtensionsMut::<K::Pin>::from_account_view_mut(&mut view).unwrap(),
            );
            assert_eq!(account.data(), expected, "{types:?}, seed {seed}");

            let account = TestAccount::new(ID, &data);
            let mut view = account.view();
            copy_extensions(&source, unsafe {
                StateWithExtensionsMut::<K::Pin>::from_account_view_unchecked_mut(&mut view)
                    .unwrap()
            });
            assert_eq!(account.data(), expected, "{types:?}, seed {seed}");
        }
    }
}

#[test]
fn mint_writes_match_the_reference() {
    writes_match_the_reference::<Mint>();
}

#[test]
fn token_account_writes_match_the_reference() {
    writes_match_the_reference::<Account>();
}

fn other_owner_is_refused<K: Kind>() {
    let (data, _) = build::<K>(K::SUPPORTED, Some(0), 1);
    let account = TestAccount::new(pinocchio_token::ID, &data);
    let mut view = account.view();

    let refused = Err(ProgramError::IncorrectProgramId);
    assert_eq!(
        StateWithExtensions::<K::Pin>::from_account_view(&view).map(drop),
        refused
    );
    assert_eq!(
        unsafe { StateWithExtensions::<K::Pin>::from_account_view_unchecked(&view) }.map(drop),
        refused
    );
    assert_eq!(
        StateWithExtensionsMut::<K::Pin>::from_account_view_mut(&mut view).map(drop),
        refused
    );
    assert_eq!(
        unsafe { StateWithExtensionsMut::<K::Pin>::from_account_view_unchecked_mut(&mut view) }
            .map(drop),
        refused
    );
}

#[test]
fn other_owner_is_refused_for_mints_and_token_accounts() {
    other_owner_is_refused::<Mint>();
    other_owner_is_refused::<Account>();
}

/// Like the base readers, the extension states accept an uninitialized base,
/// which the reference refuses.
fn uninitialized_base_is_readable<K: Kind>() {
    for len in [K::LEN, TLV_START] {
        let mut data = vec![0; len];
        if len == TLV_START {
            data[ACCOUNT_TYPE_INDEX] = K::ACCOUNT_TYPE.into();
        }
        assert!(RefState::<K>::unpack(&data).is_err());
        assert_eq!(
            read_every_way::<K::Pin>(&data).unwrap().base,
            vec![0; K::LEN]
        );
    }
}

#[test]
fn uninitialized_base_is_readable_for_mints_and_token_accounts() {
    uninitialized_base_is_readable::<Mint>();
    uninitialized_base_is_readable::<Account>();
}

/// Pinocchio sizes any combination of the extension types it supports as the
/// reference does.
fn account_len_matches_the_reference<K: Kind>() {
    let pin = |types: &[RefType]| -> Vec<ExtensionType> {
        types
            .iter()
            .map(|&ty| ExtensionType::try_from(u16::from(ty)).unwrap())
            .collect()
    };

    for subset in 0..1u32 << K::SUPPORTED.len() {
        let mut types: Vec<RefType> = (K::SUPPORTED.iter().enumerate())
            .filter(|(i, _)| subset & (1 << i) != 0)
            .map(|(_, &ty)| ty)
            .collect();
        let expected = RefType::try_calculate_account_len::<K>(&types);
        assert!(expected.is_ok());
        assert_eq!(try_calculate_account_len::<K::Pin>(&pin(&types)), expected);
        types.reverse();
        assert_eq!(try_calculate_account_len::<K::Pin>(&pin(&types)), expected);
    }

    // Every list of one or two extension types of either kind, including
    // duplicates and types Pinocchio cannot size: Pinocchio refuses more
    // lists than the reference, but any length it returns must match.
    let every_type: Vec<RefType> = (0..=28).map(|ty| RefType::try_from(ty).unwrap()).collect();
    for &first in &every_type {
        let pairs = every_type.iter().map(|&second| vec![first, second]);
        for types in pairs.chain([vec![first]]) {
            if let Ok(len) = try_calculate_account_len::<K::Pin>(&pin(&types)) {
                assert_eq!(
                    Ok(len),
                    RefType::try_calculate_account_len::<K>(&types),
                    "{types:?}"
                );
            }
        }
    }
}

#[test]
fn mint_len_matches_the_reference() {
    account_len_matches_the_reference::<Mint>();
}

#[test]
fn token_account_len_matches_the_reference() {
    account_len_matches_the_reference::<Account>();
}

// Malformed extension data, which no token program writes.

fn entry(ty: u16, len: u16, value: &[u8]) -> Vec<u8> {
    [&ty.to_le_bytes()[..], &len.to_le_bytes(), value].concat()
}

fn whole(ty: RefType, value: &[u8]) -> Vec<u8> {
    entry(ty.into(), value.len() as u16, value)
}

/// An initialized base followed by `account_type` and `entries`.
fn extended<K: Kind>(account_type: u8, entries: &[&[u8]]) -> Vec<u8> {
    let (mut data, _) = build::<K>(&[], None, 1);
    data.resize(ACCOUNT_TYPE_INDEX, 0);
    data.push(account_type);
    data.extend(entries.concat());
    data
}

/// What Pinocchio does with a malformed account.
enum Expect {
    /// Every constructor returns `InvalidAccountData`.
    Refused,
    /// The constructors succeed; looking up the given extension and listing
    /// the extension types give these results.
    Read(
        ExtensionType,
        Result<(), ProgramError>,
        Result<usize, ProgramError>,
    ),
}

/// Checks Pinocchio against `expect`, and that the reference accepts and
/// refuses the same constructions, lookups and listings, reading the same
/// values where both accept.
fn check_malformed<K: Kind>(name: &str, data: &[u8], expect: Expect) {
    let pinocchio = read_every_way::<K::Pin>(data);
    let reference = reference_reading::<K>(data);
    assert_eq!(pinocchio.is_ok(), reference.is_ok(), "{name}");

    let Expect::Read(ty, lookup, types) = expect else {
        assert_eq!(pinocchio, Err(INVALID), "{name}");
        return;
    };
    let (pinocchio, reference) = (pinocchio.unwrap(), reference.unwrap());

    let (_, found) = (pinocchio.lookups.iter())
        .find(|(found, _)| *found == ty as u16)
        .unwrap();
    assert_eq!(found.clone().map(drop), lookup, "{name}: {ty:?}");
    assert_eq!(
        pinocchio.types.clone().map(|types| types.len()),
        types,
        "{name}"
    );

    for ((ty, pinocchio), (_, reference)) in pinocchio.lookups.iter().zip(&reference.lookups) {
        assert_eq!(pinocchio.is_ok(), reference.is_ok(), "{name}: {ty}");
        if let (Ok(pinocchio), Ok(reference)) = (pinocchio, reference) {
            assert_eq!(pinocchio, reference, "{name}: {ty}");
        }
    }
    assert_eq!(pinocchio.types.is_ok(), reference.types.is_ok(), "{name}");
    if let (Ok(pinocchio), Ok(reference)) = (&pinocchio.types, &reference.types) {
        assert_eq!(pinocchio, reference, "{name}");
    }
}

#[test]
fn malformed_mints() {
    use {Expect::*, ExtensionType as T};

    let check = check_malformed::<Mint>;
    let mint = |entries: &[&[u8]]| extended::<Mint>(1, entries);
    let state = whole(RefType::DefaultAccountState, &[1]);
    let close = whole(RefType::MintCloseAuthority, &[7; 32]);

    let mut padded = mint(&[&state]);
    padded[Mint::LEN] = 1;
    check("non-zero padding", &padded, Refused);
    check(
        "token account type",
        &extended::<Mint>(2, &[&state]),
        Refused,
    );
    check(
        "uninitialized account type",
        &extended::<Mint>(0, &[&state]),
        Refused,
    );
    let mut multisig_len = mint(&[&state]);
    multisig_len.resize(Multisig::LEN, 0);
    check("multisig length", &multisig_len, Refused);
    check("shorter than the account type", &mint(&[])[..120], Refused);

    check(
        "header cut short",
        &mint(&[&state, &[3, 0]]),
        Read(T::MintCloseAuthority, Err(INVALID), Err(INVALID)),
    );
    check(
        "length past the end",
        &mint(&[&entry(3, 33, &[7; 32])]),
        Read(T::MintCloseAuthority, Err(INVALID), Err(INVALID)),
    );
    check(
        "length past the end before the target",
        &mint(&[&entry(6, 40, &[1]), &close]),
        Read(T::MintCloseAuthority, Err(INVALID), Err(INVALID)),
    );
    check(
        "value length wrong for the type",
        &mint(&[&entry(6, 2, &[1, 0])]),
        Read(T::DefaultAccountState, Err(INVALID), Ok(1)),
    );
    check(
        "unknown type before the target",
        &mint(&[&entry(u16::MAX, 2, &[9, 9]), &close]),
        Read(T::MintCloseAuthority, Ok(()), Err(INVALID)),
    );
    check(
        "token account extension in a mint",
        &mint(&[&whole(RefType::ImmutableOwner, &[]), &close]),
        Read(T::MintCloseAuthority, Ok(()), Ok(2)),
    );
    check(
        "uninitialized entry ends the list",
        &mint(&[&state, &entry(0, 0, &[]), &close]),
        Read(T::MintCloseAuthority, Err(NOT_FOUND), Ok(1)),
    );
    check(
        "duplicate entries",
        &mint(&[&state, &whole(RefType::DefaultAccountState, &[2])]),
        Read(T::DefaultAccountState, Ok(()), Ok(2)),
    );
}

#[test]
fn malformed_token_accounts() {
    use {Expect::*, ExtensionType as T};

    let check = check_malformed::<Account>;
    let account = |entries: &[&[u8]]| extended::<Account>(2, entries);
    let memo = whole(RefType::MemoTransfer, &[1]);

    check(
        "mint account type",
        &extended::<Account>(1, &[&memo]),
        Refused,
    );
    let mut multisig_len = account(&[&memo]);
    multisig_len.resize(Multisig::LEN, 0);
    check("multisig length", &multisig_len, Refused);

    check(
        "header cut short",
        &account(&[&memo, &[15]]),
        Read(T::TransferHookAccount, Err(INVALID), Ok(1)),
    );
    check(
        "length past the end",
        &account(&[&memo, &entry(15, 2, &[1])]),
        Read(T::TransferHookAccount, Err(INVALID), Err(INVALID)),
    );
    check(
        "mint extension in a token account",
        &account(&[&whole(RefType::NonTransferable, &[]), &memo]),
        Read(T::MemoTransfer, Ok(()), Ok(2)),
    );
    check(
        "uninitialized entry ends the list",
        &account(&[&entry(0, 1, &[0]), &memo]),
        Read(T::MemoTransfer, Err(NOT_FOUND), Ok(0)),
    );
}
