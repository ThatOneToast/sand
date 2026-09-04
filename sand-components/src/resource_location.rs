use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{Result, SandError};

/// Sentinel namespace for resources whose pack namespace is resolved during export.
pub(crate) const SAND_LOCAL_NS: &str = "__sand_local";

/// A validated Minecraft resource location in the form `namespace:path`.
///
/// - **namespace** must match `[a-z0-9_.-]+`
/// - **path** must match `[a-z0-9_./-]+`
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::ResourceLocation",
    aliases = ["sand::prelude::ResourceLocation"],
    module = "sand",
    summary = "Represents a validated Minecraft `namespace:path` resource identifier.",
    context = "Resource locations are Sand's typed boundary for functions, registries, datapack resources, and other namespaced references.",
    minecraft = "Construction validates the lowercase namespace and path grammar before the identifier can enter commands or generated JSON.",
    use_when = ["Naming a Minecraft or datapack resource with an explicit namespace"],
    avoid_when = ["Passing unchecked namespace:path strings through typed APIs"],
    example = "let start = sand::ResourceLocation::new(\"demo\", \"functions/start\")?;",
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    namespace: String,
    path: String,
}

impl ResourceLocation {
    /// Construct a `ResourceLocation`, returning an error if either part is invalid.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::ResourceLocation::new",
        aliases = ["sand::prelude::ResourceLocation::new"],
        module = "sand",
        kind = "method",
        summary = "Constructs a namespaced Minecraft resource identifier after validating both parts.",
        context = "ResourceLocation is the common typed boundary for functions, registries, datapack resources, and other namespace:path references.",
        minecraft = "Invalid namespace or path characters are rejected before Sand emits the identifier into commands or JSON resources.",
        use_when = ["Creating a project-owned or explicitly namespaced resource ID"],
        avoid_when = ["Passing unchecked namespace:path strings through typed APIs"],
        params(namespace = "The owning namespace without a colon.", path = "The slash-delimited resource path within that namespace."),
        returns = "A validated resource location, or an error identifying invalid syntax.",
        example = "let start = sand::ResourceLocation::new(\"demo\", \"functions/start\")?;",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::ResourceLocation::minecraft",
        aliases = ["sand::prelude::ResourceLocation::minecraft"],
        module = "sand",
        kind = "method",
        summary = "Builds a validated resource location in Minecraft's built-in namespace.",
        context = "This convenience constructor is equivalent to ResourceLocation::new(\"minecraft\", path) and keeps vanilla references typed.",
        minecraft = "The path is validated before it can enter generated JSON or commands.",
        use_when = ["Referring to a vanilla namespaced resource"],
        avoid_when = ["Referring to project-owned or third-party namespace content"],
        params(path = "The namespace-relative vanilla resource path."),
        returns = "A validated minecraft:path resource location, or a validation error.",
        example = "let stone = sand::ResourceLocation::minecraft(\"stone\")?;",
    )]
    pub fn minecraft(path: impl AsRef<str>) -> Result<Self> {
        Self::new("minecraft", path)
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::ResourceLocation::namespace",
        aliases = ["sand::prelude::ResourceLocation::namespace"],
        module = "sand",
        kind = "method",
        summary = "Returns the validated namespace portion of this resource location.",
        context = "The returned text excludes the colon and borrows from the immutable typed identifier.",
        minecraft = "Minecraft uses the namespace to select the owning pack or vanilla registry domain.",
        use_when = ["Inspecting or routing an already validated resource ID"],
        avoid_when = ["Constructing a new ID; use ResourceLocation::new instead"],
        returns = "The namespace before the resource-location colon.",
        example = "assert_eq!(sand::ResourceLocation::minecraft(\"stone\")?.namespace(), \"minecraft\");",
    )]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::ResourceLocation::path",
        aliases = ["sand::prelude::ResourceLocation::path"],
        module = "sand",
        kind = "method",
        summary = "Returns the validated path portion of this resource location.",
        context = "The returned text excludes the namespace and colon and borrows from the immutable typed identifier.",
        minecraft = "Minecraft resolves this path inside the namespace-specific registry or pack directory selected by the consuming API.",
        use_when = ["Inspecting or routing an already validated resource ID"],
        avoid_when = ["Constructing a new ID; use ResourceLocation::new instead"],
        returns = "The resource path after the namespace colon.",
        example = "assert_eq!(sand::ResourceLocation::minecraft(\"blocks/stone\")?.path(), \"blocks/stone\");",
    )]
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

