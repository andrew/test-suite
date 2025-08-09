use std::collections::HashMap;
use crate::swhid::{Swhid, ObjectType};
use crate::person::Person;
use crate::timestamp::TimestampWithTimezone;
use crate::error::SwhidError;

/// Release target type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReleaseTargetType {
    Content,
    Directory,
    Revision,
    Release,
    Snapshot,
}

impl ReleaseTargetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReleaseTargetType::Content => "content",
            ReleaseTargetType::Directory => "directory",
            ReleaseTargetType::Revision => "revision",
            ReleaseTargetType::Release => "release",
            ReleaseTargetType::Snapshot => "snapshot",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, SwhidError> {
        match s {
            "content" => Ok(ReleaseTargetType::Content),
            "directory" => Ok(ReleaseTargetType::Directory),
            "revision" => Ok(ReleaseTargetType::Revision),
            "release" => Ok(ReleaseTargetType::Release),
            "snapshot" => Ok(ReleaseTargetType::Snapshot),
            _ => Err(SwhidError::InvalidFormat(format!("Unknown release target type: {}", s))),
        }
    }
}

impl std::fmt::Display for ReleaseTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a Git release
#[derive(Debug, Clone)]
pub struct Release {
    pub name: Vec<u8>,
    pub message: Option<Vec<u8>>,
    pub target: Option<[u8; 20]>,
    pub target_type: ReleaseTargetType,
    pub synthetic: bool,
    pub author: Option<Person>,
    pub date: Option<TimestampWithTimezone>,
    pub metadata: Option<HashMap<String, String>>,
    pub id: [u8; 20],
    pub raw_manifest: Option<Vec<u8>>,
}

impl Release {
    pub fn new(
        name: Vec<u8>,
        message: Option<Vec<u8>>,
        target: Option<[u8; 20]>,
        target_type: ReleaseTargetType,
        synthetic: bool,
        author: Option<Person>,
        date: Option<TimestampWithTimezone>,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        let mut release = Self {
            name,
            message,
            target,
            target_type,
            synthetic,
            author,
            date,
            metadata,
            id: [0u8; 20],
            raw_manifest: None,
        };
        
        release.id = release.compute_hash();
        release
    }

    pub fn compute_hash(&self) -> [u8; 20] {
        let manifest = self.to_git_object();
        crate::hash::hash_git_object("tag", &manifest)
    }

    pub fn to_git_object(&self) -> Vec<u8> {
        let mut parts = Vec::new();

        // Object
        if let Some(target) = self.target {
            parts.push(format!("object {}", hex::encode(target)).into_bytes());
        }

        // Type (map SWH target type to Git object type per spec)
        let git_type = match self.target_type {
            ReleaseTargetType::Content => "blob",
            ReleaseTargetType::Directory => "tree",
            ReleaseTargetType::Revision => "commit",
            ReleaseTargetType::Release => "tag",
            ReleaseTargetType::Snapshot => "refs",
        };
        parts.push(format!("type {}", git_type).into_bytes());

        // Tag
        parts.push(format!("tag {}", String::from_utf8_lossy(&self.name)).into_bytes());

        // Tagger
        if let Some(ref author) = self.author {
            if let Some(ref date) = self.date {
                parts.push(format!("tagger {} {}", author, date).into_bytes());
            }
        }

        // Empty line
        parts.push(Vec::new());

        // Message
        if let Some(ref message) = self.message {
            parts.push(message.clone());
        }

        // Concatenate all parts
        let mut result = Vec::new();
        for part in parts {
            result.extend_from_slice(&part);
            result.push(b'\n');
        }
        result
    }

    pub fn swhid(&self) -> Swhid {
        Swhid::new(ObjectType::Release, self.id)
    }

