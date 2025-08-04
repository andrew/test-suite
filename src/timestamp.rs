use std::fmt;
use crate::error::SwhidError;
use chrono::{DateTime, Utc};

/// Represents a naive timestamp from a VCS
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Timestamp {
    pub seconds: i64,
    pub microseconds: u32,
}

impl Timestamp {
    pub const MIN_SECONDS: i64 = -62135510961; // 0001-01-02T00:00:00
    pub const MAX_SECONDS: i64 = 253402297199; // 9999-12-31T23:59:59
    pub const MIN_MICROSECONDS: u32 = 0;
    pub const MAX_MICROSECONDS: u32 = 1_000_000 - 1;

    /// Create a new timestamp from seconds and microseconds
    pub fn new(seconds: i64, microseconds: u32) -> Result<Self, SwhidError> {
        if seconds < Self::MIN_SECONDS || seconds > Self::MAX_SECONDS {
            return Err(SwhidError::InvalidFormat(format!(
                "Seconds out of range: {} (must be between {} and {})",
                seconds, Self::MIN_SECONDS, Self::MAX_SECONDS
            )));
        }

        if microseconds > Self::MAX_MICROSECONDS {
            return Err(SwhidError::InvalidFormat(format!(
                "Microseconds out of range: {} (must be between {} and {})",
                microseconds, Self::MIN_MICROSECONDS, Self::MAX_MICROSECONDS
            )));
        }

        Ok(Self {
            seconds,
            microseconds,
        })
    }

    /// Create a timestamp from Unix timestamp (seconds since epoch)
    pub fn from_unix(seconds: i64) -> Result<Self, SwhidError> {
        Self::new(seconds, 0)
    }

    /// Create a timestamp from Unix timestamp with microseconds
    pub fn from_unix_with_microseconds(seconds: i64, microseconds: u32) -> Result<Self, SwhidError> {
        Self::new(seconds, microseconds)
    }

    /// Get the timestamp as a dictionary representation
    pub fn to_dict(&self) -> serde_json::Value {
        serde_json::json!({
            "seconds": self.seconds,
            "microseconds": self.microseconds
        })
    }

    /// Create a timestamp from a dictionary
    pub fn from_dict(dict: &serde_json::Value) -> Result<Self, SwhidError> {
        let seconds = dict["seconds"]
            .as_i64()
            .ok_or_else(|| SwhidError::InvalidFormat("Missing or invalid seconds".to_string()))?;
        
        let microseconds = dict["microseconds"]
            .as_u64()
            .unwrap_or(0) as u32;

        Self::new(seconds, microseconds)
    }

    /// Format timestamp for git object
    pub fn format_for_git(&self) -> Vec<u8> {
        if self.microseconds == 0 {
            format!("{}", self.seconds).into_bytes()
        } else {
            format!("{}.{:06}", self.seconds, self.microseconds).into_bytes()
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.microseconds == 0 {
            write!(f, "{}", self.seconds)
        } else {
            write!(f, "{}.{:06}", self.seconds, self.microseconds)
        }
    }
}

/// Represents a TZ-aware timestamp from a VCS
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimestampWithTimezone {
    pub timestamp: Timestamp,
    pub offset_bytes: Vec<u8>,
}

impl TimestampWithTimezone {
    /// Create a new timestamp with timezone
    pub fn new(timestamp: Timestamp, offset_bytes: Vec<u8>) -> Self {
        Self {
            timestamp,
            offset_bytes,
        }
    }

    /// Create from numeric offset
    pub fn from_numeric_offset(timestamp: Timestamp, offset: i32, negative_utc: bool) -> Self {
        let offset_str = if negative_utc {
            format!("-{:02}:{:02}", offset / 60, offset % 60)
        } else {
            format!("+{:02}:{:02}", offset / 60, offset % 60)
        };
        
        Self {
            timestamp,
            offset_bytes: offset_str.into_bytes(),
        }
    }

    /// Create from datetime
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        let timestamp = Timestamp::from_unix(dt.timestamp()).unwrap();
        // For UTC, offset is 0
        Self::from_numeric_offset(timestamp, 0, false)
    }

