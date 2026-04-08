use crate::{
    create_instruction_struct, RANGE_PROOF_U128_FULL_LEN, RANGE_PROOF_U256_FULL_LEN,
    RANGE_PROOF_U64_FULL_LEN,
};

create_instruction_struct!(
    DOC_MAIN = "Verify a 64-bit batched range proof.",
    DOC_AUX = "A batched range proof is defined with respect to a sequence of Pedersen \
               commitments `[C_1, ..., C_N]` and bit-lengths `[n_1, ..., n_N]`. It certifies that \
               each commitment `C_i` is a commitment to a positive number of bit-length `n_i`. \
               Batch verifying range proofs is more efficient than verifying independent range \
               proofs on commitments `C_1, ..., C_N` separately.

The bit-length of a batched range proof specifies the sum of the individual bit-lengths `n_1, ..., \
               n_N`. For example, this instruction can be used to certify that two commitments \
               `C_1` and `C_2` each hold positive 32-bit numbers.",
    INSTRUCTION_NAME = VerifyBatchedRangeProofU64,
    DISCRIMINATOR = 6,
    PROOF_LEN = RANGE_PROOF_U64_FULL_LEN
);

create_instruction_struct!(
    DOC_MAIN = "Verify a 128-bit batched range proof.",
    DOC_AUX = "A batched range proof is defined with respect to a sequence of Pedersen \
               commitments `[C_1, ..., C_N]` and bit-lengths `[n_1, ..., n_N]`. It certifies that \
               each commitment `C_i` is a commitment to a positive number of bit-length `n_i`. \
               Batch verifying range proofs is more efficient than verifying independent range \
               proofs on commitments `C_1, ..., C_N` separately.

The bit-length of a batched range proof specifies the sum of the individual bit-lengths `n_1, ..., \
               n_N`. For example, this instruction can be used to certify that two commitments \
               `C_1` and `C_2` each hold positive 64-bit numbers.",
    INSTRUCTION_NAME = VerifyBatchedRangeProofU128,
    DISCRIMINATOR = 7,
    PROOF_LEN = RANGE_PROOF_U128_FULL_LEN
);

create_instruction_struct!(
    DOC_MAIN = "Verify a 256-bit batched range proof.",
    DOC_AUX = "A batched range proof is defined with respect to a sequence of Pedersen \
               commitments `[C_1, ..., C_N]` and bit-lengths `[n_1, ..., n_N]`. It certifies that \
               each commitment `C_i` is a commitment to a positive number of bit-length `n_i`. \
               Batch verifying range proofs is more efficient than verifying independent range \
               proofs on commitments `C_1, ..., C_N` separately.

The bit-length of a batched range proof specifies the sum of the individual bit-lengths `n_1, ..., \
               n_N`. For example, this instruction can be used to certify that four commitments \
               `[C_1, C_2, C_3, C_4]` each hold positive 64-bit numbers.",
    INSTRUCTION_NAME = VerifyBatchedRangeProofU256,
    DISCRIMINATOR = 8,
    PROOF_LEN = RANGE_PROOF_U256_FULL_LEN
);
