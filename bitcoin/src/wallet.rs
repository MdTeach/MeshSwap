use std::{fs, path::Path};

use bdk::{
    Wallet,
    bitcoin::{
        Network, Address,
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
use std::str::FromStr;

use crate::blockchain::create_bitcoin_rpc_client;
use crate::constants::SATOSHIS_PER_BTC;

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

/// Wallet factory for creating and managing Bitcoin wallets
pub struct WalletFactory;

/// Represents a Bitcoin wallet with associated operations
pub struct BitcoinWallet {
    pub wallet: Wallet<MemoryDatabase>,
}

impl WalletFactory {
    /// Load a wallet from a configuration file
    pub async fn load_wallet<P: AsRef<Path>>(config_file_path: P) -> Result<BitcoinWallet> {
        let path = config_file_path.as_ref();
        let wallet = Self::create_wallet_from_config(path).await?;
        
        Ok(BitcoinWallet { wallet })
    }

    /// Extract public and private keys from wallet configuration
    pub fn extract_keypair<P: AsRef<Path>>(config_file_path: P) -> Result<(PublicKey, SecretKey)> {
        let path = config_file_path.as_ref();
        if !path.exists() {
            return Err(eyre!("Wallet configuration file not found: {}", path.display()));
        }

        let config = Self::load_config(path)?;
        let (private_key, _) = Self::derive_keys_from_config(&config)?;
        let secp_context = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp_context, &private_key);

        Ok((public_key, private_key))
    }

    /// Get wallet address for a given config file
    pub async fn get_address<P: AsRef<Path>>(config_file_path: P) -> Result<Address> {
        let wallet = Self::load_wallet(config_file_path).await?;
        let address_str = wallet.get_receiving_address()?;
        Ok(Address::from_str(&address_str)?)
    }

    /// Get wallet balance in satoshis for a given config file
    pub async fn get_balance_satoshis<P: AsRef<Path>>(config_file_path: P) -> Result<u64> {
        let wallet = Self::load_wallet(config_file_path).await?;
        wallet.get_balance_satoshis().await
    }

    fn load_config(config_file_path: &Path) -> Result<WalletConfig> {
        let config_content = fs::read_to_string(config_file_path)?;
        Ok(toml::from_str(&config_content)?)
    }

    fn derive_keys_from_config(config: &WalletConfig) -> Result<(SecretKey, DerivationPath)> {
        let mnemonic = Mnemonic::parse(&config.keys.mnemonic)?;
        let extended_key: ExtendedKey = mnemonic.into_extended_key()?;
        let root_private_key = extended_key
            .into_xprv(Network::Regtest)
            .ok_or_else(|| eyre!("Invalid private key"))?;

        let derivation_path: DerivationPath = config
            .keys
            .derivation_path
            .parse()
            .map_err(|e| eyre!("Invalid derivation path: {}", e))?;

        let secp_context = Secp256k1::new();
        let derived_private_key = root_private_key
            .derive_priv(&secp_context, &derivation_path)
            .map_err(|e| eyre!("Failed to derive key: {}", e))?;

        Ok((derived_private_key.private_key, derivation_path))
    }

    async fn create_wallet_from_config(config_file_path: &Path) -> Result<Wallet<MemoryDatabase>> {
        if !config_file_path.exists() {
            return Err(eyre!("Wallet configuration file not found: {}", config_file_path.display()));
        }

        let config = Self::load_config(config_file_path)?;
        let mnemonic = Mnemonic::parse(&config.keys.mnemonic)?;
        let extended_key: ExtendedKey = mnemonic.into_extended_key()?;
        let root_private_key = extended_key
            .into_xprv(Network::Regtest)
            .ok_or_else(|| eyre!("Invalid private key"))?;

        let derivation_path: DerivationPath = config
            .keys
            .derivation_path
            .parse()
            .map_err(|e| eyre!("Invalid derivation path: {}", e))?;

        let secp_context = Secp256k1::new();
        let derived_private_key = root_private_key
            .derive_priv(&secp_context, &derivation_path)
            .map_err(|e| eyre!("Failed to derive key: {}", e))?;

        let wallet_descriptor = format!("wpkh({}/*)", derived_private_key);
        let wallet_database = MemoryDatabase::default();
        let wallet = Wallet::new(&wallet_descriptor, None, Network::Regtest, wallet_database)?;

        Ok(wallet)
    }
}

impl BitcoinWallet {
    /// Load a wallet from a configuration file
    pub async fn from_config_file<P: AsRef<Path>>(config_file_path: P) -> Result<Self> {
        WalletFactory::load_wallet(config_file_path).await
    }

    /// Get the wallet's current balance in satoshis
    pub async fn get_balance_satoshis(&self) -> Result<u64> {
        let blockchain_client = create_bitcoin_rpc_client()?;
        self.wallet.sync(&blockchain_client, SyncOptions::default())?;
        
        let balance = self.wallet.get_balance()?;
        Ok(balance.get_total())
    }


    /// Get the wallet's receiving address
    pub fn get_receiving_address(&self) -> Result<String> {
        let address_info = self.wallet.get_address(bdk::wallet::AddressIndex::Peek(0))?;
        Ok(address_info.address.to_string())
    }
}



/// Convert BTC amount to satoshis
pub fn btc_to_satoshis(btc_amount: f64) -> u64 {
    (btc_amount * SATOSHIS_PER_BTC as f64) as u64
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

