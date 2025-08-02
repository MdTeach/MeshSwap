//! Bitcoin Wallet CLI - Main Entry Point
//!
//! A command-line interface for Bitcoin wallet operations with support for
//! balance checking, transactions, and address generation on regtest network.

mod args;
mod blockchain;
pub mod constants;
mod error;
mod primitives;
mod taproot;
mod transaction;
mod wallet;

use args::{Args, Commands};
use clap::Parser;
use std::str::FromStr;

use crate::blockchain::create_bitcoin_rpc_client;
use crate::primitives::SwapInfo;
use crate::taproot::{new_atomic_swap, withdraw_from_taproot_htlc};
use crate::transaction::send_bitcoin_to_address;
use crate::wallet::{BitcoinWallet, WalletFactory, format_satoshis_to_btc, btc_to_satoshis};
use bdk::bitcoin::secp256k1::SecretKey;

/// Application entry point
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cli_args = Args::parse();

    match cli_args.command {
        Commands::Balance {
            wallet: wallet_config_path,
        } => {
            handle_balance_command(wallet_config_path).await?;
        }
        Commands::Send {
            from: source_wallet_path,
            to: destination_wallet_path,
            amount: btc_amount,
        } => {
            handle_send_command(source_wallet_path, destination_wallet_path, btc_amount).await?;
        }
        Commands::Address {
            wallet: wallet_config_path,
        } => {
            handle_address_command(wallet_config_path).await?;
        }
        Commands::Swap {
            from: source_wallet_path,
            to: destination_wallet_path,
            amount: btc_amount,
            timelock,
        } => {
            handle_swap_command(
                source_wallet_path,
                destination_wallet_path,
                btc_amount,
                timelock,
            )
            .await?;
        }
        Commands::Withdraw {
            wallet: recipient_wallet_path,
            sender: sender_wallet_path,
            amount: btc_amount,
            timelock,
            swap_secret,
        } => {
            handle_withdraw_command(
                recipient_wallet_path,
                sender_wallet_path,
                btc_amount,
                timelock,
                swap_secret,
            )
            .await?;
        }
    }

    Ok(())
}

/// Handle the balance command - display wallet balance in both BTC and satoshis
async fn handle_balance_command(
    wallet_config_path: std::path::PathBuf,
) -> eyre::Result<()> {
    let balance_satoshis = WalletFactory::get_balance_satoshis(&wallet_config_path).await?;
    let balance_btc_formatted = format_satoshis_to_btc(balance_satoshis);

    println!(
        "Balance: {} BTC ({} sats)",
        balance_btc_formatted, balance_satoshis
    );
    Ok(())
}

/// Handle the send command - transfer Bitcoin between wallets
async fn handle_send_command(
    source_wallet_path: std::path::PathBuf,
    destination_wallet_path: std::path::PathBuf,
    btc_amount: f64,
) -> eyre::Result<()> {
    let source_wallet = BitcoinWallet::from_config_file(&source_wallet_path).await?;
    let destination_address = WalletFactory::get_address(&destination_wallet_path).await?;
    let amount_satoshis = btc_to_satoshis(btc_amount);
    let blockchain_client = create_bitcoin_rpc_client()?;

    let transaction_id = send_bitcoin_to_address(
        &blockchain_client,
        &source_wallet.wallet,
        destination_address,
        amount_satoshis,
    )
    .await?;

    println!("✅ Transaction sent successfully!");
    println!("📊 Amount: {} BTC ({} sats)", btc_amount, amount_satoshis);
    println!("🔗 Transaction ID: {}", transaction_id);

    Ok(())
}

/// Handle the address command - display wallet receiving address
async fn handle_address_command(
    wallet_config_path: std::path::PathBuf,
) -> eyre::Result<()> {
    let address = WalletFactory::get_address(&wallet_config_path).await?;
    println!("{}", address);
    Ok(())
}

/// Handle the swap command - create atomic swap HTLC
async fn handle_swap_command(
    source_wallet_path: std::path::PathBuf,
    destination_wallet_path: std::path::PathBuf,
    btc_amount: f64,
    timelock_blocks: u32,
) -> eyre::Result<()> {
    let source_wallet = BitcoinWallet::from_config_file(&source_wallet_path).await?;
    let (recipient_public_key, _) = WalletFactory::extract_keypair(&destination_wallet_path)?;
    let (revocation_public_key, _) = WalletFactory::extract_keypair(&source_wallet_path)?;
    let amount_satoshis = btc_to_satoshis(btc_amount);

    let mut swap_info = SwapInfo::new(
        recipient_public_key,
        revocation_public_key,
        timelock_blocks,
        amount_satoshis,
    );

    let blockchain_client = create_bitcoin_rpc_client()?;
    let mut rng = rand::thread_rng();

    println!("🔄 Creating atomic swap...");
    println!("📊 Amount: {} BTC ({} sats)", btc_amount, amount_satoshis);
    println!("⏰ Timelock: {} blocks", timelock_blocks);

    let transaction_id = new_atomic_swap(
        &blockchain_client,
        &source_wallet.wallet,
        &mut swap_info,
        &mut rng,
    )
    .await?;

    println!("✅ Atomic swap created successfully!");
    println!("🔗 Transaction ID: {}", transaction_id);

    Ok(())
}

/// Handle the withdraw command - withdraw funds from atomic swap HTLC
async fn handle_withdraw_command(
    recipient_wallet_path: std::path::PathBuf,
    sender_wallet_path: std::path::PathBuf,
    btc_amount: f64,
    timelock_blocks: u32,
    swap_secret_hex: String,
) -> eyre::Result<()> {
    let sender_wallet = BitcoinWallet::from_config_file(&sender_wallet_path).await?;
    let (recipient_public_key, recipient_secret_key) = WalletFactory::extract_keypair(&recipient_wallet_path)?;
    let (revocation_public_key, _) = WalletFactory::extract_keypair(&sender_wallet_path)?;
    
    let swap_secret_key = SecretKey::from_str(&swap_secret_hex)
        .map_err(|e| eyre::eyre!("Invalid swap secret key format: {}", e))?;
    
    let amount_satoshis = btc_to_satoshis(btc_amount);
    let swap_info = SwapInfo::new(
        recipient_public_key,
        revocation_public_key,
        timelock_blocks,
        amount_satoshis,
    );

    let blockchain_client = create_bitcoin_rpc_client()?;
    let destination_address = WalletFactory::get_address(&recipient_wallet_path).await?;

    println!("💰 Withdrawing from atomic swap...");
    println!("📊 Amount: {} BTC ({} sats)", btc_amount, amount_satoshis);
    println!("⏰ Original timelock: {} blocks", timelock_blocks);
    println!("🔑 Using swap secret: {}...", &swap_secret_hex[..16]);

    let transaction_id = withdraw_from_taproot_htlc(
        &blockchain_client,
        &sender_wallet.wallet,
        destination_address,
        &swap_info,
        &recipient_secret_key,
        &swap_secret_key,
    )
    .await?;

    println!("✅ Atomic swap withdrawal successful!");
    println!("🔗 Transaction ID: {}", transaction_id);

    Ok(())
}
