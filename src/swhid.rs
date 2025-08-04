use std::fmt;
use std::str::FromStr;
use crate::error::SwhidError;

/// Software Heritage object types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Content,
    Directory,
    Revision,
    Release,
    Snapshot,
}

impl ObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectType::Content => "cnt",
            ObjectType::Directory => "dir",
            ObjectType::Revision => "rev",
            ObjectType::Release => "rel",
            ObjectType::Snapshot => "snp",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, SwhidError> {
        match s {
            "cnt" => Ok(ObjectType::Content),
            "dir" => Ok(ObjectType::Directory),
            "rev" => Ok(ObjectType::Revision),
            "rel" => Ok(ObjectType::Release),
            "snp" => Ok(ObjectType::Snapshot),
            _ => Err(SwhidError::InvalidObjectType(s.to_string())),
        }
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Core Software Heritage Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Swhid {
    namespace: String,
    scheme_version: u32,
    object_type: ObjectType,
    object_id: [u8; 20],
}

impl Swhid {
    pub const NAMESPACE: &'static str = "swh";
    pub const SCHEME_VERSION: u32 = 1;

    pub fn new(object_type: ObjectType, object_id: [u8; 20]) -> Self {
        Self {
            namespace: Self::NAMESPACE.to_string(),
            scheme_version: Self::SCHEME_VERSION,
            object_type,
            object_id,
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn scheme_version(&self) -> u32 {
        self.scheme_version
    }

    pub fn object_type(&self) -> ObjectType {
        self.object_type
    }

    pub fn object_id(&self) -> &[u8; 20] {
        &self.object_id
    }

    /// Parse SWHID from string
    pub fn from_string(s: &str) -> Result<Self, SwhidError> {
        let parts: Vec<&str> = s.split(':').collect();
        
        if parts.len() != 4 {
            return Err(SwhidError::InvalidFormat(format!(
                "SWHID must have 4 parts, got {}: {}", 
                parts.len(), s
            )));
        }

        let namespace = parts[0];
        if namespace != Self::NAMESPACE {
            return Err(SwhidError::InvalidNamespace(namespace.to_string()));
        }

        let scheme_version = parts[1].parse::<u32>()
            .map_err(|_| SwhidError::InvalidVersion(parts[1].to_string()))?;
        
        if scheme_version != Self::SCHEME_VERSION {
            return Err(SwhidError::InvalidVersion(scheme_version.to_string()));
        }

        let object_type = ObjectType::from_str(parts[2])?;

        let object_id_hex = parts[3];
        if object_id_hex.len() != 40 {
            return Err(SwhidError::InvalidHashLength(object_id_hex.len()));
        }

        let object_id = hex::decode(object_id_hex)
            .map_err(|_| SwhidError::InvalidHash(object_id_hex.to_string()))?;

        if object_id.len() != 20 {
            return Err(SwhidError::InvalidHashLength(object_id.len()));
        }

        let mut id_array = [0u8; 20];
        id_array.copy_from_slice(&object_id);

        Ok(Self {
            namespace: namespace.to_string(),
            scheme_version,
            object_type,
            object_id: id_array,
        })
    }
}

impl FromStr for Swhid {
    type Err = SwhidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

impl fmt::Display for Swhid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}",
            self.namespace,
            self.scheme_version,
            self.object_type,
            hex::encode(self.object_id)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swhid_creation() {
        let object_id = [0u8; 20];
        let swhid = Swhid::new(ObjectType::Content, object_id);
        
        assert_eq!(swhid.namespace(), "swh");
        assert_eq!(swhid.scheme_version(), 1);
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(swhid.object_id(), &object_id);
    }

    #[test]
    fn test_swhid_parsing() {
        let swhid_str = "swh:1:cnt:0000000000000000000000000000000000000000";
        let swhid = Swhid::from_string(swhid_str).unwrap();
        
        assert_eq!(swhid.namespace(), "swh");
        assert_eq!(swhid.scheme_version(), 1);
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(swhid.object_id(), &[0u8; 20]);
    }

    #[test]
    fn test_swhid_display() {
        let object_id = [0u8; 20];
        let swhid = Swhid::new(ObjectType::Content, object_id);
        let expected = "swh:1:cnt:0000000000000000000000000000000000000000";
        
        assert_eq!(swhid.to_string(), expected);
    }

    #[test]
    fn test_invalid_format() {
        let result = Swhid::from_string("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_namespace() {
        let result = Swhid::from_string("invalid:1:cnt:0000000000000000000000000000000000000000");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_object_type() {
        let result = Swhid::from_string("swh:1:invalid:0000000000000000000000000000000000000000");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_hash_length() {
        let result = Swhid::from_string("swh:1:cnt:123");
        assert!(result.is_err());
    }
} 