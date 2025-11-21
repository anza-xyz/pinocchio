#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StakeAuthorize {
    /// todo!()
    Staker,

    /// todo!()
    Withdrawer,
}

impl From<u8> for StakeAuthorize {
    fn from(value: u8) -> Self {
        match value {
            0 => StakeAuthorize::Staker,
            1 => StakeAuthorize::Withdrawer,
            _ => panic!("invalid stake authorize value: {value}"),
        }
    }
}

impl From<StakeAuthorize> for u8 {
    fn from(value: StakeAuthorize) -> Self {
        match value {
            StakeAuthorize::Staker => 0,
            StakeAuthorize::Withdrawer => 1,
        }
    }
}
