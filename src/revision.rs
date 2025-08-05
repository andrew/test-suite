use std::collections::HashMap;
use crate::swhid::{Swhid, ObjectType};
use crate::person::Person;
use crate::timestamp::TimestampWithTimezone;
use crate::error::SwhidError;

/// Revision type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RevisionType {
    Git,
    Tar,
    Dsc,
    Subversion,
    Mercurial,
    Cvs,
    Bazaar,
}

impl RevisionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RevisionType::Git => "git",
            RevisionType::Tar => "tar",
            RevisionType::Dsc => "dsc",
            RevisionType::Subversion => "svn",
            RevisionType::Mercurial => "hg",
            RevisionType::Cvs => "cvs",
            RevisionType::Bazaar => "bzr",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, SwhidError> {
        match s {
            "git" => Ok(RevisionType::Git),
            "tar" => Ok(RevisionType::Tar),
            "dsc" => Ok(RevisionType::Dsc),
            "svn" => Ok(RevisionType::Subversion),
            "hg" => Ok(RevisionType::Mercurial),
            "cvs" => Ok(RevisionType::Cvs),
            "bzr" => Ok(RevisionType::Bazaar),
            _ => Err(SwhidError::InvalidFormat(format!("Unknown revision type: {}", s))),
        }
    }
}

impl std::fmt::Display for RevisionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a Git revision
#[derive(Debug, Clone)]
pub struct Revision {
    pub message: Option<Vec<u8>>,
    pub author: Option<Person>,
    pub committer: Option<Person>,
    pub date: Option<TimestampWithTimezone>,
    pub committer_date: Option<TimestampWithTimezone>,
    pub revision_type: RevisionType,
    pub directory: [u8; 20],
    pub synthetic: bool,
    pub metadata: Option<HashMap<String, String>>,
    pub parents: Vec<[u8; 20]>,
    pub extra_headers: Vec<(Vec<u8>, Vec<u8>)>,
    pub id: [u8; 20],
    pub raw_manifest: Option<Vec<u8>>,
}

impl Revision {
    pub fn new(
        message: Option<Vec<u8>>,
        author: Option<Person>,
        committer: Option<Person>,
        date: Option<TimestampWithTimezone>,
        committer_date: Option<TimestampWithTimezone>,
        revision_type: RevisionType,
        directory: [u8; 20],
        synthetic: bool,
        metadata: Option<HashMap<String, String>>,
        parents: Vec<[u8; 20]>,
        extra_headers: Vec<(Vec<u8>, Vec<u8>)>,
    ) -> Self {
        let mut revision = Self {
            message,
            author,
            committer,
            date,
            committer_date,
            revision_type,
            directory,
            synthetic,
            metadata,
            parents,
            extra_headers,
            id: [0u8; 20],
            raw_manifest: None,
        };
        
        revision.id = revision.compute_hash();
        revision
    }

    pub fn compute_hash(&self) -> [u8; 20] {
        let manifest = self.to_git_object();
        crate::hash::hash_git_object("commit", &manifest)
    }

