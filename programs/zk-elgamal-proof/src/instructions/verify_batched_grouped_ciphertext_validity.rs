use crate::{
    create_instruction_struct, BATCHED_GROUPED_CIPHERTEXT_2_HANDLES_VALIDITY_PROOF_FULL_LEN,
    BATCHED_GROUPED_CIPHERTEXT_3_HANDLES_VALIDITY_PROOF_FULL_LEN,
};

create_instruction_struct!(
    DOC_MAIN = "Verify a batched grouped-ciphertext with 2 handles validity proof.",
    DOC_AUX = "A batched grouped-ciphertext validity proof certifies the validity of two grouped \
               ElGamal ciphertext that are encrypted using the same set of ElGamal public keys. A \
               batched grouped-ciphertext validity proof is shorter and more efficient than two \
               individual grouped-ciphertext validity proofs.",
    INSTRUCTION_NAME = VerifyBatchedGroupedCiphertext2HandlesValidity,
    DISCRIMINATOR = 10,
    PROOF_LEN = BATCHED_GROUPED_CIPHERTEXT_2_HANDLES_VALIDITY_PROOF_FULL_LEN
);

create_instruction_struct!(
    DOC_MAIN = "Verify a batched grouped-ciphertext with 3 handles validity proof.",
    DOC_AUX = "A batched grouped-ciphertext validity proof certifies the validity of two grouped \
               ElGamal ciphertext that are encrypted using the same set of ElGamal public keys. A \
               batched grouped-ciphertext validity proof is shorter and more efficient than two \
               individual grouped-ciphertext validity proofs.",
    INSTRUCTION_NAME = VerifyBatchedGroupedCiphertext3HandlesValidity,
    DISCRIMINATOR = 12,
    PROOF_LEN = BATCHED_GROUPED_CIPHERTEXT_3_HANDLES_VALIDITY_PROOF_FULL_LEN
);
