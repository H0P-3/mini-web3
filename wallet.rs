use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier, Signature};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// A Web3-style wallet backed by Ed25519
#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,        // Base58(SHA-256(public_key)[0..20])
    pub public_key: String,     // hex
    pub private_key: String,    // hex  ← keep secret!
}

impl Wallet {
    /// Generate a brand-new keypair
    pub fn new() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let private_key = hex::encode(signing_key.to_bytes());
        let public_key  = hex::encode(verifying_key.to_bytes());
        let address     = derive_address(verifying_key.as_bytes());

        Wallet { address, public_key, private_key }
    }

    /// Restore a wallet from a hex private key
    pub fn from_private_key(hex_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = hex::decode(hex_key)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| "key must be 32 bytes")?;
        let signing_key = SigningKey::from_bytes(&arr);
        let verifying_key = signing_key.verifying_key();

        Ok(Wallet {
            address:     derive_address(verifying_key.as_bytes()),
            public_key:  hex::encode(verifying_key.to_bytes()),
            private_key: hex_key.to_string(),
        })
    }

    /// Sign arbitrary data; returns hex-encoded signature
    pub fn sign(&self, data: &str) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = hex::decode(&self.private_key)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| "key must be 32 bytes")?;
        let signing_key = SigningKey::from_bytes(&arr);
        let sig = signing_key.sign(data.as_bytes());
        Ok(hex::encode(sig.to_bytes()))
    }
}

/// Verify that `signature` (hex) over `data` was produced by `public_key` (hex)
pub fn verify_signature(
    public_key_hex: &str,
    data: &str,
    signature_hex: &str,
) -> bool {
    let Ok(pk_bytes) = hex::decode(public_key_hex) else { return false; };
    let Ok(sig_bytes) = hex::decode(signature_hex)  else { return false; };

    let Ok(arr): Result<[u8; 32], _> = pk_bytes.try_into() else { return false; };
    let Ok(vk) = VerifyingKey::from_bytes(&arr) else { return false; };

    let Ok(sig_arr): Result<[u8; 64], _> = sig_bytes.try_into() else { return false; };
    let sig = Signature::from_bytes(&sig_arr);

    vk.verify(data.as_bytes(), &sig).is_ok()
}

/// Derive a short wallet address: Base58(first 20 bytes of SHA-256(pubkey))
fn derive_address(pubkey_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pubkey_bytes);
    let hash = hasher.finalize();
    bs58::encode(&hash[..20]).into_string()
}
