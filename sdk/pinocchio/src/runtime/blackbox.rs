use super::Runtime;

pub struct BlackBoxRuntime;

impl Runtime for BlackBoxRuntime {
    fn sol_log(message: &str) {
        core::hint::black_box(message);
    }

    fn sol_log_64(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) {
        core::hint::black_box((arg1, arg2, arg3, arg4, arg5));
    }

    fn sol_log_data(data: &[&[u8]]) {
        core::hint::black_box(data);
    }

    fn sol_log_compute_units() {
        core::hint::black_box(());
    }

    unsafe fn sol_memcpy(dst: &mut [u8], src: &[u8], n: usize) {
        core::hint::black_box((dst, src, n));
    }

    unsafe fn sol_memmove(dst: *mut u8, src: *mut u8, n: usize) {
        core::hint::black_box((dst, src, n));
    }

    unsafe fn sol_memcmp(s1: &[u8], s2: &[u8], n: usize) -> i32 {
        core::hint::black_box((s1, s2, n));
        0
    }

    unsafe fn sol_memset(s: &mut [u8], c: u8, n: usize) {
        core::hint::black_box((s, c, n));
    }

    fn invoke_signed<const ACCOUNTS: usize>(
        instruction: &crate::instruction::Instruction,
        account_infos: &[&crate::account_info::AccountInfo; ACCOUNTS],
        signers_seeds: &[crate::instruction::Signer],
    ) -> crate::ProgramResult {
        core::hint::black_box((instruction, account_infos, signers_seeds));
        Ok(())
    }

    unsafe fn invoke_signed_access_unchecked<const ACCOUNTS: usize>(
        instruction: &crate::instruction::Instruction,
        account_infos: &[&crate::account_info::AccountInfo; ACCOUNTS],
        signers_seeds: &[crate::instruction::Signer],
    ) -> crate::ProgramResult {
        core::hint::black_box((instruction, account_infos, signers_seeds));
        Ok(())
    }

    fn sol_create_program_address(
        seeds: &[&[u8]],
        program_id: &crate::pubkey::Pubkey,
    ) -> Result<crate::pubkey::Pubkey, crate::program_error::ProgramError> {
        core::hint::black_box((seeds, program_id));
        panic!("create_program_address is only available on target `solana`")
    }
}
