//! Defines the middleware entrypoint, enabling a hot path to bypass
//! entrypoint deserialization, ejecting to the cold path on failure.
#[macro_export]
macro_rules! middleware_program_entrypoint {
    ($hot:expr, $cold:expr) => {
        $crate::middleware_entrypoint!($hot, $cold, { $crate::MAX_TX_ACCOUNTS });
    };
    ($hot:expr, $cold:expr, $maximum:expr ) => {

        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            if $hot(input) == 0 {
                return $crate::SUCCESS
            }

            const UNINIT: core::mem::MaybeUninit<$crate::account_info::AccountInfo> = core::mem::MaybeUninit::<$crate::account_info::AccountInfo>::uninit();
            // Create an array of uninitialized account infos.
            let mut accounts = [UNINIT; $maximum];

            let (program_id, count, instruction_data) = unsafe {
                $crate::entrypoint::deserialize::<$maximum>(input, &mut accounts) }; 

            // Call the program's entrypoint passing `count` account infos; we know that
            // they are initialized so we cast the pointer to a slice of `[AccountInfo]`.
            match $cold(
                &program_id,
                unsafe { core::slice::from_raw_parts(accounts.as_ptr() as _, count) },
                &instruction_data,
            ) {
                Ok(()) => $crate::SUCCESS,
                Err(error) => error.into(),
            }
        }
    };
}