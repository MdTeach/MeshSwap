use bdk::bitcoin::{Address, Txid};
use bdk::blockchain::{Blockchain, RpcBlockchain};
use bdk::database::MemoryDatabase;
use bdk::{FeeRate, SignOptions, SyncOptions, Wallet};
use eyre::{Result, eyre};

use crate::constants::DEFAULT_FEE_RATE_SAT_PER_VB;

/// Transaction utilities for Bitcoin operations
pub struct TransactionUtils;


impl TransactionUtils {
    /// Create and broadcast a transaction
    pub async fn create_and_broadcast(
        blockchain_client: &RpcBlockchain,
        sender_wallet: &Wallet<MemoryDatabase>,
        recipient_address: Address,
        amount_satoshis: u64,
    ) -> Result<Txid> {
        Self::create_and_broadcast_with_fee_rate(
            blockchain_client,
            sender_wallet,
            recipient_address,
            amount_satoshis,
            FeeRate::from_sat_per_vb(DEFAULT_FEE_RATE_SAT_PER_VB),
        ).await
    }

    pub async fn create_and_broadcast_with_fee_rate(
        blockchain_client: &RpcBlockchain,
        sender_wallet: &Wallet<MemoryDatabase>,
        recipient_address: Address,
        amount_satoshis: u64,
        fee_rate: FeeRate,
    ) -> Result<Txid> {
        sender_wallet.sync(blockchain_client, SyncOptions::default())?;

        let (mut partially_signed_tx, _) = {
            let mut tx_builder = sender_wallet.build_tx();
            tx_builder
                .fee_rate(fee_rate)
                .add_recipient(recipient_address.script_pubkey(), amount_satoshis);
            tx_builder.finish()?
        };

        let is_finalized = sender_wallet.sign(&mut partially_signed_tx, SignOptions::default())?;
        if !is_finalized {
            return Err(eyre!("Failed to sign and finalize transaction"));
        }

        let transaction_id = partially_signed_tx.unsigned_tx.txid();
        blockchain_client.broadcast(&partially_signed_tx.extract_tx())?;
        Ok(transaction_id)
    }
}

/// Send Bitcoin from one wallet to a specific address
pub async fn send_bitcoin_to_address(
    blockchain_client: &RpcBlockchain,
    sender_wallet: &Wallet<MemoryDatabase>,
    recipient_address: Address,
    amount_satoshis: u64,
) -> Result<Txid> {
    TransactionUtils::create_and_broadcast(
        blockchain_client,
        sender_wallet,
        recipient_address,
        amount_satoshis,
    ).await
}
