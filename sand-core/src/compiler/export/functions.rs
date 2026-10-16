//! Collection of resources owned by lowered gameplay actions and conditions.
//!
//! Drains anonymous/branch functions registered while user factories ran and
//! resolves the `__sand_local` namespace sentinel to the real pack namespace.

use super::records::ComponentRecord;

/// Drain resources owned by gameplay IR into the validated record stream.
///
/// Loops until the registry is empty so that resources registered by other
/// resources are captured. Generated components use the normal profile-aware
/// validation and nested-resource expansion boundary.
pub(crate) fn drain_generated_resources_into(
    records: &mut Vec<ComponentRecord>,
    namespace: &str,
    ctx: Option<&super::ExportCtx<'_>>,
) -> super::records::ExportResult<()> {
    let generated_start = records.len();
    let mut emitted = std::collections::BTreeSet::new();
    let mut emitted_components = Vec::new();
    loop {
        let drained = crate::function::drain_generated_resources();
        if drained.is_empty() {
            break;
        }
        for resource in drained {
            match resource {
                crate::function::GeneratedResource::Function(path, commands) => {
                    // A parent may replay an already-drained helper. Distinct
                    // bodies at one path still reach the collision guard.
                    if !emitted.insert((path.clone(), commands.identity())) {
                        continue;
                    }
                    records.push(ComponentRecord::function(
                        namespace,
                        &path,
                        commands
                            .lower(&sand_components::ResourceLocation::new(namespace, &path)?)?,
                    ));
                }
                crate::function::GeneratedResource::Component(component) => {
                    // Keep the typed component until this profile-aware boundary.
                    let record = super::records::component_to_record(&*component, ctx)?;
                    let owner = format!("generated resource {}:{}", record.namespace, record.path);
                    let expanded = super::records::expand_with_nested(
                        vec![super::records::RegisteredComponent {
                            owner,
                            component,
                            record,
                        }],
                        ctx,
                    )?;
                    // This expansion owner is a resource-identity diagnostic,
                    // not a semantic Rust owner for runtime attribution.
                    for (_, mut record) in expanded {
                        // Numeric helpers can lower before their namespace is
                        // resolved; slot helpers lower with the final owner.
                        // Compare their shared resources by exported identity.
                        if record.namespace == crate::function::SAND_LOCAL_NS {
                            record.namespace = namespace.to_owned();
                        }
                        record.content = resolve_local_refs(&record.content, namespace);
                        if !emitted_components.contains(&record) {
                            emitted_components.push(record.clone());
                            records.push(record);
                        }
                    }
                }
                crate::function::GeneratedResource::Requirement { owner, feature } => {
                    if let Some(ctx) = ctx
                        && !ctx.caps.supports(feature)
                    {
                        return Err(sand_components::error::version_gating_error(
                            &owner.to_string(),
                            "generated gameplay resource",
                            feature,
                            ctx.requested_version,
                            ctx.is_fallback,
                        ));
                    }
                }
            }
        }
    }
    // Cold construction and cached replay can discover owned helpers in a
    // different order. Canonicalize only this generated batch, preserving the
    // registration order of authored records (including schedule bodies).
    // Contents and tag-member order are untouched; collisions still reach the
    // exporter's normal output-identity validation.
    records[generated_start..].sort_by(|left, right| {
        (&left.namespace, &left.dir, &left.path, &left.ext).cmp(&(
            &right.namespace,
            &right.dir,
            &right.path,
            &right.ext,
        ))
    });
    Ok(())
}

