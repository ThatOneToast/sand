use serde_json::Value;

use sand_version::ComponentFeature;

use crate::error::Result as SandResult;
use crate::resource_location::ResourceLocation;

/// Content of a datapack component — structured JSON or raw text.
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentContent {
    /// Structured JSON value (advancements, loot tables, recipes, etc.).
    Json(Value),
    /// Raw text content (for `.mcfunction` files).
    Text(String),
}

/// A value that can be written as a file into a Minecraft datapack.
///
/// Implementors represent datapack elements such as advancements, recipes,
/// loot tables, predicates, and item modifiers. Each component knows its
/// resource location and can serialize itself to the format Minecraft expects.
///
/// # Fallible export contract
///
/// The [`DatapackComponent::validate`] and [`DatapackComponent::try_content`]
/// hooks provide a fallible path used by `export_components` (and `sand build`)
/// to reject invalid components **before** any pack output is written. The
/// existing [`DatapackComponent::to_json`] / [`DatapackComponent::content`]
/// infallible methods remain as backward-compatible escape hatches for direct
/// callers that accept the risk of panics on invalid state.
///
/// New component implementations should override [`DatapackComponent::validate`]
/// to enforce stable builder invariants. The default
/// [`DatapackComponent::try_content`] calls `validate` and then `content`.
///
/// # Version-aware validation
///
/// Components that require a specific Minecraft feature (e.g. dialogs, jukebox
/// songs) override [`DatapackComponent::required_features`] to declare their
/// requirements. The export layer checks these against [`sand_version::VersionCaps`] resolved
/// from the target `VersionProfile` and rejects unsupported components before
/// any pack output is written.
pub trait DatapackComponent {
    /// The resource location that identifies this component within the datapack.
    fn resource_location(&self) -> &ResourceLocation;

    /// Serialize this component to the JSON value written to disk.
    fn to_json(&self) -> Value;

    /// Get the serialized content of this component (defaults to JSON).
    fn content(&self) -> ComponentContent {
        ComponentContent::Json(self.to_json())
    }

    /// Validate stable builder invariants before serialization.
    ///
    /// The default implementation is a no-op (`Ok(())`). Override this to
    /// reject invalid component state — e.g. empty required fields, missing
    /// pattern keys, or invariants documented in the public rustdoc — so the
    /// export path can surface a structured [`crate::SandError`] instead of panicking
    /// inside `to_json` / `content`.
    ///
    /// Keep this focused on *stable builder invariants*. Version-sensitive
    /// gating is handled separately via [`required_features`](Self::required_features)
    /// and the version-aware export path.
    fn validate(&self) -> SandResult<()> {
        Ok(())
    }

    /// Fallible content extraction — the hook used by the export path.
    ///
    /// Calls [`validate`](Self::validate) and then
    /// [`content`](Self::content) by default. Components whose `to_json` /
    /// `content` can panic on invalid state should override this to perform
    /// fallible serialization instead, ensuring the export path never panics.
    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    /// Version-profile-aware fallible content extraction.
    ///
    /// Some components (notably [`crate::advancement::Advancement`]) render
    /// different JSON depending on the target Minecraft version — e.g. which
    /// trigger-condition schema family a criterion uses. The default
    /// implementation ignores `caps` and delegates to
    /// [`try_content`](Self::try_content), which is correct for every
    /// component that has no version-dependent output shape.
    ///
    /// `caps` is `None` on the unprofiled compatibility export path
    /// ([`crate::component`]'s `try_export_components`-style callers); callers
    /// that resolved a target [`sand_version::VersionCaps`] should pass
    /// `Some(caps)` so profile-aware components can select the correct schema.
    fn try_content_for(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> SandResult<ComponentContent> {
        let _ = caps;
        self.try_content()
    }

    /// Declare the Minecraft feature requirements for this component.
    ///
    /// The default is an empty slice (no version-gated features required).
    /// Override this to declare requirements such as
    /// `[ComponentFeature::Dialogs]`. The export layer checks these against
    /// [`sand_version::VersionCaps`] and rejects unsupported components with a
    /// [`crate::SandError::VersionGating`] diagnostic before any pack output is written.
    ///
    /// Custom/modded components that don't map to a known feature should
    /// return `&[]` — they remain possible; version gating applies only to
    /// components that explicitly declare a known requirement.
    fn required_features(&self) -> &'static [ComponentFeature] {
        &[]
    }

    /// Project-root-relative source path to copy verbatim for binary assets.
    ///
    /// Most datapack components are generated text and should use
    /// [`DatapackComponent::content`]. Binary assets such as structure
    /// templates override this hook so the build pipeline can copy the source
    /// file without treating it as JSON or text.
    fn copy_source_path(&self) -> Option<&str> {
        None
    }

    /// The subdirectory under `data/<namespace>/` where this component lives.
    ///
    /// Examples: `"advancement"`, `"loot_table"`, `"recipe"`, `"predicate"`,
    /// `"item_modifier"`, `"tags"`.
    fn component_dir(&self) -> &'static str;

    /// The file extension for this component (without the dot). Defaults to `"json"`.
    fn file_extension(&self) -> &'static str {
        "json"
    }
}

/// A type that can produce a collection of [`DatapackComponent`]s.
pub trait IntoDatapack {
    fn into_datapack(self) -> Vec<Box<dyn DatapackComponent>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SandError;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "dummy").unwrap()
    }

    /// A minimal component that only implements the required methods.
    struct PlainComponent {
        loc: ResourceLocation,
    }

    impl DatapackComponent for PlainComponent {
        fn resource_location(&self) -> &ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::json!({"hello": "world"})
        }
        fn component_dir(&self) -> &'static str {
            "test"
        }
    }

    #[test]
    fn default_validate_is_ok() {
        let comp = PlainComponent { loc: rl() };
        assert!(comp.validate().is_ok());
    }

    #[test]
    fn default_try_content_preserves_existing_content() {
        let comp = PlainComponent { loc: rl() };
        let content = comp
            .try_content()
            .expect("default try_content should succeed");
        match content {
            ComponentContent::Json(v) => {
                assert_eq!(v, comp.to_json());
            }
            _ => panic!("expected JSON"),
        }
    }

    #[test]
    fn default_try_content_routes_through_validate() {
        struct FailingComponent {
            loc: ResourceLocation,
        }
        impl DatapackComponent for FailingComponent {
            fn resource_location(&self) -> &ResourceLocation {
                &self.loc
            }
            fn to_json(&self) -> serde_json::Value {
                panic!("to_json must not be called when validate fails")
            }
            fn validate(&self) -> crate::error::Result<()> {
                Err(SandError::ComponentValidation {
                    location: self.loc.clone(),
                    kind: "test".to_string(),
                    field: "custom".to_string(),
                    message: "always fails".to_string(),
                })
            }
            fn component_dir(&self) -> &'static str {
                "test"
            }
        }
        let comp = FailingComponent { loc: rl() };
        let result = comp.try_content();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("test:dummy"), "err: {err}");
        assert!(err.to_string().contains("test"), "err: {err}");
        assert!(err.to_string().contains("custom"), "err: {err}");
    }
}
