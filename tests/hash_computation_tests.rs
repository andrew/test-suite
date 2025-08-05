use std::fs;
use std::path::Path;
use tempfile::TempDir;
use swhid::{Content, Directory, SwhidComputer, traverse_directory_recursively, TreeObject, SwhidError, Swhid, ObjectType, EntryType, Permissions};

/// Test helper to create a temporary directory with specific structure
struct TestDir {
    temp_dir: TempDir,
}

impl TestDir {
    fn new() -> Self {
        Self {
            temp_dir: TempDir::new().unwrap(),
        }
    }

    fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    fn create_file(&self, name: &str, content: &[u8]) {
        fs::write(self.path().join(name), content).unwrap();
    }

    fn create_executable(&self, name: &str, content: &[u8]) {
        let file_path = self.path().join(name);
        fs::write(&file_path, content).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&file_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&file_path, perms).unwrap();
        }
    }

    fn create_subdir(&self, name: &str) -> std::path::PathBuf {
        let dir_path = self.path().join(name);
        fs::create_dir(&dir_path).unwrap();
        dir_path
    }

    fn create_symlink(&self, name: &str, target: &str) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(target, self.path().join(name)).unwrap();
        }
    }
}

#[test]
fn test_content_hash_basic() {
    let test_dir = TestDir::new();
    test_dir.create_file("test.txt", b"Hello, World!");
    
    let content = Content::from_file(test_dir.path().join("test.txt")).unwrap();
    let swhid = content.swhid();
    
    // Known hash for "Hello, World!" content (matches Python swh identify)
    assert_eq!(swhid.object_id(), &hex::decode("b45ef6fec89518d314f546fd6c3025367b721684").unwrap()[..]);
    assert_eq!(swhid.object_type(), swhid::ObjectType::Content);
}

#[test]
fn test_content_hash_empty() {
    let test_dir = TestDir::new();
    test_dir.create_file("empty.txt", b"");
    
    let content = Content::from_file(test_dir.path().join("empty.txt")).unwrap();
    let swhid = content.swhid();
    
    // Known hash for empty content
    assert_eq!(swhid.object_id(), &hex::decode("e69de29bb2d1d6434b8b29ae775ad8c2e48c5391").unwrap()[..]);
}

#[test]
fn test_content_hash_large() {
    let test_dir = TestDir::new();
    let large_content = vec![b'a'; 10000];
    test_dir.create_file("large.txt", &large_content);
    
    let content = Content::from_file(test_dir.path().join("large.txt")).unwrap();
    let swhid = content.swhid();
    
    // Verify it's a valid SHA1 hash
    assert_eq!(swhid.object_id().len(), 20);
}

#[test]
fn test_directory_hash_single_file() {
    let test_dir = TestDir::new();
    test_dir.create_file("file.txt", b"test content");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), swhid::ObjectType::Directory);
    assert_eq!(swhid.object_id().len(), 20);
    
    // Verify directory has exactly one entry
    assert_eq!(dir.entries().len(), 1);
    assert_eq!(dir.entries()[0].name, b"file.txt");
}

#[test]
fn test_directory_hash_multiple_files() {
    let test_dir = TestDir::new();
    test_dir.create_file("a.txt", b"content a");
    test_dir.create_file("b.txt", b"content b");
    test_dir.create_file("c.txt", b"content c");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), swhid::ObjectType::Directory);
    assert_eq!(dir.entries().len(), 3);
    
    // Verify entries are sorted (Git tree sorting)
    let names: Vec<&[u8]> = dir.entries().iter().map(|e| e.name.as_slice()).collect();
    assert_eq!(names, vec![b"a.txt", b"b.txt", b"c.txt"]);
}

#[test]
fn test_directory_hash_with_executable() {
    let test_dir = TestDir::new();
    test_dir.create_file("normal.txt", b"normal file");
    test_dir.create_executable("script.sh", b"#!/bin/bash");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), swhid::ObjectType::Directory);
    assert_eq!(dir.entries().len(), 2);
    
    // Verify executable has correct permissions
    let executable_entry = dir.entries().iter().find(|e| e.name == b"script.sh").unwrap();
    assert_eq!(executable_entry.permissions, swhid::Permissions::Executable);
    
    let normal_entry = dir.entries().iter().find(|e| e.name == b"normal.txt").unwrap();
    assert_eq!(normal_entry.permissions, swhid::Permissions::File);
}

