use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the wallet.toml configuration file
    #[arg(short, long)]
    pub wallet: PathBuf,
}