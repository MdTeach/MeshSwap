//! Taproot contract functionality for Hash Time Locked Contracts (HTLCs)
//!
//! This module provides functionality for creating and managing taproot-based contracts,
//! specifically Hash Time Locked Contracts for atomic swaps and payment channels.

use bdk::Wallet;
use bdk::bitcoin::Txid;
use bdk::bitcoin::secp256k1::{PublicKey, Secp256k1};
use bdk::blockchain::RpcBlockchain;
use bdk::database::MemoryDatabase;
use bdk::descriptor::IntoWalletDescriptor;
use bdk::miniscript::Descriptor;
use bdk::miniscript::descriptor::TapTree;
use bdk::miniscript::policy::Concrete;
use bdk::wallet::AddressIndex;
use eyre::{Context, Result};
use std::{str::FromStr, sync::Arc};

use crate::transaction::send_bitcoin_to_address;

/// Create a taproot-based Hash Time Locked Contract (HTLC)
///
/// This function creates a taproot contract with two spending conditions:
/// 1. Recipient can spend with their key after a time delay
/// 2. Sender can revoke with revocation key after timeout
pub async fn create_taproot_htlc_contract(
    blockchain_client: &RpcBlockchain,
    sender_wallet: &Wallet<MemoryDatabase>,
    recipient_public_key: &PublicKey,
    revocation_public_key: &PublicKey,
    timelock_duration_blocks: u64,
    amount_satoshis: u64,
) -> Result<Txid> {
    let secp_context = Secp256k1::new();

    // Create taproot policy: recipient can spend after timelock OR revocation key can spend
    let taproot_policy_string = format!(
        "and(older({}),pk({}))",
        timelock_duration_blocks, revocation_public_key
    );

    let taproot_policy = Concrete::<String>::from_str(&taproot_policy_string)?.compile()?;
    let tap_tree = TapTree::Leaf(Arc::new(taproot_policy));

    // Create taproot descriptor
    let taproot_descriptor = Descriptor::new_tr(recipient_public_key.to_string(), Some(tap_tree))?
        .to_string()
        .into_wallet_descriptor(&secp_context, sender_wallet.network())?
        .0;

    // Create temporary wallet for the taproot contract
    let taproot_contract_wallet = Wallet::new(
        taproot_descriptor,
        None,
        sender_wallet.network(),
        MemoryDatabase::new(),
    )?;

    let contract_address = taproot_contract_wallet
        .get_address(AddressIndex::New)?
        .address;

    // Send funds to the contract address
    let transaction_id = send_bitcoin_to_address(
        blockchain_client,
        sender_wallet,
        contract_address.clone(),
        amount_satoshis,
    )
    .await
    .wrap_err(format!(
        "Failed to send {} satoshis to taproot contract address {}",
        amount_satoshis, contract_address
    ))?;

    Ok(transaction_id)
}