impl sand_commands::IntoParticleId for ResourceLocation {
    fn into_particle_id(self) -> String {
        self.to_string()
    }
}

impl sand_commands::IntoParticleId for &ResourceLocation {
    fn into_particle_id(self) -> String {
        self.to_string()
    }
}

impl sand_commands::IntoSoundEvent for ResourceLocation {
    fn into_sound_event(self) -> String {
        self.to_string()
    }
}

impl sand_commands::IntoSoundEvent for &ResourceLocation {
    fn into_sound_event(self) -> String {
        self.to_string()
    }
}

impl sand_commands::IntoBossbarId for ResourceLocation {
    fn into_bossbar_id(self) -> sand_commands::BossbarId {
        sand_commands::BossbarId::parse(self.to_string())
            .expect("ResourceLocation is already validated")
    }
}

impl sand_commands::IntoBossbarId for &ResourceLocation {
    fn into_bossbar_id(self) -> sand_commands::BossbarId {
        sand_commands::BossbarId::parse(self.to_string())
            .expect("ResourceLocation is already validated")
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

/// The user's declared pack namespace (e.g. `"my_pack"`).
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::PackNamespace",
    module = "sand",
    summary = "Stores the validated namespace used for resources owned by a datapack.",
    context = "The user's declared pack namespace (e.g. `\"my_pack\"`). A pack namespace identifies the author-owned side of generated resource locations without conflating it with a complete namespace:path resource identifier.",
    minecraft = "Serializes as the namespace segment used by generated datapack resources.",
    use_when = ["Passing the current datapack's validated namespace to component generation"],
    avoid_when = ["Naming a complete resource; use ResourceLocation or a registry-specific ID instead"],
    example = "let namespace = sand::PackNamespace::new(\"demo\")?;",
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PackNamespace(String);

impl PackNamespace {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::PackNamespace::new",
        module = "sand",
        kind = "method",
        summary = "Validates and creates a datapack resource namespace.",
        context = "Construction rejects namespace text that Minecraft cannot use before resource generation begins.",
        minecraft = "Applies Minecraft's lowercase namespace syntax rules.",
        use_when = ["Establishing the namespace for generated datapack resources"],
        avoid_when = ["The value includes a resource path after a colon"],
        params(namespace = "The candidate namespace text to validate"),
        returns = "The validated pack namespace, or a validation error",
        example = "let namespace = sand::PackNamespace::new(\"demo\")?;",
    )]
    pub fn new(namespace: impl AsRef<str>) -> Result<Self> {
        let ns = namespace.as_ref();
        validate_namespace(ns)?;
        Ok(Self(ns.to_string()))
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::PackNamespace::as_str",
        module = "sand",
        kind = "method",
        summary = "Borrows the validated datapack namespace text.",
        context = "This accessor preserves the validation performed during construction while integrating with string-taking tooling.",
        minecraft = "Returns the namespace segment exactly as it will appear in resource locations.",
        use_when = ["Passing a validated pack namespace to an API that borrows text"],
        avoid_when = ["Constructing a full resource identifier"],
        returns = "The validated namespace as borrowed text",
        example = "let namespace = sand::PackNamespace::new(\"demo\")?; assert_eq!(namespace.as_str(), \"demo\");",
    )]
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

#[cfg(test)]
mod tests {
    use sand_commands::{RenderCommand, Validate};

    use super::*;

    #[test]
    fn resource_locations_feed_command_media_ids_directly() {
        let id = ResourceLocation::new("example", "boss").unwrap();
        assert!(
            sand_commands::Particle::named(id.clone())
                .validate(&sand_commands::CommandProfile::unprofiled())
                .is_ok()
        );
        assert!(sand_commands::Sound::play(id.clone()).try_build().is_ok());
        assert_eq!(
            sand_commands::Bossbar::remove(id),
            "bossbar remove example:boss"
        );
    }
}
