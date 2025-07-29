use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the wallet.toml configuration file
    #[arg(short, long)]
    pub wallet: PathBuf,
    
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get wallet balance
    Balance,
    /// Get wallet address
    Address,
    /// Send BTC to another wallet
    Send {
        /// Recipient wallet TOML file
        #[arg(short, long)]
        to: PathBuf,
        /// Amount in BTC to send
        #[arg(short, long)]
        amount: f64,
    },
    /// Create Hash Time Locked Contract (HTLC)
    HtlcCreate {
        /// Recipient wallet TOML file
        #[arg(short, long)]
        to: PathBuf,
        /// Amount in BTC to lock
        #[arg(short, long)]
        amount: f64,
        /// Secret text for hash lock
        #[arg(short, long)]
        secret: String,
        /// Absolute block height for timeout
        #[arg(short = 'b', long)]
        timeout_block: u32,
    },
    /// Claim HTLC with secret
    HtlcClaim {
        /// Contract ID (transaction hash)
        #[arg(short, long)]
        contract_id: String,
        /// Secret to unlock the HTLC
        #[arg(short, long)]
        secret: String,
        /// Amount in BTC (must match original HTLC amount)
        #[arg(short, long)]
        amount: f64,
        /// Timeout block height (must match original HTLC timeout)
        #[arg(short = 'b', long)]
        timeout_block: u32,
        /// Original sender wallet TOML file (who created the HTLC)
        #[arg(short = 'f', long)]
        from: PathBuf,
    },
}