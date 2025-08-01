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
}