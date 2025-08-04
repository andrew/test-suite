use sha1::{Sha1, Digest};
use sha2::{Sha256, Digest as Sha256Digest};
use crate::error::SwhidError;

/// Git-style SHA1 hash computation
pub fn sha1_git_hash(data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    let header = format!("blob {}\0", data.len());
    hasher.update(header.as_bytes());
    hasher.update(data);
    hasher.finalize().into()
}

/// Standard SHA1 hash computation
pub fn sha1_hash(data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// SHA256 hash computation
pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Git object header formatting
pub fn git_object_header(git_type: &str, length: usize) -> Vec<u8> {
    format!("{} {}\0", git_type, length).into_bytes()
}

/// Hash a Git object (header + data)
pub fn hash_git_object(git_type: &str, data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    let header = git_object_header(git_type, data.len());
    hasher.update(&header);
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_git_hash() {
        let data = b"Hello, World!";
        let hash = sha1_git_hash(data);
        
        // The hash should be different from a regular SHA1
        let regular_hash = sha1_hash(data);
        assert_ne!(hash, regular_hash);
        
        // Hash should be 20 bytes
        assert_eq!(hash.len(), 20);
    }

    #[test]
    fn test_git_object_header() {
        let header = git_object_header("blob", 1234);
        assert_eq!(header, b"blob 1234\0");
    }

    #[test]
    fn test_hash_git_object() {
        let data = b"test data";
        let hash = hash_git_object("blob", data);
        
        // Should be same as sha1_git_hash for blob type
        let expected = sha1_git_hash(data);
        assert_eq!(hash, expected);
    }
} 