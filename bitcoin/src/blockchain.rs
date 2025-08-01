use bdk::blockchain::{ConfigurableBlockchain, RpcBlockchain};
use bdk::bitcoin::Network;
use eyre::Result;

use crate::constants::{DEFAULT_RPC_URL, DEFAULT_RPC_USERNAME, DEFAULT_RPC_PASSWORD};

/// Configuration for Bitcoin RPC connection
#[derive(Debug, Clone)]
pub struct BitcoinRpcConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub network: Network,
}

impl Default for BitcoinRpcConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_RPC_URL.to_string(),
            username: DEFAULT_RPC_USERNAME.to_string(),
            password: DEFAULT_RPC_PASSWORD.to_string(),
            network: Network::Regtest,
        }
    }
}

/// Creates a configured Bitcoin RPC blockchain client
pub fn create_bitcoin_rpc_client() -> Result<RpcBlockchain> {
    create_bitcoin_rpc_client_with_config(BitcoinRpcConfig::default())
}

/// Creates a Bitcoin RPC blockchain client with custom configuration
pub fn create_bitcoin_rpc_client_with_config(config: BitcoinRpcConfig) -> Result<RpcBlockchain> {
    let rpc_config = bdk::blockchain::rpc::RpcConfig {
        url: config.url,
        auth: bdk::blockchain::rpc::Auth::UserPass {
            username: config.username,
            password: config.password,
        },
        network: config.network,
        wallet_name: String::new(), // Use empty wallet name to avoid import issues
        sync_params: None,
    };

    Ok(RpcBlockchain::from_config(&rpc_config)?)
}