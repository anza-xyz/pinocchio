use crate::{
    create_instruction_struct, GROUPED_CIPHERTEXT_2_HANDLES_VALIDITY_PROOF_FULL_LEN,
    GROUPED_CIPHERTEXT_3_HANDLES_VALIDITY_PROOF_FULL_LEN,
};

create_instruction_struct!(
    DOC_MAIN = "Verify a grouped-ciphertext with 2 handles validity proof.",
    DOC_AUX = "A grouped-ciphertext validity proof certifies that a grouped ElGamal ciphertext is \
               well-defined, i.e. the ciphertext can be decrypted by private keys associated with \
               its decryption handles.",
    INSTRUCTION_NAME = VerifyGroupedCiphertext2HandlesValidity,
    DISCRIMINATOR = 9,
    PROOF_LEN = GROUPED_CIPHERTEXT_2_HANDLES_VALIDITY_PROOF_FULL_LEN
);

create_instruction_struct!(
    DOC_MAIN = "Verify a grouped-ciphertext with 3 handles validity proof.",
    DOC_AUX = "A grouped-ciphertext validity proof certifies that a grouped ElGamal ciphertext is \
               well-defined, i.e. the ciphertext can be decrypted by private keys associated with \
               its decryption handles.",
    INSTRUCTION_NAME = VerifyGroupedCiphertext3HandlesValidity,
    DISCRIMINATOR = 11,
    PROOF_LEN = GROUPED_CIPHERTEXT_3_HANDLES_VALIDITY_PROOF_FULL_LEN
);
