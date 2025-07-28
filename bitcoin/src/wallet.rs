use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use bdk::bitcoin::Network;
use bdk::wallet::{Wallet, AddressIndex};
use bdk::database::MemoryDatabase;
use bdk::keys::{bip39::Mnemonic, DerivableKey, ExtendedKey};
use bdk::blockchain::{rpc::RpcBlockchain, ConfigurableBlockchain, Blockchain};
use bitcoincore_rpc::{Client, Auth, RpcApi};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WalletType {
    Admin,
    Maker,
    Taker,
}

impl WalletType {
    fn config_file(&self) -> &'static str {
        match self {
            WalletType::Admin => "wallet/admin.toml",
            WalletType::Maker => "wallet/maker.toml",
            WalletType::Taker => "wallet/taker.toml",
        }
    }
}

#[derive(Debug, Deserialize)]
struct WalletConfig {
    wallet: WalletInfo,
    keys: KeyInfo,
    config: Config,
}

#[derive(Debug, Deserialize)]
struct WalletInfo {
    name: String,
    #[serde(rename = "type")]
    wallet_type: String,
    network: String,
}

#[derive(Debug, Deserialize)]
struct KeyInfo {
    mnemonic: String,
    derivation_path: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    electrum_url: String,
    block_height: u32,
}

pub struct WalletManager {
    wallets: HashMap<WalletType, Wallet<MemoryDatabase>>,
    blockchain: RpcBlockchain,
    rpc_client: Client,
}

impl WalletManager {
    pub async fn new() -> Result<Self> {
        // Connect to regtest Bitcoin node
        let rpc_url = "http://127.0.0.1:18443";
        let rpc_auth = Auth::UserPass("bitcoin".to_string(), "bitcoin".to_string());
        let rpc_client = Client::new(rpc_url, rpc_auth)?;
        
        // Create BDK blockchain interface
        let blockchain = RpcBlockchain::from_config(&bdk::blockchain::rpc::RpcConfig {
            url: rpc_url.to_string(),
            auth: bdk::blockchain::rpc::Auth::UserPass {
                username: "bitcoin".to_string(),
                password: "bitcoin".to_string(),
            },
            network: Network::Regtest,
            wallet_name: "bdk_wallet".to_string(),
            sync_params: None,
        })?;
        
        let mut wallets = HashMap::new();
        
        for wallet_type in [WalletType::Admin, WalletType::Maker, WalletType::Taker] {
            let wallet = Self::load_wallet(wallet_type).await?;
            wallets.insert(wallet_type, wallet);
        }
        
        Ok(Self { 
            wallets, 
            blockchain,
            rpc_client 
        })
    }
    
    async fn load_wallet(wallet_type: WalletType) -> Result<Wallet<MemoryDatabase>> {
        let config_path = wallet_type.config_file();
        
        if !Path::new(config_path).exists() {
            return Err(anyhow!("Wallet config file not found: {}", config_path));
        }
        
        let config_content = fs::read_to_string(config_path)?;
        let config: WalletConfig = toml::from_str(&config_content)?;
        
        let network = match config.wallet.network.as_str() {
            "mainnet" => Network::Bitcoin,
            "testnet" => Network::Testnet,
            "regtest" => Network::Regtest,
            "signet" => Network::Signet,
            _ => return Err(anyhow!("Invalid network: {}", config.wallet.network)),
        };
        
        let mnemonic = Mnemonic::parse(&config.keys.mnemonic)?;
        let xkey: ExtendedKey = mnemonic.into_extended_key()?;
        let xprv = xkey.into_xprv(network).ok_or_else(|| anyhow!("Invalid private key"))?;
        
        let descriptor = format!("wpkh({}/0/*)", xprv);
        
        let database = MemoryDatabase::default();
        let wallet = Wallet::new(&descriptor, None, network, database)?;
        
        Ok(wallet)
    }
    
    pub async fn get_balance(&self, wallet_type: WalletType) -> Result<u64> {
        let wallet = self.wallets.get(&wallet_type)
            .ok_or_else(|| anyhow!("Wallet not found: {:?}", wallet_type))?;
        
        // Sync wallet with blockchain before getting balance
        wallet.sync(&self.blockchain, bdk::SyncOptions::default())?;
        
        let balance = wallet.get_balance()?;
        Ok(balance.get_total())
    }
    
    pub fn get_wallet(&self, wallet_type: WalletType) -> Result<&Wallet<MemoryDatabase>> {
        self.wallets.get(&wallet_type)
            .ok_or_else(|| anyhow!("Wallet not found: {:?}", wallet_type))
    }
    
    pub async fn sync_wallets(&mut self) -> Result<()> {
        for (wallet_type, wallet) in &mut self.wallets {
            println!("Syncing {:?} wallet...", wallet_type);
            wallet.sync(&self.blockchain, bdk::SyncOptions::default())?;
        }
        Ok(())
    }
    
    pub async fn mine_blocks(&self, num_blocks: u64) -> Result<()> {
        if let Some(admin_wallet) = self.wallets.get(&WalletType::Admin) {
            let mining_address = admin_wallet.get_address(AddressIndex::New)?;
            println!("Mining {} blocks to admin wallet address: {}", num_blocks, mining_address.address);
            
            let block_hashes = self.rpc_client.generate_to_address(num_blocks, &mining_address.address)?;
            println!("Mined {} blocks. Latest block: {:?}", block_hashes.len(), block_hashes.last());
            
            Ok(())
        } else {
            Err(anyhow!("Admin wallet not found"))
        }
    }
    
    pub async fn get_address(&self, wallet_type: WalletType) -> Result<String> {
        let wallet = self.wallets.get(&wallet_type)
            .ok_or_else(|| anyhow!("Wallet not found: {:?}", wallet_type))?;
        
        let address = wallet.get_address(AddressIndex::New)?;
        Ok(address.address.to_string())
    }
}