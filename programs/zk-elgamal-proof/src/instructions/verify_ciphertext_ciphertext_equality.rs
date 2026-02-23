use crate::{create_instruction_struct, CIPHERTEXT_CIPHERTEXT_EQUALITY_PROOF_FULL_LEN};

create_instruction_struct!(
    DOC_MAIN = "Verify a ciphertext-ciphertext equality proof.",
    DOC_AUX = "A ciphertext-ciphertext equality proof certifies that two ElGamal ciphertexts \
               encrypt the same message.",
    INSTRUCTION_NAME = VerifyCiphertextCiphertextEquality,
    DISCRIMINATOR = 2,
    PROOF_LEN = CIPHERTEXT_CIPHERTEXT_EQUALITY_PROOF_FULL_LEN
);
