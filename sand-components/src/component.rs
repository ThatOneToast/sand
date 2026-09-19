use serde_json::Value;

use sand_macros::api;
use sand_version::{ComponentFeature, VersionCaps};

use crate::error::Result as SandResult;
use crate::resource_location::ResourceLocation;

/// Content of a datapack component — structured JSON or raw text.
#[derive(Debug, Clone, PartialEq)]
#[api(
    registry = sand_api_contract,
    path = "sand::registration::ComponentContent",
    module = "sand::registration",
    summary = "Structured JSON or text supplied by a datapack resource.",
    context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
    minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
    use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
    avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
    example = "use sand::registration::DatapackComponent;",
    variants(Json = "Structured JSON for a resource file.", Text = "Rendered text for a resource file."),
    variant_fields(Json = ["The JSON document serialized into the component's Minecraft resource file."], Text = ["The complete UTF-8 text written to the component's Minecraft resource file."]),
)]
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
#[api(
    registry = sand_api_contract,
    path = "sand::registration::DatapackComponent",
    module = "sand::registration",
    summary = "The validated authoring contract for a single datapack resource.",
    context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
    minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
    use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
    avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
    example = "use sand::registration::DatapackComponent;",
)]
pub trait DatapackComponent {
    /// The resource location that identifies this component within the datapack.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackComponent::resource_location",
        kind = "trait_method",
        module = "sand::registration",
        summary = "Returns the namespace and path that identify the resource.",
        context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
        minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
        use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
        avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
        example = "use sand::registration::DatapackComponent;",
        returns = "The resource location defining the namespace and output path.",
    )]
    fn resource_location(&self) -> &ResourceLocation;

    /// Serialize this component to the JSON value written to disk.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackComponent::to_json",
        kind = "trait_method",
        module = "sand::registration",
        summary = "Serializes the resource to structured JSON.",
        context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
        minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
        use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
        avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
        example = "use sand::registration::DatapackComponent;",
        returns = "The component JSON value.",
    )]
    fn to_json(&self) -> Value;

    /// Get the serialized content of this component (defaults to JSON).
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackComponent::content",
        kind = "trait_method",
        module = "sand::registration",
        summary = "Returns the resource content, defaulting to JSON.",
        context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
        minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
        use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
        avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
        example = "use sand::registration::DatapackComponent;",
        returns = "The structured JSON or rendered text content.",
    )]
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
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackComponent::validate",
        kind = "trait_method",
        module = "sand::registration",
        summary = "Checks stable resource invariants before export.",
        context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
        minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
        use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
        avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
        example = "use sand::registration::DatapackComponent;",
        returns = "Success or a structured resource validation error.",
    )]
    fn validate(&self) -> SandResult<()> {
        Ok(())
    }

    /// Fallible content extraction — the hook used by the export path.
    ///
    /// Calls [`validate`](Self::validate) and then
    /// [`content`](Self::content) by default. Components whose `to_json` /
    /// `content` can panic on invalid state should override this to perform
    /// fallible serialization instead, ensuring the export path never panics.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackComponent::try_content",
        kind = "trait_method",
        module = "sand::registration",
        summary = "Validates and extracts the resource content.",
        context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
        minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
        use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
        avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
        example = "use sand::registration::DatapackComponent;",
        returns = "Validated content or a structured resource validation error.",
    )]
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
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackComponent::try_content_for",
        kind = "trait_method",
        module = "sand::registration",
        summary = "Validates and renders content for resolved Minecraft capabilities.",
        context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
        minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
        use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
        avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
        example = "use sand::registration::DatapackComponent;",
        returns = "Content for the requested capabilities, or a validation error.",
        params(caps = "The resolved Minecraft capabilities, or None for unprofiled export."),
    )]
    fn try_content_for(&self, caps: Option<&VersionCaps>) -> SandResult<ComponentContent> {
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
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackComponent::required_features",
        kind = "trait_method",
        module = "sand::registration",
        summary = "Declares Minecraft capabilities required by this resource.",
        context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
        minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
        use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
        avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
        example = "use sand::registration::DatapackComponent;",
        returns = "The required feature list; empty by default.",
    )]
    fn required_features(&self) -> &'static [ComponentFeature] {
        &[]
    }

    /// Additional generated components this component hoists alongside its
    /// own resource — e.g. [`crate::villager_trade::TradeSet`] hoisting its
    /// inline [`crate::villager_trade::TradeSet::entry`] closures into
    /// separate generated `villager_trade` resources, or
    /// [`crate::villager_trade::VillagerTradePoolPatch`] hoisting its
    /// `.append(...)` entries.
    ///
    /// The default implementation returns an empty list — most components
    /// map to exactly one output file. The export pipeline validates and
    /// writes every nested component the same way it does top-level
    /// components (including version gating and generated-path collision
    /// detection), after this component's own record has been produced
    /// successfully.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackComponent::nested_components",
        kind = "trait_method",
        module = "sand::registration",
        summary = "Returns resources hoisted from this resource definition.",
        context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
        minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
        use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
        avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
        example = "use sand::registration::DatapackComponent;",
        returns = "Nested resources validated with the parent registration.",
    )]
    fn nested_components(&self) -> Vec<Box<dyn DatapackComponent>> {
        Vec::new()
    }

    /// Project-root-relative source path to copy verbatim for binary assets.
    ///
    /// Most datapack components are generated text and should use
    /// [`DatapackComponent::content`]. Binary assets such as structure
    /// templates override this hook so the build pipeline can copy the source
    /// file without treating it as JSON or text.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackComponent::copy_source_path",
        kind = "trait_method",
        module = "sand::registration",
        summary = "Provides a binary resource source path when copying is required.",
        context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
        minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
        use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
        avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
        example = "use sand::registration::DatapackComponent;",
        returns = "A project-relative source path, or None for generated content.",
    )]
    fn copy_source_path(&self) -> Option<&str> {
        None
    }

    /// The subdirectory under `data/<namespace>/` where this component lives.
    ///
    /// Examples: `"advancement"`, `"loot_table"`, `"recipe"`, `"predicate"`,
    /// `"item_modifier"`, `"tags"`.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackComponent::component_dir",
        kind = "trait_method",
        module = "sand::registration",
        summary = "Selects the directory beneath data/namespace for this resource kind.",
        context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
        minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
        use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
        avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
        example = "use sand::registration::DatapackComponent;",
        returns = "The resource directory beneath data/namespace.",
    )]
    fn component_dir(&self) -> &'static str;

    /// The file extension for this component (without the dot). Defaults to `"json"`.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackComponent::file_extension",
        kind = "trait_method",
        module = "sand::registration",
        summary = "Returns the resource filename extension.",
        context = "The single-resource contract supplies the resource builders accepted by DatapackRegistration; custom implementations may override validation and content hooks.",
        minecraft = "Defines a resource's identity and validated JSON, text, or binary-copy content before export writes files.",
        use_when = ["Writing generic resource registration helpers", "Implementing a custom datapack resource"],
        avoid_when = ["Combining multiple resources or lifecycle work; implement IntoDatapack instead"],
        example = "use sand::registration::DatapackComponent;",
        returns = "The extension without a dot, defaulting to json.",
    )]
    fn file_extension(&self) -> &'static str {
        "json"
    }
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
