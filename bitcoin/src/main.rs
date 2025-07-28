mod contract;
mod deployment;
mod wallet;

use deployment::deploy_htlc_to_testnet;
use wallet::{WalletManager, WalletType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Bitcoin HTLC Contract System");
    println!("============================");
    
    let mut wallet_manager = WalletManager::new().await?;
    
    println!("\nInitial Wallet Balances:");
    println!("========================");
    
    let admin_balance = wallet_manager.get_balance(WalletType::Admin).await?;
    println!("Admin (Miner): {} sats", admin_balance);
    
    let maker_balance = wallet_manager.get_balance(WalletType::Maker).await?;
    println!("Maker: {} sats", maker_balance);
    
    let taker_balance = wallet_manager.get_balance(WalletType::Taker).await?;
    println!("Taker: {} sats", taker_balance);
    
    // Mine some blocks to fund the admin wallet
    println!("\nMining blocks to fund admin wallet...");
    wallet_manager.mine_blocks(101).await?; // Need 101 blocks for coinbase maturity
    
    // Sync and check balances again
    wallet_manager.sync_wallets().await?;
    
    println!("\nWallet Balances After Mining:");
    println!("=============================");
    
    let admin_balance = wallet_manager.get_balance(WalletType::Admin).await?;
    println!("Admin (Miner): {} sats", admin_balance);
    
    let maker_balance = wallet_manager.get_balance(WalletType::Maker).await?;
    println!("Maker: {} sats", maker_balance);
    
    let taker_balance = wallet_manager.get_balance(WalletType::Taker).await?;
    println!("Taker: {} sats", taker_balance);
    
    // Show wallet addresses
    println!("\nWallet Addresses:");
    println!("=================");
    println!("Admin: {}", wallet_manager.get_address(WalletType::Admin).await?);
    println!("Maker: {}", wallet_manager.get_address(WalletType::Maker).await?);
    println!("Taker: {}", wallet_manager.get_address(WalletType::Taker).await?);
    
    println!("\nHTLC Deployment:");
    println!("================");
    match deploy_htlc_to_testnet() {
        Ok(()) => println!("✅ Deployment completed successfully!"),
        Err(e) => println!("❌ Error deploying HTLC: {}", e),
    }
    
    Ok(())
}
