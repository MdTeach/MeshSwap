
//! Bitcoin Wallet CLI - Main Entry Point
//! 
//! A command-line interface for Bitcoin wallet operations with support for
//! balance checking, transactions, and address generation on regtest network.

mod args;
mod blockchain;
mod error;
mod transaction;
mod wallet;
pub mod constants;
mod taproot;

use args::{Args, Commands};
use clap::Parser;
use std::str::FromStr;
use bdk::bitcoin::Address;


use crate::blockchain::create_bitcoin_rpc_client;
use crate::wallet::{BitcoinWallet, format_satoshis_to_btc};
use crate::transaction::send_bitcoin_to_address;

/// Application entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args = Args::parse();
    
    match cli_args.command {
        Commands::Balance { wallet: wallet_config_path } => {
            handle_balance_command(wallet_config_path).await?;
        }
        Commands::Send { from: source_wallet_path, to: destination_wallet_path, amount: btc_amount } => {
            handle_send_command(source_wallet_path, destination_wallet_path, btc_amount).await?;
        }
        Commands::Address { wallet: wallet_config_path } => {
            handle_address_command(wallet_config_path).await?;
        }
    }

    Ok(())
}

/// Handle the balance command - display wallet balance in both BTC and satoshis
async fn handle_balance_command(wallet_config_path: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let bitcoin_wallet = BitcoinWallet::from_config_file(&wallet_config_path).await?;
    let balance_satoshis = bitcoin_wallet.get_balance_satoshis().await?;
    let balance_btc_formatted = format_satoshis_to_btc(balance_satoshis);
    
    println!("Balance: {} BTC ({} sats)", balance_btc_formatted, balance_satoshis);
    Ok(())
}

/// Handle the send command - transfer Bitcoin between wallets
async fn handle_send_command(
    source_wallet_path: std::path::PathBuf,
    destination_wallet_path: std::path::PathBuf,
    btc_amount: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load source wallet
    let source_wallet = BitcoinWallet::from_config_file(&source_wallet_path).await?;
    
    // Get destination wallet address
    let destination_wallet = BitcoinWallet::from_config_file(&destination_wallet_path).await?;
    let destination_address_string = destination_wallet.get_receiving_address()?;
    let destination_address = Address::from_str(&destination_address_string)?;
    
    // Convert BTC amount to satoshis
    let amount_satoshis = (btc_amount * 100_000_000.0) as u64;
    
    // Initialize blockchain client
    let blockchain_client = create_bitcoin_rpc_client()?;
    
    // Send transaction using the new transaction module
    let transaction_id = send_bitcoin_to_address(
        &blockchain_client,
        &source_wallet.wallet,
        destination_address,
        amount_satoshis
    ).await?;
    
    println!("✅ Transaction sent successfully!");
    println!("📊 Amount: {} BTC ({} sats)", btc_amount, amount_satoshis);
    println!("🔗 Transaction ID: {}", transaction_id);
    
    Ok(())
}

/// Handle the address command - display wallet receiving address
async fn handle_address_command(wallet_config_path: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let bitcoin_wallet = BitcoinWallet::from_config_file(&wallet_config_path).await?;
    let receiving_address = bitcoin_wallet.get_receiving_address()?;
    
    println!("{}", receiving_address);
    Ok(())
}

