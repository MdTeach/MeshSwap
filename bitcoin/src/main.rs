mod args;
mod contract;

use std::fs;
use std::path::Path;

use anyhow::{Result, anyhow};
use bdk::bitcoin::Network;
use bdk::bitcoin::bip32::DerivationPath;
use bdk::bitcoin::hashes::{Hash, sha256};
use bdk::bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use bdk::bitcoin::{Address, Amount};
use bdk::blockchain::{Blockchain, ConfigurableBlockchain, GetHeight, GetTx, rpc::RpcBlockchain};
use bdk::database::MemoryDatabase;
use bdk::keys::{DerivableKey, ExtendedKey, bip39::Mnemonic};
use bdk::wallet::Wallet;
use clap::Parser;
use serde::Deserialize;
use std::str::FromStr;

use args::{Args, Commands};

fn format_btc(sats: u64) -> String {
    let btc = sats as f64 / 100_000_000.0;
    if btc == 0.0 {
        "0".to_string()
    } else {
        let s = format!("{:.3}", btc);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[derive(Debug, Deserialize)]
struct WalletConfig {
    keys: KeyInfo,
}

#[derive(Debug, Deserialize)]
struct KeyInfo {
    mnemonic: String,
    derivation_path: String,
}

async fn load_wallet_from_config(config_path: &Path) -> Result<Wallet<MemoryDatabase>> {
    if !config_path.exists() {
        return Err(anyhow!(
            "Wallet config file not found: {}",
            config_path.display()
        ));
    }

    let config_content = fs::read_to_string(config_path)?;
    let config: WalletConfig = toml::from_str(&config_content)?;

    let mnemonic = Mnemonic::parse(&config.keys.mnemonic)?;
    let xkey: ExtendedKey = mnemonic.into_extended_key()?;
    let root_xprv = xkey
        .into_xprv(bdk::bitcoin::Network::Regtest)
        .ok_or_else(|| anyhow!("Invalid private key"))?;

    // Parse derivation path (e.g., "m/84h/1h/0h")
    let derivation_path: DerivationPath = config
        .keys
        .derivation_path
        .parse()
        .map_err(|e| anyhow!("Invalid derivation path: {}", e))?;

    // Derive the key at the specific path
    let secp = Secp256k1::new();
    let derived_xprv = root_xprv
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| anyhow!("Failed to derive key: {}", e))?;

    let descriptor = format!("wpkh({}/*)", derived_xprv);

    let database = MemoryDatabase::default();
    let wallet = Wallet::new(&descriptor, None, Network::Regtest, database)?;

    Ok(wallet)
}

fn create_blockchain() -> Result<RpcBlockchain> {
    Ok(RpcBlockchain::from_config(
        &bdk::blockchain::rpc::RpcConfig {
            url: "http://127.0.0.1:18443".to_string(),
            auth: bdk::blockchain::rpc::Auth::UserPass {
                username: "bitcoin".to_string(),
                password: "bitcoin".to_string(),
            },
            network: Network::Regtest,
            wallet_name: "".to_string(), // Use empty wallet name to avoid import issues
            sync_params: None,
        },
    )?)
}

async fn get_wallet_balance(config_path: &Path) -> Result<u64> {
    let wallet = load_wallet_from_config(config_path).await?;
    let blockchain = create_blockchain()?;

    wallet.sync(&blockchain, bdk::SyncOptions::default())?;

    let balance = wallet.get_balance()?;
    Ok(balance.get_total())
}

async fn get_wallet_address(config_path: &Path) -> Result<String> {
    let wallet = load_wallet_from_config(config_path).await?;
    let address = wallet.get_address(bdk::wallet::AddressIndex::Peek(0))?;
    Ok(address.address.to_string())
}

fn extract_keys_from_config(config_path: &Path) -> Result<(PublicKey, SecretKey)> {
    if !config_path.exists() {
        return Err(anyhow!(
            "Wallet config file not found: {}",
            config_path.display()
        ));
    }

    let config_content = fs::read_to_string(config_path)?;
    let config: WalletConfig = toml::from_str(&config_content)?;

    let mnemonic = Mnemonic::parse(&config.keys.mnemonic)?;
    let xkey: ExtendedKey = mnemonic.into_extended_key()?;
    let root_xprv = xkey
        .into_xprv(bdk::bitcoin::Network::Regtest)
        .ok_or_else(|| anyhow!("Invalid private key"))?;

    // Parse derivation path
    let derivation_path: DerivationPath = config
        .keys
        .derivation_path
        .parse()
        .map_err(|e| anyhow!("Invalid derivation path: {}", e))?;

    // Derive the key at the specific path
    let secp = Secp256k1::new();
    let derived_xprv = root_xprv
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| anyhow!("Failed to derive key: {}", e))?;

    let private_key = derived_xprv.private_key;
    let public_key = PublicKey::from_secret_key(&secp, &private_key);

    Ok((public_key, private_key))
}

