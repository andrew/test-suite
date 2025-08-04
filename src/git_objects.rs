// This module provides Git object formatting utilities
// Most functionality is already covered in the hash module

use crate::hash::{git_object_header, hash_git_object};

/// Format a Git object from parts
pub fn format_git_object_from_parts(git_type: &str, parts: &[u8]) -> Vec<u8> {
    let mut result = git_object_header(git_type, parts.len());
    result.extend_from_slice(parts);
    result
}

/// Hash a Git object from parts
pub fn hash_git_object_from_parts(git_type: &str, parts: &[u8]) -> [u8; 20] {
    hash_git_object(git_type, parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_git_object_from_parts() {
        let parts = b"test data";
        let formatted = format_git_object_from_parts("blob", parts);
        
        assert_eq!(formatted, b"blob 9\0test data");
    }

    #[test]
    fn test_hash_git_object_from_parts() {
        let parts = b"test data";
        let hash = hash_git_object_from_parts("blob", parts);
        
        assert_eq!(hash.len(), 20);
    }
} 