use crate::{create_instruction_struct, ZERO_CIPHERTEXT_PROOF_FULL_LEN};

create_instruction_struct!(
    DOC_MAIN = "Verify a zero-ciphertext proof.",
    DOC_AUX =
        "A zero-ciphertext proof certifies that an ElGamal ciphertext encrypts the value zero.",
    INSTRUCTION_NAME = VerifyZeroCiphertext,
    DISCRIMINATOR = 1,
    PROOF_LEN = ZERO_CIPHERTEXT_PROOF_FULL_LEN
);
