use std::str::FromStr;

use secp256k1::{Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicParams {
    pub secret_hash: String,
    pub public_key: String,
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
