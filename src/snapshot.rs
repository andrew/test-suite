use std::collections::HashMap;
use crate::swhid::{Swhid, ObjectType};
use crate::error::SwhidError;

/// Snapshot target type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SnapshotTargetType {
    Content,
    Directory,
    Revision,
    Release,
    Snapshot,
    Alias,
}

impl SnapshotTargetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SnapshotTargetType::Content => "content",
            SnapshotTargetType::Directory => "directory",
            SnapshotTargetType::Revision => "revision",
            SnapshotTargetType::Release => "release",
            SnapshotTargetType::Snapshot => "snapshot",
            SnapshotTargetType::Alias => "alias",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, SwhidError> {
        match s {
            "content" => Ok(SnapshotTargetType::Content),
            "directory" => Ok(SnapshotTargetType::Directory),
            "revision" => Ok(SnapshotTargetType::Revision),
            "release" => Ok(SnapshotTargetType::Release),
            "snapshot" => Ok(SnapshotTargetType::Snapshot),
            "alias" => Ok(SnapshotTargetType::Alias),
            _ => Err(SwhidError::InvalidFormat(format!("Unknown snapshot target type: {}", s))),
        }
    }
}

impl std::fmt::Display for SnapshotTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a snapshot branch
#[derive(Debug, Clone)]
pub struct SnapshotBranch {
    pub target: Vec<u8>,
    pub target_type: SnapshotTargetType,
}

impl SnapshotBranch {
    pub fn new(target: Vec<u8>, target_type: SnapshotTargetType) -> Self {
        Self {
            target,
            target_type,
        }
    }

    pub fn target(&self) -> &[u8] {
        &self.target
    }

    pub fn target_type(&self) -> SnapshotTargetType {
        self.target_type
    }

    pub fn swhid(&self) -> Option<Swhid> {
        let object_type = match self.target_type {
            SnapshotTargetType::Content => ObjectType::Content,
            SnapshotTargetType::Directory => ObjectType::Directory,
            SnapshotTargetType::Revision => ObjectType::Revision,
            SnapshotTargetType::Release => ObjectType::Release,
            SnapshotTargetType::Snapshot => ObjectType::Snapshot,
            SnapshotTargetType::Alias => return None, // Aliases don't have SWHIDs
        };
        if self.target.len() == 20 {
            let mut id = [0u8; 20];
            id.copy_from_slice(&self.target);
            Some(Swhid::new(object_type, id))
        } else {
            None
        }
    }
}

/// Represents a Git snapshot
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub branches: HashMap<Vec<u8>, Option<SnapshotBranch>>,
    pub id: [u8; 20],
    pub raw_manifest: Option<Vec<u8>>,
}

impl Snapshot {
    pub fn new(branches: HashMap<Vec<u8>, Option<SnapshotBranch>>) -> Self {
        let mut snapshot = Self {
            branches,
            id: [0u8; 20],
            raw_manifest: None,
        };
        
        snapshot.id = snapshot.compute_hash();
        snapshot
    }

    pub fn compute_hash(&self) -> [u8; 20] {
        let manifest = self.to_git_snapshot_manifest();
        // Hash using git-like header for the custom "snapshot" type per spec
        crate::hash::hash_git_object("snapshot", &manifest)
    }

    /// Build the snapshot manifest per SWHID v1.2 spec
    /// Each entry: type SP name NUL len ':' id (or alias target name / empty)
    pub fn to_git_snapshot_manifest(&self) -> Vec<u8> {
        let mut manifest = Vec::new();

        // Sort branches by name (bytes order)
        let mut sorted_branches: Vec<_> = self.branches.iter().collect();
        sorted_branches.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (name, branch_opt) in sorted_branches {
            match branch_opt {
                None => {
                    // Dangling branch
                    manifest.extend_from_slice(b"dangling");
                    manifest.push(b' ');
                    manifest.extend_from_slice(name);
                    manifest.push(0); // NUL
                    manifest.extend_from_slice(b"0:");
                    // No target bytes
                }
                Some(branch) => {
                    let type_str = match branch.target_type {
                        SnapshotTargetType::Content => b"content".as_ref(),
                        SnapshotTargetType::Directory => b"directory".as_ref(),
                        SnapshotTargetType::Revision => b"revision".as_ref(),
                        SnapshotTargetType::Release => b"release".as_ref(),
                        SnapshotTargetType::Snapshot => b"snapshot".as_ref(),
                        SnapshotTargetType::Alias => b"alias".as_ref(),
                    };

                    manifest.extend_from_slice(type_str);
                    manifest.push(b' ');
                    manifest.extend_from_slice(name);
                    manifest.push(0); // NUL

                    match branch.target_type {
                        SnapshotTargetType::Alias => {
                            // Alias: store the name of the target branch (raw bytes)
                            let alias_name = &branch.target;
                            let len_str = alias_name.len().to_string();
                            manifest.extend_from_slice(len_str.as_bytes());
                            manifest.push(b':');
                            manifest.extend_from_slice(alias_name);
                        }
                        _ => {
                            // Non-alias: target is a 20-byte identifier
                            let len_str = b"20";
                            manifest.extend_from_slice(len_str);
                            manifest.push(b':');
                            manifest.extend_from_slice(&branch.target);
                        }
                    }
                }
            }
        }

        manifest
    }

