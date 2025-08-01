//! Taproot contract functionality for Hash Time Locked Contracts (HTLCs)
//!
//! This module provides functionality for creating and managing taproot-based contracts,
//! specifically Hash Time Locked Contracts for atomic swaps and payment channels.

use bdk::bitcoin::Address as BitcoinAddress;
use bdk::bitcoin::secp256k1::{self, PublicKey, Scalar, Secp256k1, SecretKey};
use bdk::bitcoin::{PrivateKey, Txid};
use bdk::blockchain::{Blockchain, RpcBlockchain};
use bdk::database::MemoryDatabase;
use bdk::descriptor::IntoWalletDescriptor;
use bdk::miniscript::Descriptor;
use bdk::miniscript::descriptor::TapTree;
use bdk::miniscript::policy::Concrete;
use bdk::wallet::AddressIndex;
use bdk::{FeeRate, KeychainKind, SignOptions, SyncOptions, Wallet, bitcoin};
use eyre::{Context, Result, eyre};
use std::collections::BTreeMap;
use std::{str::FromStr, sync::Arc};

use crate::constants::DEFAULT_FEE_RATE_SAT_PER_VB;
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

pub async fn withdraw_from_taproot_htlc(
    bitcoin_client: &RpcBlockchain,
    sender_wallet: &Wallet<MemoryDatabase>,
    recipient_secret: &SecretKey,
    recipient_public_key: secp256k1::PublicKey,
    revocation_public_key: PublicKey,
    swap_secret: SecretKey,
    timelock_duration_blocks: u32,
) -> Result<Txid> {
    let key = swap_secret
        .add_tweak(&Scalar::from_be_bytes(recipient_secret.secret_bytes())?)
        .expect("Invalid secret key");

    let escrow_privkey = PrivateKey::new(key, sender_wallet.network());

    let revocation_pubkey = bitcoin::PublicKey {
        inner: revocation_public_key,
        compressed: true,
    };

    let taproot_descriptor = bdk::descriptor!(tr(
        escrow_privkey,
        and_v(v:pk(revocation_pubkey), older(timelock_duration_blocks))
    ))?;

    let wallet = Wallet::new(
        taproot_descriptor,
        None,
        sender_wallet.network(),
        MemoryDatabase::new(),
    )?;

    wallet
        .sync(bitcoin_client, SyncOptions::default())
        .wrap_err("failed to sync a BDK wallet")?;

    let wallet_policy = wallet.policies(KeychainKind::External)?.unwrap();
    let mut path = BTreeMap::new();
    // We need to use the first leaf of the script path spend, hence the second policy
    // If you're not sure what's happening here, no worries, this is bit tricky :)
    // You can learn more here: https://docs.rs/bdk/latest/bdk/wallet/tx_builder/struct.TxBuilder.html#method.policy_path
    path.insert(wallet_policy.id, vec![0]);

    let (mut psbt, _details) = {
        let mut builder = wallet.build_tx();

        let recepient_address = BitcoinAddress::p2wpkh(
            &bitcoin::PublicKey::new(recipient_public_key),
            sender_wallet.network(),
        )?;

        builder
            .fee_rate(FeeRate::from_sat_per_vb(DEFAULT_FEE_RATE_SAT_PER_VB))
            .drain_wallet()
            .drain_to(recepient_address.script_pubkey())
            .policy_path(path, KeychainKind::External);

        builder.finish()?
    };

    let is_finalized = wallet.sign(&mut psbt, SignOptions::default())?;

    if !is_finalized {
        return Err(eyre!("failed to sign and finalize a transaction"));
    }

    let txid = psbt.unsigned_tx.txid();

    bitcoin_client.broadcast(&psbt.extract_tx())?;

    Ok(txid)
}
