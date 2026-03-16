use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{Result, SandError};

/// A validated Minecraft resource location in the form `namespace:path`.
///
/// - **namespace** must match `[a-z0-9_.-]+`
/// - **path** must match `[a-z0-9_./-]+`
///
/// # Examples
/// ```
/// use sand_core::ResourceLocation;
///
/// let loc = ResourceLocation::new("minecraft", "oak_log").unwrap();
/// assert_eq!(loc.to_string(), "minecraft:oak_log");
///
/// let loc = ResourceLocation::minecraft("stone").unwrap();
/// assert_eq!(loc.namespace(), "minecraft");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    namespace: String,
    path: String,
}

impl ResourceLocation {
    /// Construct a `ResourceLocation`, returning an error if either part
    /// contains invalid characters or is empty.
    ///
    /// - **namespace** must match `[a-z0-9_.-]+`
    /// - **path** must match `[a-z0-9_./-]+`
    pub fn new(namespace: impl AsRef<str>, path: impl AsRef<str>) -> Result<Self> {
        let namespace = namespace.as_ref();
        let path = path.as_ref();
        validate_namespace(namespace)?;
        validate_path(path)?;
        Ok(Self {
            namespace: namespace.to_string(),
            path: path.to_string(),
        })
    }

    /// Convenience constructor that sets the namespace to `"minecraft"`.
    pub fn minecraft(path: impl AsRef<str>) -> Result<Self> {
        Self::new("minecraft", path)
    }

    /// Get the namespace part of this resource location.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Get the path part of this resource location.
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl fmt::Display for ResourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

impl FromStr for ResourceLocation {
    type Err = SandError;

    /// Parse a string of the form `namespace:path`.
    fn from_str(s: &str) -> Result<Self> {
        let (namespace, path) = s
            .split_once(':')
            .ok_or_else(|| SandError::InvalidNamespace(s.to_string()))?;
        Self::new(namespace, path)
    }
}

impl Serialize for ResourceLocation {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ResourceLocation {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Type alias for [`ResourceLocation`], matching Minecraft's "Identifier" terminology.
pub type Identifier = ResourceLocation;

/// The user's declared pack namespace (e.g. `"my_pack"`).
///
/// Follows the same character rules as a resource location namespace:
/// `[a-z0-9_.-]+`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PackNamespace(String);

impl PackNamespace {
    /// Create a `PackNamespace`, validating that it matches `[a-z0-9_.-]+`.
    pub fn new(namespace: impl AsRef<str>) -> Result<Self> {
        let ns = namespace.as_ref();
        validate_namespace(ns)?;
        Ok(Self(ns.to_string()))
    }

    /// Get this namespace as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<PackNamespace> for String {
    fn from(p: PackNamespace) -> Self {
        p.0
    }
}

impl TryFrom<String> for PackNamespace {
    type Error = SandError;

    fn try_from(s: String) -> Result<Self> {
        Self::new(s)
    }
}

// ── Validation ───────────────────────────────────────────────────────────────

fn validate_namespace(s: &str) -> Result<()> {
    if s.is_empty()
        || !s
            .chars()
            .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-'))
    {
        Err(SandError::InvalidNamespace(s.to_string()))
    } else {
        Ok(())
    }
}

fn validate_path(s: &str) -> Result<()> {
    if s.is_empty()
        || !s
            .chars()
            .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-' | '/'))
    {
        Err(SandError::InvalidPath(s.to_string()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_resource_location() {
        let loc = ResourceLocation::new("minecraft", "oak_log").unwrap();
        assert_eq!(loc.namespace(), "minecraft");
        assert_eq!(loc.path(), "oak_log");
        assert_eq!(loc.to_string(), "minecraft:oak_log");
    }

    #[test]
    fn minecraft_convenience() {
        let loc = ResourceLocation::minecraft("stone").unwrap();
        assert_eq!(loc.to_string(), "minecraft:stone");
    }

    #[test]
    fn invalid_namespace_uppercase() {
        assert!(ResourceLocation::new("Minecraft", "stone").is_err());
    }

    #[test]
    fn invalid_namespace_space() {
        assert!(ResourceLocation::new("my pack", "stone").is_err());
    }

    #[test]
    fn invalid_path_uppercase() {
        assert!(ResourceLocation::new("minecraft", "Oak_Log").is_err());
    }

    #[test]
    fn valid_path_with_slash() {
        let loc = ResourceLocation::new("minecraft", "textures/block/stone").unwrap();
        assert_eq!(loc.path(), "textures/block/stone");
    }

    #[test]
    fn invalid_path_space() {
        assert!(ResourceLocation::new("minecraft", "my block").is_err());
    }

    #[test]
    fn empty_namespace_invalid() {
        assert!(ResourceLocation::new("", "stone").is_err());
    }

    #[test]
    fn empty_path_invalid() {
        assert!(ResourceLocation::new("minecraft", "").is_err());
    }

    #[test]
    fn from_str_valid() {
        let loc: ResourceLocation = "minecraft:stone".parse().unwrap();
        assert_eq!(loc.namespace(), "minecraft");
        assert_eq!(loc.path(), "stone");
    }

    #[test]
    fn from_str_no_colon_errors() {
        assert!("minecraft_stone".parse::<ResourceLocation>().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let loc = ResourceLocation::minecraft("stone").unwrap();
        let json = serde_json::to_string(&loc).unwrap();
        assert_eq!(json, r#""minecraft:stone""#);
        let back: ResourceLocation = serde_json::from_str(&json).unwrap();
        assert_eq!(loc, back);
    }

    #[test]
    fn pack_namespace_valid() {
        let ns = PackNamespace::new("my_pack").unwrap();
        assert_eq!(ns.as_str(), "my_pack");
    }

    #[test]
    fn pack_namespace_invalid() {
        assert!(PackNamespace::new("My Pack!").is_err());
    }

    #[test]
    fn pack_namespace_empty_invalid() {
        assert!(PackNamespace::new("").is_err());
    }
}