async fn create_htlc(
    from_wallet_path: &Path,
    to_wallet_path: &Path,
    amount_btc: f64,
    secret: &str,
    timeout_block: u32,
) -> Result<String> {
    // Extract keys from wallet configs
    let (sender_pubkey, _sender_privkey) = extract_keys_from_config(from_wallet_path)?;
    let (recipient_pubkey, _recipient_privkey) = extract_keys_from_config(to_wallet_path)?;

    // Create HTLC contract
    let secret_bytes = secret.as_bytes();
    let htlc = contract::create_htlc_contract(
        recipient_pubkey,
        sender_pubkey,
        secret_bytes,
        timeout_block,
    );

    // Load sender wallet for funding
    let from_wallet = load_wallet_from_config(from_wallet_path).await?;
    let blockchain = create_blockchain()?;
    from_wallet.sync(&blockchain, bdk::SyncOptions::default())?;

    // Check balance
    let balance = from_wallet.get_balance()?;
    let amount_satoshis = (amount_btc * 100_000_000.0) as u64;
    if balance.get_total() < amount_satoshis {
        return Err(anyhow!(
            "Insufficient balance. Available: {} sats, Required: {} sats",
            balance.get_total(),
            amount_satoshis
        ));
    }

    // Create HTLC script and funding transaction
    let htlc_script = htlc.create_script();

    // Build transaction to fund the HTLC
    let mut tx_builder = from_wallet.build_tx();

    // Create P2WSH script pubkey from HTLC script
    // P2WSH uses single SHA256 for the witness script hash
    let script_hash = sha256::Hash::hash(htlc_script.as_bytes());
    let script_pubkey = bdk::bitcoin::script::Builder::new()
        .push_int(0)
        .push_slice(script_hash.as_byte_array())
        .into_script();

    tx_builder
        .add_recipient(script_pubkey, amount_satoshis)
        .enable_rbf();

    let (mut psbt, _) = tx_builder.finish()?;

    // Sign transaction
    let finalized = from_wallet.sign(&mut psbt, bdk::SignOptions::default())?;
    if !finalized {
        return Err(anyhow!("Failed to finalize HTLC funding transaction"));
    }

    // Broadcast transaction
    let tx = psbt.extract_tx();
    blockchain.broadcast(&tx)?;

    let contract_id = tx.txid().to_string();
    println!("HTLC created successfully!");
    println!("Contract ID: {}", contract_id);
    println!(
        "Amount: {} BTC ({} sats)",
        format_btc(amount_satoshis),
        amount_satoshis
    );
    println!("Secret: {}", secret);
    println!("Timeout Block: {}", timeout_block);
    println!("Recipient can claim with secret: {}", secret);

    Ok(contract_id)
}