#[test]
fn test_directory_hash_with_symlink() {
    let test_dir = TestDir::new();
    test_dir.create_file("target.txt", b"target content");
    test_dir.create_symlink("link.txt", "target.txt");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), swhid::ObjectType::Directory);
    assert_eq!(dir.entries().len(), 2);
    
    // Verify symlink has correct permissions and target
    let symlink_entry = dir.entries().iter().find(|e| e.name == b"link.txt").unwrap();
    assert_eq!(symlink_entry.permissions, swhid::Permissions::Symlink);
    
    // The target should be the hash of the symlink target content
    let target_content = Content::from_data(b"target.txt".to_vec());
    assert_eq!(symlink_entry.target, *target_content.sha1_git());
}

#[test]
fn test_directory_hash_with_subdirectory() {
    let test_dir = TestDir::new();
    test_dir.create_file("root.txt", b"root content");
    
    let subdir = test_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("sub.txt"), b"sub content").unwrap();
    
    let mut dir = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), swhid::ObjectType::Directory);
    assert_eq!(dir.entries().len(), 2);
    
    // Verify subdirectory entry
    let subdir_entry = dir.entries().iter().find(|e| e.name == b"subdir").unwrap();
    assert_eq!(subdir_entry.permissions, swhid::Permissions::Directory);
    assert_eq!(subdir_entry.entry_type, swhid::EntryType::Directory);
}

#[test]
fn test_recursive_traversal_simple() {
    let test_dir = TestDir::new();
    test_dir.create_file("file1.txt", b"content 1");
    test_dir.create_file("file2.txt", b"content 2");
    
    let objects = traverse_directory_recursively(test_dir.path(), &[], true).unwrap();
    
    // Should have 3 objects: directory + 2 files
    assert_eq!(objects.len(), 3);
    
    // Verify all objects have valid SWHIDs
    for (_, mut obj) in objects {
        let swhid = obj.swhid();
        assert_eq!(swhid.object_id().len(), 20);
    }
}

#[test]
fn test_recursive_traversal_with_subdirs() {
    let test_dir = TestDir::new();
    test_dir.create_file("root.txt", b"root");
    
    let subdir1 = test_dir.path().join("subdir1");
    fs::create_dir(&subdir1).unwrap();
    fs::write(subdir1.join("sub1.txt"), b"sub1").unwrap();
    
    let subdir2 = test_dir.path().join("subdir2");
    fs::create_dir(&subdir2).unwrap();
    fs::write(subdir2.join("sub2.txt"), b"sub2").unwrap();
    
    let objects = traverse_directory_recursively(test_dir.path(), &[], true).unwrap();
    
    // Should have 6 objects: root dir + root file + 2 subdirs + 2 subfiles
    assert_eq!(objects.len(), 6);
    
    // Verify directory objects come first (matching Python behavior)
    let first_obj = &objects[0];
    assert!(matches!(first_obj.1, TreeObject::Directory(_)));
}

#[test]
fn test_recursive_traversal_exclude_patterns() {
    let test_dir = TestDir::new();
    test_dir.create_file("keep.txt", b"keep");
    test_dir.create_file("exclude.txt", b"exclude");
    
    let exclude_dir = test_dir.path().join("exclude_dir");
    fs::create_dir(&exclude_dir).unwrap();
    fs::write(exclude_dir.join("file.txt"), b"excluded").unwrap();
    
    let objects = traverse_directory_recursively(test_dir.path(), &["exclude_dir".to_string()], true).unwrap();
    
    // Should have 3 objects: directory + keep.txt + exclude.txt (exclude_dir should be excluded)
    assert_eq!(objects.len(), 3);
    
    // Verify excluded directory is not present
    let paths: Vec<String> = objects.iter().map(|(p, _)| p.to_string_lossy().to_string()).collect();
    assert!(!paths.iter().any(|p| p.contains("exclude_dir")));
}

#[test]
fn test_recursive_traversal_hidden_files() {
    let test_dir = TestDir::new();
    test_dir.create_file("visible.txt", b"visible");
    test_dir.create_file(".hidden", b"hidden");
    
    let objects = traverse_directory_recursively(test_dir.path(), &[], true).unwrap();
    
    // Should only have 2 objects: directory + visible.txt (hidden file should be excluded)
    assert_eq!(objects.len(), 2);
    
    // Verify hidden file is not present
    let paths: Vec<String> = objects.iter().map(|(p, _)| p.to_string_lossy().to_string()).collect();
    assert!(!paths.iter().any(|p| p.contains(".hidden")));
}

