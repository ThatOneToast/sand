use std::path::{Component, Path};

use sand_components::registry_coverage::{REGISTRY_COVERAGE, TAG_COVERAGE};
use serde::Deserialize;

// ── PackNamespace ─────────────────────────────────────────────────────────────

/// A validated Minecraft namespace (lowercase letters, digits, `_`, `-`, `.`).
///
/// Validated at deserialization so downstream code can assume the value is safe
/// to use as a filesystem path component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackNamespace(String);

impl PackNamespace {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn is_valid(s: &str) -> bool {
        !s.is_empty()
            && s.bytes().all(|b| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-' | b'.')
            })
    }
}

impl<'de> Deserialize<'de> for PackNamespace {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if !PackNamespace::is_valid(&s) {
            return Err(serde::de::Error::custom(format!(
                "invalid namespace '{s}'; expected lowercase letters, digits, `_`, `-`, or `.`"
            )));
        }
        Ok(PackNamespace(s))
    }
}

impl std::fmt::Display for PackNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── RelativePackPath ──────────────────────────────────────────────────────────

/// A relative path guaranteed not to escape the pack root.
///
/// Rejects: empty strings, absolute paths, `..` components, and null bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativePackPath(String);

impl RelativePackPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn is_valid(s: &str) -> bool {
        !s.is_empty()
            && !s.contains('\0')
            && !Path::new(s).is_absolute()
            && !Path::new(s).components().any(|c| {
                matches!(
                    c,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
    }
}

impl<'de> Deserialize<'de> for RelativePackPath {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if !RelativePackPath::is_valid(&s) {
            return Err(serde::de::Error::custom(format!(
                "unsafe or empty pack path '{s}'"
            )));
        }
        Ok(RelativePackPath(s))
    }
}

impl std::fmt::Display for RelativePackPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── PackSupportedFormats / PackOverlay ─────────────────────────────────────────

/// Datapack/resource-pack format compatibility, matching vanilla's
/// `pack.mcmeta` `pack.supported_formats` field.
///
/// Accepts either a single format number in `sand.toml`:
///
/// ```toml
/// supported_formats = 71
/// ```
///
/// or an inclusive min/max range:
///
/// ```toml
/// supported_formats = { min = 71, max = 72 }
/// ```
///
/// Ranges are validated at deserialization time: `min` and `max` must both be
/// `>= 1`, and `min` must be `<= max`. Invalid ranges fail `sand.toml`
/// parsing with a diagnostic naming the offending values, rather than
/// producing a `pack.mcmeta` Minecraft would silently reject.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackSupportedFormats {
    /// A single supported format number, equivalent to `min == max`.
    Single(u32),
    /// An inclusive `[min, max]` range of supported format numbers.
    Range { min: u32, max: u32 },
}

impl PackSupportedFormats {
    /// Render the JSON value vanilla expects for `pack.supported_formats` (or
    /// an overlay entry's `formats`): a bare integer for a single format, or
    /// `{"min_inclusive": .., "max_inclusive": ..}` for a range.
    pub fn to_json(self) -> serde_json::Value {
        match self {
            PackSupportedFormats::Single(n) => serde_json::json!(n),
            PackSupportedFormats::Range { min, max } => serde_json::json!({
                "min_inclusive": min,
                "max_inclusive": max,
            }),
        }
    }

    fn validate_range(min: u32, max: u32) -> Result<(), String> {
        if min == 0 || max == 0 {
            return Err(format!(
                "supported_formats range must use format numbers >= 1 (got min={min}, max={max})"
            ));
        }
        if min > max {
            return Err(format!(
                "supported_formats range min ({min}) must be <= max ({max})"
            ));
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for PackSupportedFormats {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Single(u32),
            Range { min: u32, max: u32 },
        }
        match Raw::deserialize(d)? {
            Raw::Single(n) => {
                if n == 0 {
                    return Err(serde::de::Error::custom(
                        "supported_formats value must be >= 1",
                    ));
                }
                Ok(PackSupportedFormats::Single(n))
            }
            Raw::Range { min, max } => {
                PackSupportedFormats::validate_range(min, max).map_err(serde::de::Error::custom)?;
                Ok(PackSupportedFormats::Range { min, max })
            }
        }
    }
}

/// A `pack.mcmeta` overlay entry: an alternate content directory active for a
/// specific format range, matching vanilla's `overlays.entries`.
///
/// ```toml
/// [[pack.overlays]]
/// directory = "overlays/26_2"
/// formats = { min = 72, max = 72 }
/// ```
///
/// `directory` is a [`RelativePackPath`], so absolute paths and `..`
/// traversal are rejected at `sand.toml` parse time.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PackOverlay {
    pub directory: RelativePackPath,
    pub formats: PackSupportedFormats,
}

