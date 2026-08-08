use crate::{create_instruction_struct, CIPHERTEXT_COMMITMENT_EQUALITY_PROOF_FULL_LEN};

create_instruction_struct!(
    DOC_MAIN = "Verify a ciphertext-commitment equality proof.",
    DOC_AUX = "A ciphertext-commitment equality proof certifies that an ElGamal ciphertext and a \
               Pedersen commitment encrypt/encode the same message.",
    INSTRUCTION_NAME = VerifyCiphertextCommitmentEquality,
    DISCRIMINATOR = 3,
    PROOF_LEN = CIPHERTEXT_COMMITMENT_EQUALITY_PROOF_FULL_LEN
);
