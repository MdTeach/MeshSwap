use std::str::FromStr;

use secp256k1::{Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicParams {
    pub secret_hash: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapInfo {
    pub recipient_public_key: String,
    pub revocation_public_key: String,
    pub timelock_duration_blocks: u32,
    pub amount_satoshis: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinSwap {
    pub swap_info: SwapInfo,
    pub swap_secret: String,
    pub descriptor_string: String,
    pub contract_address: String,
    pub funding_txid: String,
    pub creation_timestamp: u64,
}

pub fn make_process(secret_key_string: &str) -> PublicParams {
    let secret_key = SecretKey::from_str(secret_key_string).unwrap();
    let secp = Secp256k1::new();
    let pub_key = secret_key.public_key(&secp);
    let secret_hash = keccak256(secret_key.as_ref());

    PublicParams {
        secret_hash: hex::encode(secret_hash),
        public_key: pub_key.to_string(),
    }
}

/// Simple interface to the [`keccak256`] hash function.
///
/// [`keccak256`]: https://en.wikipedia.org/wiki/SHA-3
pub fn keccak256<T: AsRef<[u8]>>(bytes: T) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes.as_ref());
    hasher.finalize(&mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_process_with_valid_secret_key() {
        let secret_key_string = "242b7a112ced4f1e688d117f358e3534e92f9e5fc89a5d0b2f843afebb9742f6";
        let params = make_process(secret_key_string);

        println!("Public Params: {:?}", params);
    }

}