    pub fn swhid(&self) -> Swhid {
        Swhid::new(ObjectType::Snapshot, self.id)
    }

    pub fn branches(&self) -> &HashMap<Vec<u8>, Option<SnapshotBranch>> {
        &self.branches
    }

    pub fn get_branch(&self, name: &[u8]) -> Option<&SnapshotBranch> {
        self.branches.get(name).and_then(|opt| opt.as_ref())
    }

    pub fn add_branch(&mut self, name: Vec<u8>, branch: SnapshotBranch) {
        self.branches.insert(name, Some(branch));
        // Recompute hash after modification
        self.id = self.compute_hash();
    }

    pub fn remove_branch(&mut self, name: &[u8]) {
        self.branches.remove(name);
        // Recompute hash after modification
        self.id = self.compute_hash();
    }

    pub fn id(&self) -> &[u8; 20] {
        &self.id
    }

    pub fn raw_manifest(&self) -> Option<&[u8]> {
        self.raw_manifest.as_deref()
    }

    pub fn with_raw_manifest(mut self, manifest: Vec<u8>) -> Self {
        self.raw_manifest = Some(manifest);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_snapshot_target_type() {
        assert_eq!(SnapshotTargetType::Revision.as_str(), "revision");
        assert_eq!(SnapshotTargetType::from_str("revision").unwrap(), SnapshotTargetType::Revision);
        assert!(SnapshotTargetType::from_str("invalid").is_err());
    }

    #[test]
    fn test_snapshot_branch_creation() {
        let target = [0u8; 20];
        let branch = SnapshotBranch::new(target.to_vec(), SnapshotTargetType::Revision);

        assert_eq!(branch.target(), &target);
        assert_eq!(branch.target_type(), SnapshotTargetType::Revision);
    }

    #[test]
    fn test_snapshot_branch_swhid() {
        let target = [0u8; 20];
        let branch = SnapshotBranch::new(target.to_vec(), SnapshotTargetType::Revision);

        let swhid = branch.swhid().unwrap();
        assert_eq!(swhid.object_type(), ObjectType::Revision);
        assert_eq!(swhid.object_id(), &target);
    }

    #[test]
    fn test_snapshot_branch_alias_no_swhid() {
        let target = [0u8; 20];
        let branch = SnapshotBranch::new(target.to_vec(), SnapshotTargetType::Alias);

        assert_eq!(branch.swhid(), None);
    }

    #[test]
    fn test_snapshot_creation() {
        let mut branches = HashMap::new();
        let branch = SnapshotBranch::new([0u8; 20].to_vec(), SnapshotTargetType::Revision);
        branches.insert(b"main".to_vec(), Some(branch));

        let snapshot = Snapshot::new(branches);

        assert_eq!(snapshot.branches().len(), 1);
        assert!(snapshot.get_branch(b"main").is_some());
    }

    #[test]
    fn test_snapshot_swhid() {
        let branches = HashMap::new();
        let snapshot = Snapshot::new(branches);

        let swhid = snapshot.swhid();
        assert_eq!(swhid.object_type(), ObjectType::Snapshot);
        assert_eq!(swhid.object_id(), &snapshot.id);
    }

    #[test]
    fn test_snapshot_add_branch() {
        let branches = HashMap::new();
        let mut snapshot = Snapshot::new(branches);

        let branch = SnapshotBranch::new([1u8; 20].to_vec(), SnapshotTargetType::Revision);
        snapshot.add_branch(b"main".to_vec(), branch);

        assert_eq!(snapshot.branches().len(), 1);
        assert!(snapshot.get_branch(b"main").is_some());
    }

    #[test]
    fn test_snapshot_remove_branch() {
        let mut branches = HashMap::new();
        let branch = SnapshotBranch::new([0u8; 20].to_vec(), SnapshotTargetType::Revision);
        branches.insert(b"main".to_vec(), Some(branch));

        let mut snapshot = Snapshot::new(branches);
        snapshot.remove_branch(b"main");

        assert_eq!(snapshot.branches().len(), 0);
        assert!(snapshot.get_branch(b"main").is_none());
    }
} 