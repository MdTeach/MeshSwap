use bitcoin::{
    Address, Network,
    secp256k1::{PublicKey, Secp256k1, SecretKey},
    hashes::{Hash, sha256},
};
use rand::thread_rng;

use crate::contract::{HTLCContract, create_htlc_contract};

pub struct DeploymentConfig {
    pub network: Network,
    pub timelock_blocks: u32,
    pub secret: Vec<u8>,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            network: Network::Regtest,
            timelock_blocks: 144, // ~24 hours
            secret: b"my_secret_preimage".to_vec(),
        }
    }
}

pub struct DeploymentResult {
    pub contract: HTLCContract,
    pub address: Address,
    pub recipient_secret: SecretKey,
    pub sender_secret: SecretKey,
    pub hash_lock: sha256::Hash,
}

pub fn deploy_htlc(config: DeploymentConfig) -> Result<DeploymentResult, Box<dyn std::error::Error>> {
    let secp = Secp256k1::new();
    
    // Generate random keys for demonstration
    let recipient_secret = SecretKey::new(&mut thread_rng());
    let sender_secret = SecretKey::new(&mut thread_rng());
    
    let recipient_pubkey = PublicKey::from_secret_key(&secp, &recipient_secret);
    let sender_pubkey = PublicKey::from_secret_key(&secp, &sender_secret);
    
    // Create HTLC contract
    let contract = create_htlc_contract(
        recipient_pubkey,
        sender_pubkey,
        &config.secret,
        config.timelock_blocks,
    );
    
    // Create script and address
    let htlc_script = contract.create_script();
    let address = Address::p2wsh(&htlc_script, config.network);
    
    Ok(DeploymentResult {
        contract,
        address,
        recipient_secret,
        sender_secret,
        hash_lock: sha256::Hash::hash(&config.secret),
    })
}

pub fn deploy_htlc_to_testnet() -> Result<(), Box<dyn std::error::Error>> {
    let config = DeploymentConfig::default();
    let result = deploy_htlc(config)?;
    
    println!("HTLC Contract Deployed Successfully!");
    println!("=====================================");
    println!("HTLC Script: {}", result.contract.create_script());
    println!("HTLC Address: {}", result.address);
    println!("Hash lock: {}", result.hash_lock);
    println!("Timelock: {} blocks", result.contract.timelock);
    println!();
    println!("To fund this HTLC:");
    println!("Send Bitcoin to: {}", result.address);
    println!();
    println!("To claim funds (recipient):");
    println!("Use the secret preimage: {:?}", b"my_secret_preimage");
    println!();
    println!("To refund (sender):");
    println!("Wait {} blocks then use sender private key", result.contract.timelock);
    
    Ok(())
}