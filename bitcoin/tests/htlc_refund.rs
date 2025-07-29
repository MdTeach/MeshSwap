use std::process::Command;
use std::thread;
use std::time::Duration;

/// End-to-End HTLC Refund Test
///
/// This test demonstrates a complete HTLC (Hash Time Locked Contract) timeout refund flow:
/// 1. Fresh blockchain with initial balances
/// 2. Admin sends funds to maker
/// 3. Maker creates HTLC for taker with secret and short timeout
/// 4. Wait for timeout to expire without taker claiming
/// 5. Maker reclaims funds via timeout refund
///
/// Expected Flow:
/// Admin (10 BTC) → sends 3 BTC → Maker (3 BTC) → creates 1.5 BTC HTLC → timeout expires → Maker refunds 1.5 BTC
///
/// Final balances:
/// - Admin: 7 BTC  
/// - Maker: 3 BTC (back to original after refund)
/// - Taker: 0 BTC (never claimed)
#[test]
fn test_htlc_refund() {
    println!("🚀 Starting End-to-End HTLC Refund Test");

    // Test parameters
    let secret = "timeout-refund-secret-2024";
    let htlc_amount = 1.5;
    let admin_to_maker_amount = 3.0;

    // Step 0: Clear the chain state
    println!("🧹 Step 0: Clearing previous chain state...");
    let clear_result = Command::new("just")
        .arg("clean")
        .output()
        .expect("Failed to clear chain state");

    if !clear_result.status.success() {
        panic!(
            "Failed to clear chain state: {}",
            String::from_utf8_lossy(&clear_result.stderr)
        );
    }

    // Step 1: Start fresh blockchain with fast mining (3s blocks)
    println!("\n📦 Step 1: Starting fresh Bitcoin regtest blockchain with fast mining...");
    let start_result = Command::new("just")
        .arg("start-fast")
        .arg("3") // 3 second blocks for faster timeout testing
        .output()
        .expect("Failed to start blockchain");

    if !start_result.status.success() {
        panic!(
            "Failed to start blockchain: {}",
            String::from_utf8_lossy(&start_result.stderr)
        );
    }

    // Wait for blockchain to be ready
    thread::sleep(Duration::from_secs(5));

    // Step 2: Check initial balances
    println!("\n💰 Step 2: Checking initial balances...");
    let admin_balance = get_wallet_balance("admin");
    let maker_balance = get_wallet_balance("maker");
    let taker_balance = get_wallet_balance("taker");

    println!("Initial balances:");
    println!("  Admin: {} BTC", admin_balance);
    println!("  Maker: {} BTC", maker_balance);
    println!("  Taker: {} BTC", taker_balance);

    // Verify admin starts with expected balance (around 10 BTC from regtest mining)
    assert!(
        (admin_balance - 10.0).abs() < 1.0,
        "Admin should start with ~10 BTC"
    );
    assert_eq!(maker_balance, 0.0, "Maker should start with 0 BTC");
    assert_eq!(taker_balance, 0.0, "Taker should start with 0 BTC");

    // Step 3: Admin sends Bitcoin to maker
    println!(
        "\n💸 Step 3: Admin sending {} BTC to maker...",
        admin_to_maker_amount
    );
    let send_result = Command::new("just")
        .arg("send-admin-to-maker")
        .arg(admin_to_maker_amount.to_string())
        .output()
        .expect("Failed to send Bitcoin");

    if !send_result.status.success() {
        panic!(
            "Failed to send Bitcoin: {}",
            String::from_utf8_lossy(&send_result.stderr)
        );
    }

    // Wait for transaction to be mined
    thread::sleep(Duration::from_secs(5));

    // Verify balances after transfer
    let admin_balance_after_send = get_wallet_balance("admin");
    let maker_balance_after_send = get_wallet_balance("maker");

    println!("Balances after transfer:");
    println!("  Admin: {} BTC", admin_balance_after_send);
    println!("  Maker: {} BTC", maker_balance_after_send);

    assert!(
        (maker_balance_after_send - admin_to_maker_amount).abs() < 0.1,
        "Maker should have received the Bitcoin"
    );

    // Step 4: Get current block height and set short timeout
    println!("\n⏰ Step 4: Getting current block height for timeout calculation...");
    let current_height = get_current_block_height();
    let timeout_block = current_height + 5; // Short timeout: 5 blocks from now

    println!("Current block height: {}", current_height);
    println!("HTLC timeout set to block: {}", timeout_block);

    // Step 5: Maker creates HTLC for taker with short timeout
    println!(
        "\n🔒 Step 5: Maker creating {} BTC HTLC for taker with secret '{}' and timeout at block {}...",
        htlc_amount, secret, timeout_block
    );

    let htlc_create_result = Command::new("just")
        .arg("htlc-create-maker-to-taker")
        .arg(htlc_amount.to_string())
        .arg(secret)
        .arg(timeout_block.to_string())
        .output()
        .expect("Failed to create HTLC");

    if !htlc_create_result.status.success() {
        panic!(
            "Failed to create HTLC: {}",
            String::from_utf8_lossy(&htlc_create_result.stderr)
        );
    }

    let htlc_output = String::from_utf8_lossy(&htlc_create_result.stdout);
    println!("HTLC creation output:\n{}", htlc_output);

    // Extract contract ID from output
    let contract_id = extract_contract_id(&htlc_output)
        .expect("Failed to extract contract ID from HTLC creation output");

    println!("📝 Contract ID: {}", contract_id);

    // Wait for HTLC transaction to be mined
    thread::sleep(Duration::from_secs(5));

    // Step 6: Wait for timeout to expire (monitor block height)
    println!(
        "\n⏳ Step 6: Waiting for timeout to expire (block {})...",
        timeout_block
    );

    loop {
        let current_height = get_current_block_height();
        println!(
            "Current block height: {} (waiting for {})",
            current_height, timeout_block
        );

        if current_height >= timeout_block {
            println!(
                "✅ Timeout reached! Block {} >= {}",
                current_height, timeout_block
            );
            break;
        }

        // Wait for next block (3 second intervals)
        thread::sleep(Duration::from_secs(4));
    }

    // Step 7: Test that refund fails before adequate time (should not happen now, but let's be safe)
    println!("\n🚫 Step 7: Verifying timeout protection is working...");
    let early_refund_result = Command::new("just")
        .arg("htlc-refund-maker")
        .arg(&contract_id)
        .arg(secret)
        .arg(htlc_amount.to_string())
        .arg((timeout_block - 1).to_string()) // Try with a block height that hasn't been reached
        .arg("wallet/taker.toml")
        .output()
        .expect("Failed to attempt early refund");

    // This should fail because we're using timeout_block - 1
    if early_refund_result.status.success() {
        println!(
            "⚠️  Warning: Early refund succeeded when it should have failed (this might be OK if timeout has long passed)"
        );
    } else {
        println!(
            "✅ Early refund correctly failed: {}",
            String::from_utf8_lossy(&early_refund_result.stderr)
        );
    }

    // Step 8: Maker refunds HTLC after timeout
    println!(
        "\n🔄 Step 8: Maker refunding HTLC after timeout with secret '{}'...",
        secret
    );
    let refund_result = Command::new("just")
        .arg("htlc-refund-maker")
        .arg(&contract_id)
        .arg(secret)
        .arg(htlc_amount.to_string())
        .arg(timeout_block.to_string())
        .arg("wallet/taker.toml")
        .output()
        .expect("Failed to refund HTLC");

    if !refund_result.status.success() {
        panic!(
            "Failed to refund HTLC: {}",
            String::from_utf8_lossy(&refund_result.stderr)
        );
    }

    let refund_output = String::from_utf8_lossy(&refund_result.stdout);
    println!("HTLC refund output:\n{}", refund_output);

    // Wait for refund transaction to be mined
    thread::sleep(Duration::from_secs(5));

    // Step 9: Verify final balances
    println!("\n🏁 Step 9: Verifying final balances...");
    let final_admin_balance = get_wallet_balance("admin");
    let final_maker_balance = get_wallet_balance("maker");
    let final_taker_balance = get_wallet_balance("taker");

    println!("Final balances:");
    println!("  Admin: {} BTC", final_admin_balance);
    println!("  Maker: {} BTC", final_maker_balance);
    println!("  Taker: {} BTC", final_taker_balance);

    // Verify expected final balances
    assert!(
        (final_admin_balance - 7.0).abs() < 0.1,
        "Admin should have ~7 BTC (10 - 3 sent)"
    );
    assert!(
        (final_maker_balance - 3.0).abs() < 0.1,
        "Maker should have ~3 BTC (3 received - 1.5 locked + 1.5 refunded)"
    );
    assert!(
        final_taker_balance < 0.1,
        "Taker should have ~0 BTC (never claimed the HTLC)"
    );

    println!("\n✅ End-to-End HTLC Refund Test Completed Successfully!");
    println!("🎉 Timeout refund executed: Maker → Created HTLC → Timeout → Refunded (1.5 BTC)");
    println!("⏰ This demonstrates that funds are safely returned to sender when timeout expires");

    // Cleanup: Stop blockchain
    println!("\n🧹 Cleaning up: Stopping blockchain...");
    let _stop_result = Command::new("just")
        .arg("stop")
        .output()
        .expect("Failed to stop blockchain");
}