/// Replace every `__sand_local:<path>` sentinel in an mcfunction content string
/// with `<namespace>:<path>`.
///
/// Handles both patterns:
/// - `function __sand_local:path` — bare function pointer calls
/// - `... only __sand_local:path` — advancement revoke/grant from EventHandle
pub(crate) fn resolve_local_refs(content: &str, namespace: &str) -> String {
    let sentinel = crate::function::SAND_LOCAL_NS;
    content.replace(&format!("{sentinel}:"), &format!("{namespace}:"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(path: &str, item: &str) -> sand_components::TypedTag<sand_components::ItemId> {
        sand_components::TypedTag::new(
            sand_components::ResourceLocation::new("test", path)
                .unwrap()
                .into(),
        )
        .entry(sand_components::ItemId::minecraft(item).unwrap())
    }

    #[test]
    fn owned_resources_sort_without_reordering_preexisting_records() {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        crate::register_dyn_fn("owned/z".into(), "say z");
        crate::register_dyn_fn("owned/a".into(), "say a");
        let mut records = vec![
            ComponentRecord::function("test", "authored/z", vec!["say first".into()]),
            ComponentRecord::function("test", "authored/a", vec!["say second".into()]),
        ];
        drain_generated_resources_into(&mut records, "test", None).unwrap();
        assert_eq!(
            records.iter().map(|r| r.path.as_str()).collect::<Vec<_>>(),
            ["authored/z", "authored/a", "owned/a", "owned/z"]
        );
        assert_eq!(records[0].content, "say first");
        assert_eq!(records[1].content, "say second");
    }

    #[test]
    fn repeated_owned_components_deduplicate_but_conflicts_reach_output_validation() {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        crate::function::register_generated_component(tag("shared", "diamond"));
        crate::function::register_generated_component(tag("shared", "diamond"));
        crate::function::register_generated_component(tag("shared", "emerald"));
        let mut records = Vec::new();
        drain_generated_resources_into(&mut records, "test", None).unwrap();
        assert_eq!(records.len(), 2);
        let error = super::super::records::validate_unique_output_identities(&records).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("duplicate datapack output identity")
        );
    }

    #[test]
    fn owned_component_expansion_retains_nested_resources() {
        struct Parent(sand_components::ResourceLocation);
        impl sand_components::DatapackComponent for Parent {
            fn resource_location(&self) -> &sand_components::ResourceLocation {
                &self.0
            }
            fn to_json(&self) -> serde_json::Value {
                serde_json::json!({"nested": "test:child"})
            }
            fn component_dir(&self) -> &'static str {
                "test_parent"
            }
            fn nested_components(&self) -> Vec<Box<dyn sand_components::DatapackComponent>> {
                vec![Box::new(tag("child", "diamond"))]
            }
        }
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        crate::function::register_generated_component(Parent("test:parent".parse().unwrap()));
        let mut records = Vec::new();
        drain_generated_resources_into(&mut records, "test", None).unwrap();
        assert_eq!(records.len(), 2);
        assert!(
            records
                .iter()
                .any(|record| record.dir == "tags/item" && record.path == "child")
        );
    }

    #[test]
    fn generated_requirements_use_the_export_profile_and_scope_cleanup() {
        let caps = sand_version::VersionCaps::all_disabled();
        let ctx = super::super::ExportCtx {
            caps: &caps,
            requested_version: "999.0",
            is_fallback: true,
        };
        {
            let _scope = crate::function::ExportFunctionRegistryScope::enter();
            crate::function::require_generated_feature(
                &"test:query".parse().unwrap(),
                sand_version::ComponentFeature::ItemComponents,
            );
            crate::function::register_generated_component(tag("pending", "diamond"));
            let error =
                drain_generated_resources_into(&mut Vec::new(), "test", Some(&ctx)).unwrap_err();
            assert!(error.to_string().contains("999.0"));
        }
        let mut records = Vec::new();
        drain_generated_resources_into(&mut records, "test", None).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn same_path_with_different_ir_still_validates_each_body() {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        crate::register_dyn_fn("shared".into(), "say valid");
        crate::register_dyn_fn(
            "shared".into(),
            crate::ir::Actions(vec![crate::ir::Cmd::Execute {
                operations: vec![],
                run: Box::new(crate::ir::Cmd::Raw("say invalid".into())),
            }]),
        );
        let error = drain_generated_resources_into(&mut Vec::new(), "test", None).unwrap_err();
        assert!(error.to_string().contains("SAND-COMMAND-EXECUTE-EMPTY"));
    }

    #[test]
    fn observation_samples_retain_canonical_resources_across_exports() {
        use crate::transition::observation::{
            Observation, ObservationHandler, ObservationSource, resolve,
        };
        use crate::{TransitionKind, condition::Condition, entity::StatCurve};
        use sand_commands::{DataTarget, Selector};
        use sand_components::{EffectId, ItemId, ItemPredicate};
        let items = ItemPredicate::id(ItemId::minecraft("diamond").unwrap())
            .item(ItemId::minecraft("emerald").unwrap());
        let observations = [
            (
                "effect",
                ObservationSource::Boolean(Condition::effect_presence(
                    Selector::self_(),
                    EffectId::Speed,
                )),
                TransitionKind::BecameTrue,
            ),
            (
                "inventory",
                ObservationSource::Numeric(StatCurve::inventory_count(Selector::self_(), items)),
                TransitionKind::ScoreChanged,
            ),
            (
                "health",
                ObservationSource::Numeric(StatCurve::native_nbt(
                    DataTarget::entity(Selector::self_()),
                    "Health".into(),
                )),
                TransitionKind::ScoreDecreased,
            ),
        ]
        .into_iter()
        .map(|(id, source, kind)| ObservationHandler {
            tracker_id: id.into(),
            description: id.into(),
            observation: Observation {
                source,
                declarations: Default::default(),
            },
            kind,
            path: format!("handlers/{id}"),
        })
        .collect::<Vec<_>>();
        let export = |namespace: &str| {
            let _scope = crate::function::ExportFunctionRegistryScope::enter();
            let plan = resolve(namespace, &observations).unwrap();
            assert_eq!(plan.tick_commands.len(), 1);
            assert_eq!(
                plan.functions
                    .iter()
                    .filter(|f| f.path.starts_with("__sand_transition/"))
                    .count(),
                3
            );
            let mut records = plan
                .functions
                .into_iter()
                .map(|function| {
                    ComponentRecord::function(namespace, &function.path, function.commands)
                })
                .collect::<Vec<_>>();
            drain_generated_resources_into(&mut records, namespace, None).unwrap();
            for record in &mut records {
                record.content = resolve_local_refs(&record.content, namespace);
            }
            super::super::records::validate_unique_output_identities(&records).unwrap();
            assert_eq!(records.iter().filter(|r| r.dir == "predicate").count(), 1);
            assert_eq!(records.iter().filter(|r| r.dir == "tags/item").count(), 1);
            assert_eq!(
                records
                    .iter()
                    .map(|r| r.content.matches("from entity @s Health").count())
                    .sum::<usize>(),
                1,
                "native health must be snapshotted exactly once before numeric conversion"
            );
            assert!(
                records
                    .iter()
                    .any(|r| r.content.contains("clear @s") && r.content.contains(" 0"))
            );
            assert!(records.iter().all(|r| r.namespace == namespace));
            assert!(records.iter().all(|r| !r.content.contains("__sand_local:")));
            records
        };
        let first = export("first");
        assert_eq!(first, export("first"));
        let second = export("second");
        assert!(second.iter().all(|r| !r.content.contains("first:")));
        assert_eq!(second, export("second"));
    }
}
