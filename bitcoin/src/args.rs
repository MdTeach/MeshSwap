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
}