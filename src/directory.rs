use std::fs;
use std::os::unix::fs::MetadataExt;
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
        Self::from_disk_with_hash_fn(path, exclude_patterns, follow_symlinks, |_| Ok([0u8; 20]))
    }

    pub fn from_disk_with_hash_fn<P: AsRef<Path>, F>(
        path: P,
        exclude_patterns: &[String],
        follow_symlinks: bool,
        hash_fn: F,
    ) -> Result<Self, SwhidError>
    where
        F: Fn(&Path) -> Result<[u8; 20], SwhidError>,
    {
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
                entry.metadata()? // Note: symlink_metadata() is not available on DirEntry
            };

            let entry_type = if metadata.is_dir() {
                EntryType::Directory
            } else if metadata.is_symlink() {
                EntryType::Symlink
            } else {
                EntryType::File
            };

            let permissions = Permissions::from_mode(metadata.mode());

            // Compute the target hash using the provided hash function
            let target = if entry_type == EntryType::File {
                // Compute content hash
                let content = Content::from_file(entry.path())?;
                *content.sha1_git()
            } else {
                // Use the provided hash function for directories and symlinks
                hash_fn(&entry.path())?
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



    /// Get the path associated with this directory (for recursive traversal)
    pub fn path(&self) -> Option<&Path> {
        None // TODO: Add path tracking
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
        should_exclude_str(&name_str, patterns)
    }
}

/// Check if entry should be excluded based on patterns (string version)
fn should_exclude_str(name: &str, patterns: &[String]) -> bool {
    for pattern in patterns {
        if name.contains(pattern) {
            return true;
        }
    }
    false
}

/// Recursively traverse a directory and yield all objects
pub fn traverse_directory_recursively<P: AsRef<Path>>(
    root_path: P,
    exclude_patterns: &[String],
    follow_symlinks: bool,
) -> Result<Vec<(PathBuf, TreeObject)>, SwhidError> {
    let root_path = root_path.as_ref();
    
    // Build a cache of directory hashes
    let mut hash_cache = std::collections::HashMap::new();
    
    // First pass: collect all content objects and compute their hashes
    let mut content_objects = Vec::new();
    collect_content_objects(root_path, exclude_patterns, follow_symlinks, &mut content_objects)?;
    
    // Second pass: compute directory hashes using the content hashes
    let mut directory_objects = Vec::new();
    compute_directory_hashes(root_path, exclude_patterns, follow_symlinks, &mut hash_cache, &mut directory_objects)?;
    
    // Combine all objects
    let mut all_objects = Vec::new();
    all_objects.extend(content_objects);
    all_objects.extend(directory_objects);
    
    Ok(all_objects)
}

/// Collect all content objects recursively
fn collect_content_objects(
    current_path: &Path,
    exclude_patterns: &[String],
    follow_symlinks: bool,
    objects: &mut Vec<(PathBuf, TreeObject)>,
) -> Result<(), SwhidError> {
    let metadata = if follow_symlinks {
        fs::metadata(current_path)?
    } else {
        fs::symlink_metadata(current_path)?
    };

    if metadata.is_file() {
        // Add content object
        let content = Content::from_file(current_path)?;
        objects.push((current_path.to_path_buf(), TreeObject::Content(content)));
    } else if metadata.is_dir() {
        // Process all subdirectories and files recursively
        for entry in fs::read_dir(current_path)? {
            let entry = entry?;
            let entry_path = entry.path();
            
            // Skip hidden files and excluded patterns
            if let Some(name) = entry_path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with('.') || should_exclude_str(&name_str, exclude_patterns) {
                    continue;
                }
            }
            
            collect_content_objects(&entry_path, exclude_patterns, follow_symlinks, objects)?;
        }
    }
    
    Ok(())
}

/// Compute directory hashes recursively, using cached content hashes
fn compute_directory_hashes(
    current_path: &Path,
    exclude_patterns: &[String],
    follow_symlinks: bool,
    hash_cache: &mut std::collections::HashMap<PathBuf, [u8; 20]>,
    objects: &mut Vec<(PathBuf, TreeObject)>,
) -> Result<(), SwhidError> {
    let metadata = if follow_symlinks {
        fs::metadata(current_path)?
    } else {
        fs::symlink_metadata(current_path)?
    };

    if metadata.is_dir() {
        // Check if we've already processed this directory
        if hash_cache.contains_key(current_path) {
            return Ok(());
        }
        
        // First, compute hashes for all subdirectories
        for entry in fs::read_dir(current_path)? {
            let entry = entry?;
            let entry_path = entry.path();
            
            // Skip hidden files and excluded patterns
            if let Some(name) = entry_path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with('.') || should_exclude_str(&name_str, exclude_patterns) {
                    continue;
                }
            }
            
            let entry_metadata = if follow_symlinks {
                fs::metadata(&entry_path)?
            } else {
                fs::symlink_metadata(&entry_path)?
            };

            if entry_metadata.is_dir() {
                compute_directory_hashes(&entry_path, exclude_patterns, follow_symlinks, hash_cache, objects)?;
            }
        }
        
        // Then compute the hash for this directory
        let hash_fn = |path: &Path| {
            if let Some(hash) = hash_cache.get(path) {
                Ok(*hash)
            } else {
                // For content objects, compute the hash
                let content = Content::from_file(path)?;
                Ok(*content.sha1_git())
            }
        };
        
        let mut dir = Directory::from_disk_with_hash_fn(current_path, exclude_patterns, follow_symlinks, hash_fn)?;
        let hash = dir.compute_hash();
        hash_cache.insert(current_path.to_path_buf(), hash);
        
        objects.push((current_path.to_path_buf(), TreeObject::Directory(dir)));
    }
    
    Ok(())
}

/// Represents an object in the directory tree (either content or directory)
#[derive(Debug)]
pub enum TreeObject {
    Content(Content),
    Directory(Directory),
}

impl TreeObject {
    /// Get the SWHID for this object
    pub fn swhid(&mut self) -> Swhid {
        match self {
            TreeObject::Content(content) => content.swhid(),
            TreeObject::Directory(dir) => dir.swhid(),
        }
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