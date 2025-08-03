//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use clap::Parser;
use proofimpl_atomic_swap::BitcoinSwap;
use sp1_sdk::{include_elf, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};
use std::fs;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const ATOMIC_SWAP_ELF: &[u8] = include_elf!("atomic-swap-program");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,

    #[arg(long)]
    verify: bool,

    #[arg(long, default_value = "proof.bin")]
    proof_file: String,

    #[arg(long, default_value = "vkey.bin")]
    vkey_file: String,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Parse the command line arguments.
    let args = Args::parse();

    let mode_count = [args.execute, args.prove, args.verify]
        .iter()
        .filter(|&&x| x)
        .count();
    if mode_count != 1 {
        eprintln!("❌ Error: You must specify exactly one mode: --execute, --prove, or --verify");
        std::process::exit(1);
    }

    println!("🚀 Starting Atomic Swap Prover!");

    // Setup the prover client.
    let client = ProverClient::from_env();
    println!("✅ Prover client initialized!");

    if args.verify {
        verify_mode(&args);
        return;
    }

    // Load and parse the JSON file
    let json_path = "../../bitcoin/swaps/swap_bitcoin.json";
    println!("📂 Loading swap data from: {}", json_path);
    let json_content = fs::read_to_string(json_path).expect("❌ Failed to read JSON file");
    let bitcoin_swap: BitcoinSwap =
        serde_json::from_str(&json_content).expect("❌ Failed to parse JSON");

    println!("🔍 Loaded Bitcoin swap: {:?}", bitcoin_swap.swap_info);

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&bitcoin_swap.swap_secret);

    println!("🔐 Using swap secret: {}", bitcoin_swap.swap_secret);

    if args.execute {
        execute_mode(&client, &stdin);
    } else if args.prove {
        prove_mode(&client, &stdin, &args);
    }
}

fn execute_mode(client: &sp1_sdk::EnvProver, stdin: &SP1Stdin) {
    println!("⚡ Executing program...");
    let (_output, _report) = client.execute(ATOMIC_SWAP_ELF, stdin).run().unwrap();
    println!("✅ Program executed successfully! 🎉");
}

fn prove_mode(client: &sp1_sdk::EnvProver, stdin: &SP1Stdin, args: &Args) {
    println!("🔧 Setting up proving system...");
    let (pk, vk) = client.setup(ATOMIC_SWAP_ELF);
    println!("✅ Proving key and verifying key generated!");

    // Save the verifying key
    println!("💾 Saving verifying key to: {}", args.vkey_file);
    let vkey_bytes = bincode::serialize(&vk).expect("❌ Failed to serialize verifying key");
    fs::write(&args.vkey_file, vkey_bytes).expect("❌ Failed to write verifying key file");
    println!("✅ Verifying key saved! 🔑");

    // Generate the proof
    println!("🧮 Generating proof... (this may take a while)");
    let proof = client
        .prove(&pk, stdin)
        .run()
        .expect("❌ Failed to generate proof");

    println!("✅ Successfully generated proof! 🎊");

    // Save the proof
    println!("💾 Saving proof to: {}", args.proof_file);
    let proof_bytes = bincode::serialize(&proof).expect("❌ Failed to serialize proof");
    fs::write(&args.proof_file, proof_bytes).expect("❌ Failed to write proof file");
    println!("✅ Proof saved successfully! 📦");

    // Verify the proof immediately
    println!("🔍 Verifying proof...");
    client
        .verify(&proof, &vk)
        .expect("❌ Failed to verify proof");
    println!("✅ Proof verified successfully! 🎉🔒");
}

fn verify_mode(args: &Args) {
    println!("🔍 Entering verification mode...");

    // Load the proof
    println!("📂 Loading proof from: {}", args.proof_file);
    let proof_bytes = fs::read(&args.proof_file).expect("❌ Failed to read proof file");
    let proof: SP1ProofWithPublicValues =
        bincode::deserialize(&proof_bytes).expect("❌ Failed to deserialize proof");
    println!("✅ Proof loaded! 📋");

    // Load the verifying key
    println!("📂 Loading verifying key from: {}", args.vkey_file);
    let vkey_bytes = fs::read(&args.vkey_file).expect("❌ Failed to read verifying key file");
    let vk: SP1VerifyingKey =
        bincode::deserialize(&vkey_bytes).expect("❌ Failed to deserialize verifying key");
    println!("✅ Verifying key loaded! 🔑");

    // Verify the proof
    println!("🔍 Verifying proof...");
    let client = ProverClient::from_env();
    client
        .verify(&proof, &vk)
        .expect("❌ Proof verification failed!");
    println!("✅ Proof verified successfully! 🎉🔒✨");

    // Extract and save public parameters
    println!("📝 Extracting public parameters...");
    let mut public_values = proof.public_values.clone();
    let public_params: proofimpl_atomic_swap::PublicParams = public_values.read();
    println!("🔍 Public params extracted: {:?}", public_params);

    // Save public params as JSON
    let public_params_json =
        serde_json::to_string_pretty(&public_params).expect("❌ Failed to serialize public params");
    let output_file = "public_params.json";
    fs::write(output_file, public_params_json).expect("❌ Failed to write public params file");
    println!("💾 Public parameters saved to: {} ✨", output_file);
}