#[test]
fn test_recursive_traversal_complex_structure() {
    let test_dir = TestDir::new();
    
    // Create a complex directory structure
    test_dir.create_file("root.txt", b"root");
    test_dir.create_executable("script.sh", b"#!/bin/bash");
    
    let subdir1 = test_dir.path().join("subdir1");
    fs::create_dir(&subdir1).unwrap();
    fs::write(subdir1.join("file1.txt"), b"subdir1 file").unwrap();
    
    let subdir2 = test_dir.path().join("subdir2");
    fs::create_dir(&subdir2).unwrap();
    fs::write(subdir2.join("file2.txt"), b"subdir2 file").unwrap();
    
    let nested = subdir2.join("nested");
    fs::create_dir(&nested).unwrap();
    fs::write(nested.join("nested.txt"), b"nested file").unwrap();
    
    let objects = traverse_directory_recursively(test_dir.path(), &[], true).unwrap();
    
    // Should have 9 objects: root dir + root files + 2 subdirs + subdir files + nested dir + nested file
    assert_eq!(objects.len(), 9);
    
    // Verify all objects have valid SWHIDs
    for (_, mut obj) in objects {
        let swhid = obj.swhid();
        assert_eq!(swhid.object_id().len(), 20);
    }
}

#[test]
fn test_hash_consistency() {
    let test_dir = TestDir::new();
    test_dir.create_file("test.txt", b"consistent content");
    
    // Compute hash multiple times
    let mut dir1 = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let hash1 = dir1.compute_hash();
    
    let mut dir2 = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let hash2 = dir2.compute_hash();
    
    // Hashes should be identical
    assert_eq!(hash1, hash2);
}

#[test]
fn test_hash_deterministic_ordering() {
    let test_dir = TestDir::new();
    
    // Create files in reverse alphabetical order
    test_dir.create_file("z.txt", b"z content");
    test_dir.create_file("a.txt", b"a content");
    test_dir.create_file("m.txt", b"m content");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let hash = dir.compute_hash();
    
    // Verify entries are sorted correctly
    let names: Vec<&[u8]> = dir.entries().iter().map(|e| e.name.as_slice()).collect();
    assert_eq!(names, vec![b"a.txt", b"m.txt", b"z.txt"]);
    
    // Hash should be deterministic
    let mut dir2 = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let hash2 = dir2.compute_hash();
    assert_eq!(hash, hash2);
}

#[test]
fn test_empty_directory() {
    let test_dir = TestDir::new();
    
    let mut dir = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), swhid::ObjectType::Directory);
    assert_eq!(dir.entries().len(), 0);
    
    // Empty directory should have a specific hash
    let hash = dir.compute_hash();
    assert_eq!(hash.len(), 20);
}

#[test]
fn test_directory_with_mixed_content() {
    let test_dir = TestDir::new();
    
    // Create files, directories, and executables
    test_dir.create_file("file.txt", b"file content");
    test_dir.create_executable("script.sh", b"#!/bin/bash");
    
    let subdir = test_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("subfile.txt"), b"subfile content").unwrap();
    
    let mut dir = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), swhid::ObjectType::Directory);
    assert_eq!(dir.entries().len(), 3);
    
    // Verify each entry type
    let file_entry = dir.entries().iter().find(|e| e.name == b"file.txt").unwrap();
    assert_eq!(file_entry.permissions, swhid::Permissions::File);
    
    let exec_entry = dir.entries().iter().find(|e| e.name == b"script.sh").unwrap();
    assert_eq!(exec_entry.permissions, swhid::Permissions::Executable);
    
    let dir_entry = dir.entries().iter().find(|e| e.name == b"subdir").unwrap();
    assert_eq!(dir_entry.permissions, swhid::Permissions::Directory);
}

#[test]
fn test_error_handling_nonexistent_path() {
    let result = Content::from_file("nonexistent_file.txt");
    assert!(result.is_err());
}

