mod args;

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use bdk::bitcoin::Network;
use bdk::wallet::Wallet;
use bdk::database::MemoryDatabase;
use bdk::keys::{bip39::Mnemonic, DerivableKey, ExtendedKey};
use bdk::bitcoin::bip32::DerivationPath;
use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::blockchain::{rpc::RpcBlockchain, ConfigurableBlockchain, Blockchain};
use clap::Parser;
use serde::Deserialize;

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
        return Err(anyhow!("Wallet config file not found: {}", config_path.display()));
    }
    
    let config_content = fs::read_to_string(config_path)?;
    let config: WalletConfig = toml::from_str(&config_content)?;
    
    let mnemonic = Mnemonic::parse(&config.keys.mnemonic)?;
    let xkey: ExtendedKey = mnemonic.into_extended_key()?;
    let root_xprv = xkey.into_xprv(Network::Regtest).ok_or_else(|| anyhow!("Invalid private key"))?;
    
    // Parse derivation path (e.g., "m/84h/1h/0h")
    let derivation_path: DerivationPath = config.keys.derivation_path.parse()
        .map_err(|e| anyhow!("Invalid derivation path: {}", e))?;
    
    // Derive the key at the specific path
    let secp = Secp256k1::new();
    let derived_xprv = root_xprv.derive_priv(&secp, &derivation_path)
        .map_err(|e| anyhow!("Failed to derive key: {}", e))?;
    
    let descriptor = format!("wpkh({}/*)", derived_xprv);
    
    let database = MemoryDatabase::default();
    let wallet = Wallet::new(&descriptor, None, Network::Regtest, database)?;
    
    Ok(wallet)
}

fn create_blockchain() -> Result<RpcBlockchain> {
    Ok(RpcBlockchain::from_config(&bdk::blockchain::rpc::RpcConfig {
        url: "http://127.0.0.1:18443".to_string(),
        auth: bdk::blockchain::rpc::Auth::UserPass {
            username: "bitcoin".to_string(),
            password: "bitcoin".to_string(),
        },
        network: Network::Regtest,
        wallet_name: "".to_string(), // Use empty wallet name to avoid import issues
        sync_params: None,
    })?)
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

async fn send_btc(from_wallet_path: &Path, to_wallet_path: &Path, amount_btc: f64) -> Result<String> {
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
        return Err(anyhow!("Insufficient balance. Available: {} sats, Required: {} sats", 
                          balance.get_total(), amount_satoshis));
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
        Some(Commands::Address) => {
            match get_wallet_address(&args.wallet).await {
                Ok(address) => {
                    println!("{}", address);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Send { to, amount }) => {
            match send_btc(&args.wallet, &to, amount).await {
                Ok(txid) => {
                    let amount_sats = (amount * 100_000_000.0) as u64;
                    println!("Transaction sent successfully!");
                    println!("TXID: {}", txid);
                    println!("Amount: {} BTC ({} sats)", format_btc(amount_sats), amount_sats);
                }
                Err(e) => {
                    eprintln!("Error sending transaction: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    
    Ok(())
}
