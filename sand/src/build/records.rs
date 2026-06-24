use serde::Deserialize;

// ── Typed extension for datapack components ───────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub(super) enum OutputExt {
    Json,
    Mcfunction,
}

impl OutputExt {
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            OutputExt::Json => "json",
            OutputExt::Mcfunction => "mcfunction",
        }
    }
}

impl<'de> serde::Deserialize<'de> for OutputExt {
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
pub(super) struct ComponentRecord {
    pub(super) namespace: String,
    pub(super) dir: String,
    pub(super) path: String,
    pub(super) ext: OutputExt,
    pub(super) content: String,
}

// ── Content type for resource pack assets ─────────────────────────────────────

/// How the `content` field of a [`ResourcePackRecord`] should be interpreted.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum ContentType {
    /// Write `content` as UTF-8 text (JSON).
    Json,
    /// Copy the file at the project-root-relative path in `content`.
    Copy,
    /// Decode `content` as base64 and write raw bytes.
    Bytes,
}

impl<'de> serde::Deserialize<'de> for ContentType {
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
pub(super) struct ResourcePackRecord {
    /// Full path from the pack root, e.g. `"assets/ns/font/hud.json"`.
    pub(super) path: String,
    /// How to interpret the `content` field.
    pub(super) content_type: ContentType,
    /// JSON string, project-root-relative source path, or base64-encoded bytes.
    pub(super) content: String,
}
