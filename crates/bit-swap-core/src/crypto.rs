//! Cryptography utilities
//!
//! Hash functions ve cryptographic operations

use crate::error::{BitSwapError, BitSwapResult};
use sha2::{Digest, Sha256};

/// SHA-256 hash hesapla
pub fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// SHA-256 hash'i hex string olarak hesapla
pub fn sha256_hex(data: &[u8]) -> String {
    hex::encode(sha256(data))
}

/// Hex string'i bytes'a çevir
pub fn hex_to_bytes(hex_str: &str) -> BitSwapResult<Vec<u8>> {
    hex::decode(hex_str).map_err(|e| {
        BitSwapError::Crypto(format!("Invalid hex string: {}", e))
    })
}

/// Info hash'i bytes'a çevir (32 byte SHA-256)
pub fn info_hash_to_bytes(info_hash: &str) -> BitSwapResult<[u8; 32]> {
    let bytes = hex_to_bytes(info_hash)?;
    if bytes.len() != 32 {
        return Err(BitSwapError::Crypto(format!(
            "Info hash must be 32 bytes, got {}", 
            bytes.len()
        )));
    }
    
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);
    Ok(array)
}

/// Piece hash'i doğrula
pub fn verify_piece_hash(data: &[u8], expected_hash: &str) -> bool {
    let calculated = sha256_hex(data);
    calculated == expected_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let data = b"Hello, BitSwapTorrent!";
        let hash = sha256(data);
        let hex_hash = sha256_hex(data);
        
        assert_eq!(hex_hash, hex::encode(hash));
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_piece_verification() {
        let data = b"test piece data";
        let hash = sha256_hex(data);
        
        assert!(verify_piece_hash(data, &hash));
        assert!(!verify_piece_hash(data, "invalid_hash"));
    }

    #[test]
    fn test_info_hash_conversion() {
        let hash_hex = "a".repeat(64); // 32 bytes as hex
        let hash_bytes = info_hash_to_bytes(&hash_hex).unwrap();
        
        assert_eq!(hash_bytes.len(), 32);
        assert_eq!(hex::encode(hash_bytes), hash_hex);
    }
}
