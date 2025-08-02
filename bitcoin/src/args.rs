use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bitcoin-cli")]
#[command(about = "Bitcoin wallet CLI with HTLC support")]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Get wallet balance
    Balance {
        /// Path to wallet config file
        #[arg(short, long)]
        wallet: PathBuf,
    },
    /// Send Bitcoin to another wallet
    Send {
        /// Source wallet config file
        #[arg(short, long)]
        from: PathBuf,
        /// Destination wallet config file
        #[arg(short, long)]
        to: PathBuf,
        /// Amount in BTC to send
        #[arg(short, long)]
        amount: f64,
    },
    /// Get wallet address
    Address {
        /// Path to wallet config file
        #[arg(short, long)]
        wallet: PathBuf,
    },
    /// Create atomic swap HTLC
    Swap {
        /// Source wallet config file
        #[arg(short, long)]
        from: PathBuf,
        /// Destination wallet config file
        #[arg(short, long)]
        to: PathBuf,
        /// Amount in BTC to swap
        #[arg(short, long)]
        amount: f64,
        /// Timelock duration in blocks
        #[arg(long, default_value = "144")]
        timelock: u32,
    },
    /// Withdraw from atomic swap HTLC
    Withdraw {
        /// Recipient wallet config file (claiming the funds)
        #[arg(short, long)]
        wallet: PathBuf,
        /// Original sender wallet config file (for network info)
        #[arg(short, long)]
        sender: PathBuf,
        /// Amount in BTC of the original swap
        #[arg(short, long)]
        amount: f64,
        /// Timelock duration in blocks from original swap
        #[arg(long, default_value = "144")]
        timelock: u32,
        /// Swap secret key (hex string from swap creation)
        #[arg(long)]
        swap_secret: String,
    },
}