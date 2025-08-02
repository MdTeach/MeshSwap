//! Taproot contract functionality for Hash Time Locked Contracts (HTLCs)
//!
//! This module provides functionality for creating and managing taproot-based contracts,
//! specifically Hash Time Locked Contracts for atomic swaps and payment channels.

use bdk::bitcoin::Address as BitcoinAddress;
use bdk::bitcoin::secp256k1::{self, Scalar, Secp256k1, SecretKey};
use bdk::bitcoin::{Network, PrivateKey, Txid};
use bdk::blockchain::{Blockchain, RpcBlockchain};
use bdk::database::MemoryDatabase;
use bdk::descriptor::IntoWalletDescriptor;
use bdk::miniscript::Descriptor;
use bdk::miniscript::descriptor::TapTree;
use bdk::miniscript::policy::Concrete;
use bdk::wallet::AddressIndex;
use bdk::{FeeRate, KeychainKind, SignOptions, SyncOptions, Wallet, bitcoin};
use eyre::{Context, Result, eyre};
use rand::rngs::ThreadRng;
use std::collections::BTreeMap;
use std::{str::FromStr, sync::Arc};

use crate::constants::DEFAULT_FEE_RATE_SAT_PER_VB;
use crate::primitives::SwapInfo;
use crate::transaction::TransactionUtils;

/// Creates a secp256k1 context for cryptographic operations
fn create_secp_context() -> Secp256k1<secp256k1::All> {
    Secp256k1::new()
}

/// Creates a temporary wallet with MemoryDatabase for contract operations
fn create_contract_wallet(descriptor: &str, network: Network) -> Result<Wallet<MemoryDatabase>> {
    Ok(Wallet::new(
        descriptor,
        None,
        network,
        MemoryDatabase::new(),
    )?)
}

/// Syncs a wallet with the blockchain and returns the wallet reference
fn sync_wallet_with_blockchain(
    wallet: &Wallet<MemoryDatabase>,
    blockchain_client: &RpcBlockchain,
) -> Result<()> {
    wallet
        .sync(blockchain_client, SyncOptions::default())
        .wrap_err("Failed to sync wallet with blockchain")
}


/// Signs and finalizes a PSBT, returning an error if signing fails
fn sign_and_finalize_transaction(
    wallet: &Wallet<MemoryDatabase>,
    psbt: &mut bdk::bitcoin::util::psbt::PartiallySignedTransaction,
) -> Result<()> {
    let is_finalized = wallet.sign(psbt, SignOptions::default())?;

    if !is_finalized {
        return Err(eyre!("Failed to sign and finalize transaction"));
    }

    Ok(())
}

/// Broadcasts a transaction and returns the transaction ID
fn broadcast_transaction(
    blockchain_client: &RpcBlockchain,
    signed_transaction: bdk::bitcoin::Transaction,
) -> Result<Txid> {
    blockchain_client.broadcast(&signed_transaction)?;
    Ok(signed_transaction.txid())
}

/// Creates a taproot-based Hash Time Locked Contract (HTLC)
///
/// This function creates a taproot contract with two spending conditions:
/// 1. Recipient can spend with their key after a time delay
/// 2. Sender can revoke with revocation key after timeout
///
/// # Arguments
/// * `blockchain_client` - RPC client for blockchain operations
/// * `sender_wallet` - Wallet that will fund the contract
/// * `swap_info` - Complete swap information including keys, timelock, and amount
///
/// # Returns
/// Transaction ID of the funding transaction
pub async fn create_taproot_htlc_contract(
    blockchain_client: &RpcBlockchain,
    sender_wallet: &Wallet<MemoryDatabase>,
    swap_info: &SwapInfo,
) -> Result<Txid> {
    // Validate swap info before proceeding
    swap_info
        .validate()
        .map_err(|e| eyre!("Invalid swap info: {}", e))?;

    let secp_context = create_secp_context();

    // Build taproot policy: recipient can spend after timelock OR revocation key can spend
    let policy_script = format!(
        "and(older({}),pk({}))",
        swap_info.timelock_duration_blocks, swap_info.revocation_public_key
    );

    let compiled_policy = Concrete::<String>::from_str(&policy_script)?
        .compile()
        .wrap_err("Failed to compile taproot policy")?;

    let tap_tree = TapTree::Leaf(Arc::new(compiled_policy));

    // Generate taproot descriptor
    let taproot_descriptor_string =
        Descriptor::new_tr(swap_info.recipient_public_key.to_string(), Some(tap_tree))?
            .to_string()
            .into_wallet_descriptor(&secp_context, sender_wallet.network())?
            .0;

    // Create contract wallet
    let contract_wallet = create_contract_wallet(
        &taproot_descriptor_string.to_string(),
        sender_wallet.network(),
    )?;

    let contract_address = contract_wallet.get_address(AddressIndex::New)?.address;

    // Fund the contract
    let funding_transaction_id = TransactionUtils::create_and_broadcast(
        blockchain_client,
        sender_wallet,
        contract_address.clone(),
        swap_info.amount_satoshis,
    )
    .await
    .wrap_err(format!(
        "Failed to send {} satoshis to HTLC contract address {}",
        swap_info.amount_satoshis, contract_address
    ))?;

    Ok(funding_transaction_id)
}

