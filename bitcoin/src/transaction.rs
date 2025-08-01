use bdk::bitcoin::{Address, Txid};
use bdk::blockchain::{Blockchain, RpcBlockchain};
use bdk::database::MemoryDatabase;
use bdk::{FeeRate, SignOptions, SyncOptions, Wallet};
use eyre::{Result, eyre};

use crate::constants::DEFAULT_FEE_RATE_SAT_PER_VB;

/// Transaction builder for creating and sending Bitcoin transactions
pub struct TransactionBuilder {
    fee_rate: FeeRate,
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self {
            fee_rate: FeeRate::from_sat_per_vb(DEFAULT_FEE_RATE_SAT_PER_VB),
        }
    }
}

impl TransactionBuilder {
    /// Create a new transaction builder with default fee rate
    pub fn new() -> Self {
        Self::default()
    }

    /// Send Bitcoin from one wallet to an address
    pub async fn send_to_address(
        &self,
        blockchain_client: &RpcBlockchain,
        sender_wallet: &Wallet<MemoryDatabase>,
        recipient_address: Address,
        amount_satoshis: u64,
    ) -> Result<Txid> {
        // Sync wallet with blockchain
        sender_wallet.sync(blockchain_client, SyncOptions::default())?;

        // Build transaction
        let (mut partially_signed_tx, _transaction_details) = {
            let mut tx_builder = sender_wallet.build_tx();
            tx_builder
                .fee_rate(self.fee_rate)
                .add_recipient(recipient_address.script_pubkey(), amount_satoshis);
            tx_builder.finish()?
        };

        // Sign transaction
        let is_transaction_finalized =
            sender_wallet.sign(&mut partially_signed_tx, SignOptions::default())?;

        if !is_transaction_finalized {
            return Err(eyre!("Failed to sign and finalize transaction"));
        }

        let transaction_id = partially_signed_tx.unsigned_tx.txid();

        // Broadcast transaction
        blockchain_client.broadcast(&partially_signed_tx.extract_tx())?;

        Ok(transaction_id)
    }
}

/// Send Bitcoin from one wallet to a specific address
///
/// This is a convenience function that uses default fee rates
pub async fn send_bitcoin_to_address(
    blockchain_client: &RpcBlockchain,
    sender_wallet: &Wallet<MemoryDatabase>,
    recipient_address: Address,
    amount_satoshis: u64,
) -> Result<Txid> {
    let tx_builder = TransactionBuilder::new();
    tx_builder
        .send_to_address(
            blockchain_client,
            sender_wallet,
            recipient_address,
            amount_satoshis,
        )
        .await
}
