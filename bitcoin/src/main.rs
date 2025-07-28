mod contract;
mod deployment;

use deployment::deploy_htlc_to_testnet;

fn main() {
    println!("Bitcoin HTLC Contract System");
    println!("============================");
    
    match deploy_htlc_to_testnet() {
        Ok(()) => println!("\n✅ Deployment completed successfully!"),
        Err(e) => println!("❌ Error deploying HTLC: {}", e),
    }
}