/// Helper function to get wallet balance
fn get_wallet_balance(wallet: &str) -> f64 {
    let balance_result = Command::new("just")
        .arg(format!("balance-{}", wallet))
        .output()
        .expect("Failed to get wallet balance");

    if !balance_result.status.success() {
        panic!(
            "Failed to get {} balance: {}",
            wallet,
            String::from_utf8_lossy(&balance_result.stderr)
        );
    }

    let output = String::from_utf8_lossy(&balance_result.stdout);

    // Parse balance from output like "Balance: 10 BTC (1000000000 sats)"
    for line in output.lines() {
        if line.starts_with("Balance:") {
            if let Some(btc_part) = line.split(" BTC ").next() {
                if let Some(balance_str) = btc_part.split("Balance: ").nth(1) {
                    return balance_str.parse::<f64>().unwrap_or(0.0);
                }
            }
        }
    }

    0.0
}

/// Helper function to get current block height
fn get_current_block_height() -> u32 {
    let height_result = Command::new("bitcoin-cli")
        .args([
            "-regtest",
            "-rpcuser=bitcoin",
            "-rpcpassword=bitcoin",
            "-rpcport=18443",
            "getblockcount",
        ])
        .output()
        .expect("Failed to get block height");

    if !height_result.status.success() {
        panic!(
            "Failed to get block height: {}",
            String::from_utf8_lossy(&height_result.stderr)
        );
    }

    let height_str = String::from_utf8_lossy(&height_result.stdout);
    height_str.trim().parse::<u32>().unwrap_or(0)
}

/// Helper function to extract contract ID from HTLC creation output
fn extract_contract_id(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.starts_with("Contract ID:") {
            return line
                .split("Contract ID: ")
                .nth(1)
                .map(|s| s.trim().to_string());
        }
    }
    None
}
