use std::path::{Component, Path};

use serde::Deserialize;

// ── PackNamespace ─────────────────────────────────────────────────────────────

/// A validated Minecraft namespace (lowercase letters, digits, `_`, `-`, `.`).
///
/// Validated at deserialization so downstream code can assume the value is safe
/// to use as a filesystem path component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PackNamespace(String);

impl PackNamespace {
    pub(crate) fn as_str(&self) -> &str {
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
pub(crate) struct RelativePackPath(String);

impl RelativePackPath {
    pub(crate) fn as_str(&self) -> &str {
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

// ── ComponentDirectory ────────────────────────────────────────────────────────

/// A validated datapack component directory (must be an allowed Minecraft
/// datapack subdirectory).
///
/// Validated at deserialization so unknown or dangerous directories are
/// rejected before any filesystem access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComponentDirectory(String);

impl ComponentDirectory {
    pub(crate) fn as_str(&self) -> &str {
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
    matches!(
        dir,
        "advancement"
            | "banner_pattern"
            | "chat_type"
            | "damage_type"
            | "dialog"
            | "dimension"
            | "enchantment"
            | "function"
            | "instrument"
            | "item_modifier"
            | "jukebox_song"
            | "loot_table"
            | "painting_variant"
            | "predicate"
            | "recipe"
            | "tags"
            | "tags/function"
            | "trim_material"
            | "trim_pattern"
            | "wolf_variant"
            | "worldgen/biome"
            | "worldgen/noise_settings"
            | "worldgen/placed_feature"
    )
}

// ── Typed extension for datapack components ───────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum OutputExt {
    Json,
    Mcfunction,
}

impl OutputExt {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            OutputExt::Json => "json",
            OutputExt::Mcfunction => "mcfunction",
        }
    }
}

impl<'de> Deserialize<'de> for OutputExt {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        match s.as_str() {
            "json" => Ok(OutputExt::Json),
            "mcfunction" => Ok(OutputExt::Mcfunction),
            other => Err(serde::de::Error::custom(format!(
                "unsupported component extension '{other}'; expected 'json' or 'mcfunction'"
            ))),
        }
    }
}

// ── Datapack record (from sand_export) ────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct ComponentRecord {
    pub(crate) namespace: PackNamespace,
    pub(crate) dir: ComponentDirectory,
    pub(crate) path: RelativePackPath,
    pub(crate) ext: OutputExt,
    pub(crate) content: String,
}

// ── Content type for resource pack assets ─────────────────────────────────────

/// How the `content` field of a [`ResourcePackRecord`] should be interpreted.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ContentType {
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
pub(crate) struct ResourcePackRecord {
    /// Full path from the pack root, e.g. `"assets/ns/font/hud.json"`.
    pub(crate) path: RelativePackPath,
    /// How to interpret the `content` field.
    pub(crate) content_type: ContentType,
    /// JSON string, project-root-relative source path, or base64-encoded bytes.
    pub(crate) content: String,
}
