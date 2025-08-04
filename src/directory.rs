use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::swhid::{Swhid, ObjectType};
use crate::content::Content;
use crate::hash::hash_git_object;
use crate::error::SwhidError;

/// Directory entry types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
}

impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::File => "file",
            EntryType::Directory => "dir",
            EntryType::Symlink => "symlink",
        }
    }
}

/// Directory entry permissions (Git-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permissions {
    File = 0o100644,
    Executable = 0o100755,
    Symlink = 0o120000,
    Directory = 0o040000,
}

impl Permissions {
    pub fn from_mode(mode: u32) -> Self {
        match mode & 0o170000 {
            0o040000 => Permissions::Directory,
            0o120000 => Permissions::Symlink,
            _ => {
                if mode & 0o111 != 0 {
                    Permissions::Executable
                } else {
                    Permissions::File
                }
            }
        }
    }

    pub fn as_octal(&self) -> u32 {
        *self as u32
    }
}

/// Directory entry
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: Vec<u8>,
    pub entry_type: EntryType,
    pub permissions: Permissions,
    pub target: [u8; 20], // SHA1 hash of the target object
}

impl DirectoryEntry {
    pub fn new(name: Vec<u8>, entry_type: EntryType, permissions: Permissions, target: [u8; 20]) -> Self {
        Self {
            name,
            entry_type,
            permissions,
            target,
        }
    }
}

/// Directory object
#[derive(Debug, Clone)]
pub struct Directory {
    entries: Vec<DirectoryEntry>,
    hash: Option<[u8; 20]>,
}

impl Directory {
    /// Create directory from disk path
    pub fn from_disk<P: AsRef<Path>>(
        path: P,
        exclude_patterns: &[String],
        follow_symlinks: bool,
    ) -> Result<Self, SwhidError> {
        let path = path.as_ref();
        let mut entries = Vec::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_bytes = name.to_string_lossy().as_bytes().to_vec();

            // Skip hidden files and excluded patterns
            if name_bytes.starts_with(b".") || Self::should_exclude(&name_bytes, exclude_patterns) {
                continue;
            }

            let metadata = if follow_symlinks {
                entry.metadata()?
            } else {
                entry.symlink_metadata()?
            };

            let entry_type = if metadata.is_dir() {
                EntryType::Directory
            } else if metadata.is_symlink() {
                EntryType::Symlink
            } else {
                EntryType::File
            };

            let permissions = Permissions::from_mode(metadata.mode());

            // For now, we'll need to compute the target hash
            // This is a simplified version - in practice, you'd need to handle
            // recursive directory traversal and content hashing
            let target = if entry_type == EntryType::File {
                // Compute content hash
                let content = Content::from_file(entry.path())?;
                *content.sha1_git()
            } else {
                // For directories and symlinks, we'd need to compute their hashes
                // This is a placeholder - in practice, you'd need to implement
                // recursive directory hashing
                [0u8; 20]
            };

            entries.push(DirectoryEntry::new(name_bytes, entry_type, permissions, target));
        }

        // Sort entries according to Git's tree sorting rules
        entries.sort_by(|a, b| Self::entry_sort_key(a).cmp(&Self::entry_sort_key(b)));

        Ok(Self {
            entries,
            hash: None,
        })
    }

    /// Get directory entries
    pub fn entries(&self) -> &[DirectoryEntry] {
        &self.entries
    }

    /// Compute the directory hash
    pub fn compute_hash(&mut self) -> [u8; 20] {
        if let Some(hash) = self.hash {
            return hash;
        }

        let mut components = Vec::new();

        for entry in &self.entries {
            // Format: perms + space + name + null + target
            let perms_str = format!("{:o}", entry.permissions.as_octal());
            components.extend_from_slice(perms_str.as_bytes());
            components.push(b' ');
            components.extend_from_slice(&entry.name);
            components.push(0);
            components.extend_from_slice(&entry.target);
        }

        let hash = hash_git_object("tree", &components);
        self.hash = Some(hash);
        hash
    }

    /// Compute SWHID for this directory
    pub fn swhid(&mut self) -> Swhid {
        let hash = self.compute_hash();
        Swhid::new(ObjectType::Directory, hash)
    }

    /// Entry sorting key (Git's tree sorting rules)
    fn entry_sort_key(entry: &DirectoryEntry) -> Vec<u8> {
        let mut key = entry.name.clone();
        if entry.entry_type == EntryType::Directory {
            key.push(b'/');
        }
        key
    }

    /// Check if entry should be excluded based on patterns
    fn should_exclude(name: &[u8], patterns: &[String]) -> bool {
        let name_str = String::from_utf8_lossy(name);
        for pattern in patterns {
            if name_str.matches(pattern) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("file.txt"), b"test").unwrap();

        let dir = Directory::from_disk(temp_dir.path(), &[], true).unwrap();
        
        assert!(!dir.entries().is_empty());
    }

    #[test]
    fn test_directory_hash() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test.txt"), b"test").unwrap();

        let mut dir = Directory::from_disk(temp_dir.path(), &[], true).unwrap();
        let hash = dir.compute_hash();
        
        assert_eq!(hash.len(), 20);
    }

    #[test]
    fn test_directory_swhid() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test.txt"), b"test").unwrap();

        let mut dir = Directory::from_disk(temp_dir.path(), &[], true).unwrap();
        let swhid = dir.swhid();
        
        assert_eq!(swhid.object_type(), ObjectType::Directory);
    }
} 