#[test]
fn test_error_handling_directory_as_file() {
    let test_dir = TestDir::new();
    let result = Content::from_file(test_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_swhid_computer_api() {
    let test_dir = TestDir::new();
    test_dir.create_file("test.txt", b"test content");
    
    let computer = SwhidComputer::new();
    
    // Test file SWHID computation
    let file_swhid = computer.compute_file_swhid(test_dir.path().join("test.txt")).unwrap();
    assert_eq!(file_swhid.object_type(), swhid::ObjectType::Content);
    
    // Test directory SWHID computation
    let dir_swhid = computer.compute_directory_swhid(test_dir.path()).unwrap();
    assert_eq!(dir_swhid.object_type(), swhid::ObjectType::Directory);
    
    // Test auto-detection
    let auto_swhid = computer.compute_swhid(test_dir.path().join("test.txt")).unwrap();
    assert_eq!(auto_swhid.object_type(), swhid::ObjectType::Content);
}

#[test]
fn test_swhid_computer_with_exclusions() {
    let test_dir = TestDir::new();
    test_dir.create_file("keep.txt", b"keep");
    
    let exclude_dir = test_dir.path().join("exclude");
    fs::create_dir(&exclude_dir).unwrap();
    fs::write(exclude_dir.join("file.txt"), b"excluded").unwrap();
    
    let computer = SwhidComputer::new().with_exclude_patterns(vec!["exclude".to_string()]);
    let dir_swhid = computer.compute_directory_swhid(test_dir.path()).unwrap();
    
    assert_eq!(dir_swhid.object_type(), swhid::ObjectType::Directory);
}

#[test]
fn test_swhid_string_parsing() {
    let swhid_str = "swh:1:cnt:95d09f2b10159347eece71399a7e2cc70638e9a7";
    let swhid = Swhid::from_string(swhid_str).unwrap();
    
    assert_eq!(swhid.namespace(), "swh");
    assert_eq!(swhid.scheme_version(), 1);
    assert_eq!(swhid.object_type(), swhid::ObjectType::Content);
    assert_eq!(swhid.to_string(), swhid_str);
}

#[test]
fn test_swhid_string_parsing_invalid() {
    let invalid_swhids = vec![
        "invalid",
        "swh:2:cnt:95d09f2b10159347eece71399a7e2cc70638e9a7", // wrong version
        "swh:1:invalid:95d09f2b10159347eece71399a7e2cc70638e9a7", // wrong type
        "swh:1:cnt:invalid", // wrong hash
    ];
    
    for invalid in invalid_swhids {
        assert!(Swhid::from_string(invalid).is_err());
    }
}

#[test]
fn test_permissions_encoding() {
    let test_dir = TestDir::new();
    test_dir.create_file("normal.txt", b"normal");
    test_dir.create_executable("script.sh", b"#!/bin/bash");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    
    let normal_entry = dir.entries().iter().find(|e| e.name == b"normal.txt").unwrap();
    assert_eq!(normal_entry.permissions, swhid::Permissions::File);
    
    let exec_entry = dir.entries().iter().find(|e| e.name == b"script.sh").unwrap();
    assert_eq!(exec_entry.permissions, swhid::Permissions::Executable);
}

#[test]
fn test_large_file_handling() {
    let test_dir = TestDir::new();
    
    // Create a file with 1MB of data
    let large_content = vec![b'x'; 1024 * 1024];
    test_dir.create_file("large.txt", &large_content);
    
    let content = Content::from_file(test_dir.path().join("large.txt")).unwrap();
    let swhid = content.swhid();
    
    assert_eq!(swhid.object_type(), swhid::ObjectType::Content);
    assert_eq!(swhid.object_id().len(), 20);
}

#[test]
fn test_unicode_filename_handling() {
    let test_dir = TestDir::new();
    
    // Create file with Unicode name
    let unicode_name = "测试文件.txt";
    test_dir.create_file(unicode_name, b"unicode content");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), swhid::ObjectType::Directory);
    assert_eq!(dir.entries().len(), 1);
    
    // Verify the entry name is correctly encoded
    let entry = &dir.entries()[0];
    assert_eq!(entry.name, unicode_name.as_bytes());
}

#[test]
fn test_symlink_following() {
    let test_dir = TestDir::new();
    test_dir.create_file("target.txt", b"target content");
    test_dir.create_symlink("link.txt", "target.txt");
    
    // Test with follow_symlinks = true
    let mut dir_follow = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let swhid_follow = dir_follow.swhid();
    
    // Test with follow_symlinks = false
    let mut dir_no_follow = Directory::from_disk(test_dir.path(), &[], false).unwrap();
    let swhid_no_follow = dir_no_follow.swhid();
    
    // Results should be the same - symlinks in directory entries are always
    // treated as content objects with the target path as content
    assert_eq!(swhid_follow.object_id(), swhid_no_follow.object_id());
    
    // Verify that the symlink entry has the correct permissions and target
    let symlink_entry = dir_follow.entries().iter().find(|e| e.name == b"link.txt").unwrap();
    assert_eq!(symlink_entry.permissions, swhid::Permissions::Symlink);
    assert_eq!(symlink_entry.entry_type, swhid::EntryType::Symlink);
    
    // The target should be the hash of the symlink target path as content
    let target_content = Content::from_data(b"target.txt".to_vec());
    assert_eq!(symlink_entry.target, *target_content.sha1_git());
}

