// solana-kite code is implemented here due to version conflict with original crate

use {
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::Message,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
    spl_associated_token_account::instruction::create_associated_token_account as create_ata_instruction,
    spl_token::instruction::mint_to,
    std::{fmt, fs},
};

pub fn create_associated_token_account(
    litesvm: &mut LiteSVM,
    owner: &Pubkey,
    mint: &Pubkey,
    payer: &Keypair,
) -> Result<Pubkey, SolanaKiteError> {
    let associated_token_account =
        spl_associated_token_account::get_associated_token_address(owner, mint);

    let create_ata_instruction =
        create_ata_instruction(&payer.pubkey(), owner, mint, &spl_token::id());

    let message = Message::new(&[create_ata_instruction], Some(&payer.pubkey()));
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = litesvm.latest_blockhash();
    transaction.sign(&[payer], blockhash);

    litesvm.send_transaction(transaction).map_err(|e| {
        SolanaKiteError::TokenOperationFailed(format!(
            "Failed to create associated token account: {:?}",
            e
        ))
    })?;

    Ok(associated_token_account)
}

pub fn create_token_mint(
    litesvm: &mut LiteSVM,
    mint_authority: &Keypair,
    decimals: u8,
    mint: Option<Pubkey>,
) -> Result<Pubkey, SolanaKiteError> {
    let mint = mint.unwrap_or(Pubkey::new_unique());
    let rent = litesvm.minimum_balance_for_rent_exemption(82);

    litesvm
        .set_account(
            mint,
            solana_account::Account {
                lamports: rent,
                data: vec![0u8; 82],
                owner: spl_token::ID,
                executable: false,
                rent_epoch: 0,
            },
        )
        .map_err(|e| {
            SolanaKiteError::TokenOperationFailed(format!("Failed to create mint account: {:?}", e))
        })?;

    let initialize_mint_instruction = spl_token::instruction::initialize_mint(
        &spl_token::ID,
        &mint,
        &mint_authority.pubkey(),
        None,
        decimals,
    )
    .map_err(|e| {
        SolanaKiteError::TokenOperationFailed(format!(
            "Failed to create initialize mint instruction: {:?}",
            e
        ))
    })?;

    let message = Message::new(
        &[initialize_mint_instruction],
        Some(&mint_authority.pubkey()),
    );
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = litesvm.latest_blockhash();
    transaction.sign(&[mint_authority], blockhash);

    litesvm.send_transaction(transaction).map_err(|e| {
        SolanaKiteError::TokenOperationFailed(format!("Failed to initialize mint: {:?}", e))
    })?;

    Ok(mint)
}

