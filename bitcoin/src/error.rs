#![allow(dead_code)]
use std::fmt;

/// Custom error types for the Bitcoin wallet CLI
#[derive(Debug)]
pub enum BitcoinWalletError {
    /// Configuration file related errors
    ConfigFile(ConfigFileError),
    /// Wallet operation errors
    Wallet(WalletError),
    /// Transaction related errors
    Transaction(TransactionError),
    /// Blockchain/RPC related errors
    Blockchain(BlockchainError),
    /// IO related errors
    Io(std::io::Error),
    /// Generic error with message
    Generic(String),
}

#[derive(Debug)]
pub enum ConfigFileError {
    NotFound(String),
    InvalidFormat(String),
    InvalidMnemonic(String),
    InvalidDerivationPath(String),
}

#[derive(Debug)]
pub enum WalletError {
    CreationFailed(String),
    SyncFailed(String),
    BalanceRetrievalFailed(String),
    AddresGenerationFailed(String),
    KeyDerivationFailed(String),
}

#[derive(Debug)]
pub enum TransactionError {
    BuildFailed(String),
    SigningFailed(String),
    BroadcastFailed(String),
    InsufficientFunds(String),
    InvalidAddress(String),
    InvalidAmount(String),
}

#[derive(Debug)]
pub enum BlockchainError {
    ConnectionFailed(String),
    RpcError(String),
    NetworkError(String),
}

impl fmt::Display for BitcoinWalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitcoinWalletError::ConfigFile(err) => write!(f, "Configuration error: {}", err),
            BitcoinWalletError::Wallet(err) => write!(f, "Wallet error: {}", err),
            BitcoinWalletError::Transaction(err) => write!(f, "Transaction error: {}", err),
            BitcoinWalletError::Blockchain(err) => write!(f, "Blockchain error: {}", err),
            BitcoinWalletError::Io(err) => write!(f, "IO error: {}", err),
            BitcoinWalletError::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl fmt::Display for ConfigFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigFileError::NotFound(path) => write!(f, "Configuration file not found: {}", path),
            ConfigFileError::InvalidFormat(msg) => write!(f, "Invalid configuration format: {}", msg),
            ConfigFileError::InvalidMnemonic(msg) => write!(f, "Invalid mnemonic: {}", msg),
            ConfigFileError::InvalidDerivationPath(msg) => write!(f, "Invalid derivation path: {}", msg),
        }
    }
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletError::CreationFailed(msg) => write!(f, "Wallet creation failed: {}", msg),
            WalletError::SyncFailed(msg) => write!(f, "Wallet sync failed: {}", msg),
            WalletError::BalanceRetrievalFailed(msg) => write!(f, "Balance retrieval failed: {}", msg),
            WalletError::AddresGenerationFailed(msg) => write!(f, "Address generation failed: {}", msg),
            WalletError::KeyDerivationFailed(msg) => write!(f, "Key derivation failed: {}", msg),
        }
    }
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::BuildFailed(msg) => write!(f, "Transaction build failed: {}", msg),
            TransactionError::SigningFailed(msg) => write!(f, "Transaction signing failed: {}", msg),
            TransactionError::BroadcastFailed(msg) => write!(f, "Transaction broadcast failed: {}", msg),
            TransactionError::InsufficientFunds(msg) => write!(f, "Insufficient funds: {}", msg),
            TransactionError::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
            TransactionError::InvalidAmount(msg) => write!(f, "Invalid amount: {}", msg),
        }
    }
}

impl fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockchainError::ConnectionFailed(msg) => write!(f, "Blockchain connection failed: {}", msg),
            BlockchainError::RpcError(msg) => write!(f, "RPC error: {}", msg),
            BlockchainError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for BitcoinWalletError {}
impl std::error::Error for ConfigFileError {}
impl std::error::Error for WalletError {}
impl std::error::Error for TransactionError {}
impl std::error::Error for BlockchainError {}

impl From<std::io::Error> for BitcoinWalletError {
    fn from(err: std::io::Error) -> Self {
        BitcoinWalletError::Io(err)
    }
}

impl From<ConfigFileError> for BitcoinWalletError {
    fn from(err: ConfigFileError) -> Self {
        BitcoinWalletError::ConfigFile(err)
    }
}

impl From<WalletError> for BitcoinWalletError {
    fn from(err: WalletError) -> Self {
        BitcoinWalletError::Wallet(err)
    }
}

impl From<TransactionError> for BitcoinWalletError {
    fn from(err: TransactionError) -> Self {
        BitcoinWalletError::Transaction(err)
    }
}

impl From<BlockchainError> for BitcoinWalletError {
    fn from(err: BlockchainError) -> Self {
        BitcoinWalletError::Blockchain(err)
    }
}

/// Type alias for Results using our custom error type
pub type Result<T> = std::result::Result<T, BitcoinWalletError>;