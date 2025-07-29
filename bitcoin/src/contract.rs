use bdk::bitcoin::{
    Address, Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
    hashes::{Hash, sha256},
    opcodes::all::*,
    script::Builder,
    secp256k1::{Message, PublicKey, Secp256k1, SecretKey},
    sighash::{EcdsaSighashType, SighashCache},
};

pub struct HTLCContract {
    pub recipient_pubkey: PublicKey,
    pub sender_pubkey: PublicKey,
    pub hash_lock: sha256::Hash,
    pub timelock: u32,
}

impl HTLCContract {
    pub fn new(
        recipient_pubkey: PublicKey,
        sender_pubkey: PublicKey,
        hash_lock: sha256::Hash,
        timelock: u32,
    ) -> Self {
        Self {
            recipient_pubkey,
            sender_pubkey,
            hash_lock,
            timelock,
        }
    }

    /// Creates a Hash Time Locked Contract (HTLC) Bitcoin script.
    ///
    /// ```
    ///                    ┌─────────────────────────────────────┐
    ///                    │          HTLC CONTRACT              │
    ///                    │    ₿ Locked in Smart Contract       │
    ///                    └─────────────────┬───────────────────┘
    ///                                      │
    ///                         ┌────────────▼────────────┐
    ///                         │     Two Spending Paths  │
    ///                         └─────────┬───────┬───────┘
    ///                                   │       │
    ///                    ┌──────────────▼─┐   ┌─▼──────────────┐
    ///                    │   SECRET PATH  │   │  TIMEOUT PATH  │
    ///                    │                │   │                │
    ///      ┌─────────────┤  🔑 + 📝 sig   │   │  ⏰ + 📝 sig   ├──────────────┐
    ///      │             └────────────────┘   └────────────────┘              │
    ///      │                                                                  │
    ///  ┌───▼────┐                                                      ┌─────▼───┐
    ///  │ 👤 BOB │◄──── knows secret ────┐          ┌──── after timeout ────►│ 👤 ALICE │
    ///  │(recipient)                     │          │                        │(sender) │
    ///  └────────┘                       │          │                        └─────────┘
    ///                                   │          │
    ///                              ┌────▼──────────▼────┐
    ///                              │   BITCOIN SCRIPT    │
    ///                              │                     │
    ///                              │ IF (secret provided)│
    ///                              │   ✓ hash matches    │
    ///                              │   ✓ Bob's signature │
    ///                              │ ELSE (timeout)      │
    ///                              │   ✓ time passed     │
    ///                              │   ✓ Alice signature │
    ///                              │ ENDIF               │
    ///                              └─────────────────────┘
    /// ```
    ///
    /// **How it works**: Alice locks Bitcoin that Bob can claim with a secret,
    /// or Alice gets it back after timeout. No trust needed - blockchain enforces it!
    pub fn create_script(&self) -> ScriptBuf {
        Builder::new()
            // Path 1: Recipient claims with secret preimage
            .push_opcode(OP_IF)
            .push_opcode(OP_SHA256) // Hash the provided preimage
            .push_slice(self.hash_lock.as_byte_array()) // Expected hash (commitment)
            .push_opcode(OP_EQUALVERIFY) // Verify preimage hashes correctly
            .push_slice(&self.recipient_pubkey.serialize()) // Recipient's public key
            .push_opcode(OP_CHECKSIG) // Verify recipient signed this transaction
            // Path 2: Sender reclaims after timelock expires
            .push_opcode(OP_ELSE)
            .push_int(self.timelock as i64) // Minimum block height for refund
            .push_opcode(OP_CLTV) // Check Lock Time Verify (timelock enforcement)
            .push_opcode(OP_DROP) // Remove timelock from stack
            .push_slice(&self.sender_pubkey.serialize()) // Sender's public key
            .push_opcode(OP_CHECKSIG) // Verify sender signed this transaction
            .push_opcode(OP_ENDIF) // End conditional paths
            .into_script()
    }

