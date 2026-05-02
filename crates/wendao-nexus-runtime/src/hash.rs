//! Content hashing helpers for `Wendao Nexus` dedup registries.

use sha2::{Digest, Sha256};

/// Return a stable content hash suitable for dedup registries.
pub fn sha256_content_hash(payload: &[u8]) -> String {
    let digest = Sha256::digest(payload);
    format!("sha256:{}", hex::encode(digest))
}
