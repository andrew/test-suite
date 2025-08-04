use std::fmt;
use crate::error::SwhidError;
use crate::timestamp::TimestampWithTimezone;

/// Represents the author/committer of a revision or release
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Person {
    pub fullname: Vec<u8>,
    pub name: Option<Vec<u8>>,
    pub email: Option<Vec<u8>>,
}

impl Person {
    /// Create a new Person from fullname
    pub fn new(fullname: Vec<u8>) -> Self {
        Self {
            fullname,
            name: None,
            email: None,
        }
    }

    /// Create a Person from fullname, name, and email
    pub fn with_details(fullname: Vec<u8>, name: Option<Vec<u8>>, email: Option<Vec<u8>>) -> Self {
        Self {
            fullname,
            name,
            email,
        }
    }

    /// Create a Person from a fullname string (e.g., "John Doe <john@example.com>")
    pub fn from_fullname(fullname: &str) -> Result<Self, SwhidError> {
        let fullname_bytes = fullname.as_bytes().to_vec();
        
        // Parse email if present
        if let Some(email_start) = fullname.rfind('<') {
            if let Some(email_end) = fullname.rfind('>') {
                if email_start < email_end {
                    let email = fullname[email_start + 1..email_end].as_bytes().to_vec();
                    let name_part = fullname[..email_start].trim();
                    let name = if name_part.is_empty() {
                        None
                    } else {
                        Some(name_part.as_bytes().to_vec())
                    };
                    
                    return Ok(Self {
                        fullname: fullname_bytes,
                        name,
                        email: Some(email),
                    });
                }
            }
        }
        
        // No email found, use fullname as name
        let name = if fullname.trim().is_empty() {
            None
        } else {
            Some(fullname.trim().as_bytes().to_vec())
        };
        
        Ok(Self {
            fullname: fullname_bytes,
            name,
            email: None,
        })
    }

    /// Get the fullname as a string
    pub fn fullname_str(&self) -> Result<String, SwhidError> {
        String::from_utf8(self.fullname.clone())
            .map_err(|e| SwhidError::InvalidFormat(format!("Invalid UTF-8 in fullname: {}", e)))
    }

    /// Get the name as a string
    pub fn name_str(&self) -> Result<Option<String>, SwhidError> {
        match &self.name {
            Some(name) => String::from_utf8(name.clone())
                .map(Some)
                .map_err(|e| SwhidError::InvalidFormat(format!("Invalid UTF-8 in name: {}", e))),
            None => Ok(None),
        }
    }

    /// Get the email as a string
    pub fn email_str(&self) -> Result<Option<String>, SwhidError> {
        match &self.email {
            Some(email) => String::from_utf8(email.clone())
                .map(Some)
                .map_err(|e| SwhidError::InvalidFormat(format!("Invalid UTF-8 in email: {}", e))),
            None => Ok(None),
        }
    }

    /// Format person data for git object (author/committer line)
    pub fn format_for_git(&self, date: Option<&TimestampWithTimezone>) -> Vec<u8> {
        let mut result = self.fullname.clone();
        
        if let Some(date) = date {
            result.push(b' ');
            result.extend_from_slice(&date.format_for_git());
        }
        
        result
    }

    /// Create an anonymized version of the person
    pub fn anonymize(&self) -> Person {
        Self {
            fullname: b"Anonymous <anonymous@example.com>".to_vec(),
            name: Some(b"Anonymous".to_vec()),
            email: Some(b"anonymous@example.com".to_vec()),
        }
    }
}

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.name_str(), self.email_str()) {
            (Ok(Some(name)), Ok(Some(email))) => {
                write!(f, "{} <{}>", name, email)
            }
            (Ok(Some(name)), _) => {
                write!(f, "{}", name)
            }
            _ => {
                write!(f, "{}", String::from_utf8_lossy(&self.fullname))
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_from_fullname() {
        let person = Person::from_fullname("John Doe <john@example.com>").unwrap();
        assert_eq!(person.name_str().unwrap(), Some("John Doe".to_string()));
        assert_eq!(person.email_str().unwrap(), Some("john@example.com".to_string()));
    }

    #[test]
    fn test_person_from_fullname_no_email() {
        let person = Person::from_fullname("John Doe").unwrap();
        assert_eq!(person.name_str().unwrap(), Some("John Doe".to_string()));
        assert_eq!(person.email_str().unwrap(), None);
    }

    #[test]
    fn test_person_anonymize() {
        let person = Person::from_fullname("John Doe <john@example.com>").unwrap();
        let anonymized = person.anonymize();
        assert_eq!(anonymized.name_str().unwrap(), Some("Anonymous".to_string()));
        assert_eq!(anonymized.email_str().unwrap(), Some("anonymous@example.com".to_string()));
    }

    #[test]
    fn test_person_display() {
        let person = Person::from_fullname("John Doe <john@example.com>").unwrap();
        assert_eq!(person.to_string(), "John Doe <john@example.com>");
    }
} 