#[test]
fn test_symlink_as_object() {
    let test_dir = TestDir::new();
    test_dir.create_file("target.txt", b"target content");
    test_dir.create_symlink("link.txt", "target.txt");
    
    // When identifying the symlink itself, follow_symlinks should matter
    let link_path = test_dir.path().join("link.txt");
    let target_path = test_dir.path().join("target.txt");
    
    // Test with follow_symlinks = true (should follow the symlink)
    let computer_follow = SwhidComputer::new().with_follow_symlinks(true);
    let swhid_follow = computer_follow.compute_swhid(&link_path).unwrap();
    
    // Test with follow_symlinks = false (should treat symlink as content)
    let computer_no_follow = SwhidComputer::new().with_follow_symlinks(false);
    let swhid_no_follow = computer_no_follow.compute_swhid(&link_path).unwrap();
    
    // Results should be different
    assert_ne!(swhid_follow.object_id(), swhid_no_follow.object_id());
    
    // The follow_symlinks=true should match the target file
    let target_swhid = SwhidComputer::new().compute_swhid(&target_path).unwrap();
    assert_eq!(swhid_follow.object_id(), target_swhid.object_id());
    
    // The follow_symlinks=false should be the symlink content (target path)
    let symlink_content = Content::from_data(b"target.txt".to_vec());
    let expected_symlink_swhid = symlink_content.swhid();
    assert_eq!(swhid_no_follow.object_id(), expected_symlink_swhid.object_id());
}

#[test]
fn test_recursive_hash_consistency() {
    let test_dir = TestDir::new();
    
    // Create a simple structure: root dir with one subdir containing one file
    test_dir.create_file("root.txt", b"root");
    
    let subdir = test_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("subfile.txt"), b"subfile").unwrap();
    
    // Compute hashes using recursive traversal
    let objects = traverse_directory_recursively(test_dir.path(), &[], true).unwrap();
    
    // Should have 4 objects: root dir, root file, subdir, subfile
    assert_eq!(objects.len(), 4);
    
    // Verify all hashes are consistent
    let mut hashes = Vec::new();
    for (_, mut obj) in objects {
        let swhid = obj.swhid();
        hashes.push(swhid.object_id().to_vec());
    }
    
    // All hashes should be unique and 20 bytes
    for hash in &hashes {
        assert_eq!(hash.len(), 20);
    }
    
    // Check for duplicates
    let unique_hashes: std::collections::HashSet<_> = hashes.iter().collect();
    assert_eq!(unique_hashes.len(), hashes.len());
}

#[test]
fn test_edge_case_single_byte_file() {
    let test_dir = TestDir::new();
    test_dir.create_file("single.txt", b"a");
    
    let content = Content::from_file(test_dir.path().join("single.txt")).unwrap();
    let swhid = content.swhid();
    
    assert_eq!(swhid.object_type(), swhid::ObjectType::Content);
    assert_eq!(swhid.object_id().len(), 20);
}

#[test]
fn test_edge_case_very_long_filename() {
    let test_dir = TestDir::new();
    
    // Create a file with a very long name
    let long_name = "a".repeat(255);
    test_dir.create_file(&long_name, b"content");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), swhid::ObjectType::Directory);
    assert_eq!(dir.entries().len(), 1);
    assert_eq!(dir.entries()[0].name, long_name.as_bytes());
}

#[test]
fn test_edge_case_special_characters() {
    let test_dir = TestDir::new();
    
    // Create files with special characters in names
    test_dir.create_file("file with spaces.txt", b"content");
    test_dir.create_file("file-with-dashes.txt", b"content");
    test_dir.create_file("file_with_underscores.txt", b"content");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[], true).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), swhid::ObjectType::Directory);
    assert_eq!(dir.entries().len(), 3);
    
    // Verify all files are present
    let names: Vec<&[u8]> = dir.entries().iter().map(|e| e.name.as_slice()).collect();
    assert!(names.iter().any(|n| n == b"file with spaces.txt"));
    assert!(names.iter().any(|n| n == b"file-with-dashes.txt"));
    assert!(names.iter().any(|n| n == b"file_with_underscores.txt"));
} 