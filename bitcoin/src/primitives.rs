#![allow(dead_code)]
//! Primitive data structures for Bitcoin operations
//!
//! This module contains core data structures used throughout the Bitcoin CLI,
//! including swap information and contract parameters.

use bdk::bitcoin::secp256k1::PublicKey;
use serde::{Deserialize, Serialize};

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