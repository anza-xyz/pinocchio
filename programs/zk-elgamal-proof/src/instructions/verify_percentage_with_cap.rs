use crate::{create_instruction_struct, PERCENTAGE_WITH_CAP_PROOF_FULL_LEN};

create_instruction_struct!(
    DOC_MAIN = "Verify a percentage-with-cap proof.",
    DOC_AUX = "A percentage-with-cap proof certifies that a tuple of Pedersen commitments satisfy \
               a percentage relation.",
    INSTRUCTION_NAME = VerifyPercentageWithCap,
    DISCRIMINATOR = 5,
    PROOF_LEN = PERCENTAGE_WITH_CAP_PROOF_FULL_LEN
);