/// Deploys a program to the LiteSVM test environment.
///
/// This function reads a program binary from the filesystem and deploys it to the
/// specified program ID in the LiteSVM instance. The program will be marked as executable
/// and owned by the BPF loader.
///
/// # Arguments
///
/// * `litesvm` - Mutable reference to the LiteSVM instance
/// * `program_id` - The public key where the program should be deployed
/// * `program_path` - Path to the compiled program binary (.so file)
///
/// # Returns
///
/// Returns `Ok(())` on successful deployment, or a [`SolanaKiteError`] on failure.
///
/// # Errors
///
/// This function will return an error if:
/// - The program binary file cannot be read
/// - The program deployment to LiteSVM fails
///
/// # Example
///
/// ```rust
/// use solana_kite::deploy_program;
/// use litesvm::LiteSVM;
/// use solana_pubkey::Pubkey;
///
/// let mut litesvm = LiteSVM::new();
/// let program_id = Pubkey::new_unique();
///
/// // Deploy a program (this would fail in tests without an actual .so file)
/// // deploy_program(&mut litesvm, &program_id, "./target/deploy/my_program.so")?;
/// ```
pub fn deploy_program(
    litesvm: &mut LiteSVM,
    program_id: &Pubkey,
    program_path: &str,
) -> Result<(), SolanaKiteError> {
    let program_bytes = fs::read(program_path).map_err(|e| {
        SolanaKiteError::ProgramDeploymentFailed(format!(
            "Failed to read program binary at {}: {}",
            program_path, e
        ))
    })?;

    litesvm
        .set_account(
            *program_id,
            solana_account::Account {
                lamports: litesvm.minimum_balance_for_rent_exemption(program_bytes.len()),
                data: program_bytes,
                owner: solana_program::bpf_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        )
        .map_err(|e| {
            SolanaKiteError::ProgramDeploymentFailed(format!("Failed to deploy program: {:?}", e))
        })?;

    Ok(())
}

/// Gets the token balance of a token account.
///
/// This function reads the token account data and extracts the balance from the
/// SPL token account layout.
///
/// # Arguments
///
/// * `litesvm` - Reference to the LiteSVM instance
/// * `token_account` - Public key of the token account to query
///
/// # Returns
///
/// Returns the token balance as a u64 in base units.
///
/// # Errors
///
/// This function will return an error if the token account doesn't exist or
/// the balance cannot be parsed.
///
/// # Example
///
/// ```rust
/// use solana_kite::get_token_account_balance;
/// use litesvm::LiteSVM;
/// use solana_pubkey::Pubkey;
///
/// let litesvm = LiteSVM::new();
/// let token_account = Pubkey::new_unique();
///
/// // This would fail in practice without a real token account
/// // let balance = get_token_account_balance(&litesvm, &token_account)?;
/// ```
pub fn get_token_account_balance(
    litesvm: &LiteSVM,
    token_account: &Pubkey,
) -> Result<u64, SolanaKiteError> {
    let account = litesvm.get_account(token_account).ok_or_else(|| {
        SolanaKiteError::TokenOperationFailed("Token account not found".to_string())
    })?;

    let data = &account.data;
    if data.len() < 72 {
        return Err(SolanaKiteError::TokenOperationFailed(
            "Invalid token account data length".to_string(),
        ));
    }

    // SPL Token account layout: amount is at bytes 64..72 (u64, little endian)
    let amount_bytes = &data[64..72];
    let amount = u64::from_le_bytes(amount_bytes.try_into().map_err(|_| {
        SolanaKiteError::TokenOperationFailed("Failed to parse token amount".to_string())
    })?);

    Ok(amount)
}

/// Mints tokens to a specified token account.
///
/// This function creates a mint_to instruction and sends it as a transaction.
/// The mint authority must have permission to mint tokens for the specified mint.
///
/// # Arguments
///
/// * `litesvm` - Mutable reference to the LiteSVM instance
/// * `mint` - Public key of the token mint
/// * `token_account` - Public key of the destination token account
/// * `amount` - Number of tokens to mint (in base units)
/// * `mint_authority` - Keypair with mint authority
///
/// # Errors
///
/// This function will return an error if the minting transaction fails.
///
/// # Example
///
/// ```rust
/// use solana_kite::{create_token_mint, create_associated_token_account, mint_tokens_to_account, create_wallet};
/// use litesvm::LiteSVM;
/// use solana_signer::Signer;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut litesvm = LiteSVM::new();
/// let mint_authority = create_wallet(&mut litesvm, 1_000_000_000)?;
/// let owner = create_wallet(&mut litesvm, 1_000_000_000)?;
/// let mint = create_token_mint(&mut litesvm, &mint_authority, 6)?;
/// let token_account = create_associated_token_account(&mut litesvm, &owner, &mint.pubkey(), &owner)?;
///
/// mint_tokens_to_account(
///     &mut litesvm,
///     &mint.pubkey(),
///     &token_account,
///     1_000_000, // 1 token with 6 decimals
///     &mint_authority,
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn mint_tokens_to_account(
    litesvm: &mut LiteSVM,
    mint: &Pubkey,
    token_account: &Pubkey,
    amount: u64,
    mint_authority: &Keypair,
) -> Result<(), SolanaKiteError> {
    let mint_to_instruction = mint_to(
        &spl_token::id(),
        mint,
        token_account,
        &mint_authority.pubkey(),
        &[],
        amount,
    )
    .map_err(|e| {
        SolanaKiteError::TokenOperationFailed(format!(
            "Failed to create mint_to instruction: {:?}",
            e
        ))
    })?;

    let message = Message::new(&[mint_to_instruction], Some(&mint_authority.pubkey()));
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = litesvm.latest_blockhash();
    transaction.sign(&[mint_authority], blockhash);

    litesvm.send_transaction(transaction).map_err(|e| {
        SolanaKiteError::TokenOperationFailed(format!("Failed to mint tokens: {:?}", e))
    })?;

    Ok(())
}

/// Main error type for Solana Kite operations.
#[derive(Debug)]
pub enum SolanaKiteError {
    /// Transaction failed with an error message.
    TransactionFailed(String),
    /// Program deployment failed.
    ProgramDeploymentFailed(String),
    /// Token operation failed.
    TokenOperationFailed(String),
    /// Account operation failed.
    AccountOperationFailed(String),
    /// I/O error occurred.
    IoError(std::io::Error),
}

impl fmt::Display for SolanaKiteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolanaKiteError::TransactionFailed(msg) => {
                write!(f, "Transaction failed: {}", msg)
            }
            SolanaKiteError::ProgramDeploymentFailed(msg) => {
                write!(f, "Program deployment failed: {}", msg)
            }
            SolanaKiteError::TokenOperationFailed(msg) => {
                write!(f, "Token operation failed: {}", msg)
            }
            SolanaKiteError::AccountOperationFailed(msg) => {
                write!(f, "Account operation failed: {}", msg)
            }
            SolanaKiteError::IoError(err) => {
                write!(f, "I/O error: {}", err)
            }
        }
    }
}

impl std::error::Error for SolanaKiteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SolanaKiteError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SolanaKiteError {
    fn from(err: std::io::Error) -> Self {
        SolanaKiteError::IoError(err)
    }
}

/// Legacy error type for backward compatibility.
///
/// This is kept for backward compatibility with existing code.
/// New code should use [`SolanaKiteError`] instead.
#[deprecated(since = "0.1.0", note = "Use SolanaKiteError instead")]
#[derive(Debug)]
pub enum TestError {
    /// Transaction failed with an error message.
    TransactionFailed(String),
}

#[allow(deprecated)]
impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestError::TransactionFailed(msg) => {
                write!(f, "Transaction failed: {}", msg)
            }
        }
    }
}

#[allow(deprecated)]
impl std::error::Error for TestError {}

#[allow(deprecated)]
impl From<TestError> for SolanaKiteError {
    fn from(err: TestError) -> Self {
        match err {
            TestError::TransactionFailed(msg) => SolanaKiteError::TransactionFailed(msg),
        }
    }
}
