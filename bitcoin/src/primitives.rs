#![allow(dead_code)]
//! Primitive data structures for Bitcoin operations
//!
//! This module contains core data structures used throughout the Bitcoin CLI,
//! including swap information and contract parameters.

use bdk::bitcoin::secp256k1::{PublicKey, SecretKey};
use bdk::bitcoin::{Address as BitcoinAddress, Txid};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Information required for atomic swap operations
/// 
/// This struct encapsulates the public parameters needed to create and manage
/// Hash Time Locked Contracts (HTLCs) for atomic swaps between parties.
/// Private keys are handled separately for security reasons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapInfo {
    /// Public key of the recipient who will receive the funds
    pub recipient_public_key: PublicKey,
    
    /// Public key that can revoke/refund the contract after timeout
    pub revocation_public_key: PublicKey,
    
    /// Number of blocks before the timelock expires
    pub timelock_duration_blocks: u32,
    
    /// Amount to be locked in the contract (in satoshis)
    pub amount_satoshis: u64,
}

impl SwapInfo {
    /// Creates a new SwapInfo instance
    /// 
    /// # Arguments
    /// * `recipient_public_key` - Public key of the recipient
    /// * `revocation_public_key` - Public key for contract revocation
    /// * `timelock_duration_blocks` - Timelock duration in blocks
    /// * `amount_satoshis` - Amount in satoshis
    pub fn new(
        recipient_public_key: PublicKey,
        revocation_public_key: PublicKey,
        timelock_duration_blocks: u32,
        amount_satoshis: u64,
    ) -> Self {
        Self {
            recipient_public_key,
            revocation_public_key,
            timelock_duration_blocks,
            amount_satoshis,
        }
    }
    
    /// Validates that the swap info contains valid parameters
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.amount_satoshis == 0 {
            return Err("Amount must be greater than zero");
        }
        
        if self.timelock_duration_blocks == 0 {
            return Err("Timelock duration must be greater than zero");
        }
        
        Ok(())
    }
    
    /// Returns the amount in BTC format
    pub fn amount_btc(&self) -> f64 {
        self.amount_satoshis as f64 / 100_000_000.0
    }
}

/// Complete record of an atomic swap including all persistent data
/// 
/// This struct wraps SwapInfo and includes additional data that needs to be
/// persisted to JSON for swap tracking and recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRecord {
    /// Core swap information
    pub swap_info: SwapInfo,
    
    /// Secret key used in the atomic swap mechanism (hex encoded)
    pub swap_secret: String,
    
    /// Taproot descriptor string for the contract
    pub descriptor_string: String,
    
    /// HTLC contract address
    pub contract_address: String,
    
    /// Transaction ID of the funding transaction
    pub funding_txid: String,
    
    /// Unix timestamp when the swap was created
    pub creation_timestamp: u64,
}

impl SwapRecord {
    /// Creates a new SwapRecord
    pub fn new(
        swap_info: SwapInfo,
        swap_secret: &SecretKey,
        descriptor_string: String,
        contract_address: BitcoinAddress,
        funding_txid: Txid,
    ) -> Self {
        let swap_secret_hex = hex::encode(swap_secret.secret_bytes());
        let creation_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            swap_info,
            swap_secret: swap_secret_hex,
            descriptor_string,
            contract_address: contract_address.to_string(),
            funding_txid: funding_txid.to_string(),
            creation_timestamp,
        }
    }
    
    /// Validates the swap record
    pub fn validate(&self) -> Result<(), &'static str> {
        self.swap_info.validate()?;
        
        if self.swap_secret.is_empty() {
            return Err("Swap secret cannot be empty");
        }
        
        if self.descriptor_string.is_empty() {
            return Err("Descriptor string cannot be empty");
        }
        
        if self.contract_address.is_empty() {
            return Err("Contract address cannot be empty");
        }
        
        if self.funding_txid.is_empty() {
            return Err("Funding transaction ID cannot be empty");
        }
        
        Ok(())
    }
    
    /// Saves the swap record to JSON file
    pub fn save_to_json(&self, file_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        self.validate().map_err(|e| format!("Validation failed: {}", e))?;
        
        let json_string = serde_json::to_string_pretty(self)?;
        std::fs::write(file_path, json_string)?;
        
        Ok(())
    }
    
    /// Loads a swap record from JSON file
    pub fn load_from_json(file_path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json_string = std::fs::read_to_string(file_path)?;
        let swap_record: SwapRecord = serde_json::from_str(&json_string)?;
        
        swap_record.validate().map_err(|e| format!("Validation failed: {}", e))?;
        
        Ok(swap_record)
    }
}