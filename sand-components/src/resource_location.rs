use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{Result, SandError};

/// A validated Minecraft resource location in the form `namespace:path`.
///
/// - **namespace** must match `[a-z0-9_.-]+`
/// - **path** must match `[a-z0-9_./-]+`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    namespace: String,
    path: String,
}

impl ResourceLocation {
    /// Construct a `ResourceLocation`, returning an error if either part is invalid.
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

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

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

/// Type alias for [`ResourceLocation`].
pub type Identifier = ResourceLocation;

/// The user's declared pack namespace (e.g. `"my_pack"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PackNamespace(String);

impl PackNamespace {
    pub fn new(namespace: impl AsRef<str>) -> Result<Self> {
        let ns = namespace.as_ref();
        validate_namespace(ns)?;
        Ok(Self(ns.to_string()))
    }

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