    /// Creates a claim transaction for the recipient to spend the HTLC with the secret
    pub fn create_claim_transaction(
        &self,
        htlc_txid: Txid,
        htlc_vout: u32,
        amount: Amount,
        recipient_address: &Address,
        secret: &[u8],
        recipient_privkey: &SecretKey,
    ) -> Result<Transaction, String> {
        let secp = Secp256k1::new();
        let htlc_script = self.create_script();

        // Create the transaction
        let mut tx = Transaction {
            version: 2,
            lock_time: bdk::bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: htlc_txid,
                    vout: htlc_vout,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: amount.to_sat() - 1000, // Subtract fee in sats
                script_pubkey: recipient_address.script_pubkey(),
            }],
        };

        // Create sighash for signing
        let mut sighash_cache = SighashCache::new(&mut tx);
        let sighash = sighash_cache
            .segwit_signature_hash(0, &htlc_script, amount.to_sat(), EcdsaSighashType::All)
            .map_err(|e| format!("Failed to create sighash: {}", e))?;

        // Sign the transaction
        let message = Message::from_slice(sighash.as_byte_array())
            .map_err(|e| format!("Failed to create message: {}", e))?;
        let signature = secp.sign_ecdsa(&message, recipient_privkey);
        let mut sig_bytes = signature.serialize_der().to_vec();
        sig_bytes.push(EcdsaSighashType::All as u8);

        // Build witness stack for secret path (IF branch)
        // P2WSH witness stack: [signature] [preimage] [1] [witness_script]
        let mut witness = Witness::new();
        witness.push(&sig_bytes); // Signature for recipient
        witness.push(secret); // Secret preimage
        witness.push(&[1]); // TRUE for IF branch
        witness.push(htlc_script.as_bytes()); // The actual witness script

        tx.input[0].witness = witness;
        Ok(tx)
    }

    /// Creates a refund transaction for the sender to reclaim the HTLC after timeout
    pub fn create_refund_transaction(
        &self,
        htlc_txid: Txid,
        htlc_vout: u32,
        amount: Amount,
        sender_address: &Address,
        sender_privkey: &SecretKey,
    ) -> Result<Transaction, String> {
        let secp = Secp256k1::new();
        let htlc_script = self.create_script();

        // Create the transaction
        let mut tx = Transaction {
            version: 2,
            lock_time: bdk::bitcoin::absolute::LockTime::from_height(self.timelock)
                .map_err(|e| format!("Invalid timelock height: {}", e))?,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: htlc_txid,
                    vout: htlc_vout,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: amount.to_sat() - 1000, // Subtract fee in sats
                script_pubkey: sender_address.script_pubkey(),
            }],
        };

        // Create sighash for signing
        let mut sighash_cache = SighashCache::new(&mut tx);
        let sighash = sighash_cache
            .segwit_signature_hash(0, &htlc_script, amount.to_sat(), EcdsaSighashType::All)
            .map_err(|e| format!("Failed to create sighash: {}", e))?;

        // Sign the transaction
        let message = Message::from_slice(sighash.as_byte_array())
            .map_err(|e| format!("Failed to create message: {}", e))?;
        let signature = secp.sign_ecdsa(&message, sender_privkey);
        let mut sig_bytes = signature.serialize_der().to_vec();
        sig_bytes.push(EcdsaSighashType::All as u8);

        // Build witness stack for timeout path (ELSE branch)
        // P2WSH witness stack: [signature] [0] [witness_script]
        let mut witness = Witness::new();
        witness.push(&sig_bytes); // Signature for sender
        witness.push(&[]); // FALSE for ELSE branch (empty array = false)
        witness.push(htlc_script.as_bytes()); // The actual witness script

        tx.input[0].witness = witness;
        Ok(tx)
    }
}

pub fn create_htlc_contract(
    recipient_pubkey: PublicKey,
    sender_pubkey: PublicKey,
    secret: &[u8],
    timelock_blocks: u32,
) -> HTLCContract {
    let hash_lock = sha256::Hash::hash(secret);
    HTLCContract::new(recipient_pubkey, sender_pubkey, hash_lock, timelock_blocks)
}
