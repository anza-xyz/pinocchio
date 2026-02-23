use crate::{create_instruction_struct, PUBKEY_VALIDITY_PROOF_FULL_LEN};

create_instruction_struct!(
    DOC_MAIN = "Verify a public key validity zero-knowledge proof.",
    DOC_AUX = "A public key validity proof certifies that an ElGamal public key is well-formed \
               and the prover knows the corresponding secret key.",
    INSTRUCTION_NAME = VerifyPubkeyValidity,
    DISCRIMINATOR = 4,
    PROOF_LEN = PUBKEY_VALIDITY_PROOF_FULL_LEN
);
