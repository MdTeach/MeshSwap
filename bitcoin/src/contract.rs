use bitcoin::{
    Amount, OutPoint, Script, ScriptBuf, Transaction, TxIn, TxOut, Txid, Witness,
    hashes::{Hash, sha256},
    opcodes::all::*,
    script::Builder,
    secp256k1::PublicKey,
    transaction::Version,
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
                .push_opcode(OP_SHA256)                           // Hash the provided preimage
                .push_slice(self.hash_lock.as_byte_array())       // Expected hash (commitment)
                .push_opcode(OP_EQUALVERIFY)                     // Verify preimage hashes correctly
                .push_slice(&self.recipient_pubkey.serialize())   // Recipient's public key
                .push_opcode(OP_CHECKSIG)                        // Verify recipient signed this transaction
            // Path 2: Sender reclaims after timelock expires
            .push_opcode(OP_ELSE)
                .push_int(self.timelock as i64)                  // Minimum block height for refund
                .push_opcode(OP_CLTV)                            // Check Lock Time Verify (timelock enforcement)
                .push_opcode(OP_DROP)                            // Remove timelock from stack
                .push_slice(&self.sender_pubkey.serialize())     // Sender's public key
                .push_opcode(OP_CHECKSIG)                        // Verify sender signed this transaction
            .push_opcode(OP_ENDIF)                               // End conditional paths
            .into_script()
    }

    pub fn create_funding_transaction(
        &self,
        funding_txid: Txid,
        funding_vout: u32,
        amount: Amount,
        recipient_script_pubkey: &Script,
    ) -> Transaction {
        Transaction {
            version: Version(2),
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: funding_txid,
                    vout: funding_vout,
                },
                script_sig: Script::new().into(),
                sequence: bitcoin::Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: amount,
                script_pubkey: recipient_script_pubkey.into(),
            }],
        }
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