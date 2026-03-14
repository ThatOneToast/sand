use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{Result, SandError};

/// A parsed Minecraft release version (e.g. `1.21.4` or `1.21`).
///
/// Supports two-part (`major.minor`) and three-part (`major.minor.patch`)
/// version strings. When only two parts are provided, `patch` defaults to `0`.
///
/// Snapshot versions (e.g. `24w45a`) are not supported and will return an
/// error from `parse`.
///
/// # Ordering
/// Versions are ordered lexicographically by `(major, minor, patch)`.
///
/// # Examples
/// ```
/// use sand_core::McVersion;
///
/// let v: McVersion = "1.21.4".parse().unwrap();
/// assert_eq!(v.major, 1);
/// assert_eq!(v.minor, 21);
/// assert_eq!(v.patch, 4);
///
/// let v2: McVersion = "1.21".parse().unwrap();
/// assert_eq!(v2.patch, 0);
/// assert!(v2 < v);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct McVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl McVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse a version string. Equivalent to `s.parse::<McVersion>()`.
    pub fn parse(s: &str) -> Result<Self> {
        s.parse()
    }
}

impl FromStr for McVersion {
    type Err = SandError;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        match parts.as_slice() {
            [major, minor] => Ok(Self {
                major: parse_u32(major, s)?,
                minor: parse_u32(minor, s)?,
                patch: 0,
            }),
            [major, minor, patch] => Ok(Self {
                major: parse_u32(major, s)?,
                minor: parse_u32(minor, s)?,
                patch: parse_u32(patch, s)?,
            }),
            _ => Err(SandError::InvalidVersion(s.to_string())),
        }
    }
}

fn parse_u32(s: &str, full: &str) -> Result<u32> {
    s.parse::<u32>()
        .map_err(|_| SandError::InvalidVersion(full.to_string()))
}

impl fmt::Display for McVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Serialize for McVersion {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for McVersion {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_three_part() {
        let v: McVersion = "1.21.4".parse().unwrap();
        assert_eq!(v, McVersion::new(1, 21, 4));
    }

    #[test]
    fn parse_two_part() {
        let v: McVersion = "1.21".parse().unwrap();
        assert_eq!(v, McVersion::new(1, 21, 0));
    }

    #[test]
    fn parse_invalid_alpha() {
        assert!("1.21.abc".parse::<McVersion>().is_err());
    }

    #[test]
    fn parse_invalid_single_part() {
        assert!("1".parse::<McVersion>().is_err());
    }

    #[test]
    fn parse_empty() {
        assert!("".parse::<McVersion>().is_err());
    }

    #[test]
    fn parse_snapshot_invalid() {
        assert!("24w45a".parse::<McVersion>().is_err());
    }

    #[test]
    fn ordering() {
        let v1: McVersion = "1.20.0".parse().unwrap();
        let v2: McVersion = "1.21.4".parse().unwrap();
        let v3: McVersion = "1.21.11".parse().unwrap();
        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn display() {
        let v = McVersion::new(1, 21, 4);
        assert_eq!(v.to_string(), "1.21.4");
    }

    #[test]
    fn serde_roundtrip() {
        let v = McVersion::new(1, 21, 11);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, r#""1.21.11""#);
        let back: McVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }
}
