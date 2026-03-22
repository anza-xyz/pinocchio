use {
    crate::{
        instructions::{Batchable, MAX_MULTISIG_SIGNERS},
        write_bytes, UNINIT_BYTE,
    },
    core::{mem::MaybeUninit, ptr, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

const APPROVE_INSTRUCTION_DATA_LEN: usize = 9;

/// Approves a delegate.  A delegate is given the authority over tokens on
/// behalf of the source account's owner.
///
/// Accounts expected by this instruction:
///
///   * Single owner
///   0. `[writable]` The source account.
///   1. `[]` The delegate.
///   2. `[signer]` The source account owner.
///
///   * Multisignature owner
///   0. `[writable]` The source account.
///   1. `[]` The delegate.
///   2. `[]` The source account's multisignature owner.
///   3. `..+M` `[signer]` M signer accounts.
pub struct Approve<'account, 'multisig, MultisigSigner: AsRef<AccountView>> {
    /// The source account.
    pub source: &'account AccountView,

    /// The delegate.
    pub delegate: &'account AccountView,

    /// The source account owner.
    pub authority: &'account AccountView,

    /// Multisignature signers.
    pub multisig_signers: &'multisig [MultisigSigner],

    /// The amount of tokens the delegate is approved for.
    pub amount: u64,
}

impl<'account> Approve<'account, '_, &'account AccountView> {
    /// Creates a new `Approve` instruction with a single owner authority.
    #[inline(always)]
    pub fn new(
        source: &'account AccountView,
        delegate: &'account AccountView,
        authority: &'account AccountView,
        amount: u64,
    ) -> Self {
        Self::with_multisig_signers(source, delegate, authority, amount, &[])
    }
}

impl<'account, 'multisig, MultisigSigner: AsRef<AccountView>>
    Approve<'account, 'multisig, MultisigSigner>
{
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 4;

    /// Creates a new `Approve` instruction with a
    /// multisignature owner authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        source: &'account AccountView,
        delegate: &'account AccountView,
        authority: &'account AccountView,
        amount: u64,
        multisig_signers: &'multisig [MultisigSigner],
    ) -> Self {
        Self {
            source,
            delegate,
            authority,
            multisig_signers,
            amount,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            Err(ProgramError::InvalidArgument)?;
        }

        // Instruction accounts.

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; 3 + MAX_MULTISIG_SIGNERS];

        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts);

        // Accounts.

        let mut accounts =
            [const { MaybeUninit::<CpiAccount>::uninit() }; 3 + MAX_MULTISIG_SIGNERS];

        let written_accounts = self.write_accounts(&mut accounts);

        // Instruction data.
        // - [0]: instruction discriminator (1 byte, u8)
        // - [1..9]: amount (8 bytes, u64)

        let mut instruction_data = [UNINIT_BYTE; APPROVE_INSTRUCTION_DATA_LEN];
        let written_instruction_data = self.write_instruction_data(&mut instruction_data);

        unsafe {
            invoke_signed_unchecked(
                &InstructionView {
                    program_id: &crate::ID,
                    accounts: from_raw_parts(
                        instruction_accounts.as_ptr() as _,
                        written_instruction_accounts,
                    ),
                    data: from_raw_parts(instruction_data.as_ptr() as _, written_instruction_data),
                },
                from_raw_parts(accounts.as_ptr() as *const CpiAccount, written_accounts),
                signers,
            );
        }

        Ok(())
    }
}

impl<MultisigSigner: AsRef<AccountView>> super::sealed::Sealed for Approve<'_, '_, MultisigSigner> {}

impl<MultisigSigner: AsRef<AccountView>> Batchable for Approve<'_, '_, MultisigSigner> {
    #[inline(always)]
    fn write_accounts(&self, accounts: &mut [MaybeUninit<CpiAccount>]) -> usize {
        let expected_accounts = 3 + self.multisig_signers.len();

        accounts[0].write(CpiAccount::from(self.source));

        accounts[1].write(CpiAccount::from(self.delegate));

        accounts[2].write(CpiAccount::from(self.authority));

        for (account, signer) in accounts[3..expected_accounts]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            account.write(CpiAccount::from(signer.as_ref()));
        }

        expected_accounts
    }

    #[inline(always)]
    fn write_instruction_accounts(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount>],
    ) -> usize {
        let expected_accounts = 3 + self.multisig_signers.len();

        // SAFETY: The written address references are borrowed from `self`, and
        // callers must not use the output buffer after `self` expires.
        unsafe {
            ptr::write(
                accounts[0].as_mut_ptr(),
                InstructionAccount::writable(&*(self.source.address() as *const _)),
            );
            ptr::write(
                accounts[1].as_mut_ptr(),
                InstructionAccount::readonly(&*(self.delegate.address() as *const _)),
            );
            ptr::write(
                accounts[2].as_mut_ptr(),
                InstructionAccount::new(
                    &*(self.authority.address() as *const _),
                    false,
                    self.multisig_signers.is_empty(),
                ),
            );
        }

        for (account, signer) in accounts[3..expected_accounts]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            // SAFETY: Same lifetime requirement as above.
            unsafe {
                ptr::write(
                    account.as_mut_ptr(),
                    InstructionAccount::readonly_signer(&*(signer.as_ref().address() as *const _)),
                );
            }
        }

        expected_accounts
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> usize {
        data[0].write(Self::DISCRIMINATOR);
        write_bytes(
            &mut data[1..APPROVE_INSTRUCTION_DATA_LEN],
            &self.amount.to_le_bytes(),
        );
        APPROVE_INSTRUCTION_DATA_LEN
    }
}
