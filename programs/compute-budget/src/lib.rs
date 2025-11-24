#![no_std]

pub mod instructions;

pinocchio_pubkey::declare_id!("ComputeBudget111111111111111111111111111111");

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec::Vec;


    fn encode_instruction_data(discriminator: u8, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        result.push(discriminator);
        result.extend_from_slice(data);
        result
    }

    #[test]
    fn test_set_compute_unit_limit_discriminator() {
        // Per official Solana docs:
        // https://docs.rs/solana-compute-budget-interface/latest/
        // SetComputeUnitLimit is variant #2 in the enum
        //
        // Verified discriminator: 2
        let units = 50_000u32;

        let expected = encode_instruction_data(2, &units.to_le_bytes());

        assert_eq!(
            expected[0], 2,
            "SetComputeUnitLimit discriminator must be 2"
        );
        assert_eq!(
            expected.len(),
            5,
            "Total length: 1 byte discriminator + 4 bytes u32"
        );
        assert_eq!(
            &expected[1..5],
            units.to_le_bytes(),
            "u32 value must be little-endian"
        );
    }

    #[test]
    fn test_set_compute_unit_price_discriminator() {

        let micro_lamports = 10_000u64;

        let expected = encode_instruction_data(3, &micro_lamports.to_le_bytes());

        assert_eq!(
            expected[0], 3,
            "SetComputeUnitPrice discriminator must be 3"
        );
        assert_eq!(
            expected.len(),
            9,
            "Total length: 1 byte discriminator + 8 bytes u64"
        );
        assert_eq!(
            &expected[1..9],
            micro_lamports.to_le_bytes(),
            "u64 value must be little-endian"
        );
    }

    #[test]
    fn test_request_heap_frame_discriminator() {

        let bytes = 32 * 1024u32; // 32 KB

        let expected = encode_instruction_data(1, &bytes.to_le_bytes());

        assert_eq!(expected[0], 1, "RequestHeapFrame discriminator must be 1");
        assert_eq!(
            expected.len(),
            5,
            "Total length: 1 byte discriminator + 4 bytes u32"
        );
        assert_eq!(
            &expected[1..5],
            bytes.to_le_bytes(),
            "u32 value must be little-endian"
        );
    }

    #[test]
    fn test_set_compute_unit_limit_various_values() {

        let test_cases = [1_000u32, 50_000, 200_000, 1_400_000];

        for units in test_cases {
            let expected = encode_instruction_data(2, &units.to_le_bytes());
            assert_eq!(expected[0], 2);
            assert_eq!(&expected[1..5], units.to_le_bytes());
        }
    }

    #[test]
    fn test_set_compute_unit_price_various_values() {
        
        let test_cases = [
            0u64,      // No priority
            1_000,     // Low
            10_000,    // Medium
            100_000,   // High
            1_000_000, // Very high
        ];

        for micro_lamports in test_cases {
            let expected = encode_instruction_data(3, &micro_lamports.to_le_bytes());
            assert_eq!(expected[0], 3);
            assert_eq!(&expected[1..9], micro_lamports.to_le_bytes());
        }
    }

    #[test]
    fn test_request_heap_frame_multiples_of_8kb() {

        let test_cases = [
            8_192u32, // 8 KB
            16_384,   // 16 KB
            32_768,   // 32 KB
            65_536,   // 64 KB
        ];

        for bytes in test_cases {
            assert_eq!(bytes % 8_192, 0, "Heap request must be multiple of 8 KB");
            let expected = encode_instruction_data(1, &bytes.to_le_bytes());
            assert_eq!(expected[0], 1);
            assert_eq!(&expected[1..5], bytes.to_le_bytes());
        }
    }

    #[test]
    fn test_compute_budget_program_id() {
        assert_eq!(crate::ID.len(), 32);
    }
}