async fn claim_htlc(
    contract_id: &str,
    wallet_path: &Path,
    secret: &str,
    _amount: f64, // Amount parameter kept for API compatibility but actual output value is used
    timeout_block: u32,
    from_wallet_path: &Path,
) -> Result<String> {
    // Parse contract ID as transaction hash
    let contract_txid = bdk::bitcoin::Txid::from_str(contract_id)
        .map_err(|e| anyhow!("Invalid contract ID: {}", e))?;

    // Extract keys from wallet config
    let (recipient_pubkey, recipient_privkey) = extract_keys_from_config(wallet_path)?;

    // Load wallet and get address
    let recipient_wallet = load_wallet_from_config(wallet_path).await?;
    let recipient_address_info = recipient_wallet.get_address(bdk::wallet::AddressIndex::New)?;
    let recipient_address = Address::from_str(&recipient_address_info.address.to_string())
        .map_err(|e| anyhow!("Failed to parse address: {}", e))?
        .require_network(bdk::bitcoin::Network::Regtest)
        .map_err(|e| anyhow!("Address network mismatch: {}", e))?;

    // Extract sender keys from the provided wallet path
    let (sender_pubkey, _) = extract_keys_from_config(from_wallet_path)?;

    // Create HTLC contract (we need sender pubkey to recreate the contract)
    let secret_bytes = secret.as_bytes();
    let htlc = contract::create_htlc_contract(
        recipient_pubkey,
        sender_pubkey,
        secret_bytes,
        timeout_block,
    );

    // Find the HTLC output by matching script hash
    let blockchain = create_blockchain()?;
    let htlc_tx = blockchain
        .get_tx(&contract_txid)
        .map_err(|e| anyhow!("Failed to get HTLC transaction: {}", e))?
        .ok_or_else(|| anyhow!("HTLC transaction not found"))?;

    let script = htlc.create_script();
    let expected_script_hash = sha256::Hash::hash(script.as_bytes());
    let expected_script_pubkey = bdk::bitcoin::script::Builder::new()
        .push_int(0)
        .push_slice(expected_script_hash.as_byte_array())
        .into_script();

    // Find the output that matches our P2WSH script
    let htlc_vout = htlc_tx
        .output
        .iter()
        .enumerate()
        .find(|(_, output)| output.script_pubkey == expected_script_pubkey)
        .map(|(index, _)| index as u32)
        .ok_or_else(|| anyhow!("HTLC output not found in transaction"))?;

    let htlc_output = &htlc_tx.output[htlc_vout as usize];

    // Create claim transaction
    let amount_btc = Amount::from_sat(htlc_output.value); // Use the actual output amount
    let claim_tx = htlc
        .create_claim_transaction(
            contract_txid,
            htlc_vout,
            amount_btc,
            &recipient_address,
            secret_bytes,
            &recipient_privkey,
        )
        .map_err(|e| anyhow!("Failed to create claim transaction: {}", e))?;

    // Broadcast claim transaction
    blockchain.broadcast(&claim_tx)?;

    let claim_txid = claim_tx.txid().to_string();
    println!("HTLC claimed successfully!");
    println!("Claim transaction ID: {}", claim_txid);

    Ok(claim_txid)
}

async fn refund_htlc(
    contract_id: &str,
    sender_wallet_path: &Path,
    _amount: f64, // Amount parameter kept for API compatibility but actual output value is used
    timeout_block: u32,
    recipient_wallet_path: &Path,
    secret: &str, // Need the original secret to reconstruct the script
) -> Result<String> {
    // Parse contract ID as transaction hash
    let contract_txid = bdk::bitcoin::Txid::from_str(contract_id)
        .map_err(|e| anyhow!("Invalid contract ID: {}", e))?;

    // Extract keys from wallet config
    let (sender_pubkey, sender_privkey) = extract_keys_from_config(sender_wallet_path)?;

    // Load sender wallet and get address for refund
    let sender_wallet = load_wallet_from_config(sender_wallet_path).await?;
    let sender_address_info = sender_wallet.get_address(bdk::wallet::AddressIndex::New)?;
    let sender_address = Address::from_str(&sender_address_info.address.to_string())
        .map_err(|e| anyhow!("Failed to parse address: {}", e))?
        .require_network(bdk::bitcoin::Network::Regtest)
        .map_err(|e| anyhow!("Address network mismatch: {}", e))?;

    // Extract recipient keys from the provided wallet path
    let (recipient_pubkey, _) = extract_keys_from_config(recipient_wallet_path)?;

    // Create HTLC contract (we need recipient pubkey to recreate the contract)
    // Use the original secret to ensure we get the same script hash
    let secret_bytes = secret.as_bytes();
    let htlc = contract::create_htlc_contract(
        recipient_pubkey,
        sender_pubkey,
        secret_bytes,
        timeout_block,
    );

    // Find the HTLC output by matching script hash
    let blockchain = create_blockchain()?;
    let htlc_tx = blockchain
        .get_tx(&contract_txid)
        .map_err(|e| anyhow!("Failed to get HTLC transaction: {}", e))?
        .ok_or_else(|| anyhow!("HTLC transaction not found"))?;

    let script = htlc.create_script();
    let expected_script_hash = sha256::Hash::hash(script.as_bytes());
    let expected_script_pubkey = bdk::bitcoin::script::Builder::new()
        .push_int(0)
        .push_slice(expected_script_hash.as_byte_array())
        .into_script();

    // Find the output that matches our P2WSH script
    let htlc_vout = htlc_tx
        .output
        .iter()
        .enumerate()
        .find(|(_, output)| output.script_pubkey == expected_script_pubkey)
        .map(|(index, _)| index as u32)
        .ok_or_else(|| anyhow!("HTLC output not found in transaction"))?;

    let htlc_output = &htlc_tx.output[htlc_vout as usize];

    // Check if the timeout has been reached
    let current_height = blockchain
        .get_height()
        .map_err(|e| anyhow!("Failed to get current block height: {}", e))?;

    if current_height < timeout_block {
        return Err(anyhow!(
            "Timeout not reached yet. Current height: {}, timeout height: {}",
            current_height,
            timeout_block
        ));
    }

    // Create refund transaction
    let amount_btc = Amount::from_sat(htlc_output.value); // Use the actual output amount
    let refund_tx = htlc
        .create_refund_transaction(
            contract_txid,
            htlc_vout,
            amount_btc,
            &sender_address,
            &sender_privkey,
        )
        .map_err(|e| anyhow!("Failed to create refund transaction: {}", e))?;

    // Broadcast refund transaction
    blockchain.broadcast(&refund_tx)?;

    let refund_txid = refund_tx.txid().to_string();
    println!("HTLC refunded successfully!");
    println!("Refund transaction ID: {}", refund_txid);
    println!(
        "Refunded amount: {} BTC ({} sats)",
        format_btc(htlc_output.value - 1000),
        htlc_output.value - 1000
    );

    Ok(refund_txid)
}