impl PackOverlay {
    /// Render as one entry of vanilla's `pack.mcmeta` `overlays.entries` array.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "formats": self.formats.to_json(),
            "directory": self.directory.as_str(),
        })
    }
}

// ── ComponentDirectory ────────────────────────────────────────────────────────

/// A validated datapack component directory (must be an allowed Minecraft
/// datapack subdirectory).
///
/// Validated at deserialization so unknown or dangerous directories are
/// rejected before any filesystem access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDirectory(String);

impl ComponentDirectory {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ComponentDirectory {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if !supported_component_dir(&s) {
            return Err(serde::de::Error::custom(format!(
                "unsupported component directory '{s}'"
            )));
        }
        Ok(ComponentDirectory(s))
    }
}

impl std::fmt::Display for ComponentDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn supported_component_dir(dir: &str) -> bool {
    REGISTRY_COVERAGE.iter().any(|entry| {
        entry.datapack_dir == dir || entry.tag_dir.is_some_and(|tag_dir| tag_dir == dir)
    }) || TAG_COVERAGE.iter().any(|entry| entry.datapack_dir == dir)
        || matches!(dir, "tags" | "structure")
}

// ── Typed extension for datapack components ───────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputExt {
    Json,
    Mcfunction,
    Nbt,
}

impl OutputExt {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputExt::Json => "json",
            OutputExt::Mcfunction => "mcfunction",
            OutputExt::Nbt => "nbt",
        }
    }
}

impl<'de> Deserialize<'de> for OutputExt {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        match s.as_str() {
            "json" => Ok(OutputExt::Json),
            "mcfunction" => Ok(OutputExt::Mcfunction),
            "nbt" => Ok(OutputExt::Nbt),
            other => Err(serde::de::Error::custom(format!(
                "unsupported component extension '{other}'; expected 'json', 'mcfunction', or 'nbt'"
            ))),
        }
    }
}

// ── Content type for datapack components ─────────────────────────────────────

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ComponentContentType {
    #[default]
    Text,
    Copy,
}

impl<'de> Deserialize<'de> for ComponentContentType {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        match s.as_str() {
            "text" => Ok(ComponentContentType::Text),
            "copy" => Ok(ComponentContentType::Copy),
            other => Err(serde::de::Error::custom(format!(
                "unknown datapack component content_type '{other}'; expected 'text' or 'copy'"
            ))),
        }
    }
}

// ── Datapack record (from sand_export) ────────────────────────────────────────

#[derive(Deserialize)]
pub struct ComponentRecord {
    pub namespace: PackNamespace,
    pub dir: ComponentDirectory,
    pub path: RelativePackPath,
    pub ext: OutputExt,
    #[serde(default)]
    pub content_type: ComponentContentType,
    pub content: String,
}

// ── Content type for resource pack assets ─────────────────────────────────────

/// How the `content` field of a [`ResourcePackRecord`] should be interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// Write `content` as UTF-8 text (JSON).
    Json,
    /// Copy the file at the project-root-relative path in `content`.
    Copy,
    /// Decode `content` as base64 and write raw bytes.
    Bytes,
}

impl<'de> Deserialize<'de> for ContentType {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        match s.as_str() {
            "json" => Ok(ContentType::Json),
            "copy" => Ok(ContentType::Copy),
            "bytes" => Ok(ContentType::Bytes),
            other => Err(serde::de::Error::custom(format!(
                "unknown resource-pack content_type '{other}'; expected 'json', 'copy', or 'bytes'"
            ))),
        }
    }
}

// ── Resource pack record (from sand_resource_export) ─────────────────────────

#[derive(Deserialize)]
pub struct ResourcePackRecord {
    /// Full path from the pack root, e.g. `"assets/ns/font/hud.json"`.
    pub path: RelativePackPath,
    /// How to interpret the `content` field.
    pub content_type: ContentType,
    /// JSON string, project-root-relative source path, or base64-encoded bytes.
    pub content: String,
}