    /// Convert to datetime
    pub fn to_datetime(&self) -> Result<DateTime<Utc>, SwhidError> {
        let offset_minutes = self.offset_minutes()?;
        let offset_seconds = offset_minutes as i64 * 60;
        
        let utc_seconds = self.timestamp.seconds - offset_seconds;
        
        DateTime::from_timestamp(utc_seconds, self.timestamp.microseconds * 1000)
            .ok_or_else(|| SwhidError::InvalidFormat("Invalid timestamp".to_string()))
    }

    /// Parse offset bytes to get offset in minutes
    pub fn offset_minutes(&self) -> Result<i32, SwhidError> {
        let offset_str = String::from_utf8(self.offset_bytes.clone())
            .map_err(|e| SwhidError::InvalidFormat(format!("Invalid UTF-8 in offset: {}", e)))?;
        
        if offset_str.len() != 6 || !offset_str.starts_with(['+', '-']) {
            return Err(SwhidError::InvalidFormat(format!("Invalid offset format: {}", offset_str)));
        }
        
        let sign = if offset_str.starts_with('+') { 1 } else { -1 };
        let hours: i32 = offset_str[1..3].parse()
            .map_err(|_| SwhidError::InvalidFormat("Invalid hours in offset".to_string()))?;
        let minutes: i32 = offset_str[4..6].parse()
            .map_err(|_| SwhidError::InvalidFormat("Invalid minutes in offset".to_string()))?;
        
        Ok(sign * (hours * 60 + minutes))
    }

    /// Get the timestamp as a dictionary representation
    pub fn to_dict(&self) -> serde_json::Value {
        serde_json::json!({
            "timestamp": self.timestamp.to_dict(),
            "offset_bytes": String::from_utf8_lossy(&self.offset_bytes)
        })
    }

    /// Create from a dictionary
    pub fn from_dict(dict: &serde_json::Value) -> Result<Self, SwhidError> {
        let timestamp = Timestamp::from_dict(&dict["timestamp"])?;
        let offset_str = dict["offset_bytes"]
            .as_str()
            .ok_or_else(|| SwhidError::InvalidFormat("Missing or invalid offset_bytes".to_string()))?;
        
        Ok(Self {
            timestamp,
            offset_bytes: offset_str.as_bytes().to_vec(),
        })
    }

    /// Format timestamp with timezone for git object
    pub fn format_for_git(&self) -> Vec<u8> {
        let mut result = self.timestamp.format_for_git();
        result.extend_from_slice(&self.offset_bytes);
        result
    }
}

impl fmt::Display for TimestampWithTimezone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.timestamp, String::from_utf8_lossy(&self.offset_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_creation() {
        let ts = Timestamp::new(1234567890, 123456).unwrap();
        assert_eq!(ts.seconds, 1234567890);
        assert_eq!(ts.microseconds, 123456);
    }

    #[test]
    fn test_timestamp_validation() {
        assert!(Timestamp::new(Timestamp::MIN_SECONDS - 1, 0).is_err());
        assert!(Timestamp::new(Timestamp::MAX_SECONDS + 1, 0).is_err());
        assert!(Timestamp::new(0, Timestamp::MAX_MICROSECONDS + 1).is_err());
    }

    #[test]
    fn test_timestamp_format_for_git() {
        let ts = Timestamp::new(1234567890, 0).unwrap();
        assert_eq!(ts.format_for_git(), b"1234567890");
        
        let ts = Timestamp::new(1234567890, 123456).unwrap();
        assert_eq!(ts.format_for_git(), b"1234567890.123456");
    }

    #[test]
    fn test_timestamp_with_timezone() {
        let ts = Timestamp::new(1234567890, 0).unwrap();
        let tz = TimestampWithTimezone::from_numeric_offset(ts, 300, false); // +05:00
        assert_eq!(tz.offset_minutes().unwrap(), 300);
    }

    #[test]
    fn test_timestamp_with_timezone_format() {
        let ts = Timestamp::new(1234567890, 0).unwrap();
        let tz = TimestampWithTimezone::from_numeric_offset(ts, 300, false);
        assert_eq!(tz.format_for_git(), b"1234567890+05:00");
    }
} 