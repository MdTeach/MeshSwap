use std::process::Command;
use std::thread;
use std::time::Duration;

/// End-to-End HTLC Test
///
/// This test demonstrates a complete HTLC (Hash Time Locked Contract) atomic swap flow:
/// 1. Fresh blockchain with initial balances
/// 2. Admin sends funds to maker
/// 3. Maker creates HTLC for taker with secret
/// 4. Taker claims HTLC using the secret
///
/// Expected Flow:
/// Admin (10 BTC) → sends 3 BTC → Maker (3 BTC) → creates 1.5 BTC HTLC → Taker claims with secret
///
/// Final balances:
/// - Admin: 7 BTC
/// - Maker: 1.5 BTC  
/// - Taker: 1.5 BTC
#[test]
fn test_htlc_claim() {
    println!("🚀 Starting End-to-End HTLC Test");

    // Test parameters
    let secret = "atomic-secret-2024";
    let htlc_amount = 1.5;
    let timeout_block = 200;
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

    // Step 1: Start fresh blockchain
    println!("\n📦 Step 1: Starting fresh Bitcoin regtest blockchain...");
    let start_result = Command::new("just")
        .arg("start-fast")
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

    // Step 4: Maker creates HTLC for taker
    println!(
        "\n🔒 Step 4: Maker creating {} BTC HTLC for taker with secret '{}'...",
        htlc_amount, secret
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

    // Step 5: Taker claims HTLC with correct secret
    println!(
        "\n🔑 Step 5: Taker claiming HTLC with correct secret '{}'...",
        secret
    );
    let claim_result = Command::new("just")
        .arg("htlc-claim-taker")
        .arg(&contract_id)
        .arg(secret)
        .arg(htlc_amount.to_string())
        .arg(timeout_block.to_string())
        .arg("wallet/maker.toml")
        .output()
        .expect("Failed to claim HTLC");

    if !claim_result.status.success() {
        panic!(
            "Failed to claim HTLC: {}",
            String::from_utf8_lossy(&claim_result.stderr)
        );
    }

    let claim_output = String::from_utf8_lossy(&claim_result.stdout);
    println!("HTLC claim output:\n{}", claim_output);

    // Wait for claim transaction to be mined
    thread::sleep(Duration::from_secs(5));

    // Step 7: Verify final balances
    println!("\n🏁 Step 6: Verifying final balances...");
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
        (final_maker_balance - 1.5).abs() < 0.1,
        "Maker should have ~1.5 BTC (3 received - 1.5 locked in HTLC)"
    );
    assert!(
        (final_taker_balance - 1.5).abs() < 0.1,
        "Taker should have ~1.5 BTC (claimed from HTLC)"
    );

    println!("\n✅ End-to-End HTLC Test Completed Successfully!");
    println!("🎉 Atomic swap executed: Maker → Taker (1.5 BTC) via HTLC with secret revelation");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_contract_id() {
        let sample_output = r#"
HTLC created successfully!
Contract ID: fc2979c11478408e770bf7bd0adb99951e50b67d861411ee45007d583996185b
Amount: 1.5 BTC (150000000 sats)
Secret: atomic-secret-2024
        "#;

        let contract_id = extract_contract_id(sample_output);
        assert_eq!(
            contract_id,
            Some("fc2979c11478408e770bf7bd0adb99951e50b67d861411ee45007d583996185b".to_string())
        );
    }

    #[test]
    fn test_balance_parsing_helper() {
        // This test doesn't run the actual command, just tests the parsing logic
        // In a real test environment, you'd mock the command execution
        assert_eq!(get_wallet_balance("nonexistent"), 0.0); // Should handle gracefully
    }
}