async fn send_btc(
    from_wallet_path: &Path,
    to_wallet_path: &Path,
    amount_btc: f64,
) -> Result<String> {
    let from_wallet = load_wallet_from_config(from_wallet_path).await?;
    let to_wallet = load_wallet_from_config(to_wallet_path).await?;
    let blockchain = create_blockchain()?;

    // Sync sender wallet
    from_wallet.sync(&blockchain, bdk::SyncOptions::default())?;

    // Get recipient address (first address from their wallet)
    let recipient_address = to_wallet.get_address(bdk::wallet::AddressIndex::Peek(0))?;

    // Convert BTC to satoshis
    let amount_satoshis = (amount_btc * 100_000_000.0) as u64;

    // Check if sender has enough balance
    let balance = from_wallet.get_balance()?;
    if balance.get_total() < amount_satoshis {
        return Err(anyhow!(
            "Insufficient balance. Available: {} sats, Required: {} sats",
            balance.get_total(),
            amount_satoshis
        ));
    }

    // Create transaction
    let mut tx_builder = from_wallet.build_tx();
    tx_builder
        .add_recipient(recipient_address.script_pubkey(), amount_satoshis)
        .enable_rbf();

    let (mut psbt, _) = tx_builder.finish()?;

    // Sign transaction
    let finalized = from_wallet.sign(&mut psbt, bdk::SignOptions::default())?;
    if !finalized {
        return Err(anyhow!("Failed to finalize transaction"));
    }

    // Broadcast transaction
    let tx = psbt.extract_tx();
    blockchain.broadcast(&tx)?;

    Ok(tx.txid().to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Some(Commands::Balance) | None => {
            // Default behavior: show balance
            match get_wallet_balance(&args.wallet).await {
                Ok(balance) => {
                    println!("Balance: {} BTC ({} sats)", format_btc(balance), balance);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Address) => match get_wallet_address(&args.wallet).await {
            Ok(address) => {
                println!("{}", address);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Some(Commands::Send { to, amount }) => match send_btc(&args.wallet, &to, amount).await {
            Ok(txid) => {
                let amount_sats = (amount * 100_000_000.0) as u64;
                println!("Transaction sent successfully!");
                println!("TXID: {}", txid);
                println!(
                    "Amount: {} BTC ({} sats)",
                    format_btc(amount_sats),
                    amount_sats
                );
            }
            Err(e) => {
                eprintln!("Error sending transaction: {}", e);
                std::process::exit(1);
            }
        },
        Some(Commands::HtlcCreate {
            to,
            amount,
            secret,
            timeout_block,
        }) => {
            match create_htlc(&args.wallet, &to, amount, &secret, timeout_block).await {
                Ok(_contract_id) => {
                    // Success message already printed in create_htlc function
                }
                Err(e) => {
                    eprintln!("Error creating HTLC: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::HtlcClaim {
            contract_id,
            secret,
            amount,
            timeout_block,
            from,
        }) => {
            match claim_htlc(
                &contract_id,
                &args.wallet,
                &secret,
                amount,
                timeout_block,
                &from,
            )
            .await
            {
                Ok(_claim_txid) => {
                    // Success message already printed in claim_htlc function
                }
                Err(e) => {
                    eprintln!("Error claiming HTLC: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::HtlcRefund {
            contract_id,
            secret,
            amount,
            timeout_block,
            to,
        }) => {
            match refund_htlc(
                &contract_id,
                &args.wallet,
                amount,
                timeout_block,
                &to,
                &secret,
            )
            .await
            {
                Ok(_refund_txid) => {
                    // Success message already printed in refund_htlc function
                }
                Err(e) => {
                    eprintln!("Error refunding HTLC: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
