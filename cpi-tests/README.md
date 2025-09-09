# Pinocchio Interface Test Setup for SPL Token-2022

A comprehensive testing framework for validating the Pinocchio interface implementation against the SPL Token-2022 program, enabling developers to contribute confidently to the Pinocchio interface repository.

## Problem Statement

Before this test setup, the Pinocchio interface lacked comprehensive testing coverage, which meant:

- **Late Error Discovery**: Interface implementation errors were only discovered when used in downstream projects
- **No Validation Framework**: Contributors had no way to verify their interface implementations worked correctly
- **Manual Testing Burden**: Developers had to manually construct instructions and test scenarios
- **Inconsistent Testing**: No standardized approach for testing Cross-Program Invocations (CPI) and state reading functionality

This test setup solves these problems by providing a complete, automated testing environment that validates both the interface implementation and its real-world usage patterns.

## Architecture Overview

![Architecture Diagram](./diagrams/pinocchio_cpi_test_setup_architecture.drawio.svg)

The test setup consists of four main components:

### 1. Token-2022 Program
- **Purpose**: The actual SPL Token-2022 program implementation using Pinocchio interface
- **Key Features**: 
  - Implements token operations via Pinocchio interface
  - Handles state validation and initialization checks
  - Serves as the target program for CPI calls

### 2. Token-2022-Proxy Program
- **Purpose**: Demonstrates real-world usage of Pinocchio interface for CPI calls
- **Function**: Acts as an intermediary that executes Token-2022 operations through CPI
- **Why Important**: Simulates how other programs would integrate with Pinocchio interface in production

### 3. Proxy Handlers (Test Functions)
- **Naming Convention**: Functions prefixed with `token_2022_proxy_*`
- **Purpose**: Execute Token-2022-proxy program instructions using SPL Token-2022 interface
- **Benefits**:
  - Simplified test creation without manual instruction building
  - Consistent testing patterns across different instructions
  - Easy state reading using Pinocchio interface

### 4. SPL Handlers (Test Functions)
- **Naming Convention**: Functions prefixed with `token_2022_*` (without proxy)
- **Purpose**: Direct execution of Token-2022 program instructions using native SPL interface
- **Use Case**: Setup and prerequisite operations for testing specific instructions
- **Example**: When testing `mint_to`, you first need `initialize_mint` - SPL handlers make this setup trivial

## Code Structure

### Trait Organization
Each Token-2022 instruction has its own trait containing both proxy and SPL handlers:

```rust
pub trait Token2022InitializeMintExtension {
    // Account management
    fn token_2022_try_create_mint_account(...) -> TestResult<(TransactionMetadata, Keypair)>;
    
    // Direct SPL execution
    fn token_2022_try_initialize_mint(...) -> TestResult<TransactionMetadata>;
    
    // Proxy execution (tests Pinocchio interface)
    fn token_2022_proxy_try_initialize_mint(...) -> TestResult<TransactionMetadata>;
    
    // State reading via SPL interface
    fn token_2022_query_mint_state(...) -> TestResult<spl_token_2022_interface::state::Mint>;
    
    // State reading via Pinocchio interface  
    fn token_2022_proxy_query_mint_state(...) -> TestResult<spl_token_2022_interface::state::Mint>;
}
```

### Test Categories

**1. Direct SPL Tests**
```rust
#[test]
fn initialize_mint() -> TestResult<()> {
    // Tests direct Token-2022 program execution
    app.token_2022_try_initialize_mint(...)?;
    assert_eq!(app.token_2022_query_mint_state(mint)?, expected_state);
}
```

**2. Proxy/CPI Tests**
```rust
#[test] 
fn proxy_initialize_mint() -> TestResult<()> {
    // Tests Pinocchio interface through proxy program
    app.token_2022_proxy_try_initialize_mint(...)?;
    assert_eq!(app.token_2022_proxy_query_mint_state(mint)?, expected_state);
}
```

## Getting Started

### Prerequisites
- Rust toolchain with Solana program development setup
- LiteSVM testing environment

### Building and Testing

```bash
# Build and test with detailed output
./test.sh s

# Build and test (standard output)  
./test.sh
```

### Adding New Instruction Tests

1. **Create a new trait** for your instruction:
```rust
pub trait Token2022YourInstructionExtension {
    fn token_2022_try_your_instruction(...) -> TestResult<TransactionMetadata>;
    fn token_2022_proxy_try_your_instruction(...) -> TestResult<TransactionMetadata>;
    fn token_2022_query_your_state(...) -> TestResult<YourState>;
    fn token_2022_proxy_query_your_state(...) -> TestResult<YourState>;
}
```

2. **Implement the trait** following the established patterns:
   - SPL handlers for direct program interaction
   - Proxy handlers for CPI testing
   - State query methods for both interfaces

3. **Create test file** with both direct and proxy tests:
   - Test the instruction execution
   - Verify state changes
   - Test error conditions
   - Validate CPI behavior

## Testing Strategy

### What Gets Tested

**Interface Correctness**
- CPI calls work as expected
- State reading returns correct data
- Error handling behaves properly

**Implementation Validation**
- Pinocchio interface matches SPL interface behavior
- State structures are correctly mapped
- Account validation logic is consistent

**Integration Testing**
- Proxy programs can successfully call Token-2022 via Pinocchio
- Cross-program invocations maintain state consistency
- Complex instruction sequences work correctly

### Test Patterns

1. **Setup Phase**: Use SPL handlers to create necessary accounts and initial state
2. **Execution Phase**: Test both direct and proxy instruction execution
3. **Validation Phase**: Compare state using both SPL and Pinocchio interfaces
4. **Edge Cases**: Test error conditions and boundary cases

## Contributing

When contributing to the Pinocchio interface:

1. **Add Tests First**: Create comprehensive tests for any new interface implementations
2. **Follow Patterns**: Use the established trait and testing patterns
3. **Test Both Paths**: Always test both direct SPL and proxy execution
4. **Validate State**: Ensure state reading works correctly through Pinocchio interface
5. **Document Edge Cases**: Add tests for error conditions and special cases

## Benefits for Contributors

- **Confidence**: Know your interface implementation works before integration
- **Fast Feedback**: Catch errors immediately during development
- **Standardized Testing**: Consistent patterns across all instructions
- **Real-world Validation**: Test actual CPI usage scenarios
- **Easy Debugging**: Clear test structure makes issues easy to identify and fix

## Future Extensions

This framework can be extended to test:
- Additional SPL Token-2022 instructions
- Token-2022 extensions
- Complex multi-instruction scenarios
- Performance characteristics
- Error recovery patterns

By using this test setup, contributors can ensure their Pinocchio interface implementations are robust, correct, and ready for production use.