/// Withdraws funds from a taproot-based Hash Time Locked Contract (HTLC)
///
/// This function allows the recipient to claim funds from an HTLC by providing
/// the correct secret keys and satisfying the contract conditions.
///
/// # Arguments
/// * `blockchain_client` - RPC client for blockchain operations
/// * `sender_wallet` - Original sender's wallet (used for network info)
/// * `swap_info` - Swap information including public keys, timelock, and amount
/// * `recipient_secret_key` - Secret key of the recipient
/// * `swap_secret_key` - Secret key for the atomic swap mechanism
///
/// # Returns
/// Transaction ID of the withdrawal transaction
pub async fn withdraw_from_taproot_htlc(
    blockchain_client: &RpcBlockchain,
    sender_wallet: &Wallet<MemoryDatabase>,
    destination_address: BitcoinAddress,
    swap_info: &SwapInfo,
    recipient_secret_key: &SecretKey,
    swap_secret_key: &SecretKey,
) -> Result<Txid> {
    // Validate swap info before proceeding
    swap_info
        .validate()
        .map_err(|e| eyre!("Invalid swap info: {}", e))?;

    // Combine secret keys for escrow - this gives us the private key for the taproot internal key
    let combined_secret_key = swap_secret_key
        .add_tweak(&Scalar::from_be_bytes(recipient_secret_key.secret_bytes())?)
        .map_err(|_| eyre!("Failed to combine secret keys for escrow"))?;

    let escrow_private_key = PrivateKey::new(combined_secret_key, sender_wallet.network());


    // Generate the same taproot descriptor as in create function
     let revocation_pubkey = bitcoin::PublicKey::new(swap_info.revocation_public_key);
     let taproot_descriptor = bdk::descriptor!(tr(
            escrow_private_key,
            and_v(v:pk(revocation_pubkey), older(swap_info.timelock_duration_blocks))
        ))?;
    
    // Create and sync withdrawal wallet
    let withdrawal_wallet = Wallet::new(
        taproot_descriptor,
        None,
        sender_wallet.network(),
        MemoryDatabase::new(),
    )?;

    sync_wallet_with_blockchain(&withdrawal_wallet, blockchain_client)?;

    // Debug: Check if withdrawal wallet has any UTXOs
    let withdrawal_balance = withdrawal_wallet.get_balance()?;
    println!("🔍 Withdrawal wallet balance: {} sats", withdrawal_balance.confirmed);
    
    if withdrawal_balance.confirmed == 0 {
        return Err(eyre!("Withdrawal wallet has no confirmed balance. Expected UTXO might not be found."));
    }

    // Set up policy path for script spending
    let wallet_policy = withdrawal_wallet
        .policies(KeychainKind::External)?
        .ok_or_else(|| eyre!("No spending policy found for withdrawal wallet"))?;

    println!("🔍 Available wallet policy: {:?}", wallet_policy);

    let mut spending_policy_path = BTreeMap::new();
    // Use the first item (key path spend) - recipient can spend with combined secret
    // This allows immediate claiming by the recipient who has both secrets
    spending_policy_path.insert(wallet_policy.id, vec![0]);
    
    println!("🔍 Using spending policy path: {:?}", spending_policy_path);

  

    let (mut withdrawal_psbt, _transaction_details) = {
        let mut transaction_builder = withdrawal_wallet.build_tx();

        transaction_builder
            .fee_rate(FeeRate::from_sat_per_vb(DEFAULT_FEE_RATE_SAT_PER_VB))
            .drain_wallet()
            .drain_to(destination_address.script_pubkey())
            .policy_path(spending_policy_path, KeychainKind::External);

        transaction_builder
            .finish()
            .wrap_err("Failed to build withdrawal transaction")?
    };

    // Sign and finalize the withdrawal transaction
    sign_and_finalize_transaction(&withdrawal_wallet, &mut withdrawal_psbt)?;

    // Extract and broadcast the signed transaction
    let signed_withdrawal_transaction = withdrawal_psbt.extract_tx();
    let withdrawal_transaction_id =
        broadcast_transaction(blockchain_client, signed_withdrawal_transaction)?;

    Ok(withdrawal_transaction_id)
}

/// Creates a new atomic swap using taproot-based Hash Time Locked Contract (HTLC)
///
/// This is a wrapper function around `create_taproot_htlc_contract` that provides
/// a simplified interface for creating atomic swaps.
///
/// # Arguments
/// * `blockchain_client` - RPC client for blockchain operations
/// * `sender_wallet` - Wallet that will fund the atomic swap
/// * `swap_info` - Swap information including public keys, timelock, and amount
///
/// # Returns
/// Transaction ID of the funding transaction for the atomic swap
pub async fn new_atomic_swap(
    blockchain_client: &RpcBlockchain,
    sender_wallet: &Wallet<MemoryDatabase>,
    swap_info: &mut SwapInfo,
    rng: &mut ThreadRng,
) -> Result<Txid> {
    let swap_secret = secp256k1::SecretKey::new(rng);
    println!("| Swap k secret: {}", swap_secret.display_secret());

    let secp_ctx = create_secp_context();
    let swap_pubkey = swap_secret.public_key(&secp_ctx);

    let escrow_pubkey = swap_pubkey
        .combine(&swap_info.recipient_public_key)
        .expect("It's impossible to fail for 2 different public keys");

    swap_info.recipient_public_key = escrow_pubkey;

    let txid = create_taproot_htlc_contract(blockchain_client, sender_wallet, swap_info).await?;
    Ok(txid)
}
