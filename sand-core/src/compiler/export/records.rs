//! Record boundary of the export pipeline: converts a single
//! [`DatapackComponent`] into a serializable [`ComponentRecord`], validating
//! exactly once and applying version-gating before any content is accepted.
#![allow(clippy::result_large_err)]

use serde::Serialize;

use sand_components::component::{ComponentContent, DatapackComponent};
use sand_components::error::SandError as ComponentExportError;

use super::ExportCtx;

// ── sand-core-specific types ──────────────────────────────────────────────────

/// A serializable record of a datapack component for output during the build
/// process.  Consumed by `sand-build` / the generated `sand_export` binary.
#[derive(Serialize, Debug)]
pub struct ComponentRecord {
    /// The namespace (e.g. `"my_pack"`).
    pub namespace: String,
    /// The component type directory (e.g. `"function"`, `"advancement"`).
    pub dir: String,
    /// The resource location path (e.g. `"my_tick"`, `"utils/helper"`).
    pub path: String,
    /// The file extension without the dot (e.g. `"mcfunction"`, `"json"`).
    pub ext: String,
    /// `"text"` writes `content` directly; `"copy"` copies the source path in `content`.
    pub content_type: String,
    /// The serialized content of the component.
    pub content: String,
}

/// Error returned by [`try_export_components`](super::try_export_components) when a registered component fails
/// validation or serialization.
pub type ExportResult<T> = std::result::Result<T, ComponentExportError>;

/// Convert a single [`DatapackComponent`] into a [`ComponentRecord`],
/// validating exactly once before any content is accepted, and checking
/// version-gated features against the export context.
pub(crate) fn component_to_record(
    comp: &dyn DatapackComponent,
    ctx: Option<&ExportCtx>,
) -> ExportResult<ComponentRecord> {
    let rl = comp.resource_location().clone();
    let kind = comp.component_dir().to_string();

    // Version-gate check: reject if a required feature is not supported.
    if let Some(ctx) = ctx {
        for feature in comp.required_features() {
            if !ctx.caps.supports(*feature) {
                return Err(sand_components::error::version_gating_error(
                    &rl.to_string(),
                    &kind,
                    *feature,
                    ctx.requested_version,
                    ctx.is_fallback,
                ));
            }
        }
    }

    let (content_type, content) = if let Some(path) = comp.copy_source_path() {
        comp.validate().map_err(|e| enrich_error(e, &rl, &kind))?;
        ("copy", path.to_string())
    } else {
        let content = comp
            .try_content_for(ctx.map(|c| c.caps))
            .map_err(|e| enrich_error(e, &rl, &kind))?;
        match content {
            ComponentContent::Json(v) => {
                let text = serde_json::to_string_pretty(&v).map_err(|serde_err| {
                    sand_components::error::SandError::ComponentValidation {
                        location: rl.clone(),
                        kind: kind.clone(),
                        field: "<serialization>".to_string(),
                        message: serde_err.to_string(),
                    }
                })?;
                ("text", text)
            }
            ComponentContent::Text(t) => ("text", t),
        }
    };

    Ok(ComponentRecord {
        namespace: rl.namespace().to_string(),
        dir: kind,
        path: rl.path().to_string(),
        ext: comp.file_extension().to_string(),
        content_type: content_type.to_string(),
        content,
    })
}