    pub fn to_git_object(&self) -> Vec<u8> {
        let mut parts = Vec::new();

        // Tree
        parts.push(format!("tree {}", hex::encode(self.directory)).into_bytes());

        // Parents
        for parent in &self.parents {
            parts.push(format!("parent {}", hex::encode(parent)).into_bytes());
        }

        // Author
        if let Some(ref author) = self.author {
            if let Some(ref date) = self.date {
                parts.push(format!("author {} {}", author, date).into_bytes());
            }
        }

        // Committer
        if let Some(ref committer) = self.committer {
            if let Some(ref committer_date) = self.committer_date {
                parts.push(format!("committer {} {}", committer, committer_date).into_bytes());
            }
        }

        // Extra headers
        for (key, value) in &self.extra_headers {
            parts.push([key.as_slice(), b" ", value.as_slice()].concat());
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
        Swhid::new(ObjectType::Revision, self.id)
    }

    pub fn directory_swhid(&self) -> Swhid {
        Swhid::new(ObjectType::Directory, self.directory)
    }

    pub fn parent_swhids(&self) -> Vec<Swhid> {
        self.parents.iter().map(|p| Swhid::new(ObjectType::Revision, *p)).collect()
    }

    pub fn message(&self) -> Option<&[u8]> {
        self.message.as_deref()
    }

    pub fn author(&self) -> Option<&Person> {
        self.author.as_ref()
    }

    pub fn committer(&self) -> Option<&Person> {
        self.committer.as_ref()
    }

    pub fn date(&self) -> Option<&TimestampWithTimezone> {
        self.date.as_ref()
    }

    pub fn committer_date(&self) -> Option<&TimestampWithTimezone> {
        self.committer_date.as_ref()
    }

    pub fn revision_type(&self) -> RevisionType {
        self.revision_type
    }

    pub fn directory(&self) -> &[u8; 20] {
        &self.directory
    }

    pub fn synthetic(&self) -> bool {
        self.synthetic
    }

    pub fn metadata(&self) -> Option<&HashMap<String, String>> {
        self.metadata.as_ref()
    }

    pub fn parents(&self) -> &[[u8; 20]] {
        &self.parents
    }

    pub fn extra_headers(&self) -> &[(Vec<u8>, Vec<u8>)] {
        &self.extra_headers
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
    fn test_revision_type() {
        assert_eq!(RevisionType::Git.as_str(), "git");
        assert_eq!(RevisionType::from_str("git").unwrap(), RevisionType::Git);
        assert!(RevisionType::from_str("invalid").is_err());
    }

    #[test]
    fn test_revision_creation() {
        let directory = [0u8; 20];
        let revision = Revision::new(
            Some(b"Initial commit".to_vec()),
            None,
            None,
            None,
            None,
            RevisionType::Git,
            directory,
            false,
            None,
            vec![],
            vec![],
        );

        assert_eq!(revision.message(), Some(b"Initial commit".as_slice()));
        assert_eq!(revision.revision_type(), RevisionType::Git);
        assert_eq!(revision.directory(), &directory);
        assert!(!revision.synthetic());
        assert_eq!(revision.parents().len(), 0);
    }

    #[test]
    fn test_revision_with_author_and_committer() {
        let author = Person::from_fullname("John Doe <john@example.com>").unwrap();
        let committer = Person::from_fullname("Jane Smith <jane@example.com>").unwrap();
        let timestamp = Timestamp::new(1234567890, 0).unwrap();
        let date = TimestampWithTimezone::from_numeric_offset(timestamp, 0, false);

        let revision = Revision::new(
            Some(b"Test commit".to_vec()),
            Some(author.clone()),
            Some(committer.clone()),
            Some(date.clone()),
            Some(date.clone()),
            RevisionType::Git,
            [0u8; 20],
            false,
            None,
            vec![],
            vec![],
        );

        assert_eq!(revision.author(), Some(&author));
        assert_eq!(revision.committer(), Some(&committer));
        assert_eq!(revision.date(), Some(&date));
        assert_eq!(revision.committer_date(), Some(&date));
    }

    #[test]
    fn test_revision_swhid() {
        let revision = Revision::new(
            Some(b"Test".to_vec()),
            None,
            None,
            None,
            None,
            RevisionType::Git,
            [0u8; 20],
            false,
            None,
            vec![],
            vec![],
        );

        let swhid = revision.swhid();
        assert_eq!(swhid.object_type(), ObjectType::Revision);
        assert_eq!(swhid.object_id(), &revision.id);
    }

    #[test]
    fn test_revision_with_parents() {
        let parent1 = [1u8; 20];
        let parent2 = [2u8; 20];
        let parents = vec![parent1, parent2];

        let revision = Revision::new(
            Some(b"Merge commit".to_vec()),
            None,
            None,
            None,
            None,
            RevisionType::Git,
            [0u8; 20],
            false,
            None,
            parents.clone(),
            vec![],
        );

        assert_eq!(revision.parents(), &[parent1, parent2]);
        assert_eq!(revision.parent_swhids().len(), 2);
    }
} 