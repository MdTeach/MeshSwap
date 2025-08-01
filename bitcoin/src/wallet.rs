#![allow(dead_code)]
use std::{fs, path::Path};

use bdk::{
    Wallet,
    bitcoin::{
        Network,
        secp256k1::{PublicKey, Secp256k1, SecretKey},
        util::bip32::DerivationPath,
    },
    database::MemoryDatabase,
    keys::{DerivableKey, ExtendedKey},
    SyncOptions,
};
use bip39::Mnemonic;
use eyre::{Result, eyre};
use serde::Deserialize;

use crate::blockchain::create_bitcoin_rpc_client;

const SATOSHIS_PER_BTC: u64 = 100_000_000;

/// Wallet configuration loaded from TOML files
#[derive(Debug, Deserialize)]
pub struct WalletConfig {
    pub keys: KeyConfiguration,
}

/// Key configuration containing mnemonic and derivation path
#[derive(Debug, Deserialize)]
pub struct KeyConfiguration {
    pub mnemonic: String,
    pub derivation_path: String,
}

/// Represents a Bitcoin wallet with associated operations
pub struct BitcoinWallet {
    pub wallet: Wallet<MemoryDatabase>,
    pub config_path: String,
}

impl BitcoinWallet {
    /// Load a wallet from a configuration file
    pub async fn from_config_file<P: AsRef<Path>>(config_file_path: P) -> Result<Self> {
        let path = config_file_path.as_ref();
        let wallet = load_wallet_from_config_file(path).await?;
        
        Ok(Self {
            wallet,
            config_path: path.to_string_lossy().to_string(),
        })
    }

    /// Get the wallet's current balance in satoshis
    pub async fn get_balance_satoshis(&self) -> Result<u64> {
        let blockchain_client = create_bitcoin_rpc_client()?;
        self.wallet.sync(&blockchain_client, SyncOptions::default())?;
        
        let balance = self.wallet.get_balance()?;
        Ok(balance.get_total())
    }

    /// Get the wallet's current balance formatted as BTC string
    pub async fn get_balance_btc_formatted(&self) -> Result<String> {
        let balance_satoshis = self.get_balance_satoshis().await?;
        Ok(format_satoshis_to_btc(balance_satoshis))
    }

    /// Get the wallet's receiving address
    pub fn get_receiving_address(&self) -> Result<String> {
        let address_info = self.wallet.get_address(bdk::wallet::AddressIndex::Peek(0))?;
        Ok(address_info.address.to_string())
    }
}

/// Load a wallet from a TOML configuration file
async fn load_wallet_from_config_file(config_file_path: &Path) -> Result<Wallet<MemoryDatabase>> {
    if !config_file_path.exists() {
        return Err(eyre!(
            "Wallet configuration file not found: {}",
            config_file_path.display()
        ));
    }

    let config_content = fs::read_to_string(config_file_path)?;
    let wallet_config: WalletConfig = toml::from_str(&config_content)?;

    let mnemonic = Mnemonic::parse(&wallet_config.keys.mnemonic)?;
    let extended_key: ExtendedKey = mnemonic.into_extended_key()?;
    let root_private_key = extended_key
        .into_xprv(Network::Regtest)
        .ok_or_else(|| eyre!("Invalid private key"))?;

    // Parse derivation path (e.g., "m/84h/1h/0h")
    let derivation_path: DerivationPath = wallet_config
        .keys
        .derivation_path
        .parse()
        .map_err(|e| eyre!("Invalid derivation path: {}", e))?;

    // Derive the key at the specific path
    let secp_context = Secp256k1::new();
    let derived_private_key = root_private_key
        .derive_priv(&secp_context, &derivation_path)
        .map_err(|e| eyre!("Failed to derive key: {}", e))?;

    let wallet_descriptor = format!("wpkh({}/*)", derived_private_key);

    let wallet_database = MemoryDatabase::default();
    let wallet = Wallet::new(&wallet_descriptor, None, Network::Regtest, wallet_database)?;

    Ok(wallet)
}

/// Extract public and private keys from wallet configuration
pub fn extract_keypair_from_config(config_file_path: &Path) -> Result<(PublicKey, SecretKey)> {
    if !config_file_path.exists() {
        return Err(eyre!(
            "Wallet configuration file not found: {}",
            config_file_path.display()
        ));
    }

    let config_content = fs::read_to_string(config_file_path)?;
    let wallet_config: WalletConfig = toml::from_str(&config_content)?;

    let mnemonic = Mnemonic::parse(&wallet_config.keys.mnemonic)?;
    let extended_key: ExtendedKey = mnemonic.into_extended_key()?;
    let root_private_key = extended_key
        .into_xprv(Network::Regtest)
        .ok_or_else(|| eyre!("Invalid private key"))?;

    // Parse derivation path
    let derivation_path: DerivationPath = wallet_config
        .keys
        .derivation_path
        .parse()
        .map_err(|e| eyre!("Invalid derivation path: {}", e))?;

    // Derive the key at the specific path
    let secp_context = Secp256k1::new();
    let derived_private_key = root_private_key
        .derive_priv(&secp_context, &derivation_path)
        .map_err(|e| eyre!("Failed to derive key: {}", e))?;

    let private_key = derived_private_key.private_key;
    let public_key = PublicKey::from_secret_key(&secp_context, &private_key);

    Ok((public_key, private_key))
}

/// Format satoshis to a clean BTC string representation
pub fn format_satoshis_to_btc(satoshis: u64) -> String {
    let btc_amount = satoshis as f64 / SATOSHIS_PER_BTC as f64;
    if btc_amount == 0.0 {
        "0".to_string()
    } else {
        let formatted = format!("{:.8}", btc_amount);
        formatted.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Get wallet balance in satoshis for a given config file
pub async fn get_wallet_balance_satoshis<P: AsRef<Path>>(config_file_path: P) -> Result<u64> {
    let bitcoin_wallet = BitcoinWallet::from_config_file(config_file_path).await?;
    bitcoin_wallet.get_balance_satoshis().await
}

/// Get wallet address for a given config file
pub async fn get_wallet_address<P: AsRef<Path>>(config_file_path: P) -> Result<String> {
    let bitcoin_wallet = BitcoinWallet::from_config_file(config_file_path).await?;
    bitcoin_wallet.get_receiving_address()
}