fn enrich_error(
    e: sand_components::error::SandError,
    rl: &crate::resource_location::ResourceLocation,
    kind: &str,
) -> ComponentExportError {
    match e {
        sand_components::error::SandError::Serialization(serde_err) => {
            sand_components::error::SandError::ComponentValidation {
                location: rl.clone(),
                kind: kind.to_string(),
                field: "<serialization>".to_string(),
                message: serde_err.to_string(),
            }
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        Advancement, AdvancementRewards, AdvancementTrigger, Criterion, DatapackComponent,
        ResourceLocation,
    };

    #[test]
    fn invalid_advancement_fails_at_component_record_boundary_with_owner_context() {
        let advancement = Advancement::new(ResourceLocation::new("test", "invalid").unwrap());
        let error = super::component_to_record(&advancement, None).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("test:invalid"), "{message}");
        assert!(message.contains("(advancement)"), "{message}");
        assert!(message.contains("field: criteria"), "{message}");
    }

    #[test]
    fn generated_event_advancement_json_remains_unchanged_through_fallible_export() {
        let advancement = Advancement::new(ResourceLocation::new("test", "event").unwrap())
            .criterion("event", Criterion::new(AdvancementTrigger::Tick))
            .rewards(AdvancementRewards::new().function("test:event"));

        let legacy = serde_json::to_string_pretty(&advancement.to_json()).unwrap();
        let record = super::component_to_record(&advancement, None).unwrap();
        assert_eq!(record.content, legacy);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&record.content).unwrap(),
            json!({
                "criteria": {"event": {"trigger": "minecraft:tick"}},
                "requirements": [["event"]],
                "rewards": {"function": "test:event"}
            })
        );
    }

    // ── Fallible component-to-record contract tests (#145) ─────────────────────
    //
    // All tests use `component_to_record` with local component values — no
    // global `inventory::submit!` and no process-global atomic toggles that
    // could affect other export tests running concurrently.

    use super::component_to_record;
    use sand_components::component::ComponentContent;
    use sand_components::error::SandError;

    fn test_rl(ns: &str, path: &str) -> crate::resource_location::ResourceLocation {
        crate::resource_location::ResourceLocation::new(ns, path).unwrap()
    }

    // ── Validation call counter ─────────────────────────────────────────────────

    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A component that counts how many times `validate()` is called.
    /// Shared counter lets tests assert exactly-once validation.
    struct CountingJsonComponent {
        loc: crate::resource_location::ResourceLocation,
        counter: &'static AtomicUsize,
    }
    impl super::DatapackComponent for CountingJsonComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::json!({"ok": true})
        }
        fn validate(&self) -> sand_components::error::Result<()> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn component_dir(&self) -> &'static str {
            "test_count_json"
        }
    }

    struct CountingCopyComponent {
        loc: crate::resource_location::ResourceLocation,
        source_path: String,
        counter: &'static AtomicUsize,
    }
    impl super::DatapackComponent for CountingCopyComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::Value::Null
        }
        fn copy_source_path(&self) -> Option<&str> {
            Some(&self.source_path)
        }
        fn validate(&self) -> sand_components::error::Result<()> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn component_dir(&self) -> &'static str {
            "test_count_copy"
        }
    }

    #[test]
    fn json_component_validates_exactly_once() {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        let comp = CountingJsonComponent {
            loc: test_rl("test", "count_json"),
            counter: &COUNT,
        };
        COUNT.store(0, Ordering::SeqCst);
        component_to_record(&comp, None).expect("valid JSON component should succeed");
        assert_eq!(
            COUNT.load(Ordering::SeqCst),
            1,
            "JSON/text components must validate exactly once \
             (try_content includes validation)"
        );
    }

    #[test]
    fn copy_component_validates_exactly_once() {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        let comp = CountingCopyComponent {
            loc: test_rl("test", "count_copy"),
            source_path: "structures/x.nbt".to_string(),
            counter: &COUNT,
        };
        COUNT.store(0, Ordering::SeqCst);
        component_to_record(&comp, None).expect("valid copy component should succeed");
        assert_eq!(
            COUNT.load(Ordering::SeqCst),
            1,
            "copy-backed components must validate exactly once"
        );
    }

    // ── Test fixture components ─────────────────────────────────────────────────

    struct ValidJsonComponent {
        loc: crate::resource_location::ResourceLocation,
    }
    impl super::DatapackComponent for ValidJsonComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::json!({"hello": "world"})
        }
        fn component_dir(&self) -> &'static str {
            "test_json"
        }
    }

    struct ValidTextComponent {
        loc: crate::resource_location::ResourceLocation,
    }
    impl super::DatapackComponent for ValidTextComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::Value::Null
        }
        fn content(&self) -> ComponentContent {
            ComponentContent::Text("say hello from text component".to_string())
        }
        fn component_dir(&self) -> &'static str {
            "function"
        }
        fn file_extension(&self) -> &'static str {
            "mcfunction"
        }
    }

    struct ValidCopyComponent {
        loc: crate::resource_location::ResourceLocation,
        source_path: String,
    }
    impl super::DatapackComponent for ValidCopyComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::Value::Null
        }
        fn copy_source_path(&self) -> Option<&str> {
            Some(&self.source_path)
        }
        fn component_dir(&self) -> &'static str {
            "structure"
        }
        fn file_extension(&self) -> &'static str {
            "nbt"
        }
    }

    struct InvalidJsonComponent {
        loc: crate::resource_location::ResourceLocation,
    }
    impl super::DatapackComponent for InvalidJsonComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        fn validate(&self) -> sand_components::error::Result<()> {
            Err(SandError::ComponentValidation {
                location: self.loc.clone(),
                kind: "test_invalid_json".to_string(),
                field: "test_field".to_string(),
                message: "intentional JSON validation failure".to_string(),
            })
        }
        fn component_dir(&self) -> &'static str {
            "test_invalid_json"
        }
    }

    struct InvalidCopyComponent {
        loc: crate::resource_location::ResourceLocation,
    }
    impl super::DatapackComponent for InvalidCopyComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::Value::Null
        }
        fn copy_source_path(&self) -> Option<&str> {
            Some("structures/should_not_be_accepted.nbt")
        }
        fn validate(&self) -> sand_components::error::Result<()> {
            Err(SandError::ComponentValidation {
                location: self.loc.clone(),
                kind: "test_invalid_copy".to_string(),
                field: "source_check".to_string(),
                message: "intentional copy-backed validation failure".to_string(),
            })
        }
        fn component_dir(&self) -> &'static str {
            "test_invalid_copy"
        }
    }

    // ── Tests ───────────────────────────────────────────────────────────────────

    #[test]
    fn component_to_record_valid_json_preserves_output() {
        let comp = ValidJsonComponent {
            loc: test_rl("test", "valid_json"),
        };
        let record = component_to_record(&comp, None).expect("valid JSON component should succeed");
        assert_eq!(record.namespace, "test");
        assert_eq!(record.dir, "test_json");
        assert_eq!(record.path, "valid_json");
        assert_eq!(record.ext, "json");
        assert_eq!(record.content_type, "text");
        assert!(record.content.contains("hello"));
    }

    #[test]
    fn component_to_record_text_content_exports_correctly() {
        let comp = ValidTextComponent {
            loc: test_rl("test", "valid_text"),
        };
        let record = component_to_record(&comp, None).expect("valid text component should succeed");
        assert_eq!(record.content_type, "text");
        assert_eq!(record.content, "say hello from text component");
    }

    #[test]
    fn component_to_record_valid_copy_exports_correctly() {
        let comp = ValidCopyComponent {
            loc: test_rl("test", "valid_copy"),
            source_path: "structures/castle.nbt".to_string(),
        };
        let record = component_to_record(&comp, None).expect("valid copy component should succeed");
        assert_eq!(record.content_type, "copy");
        assert_eq!(record.content, "structures/castle.nbt");
    }

    #[test]
    fn component_to_record_invalid_json_returns_err_with_context() {
        let comp = InvalidJsonComponent {
            loc: test_rl("test", "invalid_json"),
        };
        let err = component_to_record(&comp, None).expect_err("invalid JSON component must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("test:invalid_json"),
            "must include location: {msg}"
        );
        assert!(
            msg.contains("test_invalid_json"),
            "must include kind: {msg}"
        );
        assert!(msg.contains("test_field"), "must include field: {msg}");
    }

    #[test]
    fn component_to_record_invalid_copy_returns_err_with_context() {
        let comp = InvalidCopyComponent {
            loc: test_rl("test", "invalid_copy"),
        };
        let err = component_to_record(&comp, None).expect_err("invalid copy component must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("test:invalid_copy"),
            "must include location: {msg}"
        );
        assert!(
            msg.contains("test_invalid_copy"),
            "must include kind: {msg}"
        );
        assert!(
            !msg.contains("should_not_be_accepted"),
            "source path must not be accepted when validation fails: {msg}"
        );
    }

    #[test]
    fn component_to_record_serialization_failure_never_becomes_null() {
        struct FailingSerializationComponent {
            loc: crate::resource_location::ResourceLocation,
        }
        impl super::DatapackComponent for FailingSerializationComponent {
            fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
                &self.loc
            }
            fn to_json(&self) -> serde_json::Value {
                serde_json::Value::Null
            }
            fn try_content(&self) -> sand_components::error::Result<ComponentContent> {
                self.validate()?;
                Err(SandError::Serialization(
                    serde_json::from_str::<serde_json::Value>("not json").unwrap_err(),
                ))
            }
            fn component_dir(&self) -> &'static str {
                "test_ser_fail"
            }
        }

        let comp = FailingSerializationComponent {
            loc: test_rl("test", "ser_fail"),
        };
        let result = component_to_record(&comp, None);
        assert!(
            result.is_err(),
            "serialization failure must return Err, not Value::Null"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("test:ser_fail") || msg.contains("serialization"),
            "err: {msg}"
        );
    }

    #[test]
    fn component_to_record_rejects_empty_item_modifier_with_owner_context() {
        let modifier = sand_components::ItemModifier::new(test_rl("test", "empty_modifier"));
        let error = component_to_record(&modifier, None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("test:empty_modifier"));
        assert!(error.contains("item_modifier"));
        assert!(error.contains("functions"));
    }

    #[test]
    fn component_to_record_retains_nested_item_modifier_function_path() {
        let modifier = sand_components::ItemModifier::new(test_rl("test", "bad_modifier"))
            .function(sand_components::LootFunction::SetDamage {
                damage: sand_components::NumberProvider::Uniform {
                    min: 0.0,
                    max: f64::INFINITY,
                },
                add: false,
            });
        let error = component_to_record(&modifier, None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("test:bad_modifier"));
        assert!(error.contains("functions[0].damage.max"));
        assert!(error.contains("finite"));
    }

    #[test]
    fn component_to_record_preserves_item_modifier_root_shapes() {
        let single = sand_components::ItemModifier::new(test_rl("test", "single_modifier"))
            .function(sand_components::LootFunction::ExplosionDecay);
        let single_record = component_to_record(&single, None).unwrap();
        assert_eq!(single_record.namespace, "test");
        assert_eq!(single_record.dir, "item_modifier");
        assert_eq!(single_record.path, "single_modifier");
        assert_eq!(single_record.ext, "json");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&single_record.content).unwrap(),
            serde_json::json!({"function": "minecraft:explosion_decay"})
        );

        let multiple = sand_components::ItemModifier::new(test_rl("test", "multi_modifier"))
            .function(sand_components::LootFunction::ExplosionDecay)
            .function(sand_components::LootFunction::FurnaceSmelt);
        let multiple_record = component_to_record(&multiple, None).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&multiple_record.content).unwrap(),
            serde_json::json!([
                {"function": "minecraft:explosion_decay"},
                {"function": "minecraft:furnace_smelt"}
            ])
        );
    }

    // ── Version-aware gating tests (#147) ──────────────────────────────────────

    use super::ExportCtx;
    use sand_version::VersionCaps;

    /// A component that requires dialogs.
    struct DialogComponent {
        loc: crate::resource_location::ResourceLocation,
    }
    impl super::DatapackComponent for DialogComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        fn component_dir(&self) -> &'static str {
            "dialog"
        }
        fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
            &[sand_version::ComponentFeature::Dialogs]
        }
    }

    #[test]
    fn dialog_component_succeeds_when_dialogs_supported() {
        let comp = DialogComponent {
            loc: test_rl("test", "dialog_ok"),
        };
        let caps = VersionCaps::all_enabled();
        let ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.21.6",
            is_fallback: false,
        };
        let record = component_to_record(&comp, Some(&ctx))
            .expect("dialog should succeed when dialogs feature is supported");
        assert_eq!(record.dir, "dialog");
    }

    #[test]
    fn dialog_component_fails_when_dialogs_not_supported() {
        let comp = DialogComponent {
            loc: test_rl("test", "dialog_bad"),
        };
        let caps = VersionCaps::all_disabled();
        let ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.19.4",
            is_fallback: false,
        };
        let err = component_to_record(&comp, Some(&ctx))
            .expect_err("dialog should fail when dialogs feature is not supported");
        let msg = err.to_string();
        assert!(msg.contains("dialog"), "must include kind: {msg}");
        assert!(msg.contains("dialogs"), "must include feature name: {msg}");
        assert!(
            msg.contains("1.19.4"),
            "must include requested version: {msg}"
        );
    }

    #[test]
    fn version_gating_error_includes_fallback_note() {
        let comp = DialogComponent {
            loc: test_rl("test", "dialog_fallback"),
        };
        let caps = VersionCaps::all_disabled();
        let ctx = ExportCtx {
            caps: &caps,
            requested_version: "999.0",
            is_fallback: true,
        };
        let err =
            component_to_record(&comp, Some(&ctx)).expect_err("should fail for fallback profile");
        let msg = err.to_string();
        assert!(msg.contains("fallback"), "must mention fallback: {msg}");
        assert!(
            msg.contains("999.0"),
            "must include requested version: {msg}"
        );
    }

    #[test]
    fn unprofiled_export_does_not_gate_components() {
        let comp = DialogComponent {
            loc: test_rl("test", "dialog_unprofiled"),
        };
        // No ctx → no version gating
        let record = component_to_record(&comp, None).expect("unprofiled export should not gate");
        assert_eq!(record.dir, "dialog");
    }

    #[test]
    fn resolve_export_caps_latest_enables_all_features() {
        let resolved = crate::version::resolve_export_caps("latest").unwrap();
        assert!(!resolved.is_fallback, "latest should be a known profile");
        for feature in sand_version::ComponentFeature::ALL {
            assert!(
                resolved.caps.supports(*feature),
                "latest should support {:?}",
                feature
            );
        }
    }

    #[test]
    fn resolve_export_caps_unknown_version_disables_all() {
        let resolved = crate::version::resolve_export_caps("999.0").unwrap();
        assert!(resolved.is_fallback, "unknown version should be fallback");
        for feature in sand_version::ComponentFeature::ALL {
            assert!(
                !resolved.caps.supports(*feature),
                "unknown version should not support {:?}",
                feature
            );
        }
    }

    #[test]
    fn resolve_export_caps_known_version_gates_correctly() {
        // 1.19.4 supports damage_types and trim_assets but not dialogs or jukebox_songs.
        let resolved = crate::version::resolve_export_caps("1.19.4").unwrap();
        assert!(!resolved.is_fallback, "1.19.4 should be a known profile");
        assert!(
            resolved
                .caps
                .supports(sand_version::ComponentFeature::DamageTypes)
        );
        assert!(
            resolved
                .caps
                .supports(sand_version::ComponentFeature::TrimAssets)
        );
        assert!(
            !resolved
                .caps
                .supports(sand_version::ComponentFeature::Dialogs)
        );
        assert!(
            !resolved
                .caps
                .supports(sand_version::ComponentFeature::JukeboxSongs)
        );
    }

    #[test]
    fn resolve_export_caps_rejects_malformed_version() {
        let err = crate::version::resolve_export_caps("not-a-version")
            .expect_err("malformed export version must not silently use a fallback");
        assert!(err.to_string().contains("not-a-version"));
    }

    // ── Component-bearing recipe result version gating (#226) ─────────────────

    fn elevator_recipe(
        loc: crate::resource_location::ResourceLocation,
    ) -> sand_components::recipe::ShapedRecipe {
        let elevator = sand_components::CustomItem::new("minecraft:white_wool")
            .custom_data("elevator_block_item")
            .component(sand_components::ItemComponent::EnchantmentGlintOverride(
                true,
            ));
        let result = sand_components::recipe::RecipeResult::custom_item(&elevator)
            .expect("component-bearing custom item should convert to a recipe result");
        sand_components::recipe::ShapedRecipe::new(loc)
            .pattern(["X"])
            .key(
                'X',
                sand_components::recipe::Ingredient::item("minecraft:white_wool"),
            )
            .result(result)
    }

    #[test]
    fn component_bearing_recipe_result_rejected_when_item_components_unsupported() {
        let recipe = elevator_recipe(test_rl("test", "elevator_gated"));
        let caps = VersionCaps::all_disabled();
        let ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.19.4",
            is_fallback: false,
        };
        let err = component_to_record(&recipe, Some(&ctx))
            .expect_err("component-bearing recipe result must be gated on item_components");
        let msg = err.to_string();
        assert!(msg.contains("item_components"), "err: {msg}");
        assert!(msg.contains("1.19.4"), "err: {msg}");
    }

    #[test]
    fn component_bearing_recipe_result_accepted_when_item_components_supported() {
        let recipe = elevator_recipe(test_rl("test", "elevator_ok"));
        let caps = VersionCaps::all_enabled();
        let ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.21.4",
            is_fallback: false,
        };
        let record = component_to_record(&recipe, Some(&ctx))
            .expect("component-bearing recipe result should succeed when supported");
        assert_eq!(record.dir, "recipe");
        assert!(record.content.contains("elevator_block_item"));
    }

    #[test]
    fn component_free_recipe_result_never_gated() {
        let recipe = sand_components::recipe::ShapedRecipe::new(test_rl("test", "plain_recipe"))
            .pattern(["X"])
            .key(
                'X',
                sand_components::recipe::Ingredient::item("minecraft:stick"),
            )
            .result(sand_components::recipe::RecipeResult::raw(
                "minecraft:diamond",
                1,
            ));
        let caps = VersionCaps::all_disabled();
        let ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.18.1",
            is_fallback: false,
        };
        component_to_record(&recipe, Some(&ctx))
            .expect("component-free recipe results must never be version-gated");
    }

    #[test]
    fn advancement_export_uses_the_resolved_target_profile() {
        let advancement = sand_components::Advancement::new(test_rl("test", "profiled_trigger"))
            .criterion(
                "kill",
                sand_components::Criterion::new(
                    sand_components::AdvancementTrigger::PlayerKilledEntity {
                        entity: Some(sand_components::EntityPredicate::type_("minecraft:zombie")),
                        killing_blow: None,
                    },
                ),
            );

        let stable = crate::version::resolve_export_caps("1.21.4").unwrap();
        let stable_ctx = ExportCtx {
            caps: &stable.caps,
            requested_version: "1.21.4",
            is_fallback: stable.is_fallback,
        };
        let stable_record = component_to_record(&advancement, Some(&stable_ctx)).unwrap();
        assert!(stable_record.content.contains("\"type\""));
        assert!(!stable_record.content.contains("minecraft:entity_type"));

        let latest = crate::version::resolve_export_caps("26.2").unwrap();
        let latest_ctx = ExportCtx {
            caps: &latest.caps,
            requested_version: "26.2",
            is_fallback: latest.is_fallback,
        };
        let latest_record = component_to_record(&advancement, Some(&latest_ctx)).unwrap();
        assert!(latest_record.content.contains("minecraft:entity_type"));
        assert_ne!(stable_record.content, latest_record.content);
    }

    // ── Structured validation export integration (#138, #139, #140) ──────────

    #[test]
    fn invalid_damage_type_fails_at_record_boundary() {
        let dt = sand_components::DamageType::new(test_rl("test", "spike"));
        let err = component_to_record(&dt, None).unwrap_err().to_string();
        assert!(err.contains("test:spike"), "{err}");
        assert!(err.contains("damage_type"), "{err}");
        assert!(err.contains("message_id"), "{err}");
    }

    #[test]
    fn invalid_enchantment_fails_at_record_boundary() {
        let ench = sand_components::Enchantment::new(test_rl("test", "swift_step"));
        let err = component_to_record(&ench, None).unwrap_err().to_string();
        assert!(err.contains("test:swift_step"), "{err}");
        assert!(err.contains("enchantment"), "{err}");
        assert!(err.contains("supported_items"), "{err}");
    }

    #[test]
    fn invalid_instrument_fails_at_record_boundary() {
        let inst = sand_components::Instrument::new(test_rl("test", "horn"));
        let err = component_to_record(&inst, None).unwrap_err().to_string();
        assert!(err.contains("test:horn"), "{err}");
        assert!(err.contains("instrument"), "{err}");
        assert!(err.contains("sound_event"), "{err}");
    }

    #[test]
    fn invalid_jukebox_song_fails_at_record_boundary() {
        let song = sand_components::JukeboxSong::new(test_rl("test", "theme"))
            .sound_event("minecraft:music.disc.13")
            .song_length(10.0)
            .comparator_output(0);
        let err = component_to_record(&song, None).unwrap_err().to_string();
        assert!(err.contains("test:theme"), "{err}");
        assert!(err.contains("jukebox_song"), "{err}");
        assert!(err.contains("comparator_output"), "{err}");
    }

    #[test]
    fn valid_damage_type_exports_deterministically() {
        let dt = sand_components::DamageType::new(test_rl("test", "spike"))
            .message_id("spike")
            .exhaustion(0.1);
        let a = component_to_record(&dt, None).unwrap();
        let b = component_to_record(&dt, None).unwrap();
        assert_eq!(a.content, b.content);
        assert_eq!(a.dir, "damage_type");
    }

    #[test]
    fn valid_enchantment_exports_deterministically() {
        let ench = sand_components::Enchantment::new(test_rl("test", "swift_step"))
            .description(serde_json::json!("Swift Step"))
            .supported_items("#minecraft:enchantable/foot_armor")
            .slot_typed(sand_components::EnchantmentSlot::Feet);
        let a = component_to_record(&ench, None).unwrap();
        let b = component_to_record(&ench, None).unwrap();
        assert_eq!(a.content, b.content);
        assert_eq!(a.dir, "enchantment");
    }

    #[test]
    fn valid_instrument_exports_deterministically() {
        let inst = sand_components::Instrument::new(test_rl("test", "horn"))
            .sound_event("minecraft:item.goat_horn.sound.0");
        let a = component_to_record(&inst, None).unwrap();
        let b = component_to_record(&inst, None).unwrap();
        assert_eq!(a.content, b.content);
        assert_eq!(a.dir, "instrument");
    }

    #[test]
    fn valid_jukebox_song_exports_deterministically() {
        let song = sand_components::JukeboxSong::new(test_rl("test", "theme"))
            .sound_event("minecraft:music.disc.13")
            .song_length(178.0)
            .comparator_output(5);
        let a = component_to_record(&song, None).unwrap();
        let b = component_to_record(&song, None).unwrap();
        assert_eq!(a.content, b.content);
        assert_eq!(a.dir, "jukebox_song");
    }
}
