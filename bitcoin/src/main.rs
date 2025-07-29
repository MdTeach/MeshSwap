mod args;

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use bdk::bitcoin::Network;
use bdk::wallet::Wallet;
use bdk::database::MemoryDatabase;
use bdk::keys::{bip39::Mnemonic, DerivableKey, ExtendedKey};
use bdk::blockchain::{rpc::RpcBlockchain, ConfigurableBlockchain};
use clap::Parser;
use serde::Deserialize;

use args::Args;

#[derive(Debug, Deserialize)]
struct WalletConfig {
    keys: KeyInfo,
}

#[derive(Debug, Deserialize)]
struct KeyInfo {
    mnemonic: String,
}

async fn load_wallet_from_config(config_path: &Path) -> Result<Wallet<MemoryDatabase>> {
    if !config_path.exists() {
        return Err(anyhow!("Wallet config file not found: {}", config_path.display()));
    }
    
    let config_content = fs::read_to_string(config_path)?;
    let config: WalletConfig = toml::from_str(&config_content)?;
    
    let mnemonic = Mnemonic::parse(&config.keys.mnemonic)?;
    let xkey: ExtendedKey = mnemonic.into_extended_key()?;
    let xprv = xkey.into_xprv(Network::Regtest).ok_or_else(|| anyhow!("Invalid private key"))?;
    
    let descriptor = format!("wpkh({}/0/*)", xprv);
    
    let database = MemoryDatabase::default();
    let wallet = Wallet::new(&descriptor, None, Network::Regtest, database)?;
    
    Ok(wallet)
}

async fn get_wallet_balance(config_path: &Path) -> Result<u64> {
    let wallet = load_wallet_from_config(config_path).await?;
    
    let blockchain = RpcBlockchain::from_config(&bdk::blockchain::rpc::RpcConfig {
        url: "http://127.0.0.1:18443".to_string(),
        auth: bdk::blockchain::rpc::Auth::UserPass {
            username: "bitcoin".to_string(),
            password: "bitcoin".to_string(),
        },
        network: Network::Regtest,
        wallet_name: "".to_string(), // Use empty wallet name to avoid import issues
        sync_params: None,
    })?;
    
    wallet.sync(&blockchain, bdk::SyncOptions::default())?;
    
    
    let balance = wallet.get_balance()?;
    Ok(balance.get_total())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    match get_wallet_balance(&args.wallet).await {
        Ok(balance) => {
            println!("{}", balance);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}