    pub fn target_swhid(&self) -> Option<Swhid> {
        self.target.map(|target| {
            let object_type = match self.target_type {
                ReleaseTargetType::Content => ObjectType::Content,
                ReleaseTargetType::Directory => ObjectType::Directory,
                ReleaseTargetType::Revision => ObjectType::Revision,
                ReleaseTargetType::Release => ObjectType::Release,
                ReleaseTargetType::Snapshot => ObjectType::Snapshot,
            };
            Swhid::new(object_type, target)
        })
    }

    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub fn message(&self) -> Option<&[u8]> {
        self.message.as_deref()
    }

    pub fn target(&self) -> Option<&[u8; 20]> {
        self.target.as_ref()
    }

    pub fn target_type(&self) -> ReleaseTargetType {
        self.target_type
    }

    pub fn synthetic(&self) -> bool {
        self.synthetic
    }

    pub fn author(&self) -> Option<&Person> {
        self.author.as_ref()
    }

    pub fn date(&self) -> Option<&TimestampWithTimezone> {
        self.date.as_ref()
    }

    pub fn metadata(&self) -> Option<&HashMap<String, String>> {
        self.metadata.as_ref()
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
    use crate::person::Person;
    use crate::timestamp::{Timestamp, TimestampWithTimezone};

    #[test]
    fn test_release_target_type() {
        assert_eq!(ReleaseTargetType::Revision.as_str(), "revision");
        assert_eq!(ReleaseTargetType::from_str("revision").unwrap(), ReleaseTargetType::Revision);
        assert!(ReleaseTargetType::from_str("invalid").is_err());
    }

    #[test]
    fn test_release_creation() {
        let target = [0u8; 20];
        let release = Release::new(
            b"v1.0.0".to_vec(),
            Some(b"Release v1.0.0".to_vec()),
            Some(target),
            ReleaseTargetType::Revision,
            false,
            None,
            None,
            None,
        );

        assert_eq!(release.name(), b"v1.0.0");
        assert_eq!(release.message(), Some(b"Release v1.0.0".as_slice()));
        assert_eq!(release.target(), Some(&target));
        assert_eq!(release.target_type(), ReleaseTargetType::Revision);
        assert!(!release.synthetic());
    }

    #[test]
    fn test_release_with_author_and_date() {
        let author = Person::from_fullname("John Doe <john@example.com>").unwrap();
        let timestamp = Timestamp::new(1234567890, 0).unwrap();
        let date = TimestampWithTimezone::from_numeric_offset(timestamp, 0, false);

        let release = Release::new(
            b"v2.0.0".to_vec(),
            Some(b"Release v2.0.0".to_vec()),
            Some([0u8; 20]),
            ReleaseTargetType::Revision,
            false,
            Some(author.clone()),
            Some(date.clone()),
            None,
        );

        assert_eq!(release.author(), Some(&author));
        assert_eq!(release.date(), Some(&date));
    }

    #[test]
    fn test_release_swhid() {
        let release = Release::new(
            b"v1.0.0".to_vec(),
            Some(b"Test release".to_vec()),
            Some([0u8; 20]),
            ReleaseTargetType::Revision,
            false,
            None,
            None,
            None,
        );

        let swhid = release.swhid();
        assert_eq!(swhid.object_type(), ObjectType::Release);
        assert_eq!(swhid.object_id(), &release.id);
    }

    #[test]
    fn test_release_target_swhid() {
        let target = [1u8; 20];
        let release = Release::new(
            b"v1.0.0".to_vec(),
            None,
            Some(target),
            ReleaseTargetType::Revision,
            false,
            None,
            None,
            None,
        );

        let target_swhid = release.target_swhid().unwrap();
        assert_eq!(target_swhid.object_type(), ObjectType::Revision);
        assert_eq!(target_swhid.object_id(), &target);
    }

    #[test]
    fn test_release_without_target() {
        let release = Release::new(
            b"v1.0.0".to_vec(),
            None,
            None,
            ReleaseTargetType::Revision,
            false,
            None,
            None,
            None,
        );

        assert_eq!(release.target_swhid(), None);
    }
} 