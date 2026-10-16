//! Deterministic reporting over the same records that are published to a pack.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    RuntimePlan,
    estimate::{Estimator, Work},
};
use crate::compiler::export::ComponentRecord;

/// Compilation-local report. Counts describe explicit command evaluations for
/// an uninterrupted invocation of a successfully loaded function, not MSPT or
/// command-limit units. Runtime aborts and server budget exhaustion are outside
/// this static model.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[sand_macros::api(
    registry = sand_api_contract, path = "sand::advanced::compiler::RuntimeReport", module = "sand::advanced::compiler",
    summary = "Deterministic runtime estimates and compiler-owned provenance for one compiled pack.",
    context = "Serialize this report to inspect ordered tick entries, per-function explicit evaluation ranges, ownership, cadence, and limitations.",
    minecraft = "Counts explicit command evaluations under successful function loading, not MSPT or command-budget units; unknown work remains explicit.",
    use_when = ["Inspecting generated runtime work in tools and regression tests"],
    avoid_when = ["Predicting server performance without runtime measurement"],
    example = "let pack = sand::advanced::compiler::Compiler::compile(&program)?; let report = serde_json::to_value(&pack.runtime_report)?;"
)]
pub struct RuntimeReport {
    /// Version of this report's interpretation and serialized structure.
    #[serde(deserialize_with = "report_version")]
    schema_version: u32,
    /// Compiler-owned origins and cadence before publication erases them.
    provenance: RuntimePlan,
    /// Ordered direct entries in this pack's minecraft:tick tag.
    tick: Vec<LifecycleEntry>,
    /// Ordered direct entries in the final minecraft:load tag.
    load: Vec<LifecycleEntry>,
    /// Function definitions, sorted by full resource id; shared definitions are
    /// listed once even when several owners or invocations refer to them.
    functions: BTreeMap<String, FunctionCost>,
    /// Conditions that prevent interpreting this report as a complete cost bound.
    limitations: BTreeSet<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LifecycleEntry {
    reference: String,
    required: bool,
    contributors: BTreeSet<String>,
    evaluations: Work,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct FunctionCost {
    owners: BTreeSet<String>,
    evaluations: Work,
    selector_sites: Vec<SelectorSite>,
}

/// A lexical execution-context or entity-condition selector site. A site can
/// execute zero, one, or many times; this is not a runtime scan count.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SelectorSite {
    line: usize,
    operation: String,
    selector: String,
}

/// Record canonical execute selector operands while leaving quoted chat untouched.
fn selector_sites(body: &str) -> Vec<SelectorSite> {
    let mut sites = Vec::new();
    for (index, line) in body.lines().enumerate() {
        let mut command = line.trim();
        loop {
            if let Some(tail) = sand_commands::render::collected_return_command(command) {
                command = tail;
                continue;
            }
            let Some(execute) = sand_commands::render::collected_execute(command) else {
                break;
            };
            for operation in execute.operations {
                let args = &operation.arguments;
                let indexes: &[usize] = match operation.keyword {
                    "as" | "at" => &[0],
                    "positioned" | "rotated" if args.first() == Some(&"as") => &[1],
                    "facing" | "if" | "unless" if args.first() == Some(&"entity") => &[1],
                    "if" | "unless"
                        if matches!(args.first(), Some(&"data" | &"items"))
                            && args.get(1) == Some(&"entity") =>
                    {
                        &[2]
                    }
                    "if" | "unless" if args.first() == Some(&"score") => &[1, 4],
                    "store" if matches!(args.get(1), Some(&"entity" | &"score")) => &[2],
                    _ => &[],
                };
                for operand in indexes {
                    if let Some(selector) =
                        args.get(*operand).filter(|value| value.starts_with('@'))
                    {
                        sites.push(SelectorSite {
                            line: index + 1,
                            operation: operation.keyword.into(),
                            selector: (*selector).into(),
                        });
                    }
                }
            }
            command = execute.command;
        }
    }
    sites
}

/// A host must not interpret a different report schema as the current model.
fn report_version<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    let version = <u32 as serde::Deserialize>::deserialize(deserializer)?;
    if version != 1 {
        return Err(serde::de::Error::custom(
            "unsupported runtime report schema version",
        ));
    }
    Ok(version)
}

impl RuntimeReport {
    /// Recompute costs and lifecycle entries from a host's final resource bytes.
    ///
    /// Use the same pack-relative path map as [`crate::advanced::compiler::CompiledPack::resources`]
    /// after merging world-build resources or resolving copy-backed files. This
    /// preserves collected semantic provenance and analyzes newly added functions;
    /// new resources without provenance have empty owner sets. Unsupported UTF-8
    /// remains an explicit limitation. This method performs no host I/O.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::advanced::compiler::RuntimeReport::reanalyze", module = "sand::advanced::compiler",
        summary = "Recompute runtime estimates over final pack resources while retaining semantic provenance.",
        context = "Host adapters merge world-build resources and resolve copied files before publishing a pack and its report.",
        minecraft = "Analyzes final function bodies and lifecycle tags without changing any datapack bytes.",
        use_when = ["Publishing a report after host-side resource assembly"],
        avoid_when = ["Using unchanged CompiledPack resources, which already have a report"],
        params(resources = "Final pack-relative paths and bytes, as in CompiledPack.resources."),
        returns = "A report for the final resources with retained compiler provenance and explicit unknowns.",
        example = "let report = pack.runtime_report.reanalyze(&pack.resources);"
    )]
    pub fn reanalyze(&self, resources: &BTreeMap<String, Vec<u8>>) -> Self {
        let mut records = Vec::new();
        let mut unavailable = BTreeSet::new();
        for (file, bytes) in resources {
            let Some((namespace, relative)) = file
                .strip_prefix("data/")
                .and_then(|path| path.split_once('/'))
            else {
                if file.contains("/data/") && file.contains("/function/") {
                    unavailable.insert(format!("overlay runtime resource is not analyzed: {file}"));
                }
                continue;
            };
            let identity = relative
                .strip_prefix("function/")
                .and_then(|path| path.strip_suffix(".mcfunction"))
                .map(|path| ("function", path, "mcfunction"))
                .or_else(|| {
                    relative
                        .strip_prefix("tags/function/")
                        .and_then(|path| path.strip_suffix(".json"))
                        .map(|path| ("tags/function", path, "json"))
                });
            let Some((dir, path, ext)) = identity else {
                if file.contains("/data/") && file.contains("/function/") {
                    unavailable.insert(format!("overlay runtime resource is not analyzed: {file}"));
                }
                continue;
            };
            let Ok(content) = std::str::from_utf8(bytes) else {
                unavailable.insert(format!("runtime resource is not UTF-8: {file}"));
                continue;
            };
            records.push(ComponentRecord {
                namespace: namespace.into(),
                dir: dir.into(),
                path: path.into(),
                ext: ext.into(),
                content_type: "text".into(),
                content: content.into(),
            });
        }
        let mut report = Self::collect(&records, &self.provenance);
        report.limitations.extend(unavailable);
        report
    }

    /// Inspect final records without publishing metadata as a synthetic resource.
    pub(crate) fn collect(records: &[ComponentRecord], plan: &RuntimePlan) -> Self {
        let functions: BTreeMap<_, _> = records
            .iter()
            .filter(|record| {
                record.dir == "function"
                    && record.ext == "mcfunction"
                    && record.content_type == "text"
            })
            .map(|record| {
                (
                    format!("{}:{}", record.namespace, record.path),
                    record.content.clone(),
                )
            })
            .collect();
        let mut estimator = Estimator::new(&functions);
        let costs = records
            .iter()
            .filter(|record| record.dir == "function" && record.ext == "mcfunction")
            .map(|record| {
                let id = format!("{}:{}", record.namespace, record.path);
                (
                    id.clone(),
                    FunctionCost {
                        owners: plan.function_owners.get(&id).cloned().unwrap_or_default(),
                        evaluations: estimator.function(&id),
                        selector_sites: functions
                            .get(&id)
                            .map(|body| selector_sites(body))
                            .unwrap_or_default(),
                    },
                )
            })
            .collect();
        let mut tick = Vec::new();
        let mut load = Vec::new();
        let mut limitations = BTreeSet::from([
            "Counts are explicit command evaluations, not selector search cost, terminal execution multiplicity, MSPT, or Minecraft command-budget units.".into(),
            "Estimates assume successfully loaded functions and uninterrupted execution; runtime aborts, server budget exhaustion, external pack contributions, and implicit resource-triggered callbacks are not fully analyzed.".into(),
            "Selector sites cover execute contexts, entity/score conditions, and stores, not selectors in terminal commands; sites are not runtime scan counts.".into(),
            "Each function retains at most eight unknown-cause messages of at most 256 UTF-8 bytes each; truncation and omitted causes are marked explicitly.".into(),
            "An empty owner set means provenance is unavailable; resource paths are not inferred semantic owners.".into(),
        ]);
        for record in records.iter().filter(|record| {
            record.dir == "function" && record.ext == "mcfunction" && record.content_type != "text"
        }) {
            limitations.insert(format!(
                "function {}:{} awaits host resource resolution ({})",
                record.namespace, record.path, record.content_type
            ));
        }
        for (record, phase) in records.iter().filter_map(|record| {
            let phase = record.function_tag_path()?;
            (record.namespace == "minecraft" && matches!(phase, "tick" | "load"))
                .then_some((record, phase))
        }) {
            let values = if record.content_type == "text" {
                serde_json::from_str::<serde_json::Value>(&record.content)
                    .ok()
                    .and_then(|value| value.get("values").and_then(|v| v.as_array()).cloned())
            } else {
                None
            };
            let Some(values) = values else {
                limitations.insert(format!(
                    "minecraft:{} contents are unavailable or unsupported",
                    phase
                ));
                continue;
            };
            for value in values {
                let Some(reference) = value
                    .as_str()
                    .or_else(|| value.get("id").and_then(|id| id.as_str()))
                else {
                    limitations
                        .insert(format!("minecraft:{} contains an unsupported entry", phase));
                    continue;
                };
                let required = value
                    .get("required")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let entries = if phase == "tick" {
                    &mut tick
                } else {
                    &mut load
                };
                entries.push(LifecycleEntry {
                    reference: reference.into(),
                    required,
                    contributors: plan
                        .tag_contributors
                        .get(&(format!("minecraft:{}", phase), reference.into()))
                        .cloned()
                        .unwrap_or_default(),
                    evaluations: estimator.function(reference),
                });
            }
        }
        Self {
            schema_version: 1,
            provenance: plan.clone(),
            tick,
            load,
            functions: costs,
            limitations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generic-directory tags agree with typed tags and published-file analysis.
    #[test]
    fn generic_function_tags_keep_lifecycle_entries_and_contributors() {
        use crate::compiler::export::provenance;
        use crate::function::GeneratedOrigin;

        for phase in ["load", "tick"] {
            let function = ComponentRecord::function("demo", "shared", vec!["say shared".into()]);
            let mut tag = ComponentRecord {
                namespace: "minecraft".into(),
                dir: "tags".into(),
                path: format!("function/{phase}"),
                ext: "json".into(),
                content_type: "text".into(),
                content: r#"{"values":["demo:shared",{"id":"demo:shared","required":false}]}"#
                    .into(),
            };
            let mut plan = RuntimePlan::default();
            provenance::function_tag(
                &tag,
                "demo",
                [GeneratedOrigin::Logical("demo:factory".into())].into(),
                &mut plan,
            );
            plan.resolve_generated_owners();
            let report = RuntimeReport::collect(&[function.clone(), tag.clone()], &plan);
            let json = serde_json::to_value(&report).unwrap();
            assert_eq!(json[phase][0]["reference"], "demo:shared");
            assert_eq!(
                json[phase][0]["contributors"],
                serde_json::json!(["demo:factory"])
            );
            assert_eq!(json[phase][1]["required"], false);
            let published = report.reanalyze(&BTreeMap::from([
                (
                    "data/demo/function/shared.mcfunction".into(),
                    function.content.as_bytes().to_vec(),
                ),
                (
                    format!("data/minecraft/tags/function/{phase}.json"),
                    tag.content.as_bytes().to_vec(),
                ),
            ]));
            assert_eq!(json, serde_json::to_value(published).unwrap());
            tag.dir = "tags/function".into();
            tag.path = phase.into();
            assert_eq!(
                json,
                serde_json::to_value(RuntimeReport::collect(&[function, tag], &plan)).unwrap()
            );
        }
    }

    /// Reporting preserves invocation order and separates tag contributors from
    /// the owners of shared implementation resources.
    #[test]
    fn shared_functions_keep_distinct_contributors_and_body_owners() {
        let records = vec![
            ComponentRecord::function("demo", "shared", vec!["say shared".into()]),
            ComponentRecord {
                namespace: "minecraft".into(),
                dir: "tags/function".into(),
                path: "tick".into(),
                ext: "json".into(),
                content_type: "text".into(),
                content: r##"{"values":["demo:shared",{"id":"#other:tick","required":false}]}"##
                    .into(),
            },
        ];
        let mut plan = RuntimePlan::default();
        plan.own_function("demo:shared".into(), "implementation".into());
        plan.contribute_tag(
            "minecraft:tick".into(),
            "demo:shared".into(),
            "module-a".into(),
        );
        plan.contribute_tag(
            "minecraft:tick".into(),
            "demo:shared".into(),
            "module-b".into(),
        );
        let report = RuntimeReport::collect(&records, &plan);
        assert_eq!(report.tick.len(), 2);
        assert_eq!(
            report.tick[0].contributors,
            BTreeSet::from(["module-a".into(), "module-b".into()])
        );
        assert_eq!(
            report.functions["demo:shared"].owners,
            BTreeSet::from(["implementation".into()])
        );
        assert_eq!(report.tick[0].evaluations.maximum, Some(1));
        assert!(!report.tick[1].required);
        assert_eq!(report.tick[1].evaluations.maximum, None);
        assert!(!report.tick[1].evaluations.unknown.is_empty());
        assert_eq!(
            serde_json::to_value(&report).unwrap(),
            serde_json::to_value(RuntimeReport::collect(&records, &plan)).unwrap()
        );
    }
    /// Overlay directories can themselves be named data; valid root paths remain analyzable.
    #[test]
    fn overlay_locations_remain_explicit_without_hiding_root_functions() {
        let report =
            RuntimeReport::collect(&[], &RuntimePlan::default()).reanalyze(&BTreeMap::from([
                (
                    "overlay/data/demo/function/extra.mcfunction".into(),
                    b"say overlay".to_vec(),
                ),
                (
                    "data/data/demo/function/extra.mcfunction".into(),
                    b"say data overlay".to_vec(),
                ),
                (
                    "data/demo/function/data/valid.mcfunction".into(),
                    b"say root".to_vec(),
                ),
            ]));
        assert_eq!(
            report.functions["demo:data/valid"].evaluations.maximum,
            Some(1)
        );
        for path in [
            "overlay/data/demo/function/extra.mcfunction",
            "data/data/demo/function/extra.mcfunction",
        ] {
            assert!(
                report
                    .limitations
                    .iter()
                    .any(|reason| reason.contains(path))
            );
        }
    }

    /// File-backed functions stay visible and unknown until the host supplies bytes.
    #[test]
    fn copy_backed_functions_are_unknown_until_materialized() {
        let mut record = ComponentRecord::function("demo", "copied", vec![]);
        record.content_type = "copy".into();
        record.content = "fixtures/copied.mcfunction".into();
        let report = RuntimeReport::collect(&[record], &RuntimePlan::default());
        let cost = &report.functions["demo:copied"].evaluations;
        assert_eq!(cost.maximum, None);
        assert!(!cost.unknown.is_empty());
        assert!(
            report
                .limitations
                .iter()
                .any(|reason| reason.contains("demo:copied"))
        );
        let resolved = report.reanalyze(&BTreeMap::from([(
            "data/demo/function/copied.mcfunction".into(),
            b"say loaded".to_vec(),
        )]));
        assert_eq!(
            resolved.functions["demo:copied"].evaluations.maximum,
            Some(1)
        );
        assert!(
            !resolved
                .limitations
                .iter()
                .any(|reason| reason.contains("demo:copied"))
        );
    }

    /// Quoted text is not a selector; nested execution contexts retain their sites.
    #[test]
    fn selector_sites_preserve_line_and_operation_without_parsing_chat() {
        let sites = selector_sites(
            "say @e\n# comment\nexecute as @e[type=minecraft:pig] at @s run execute if entity @a run say hello\nreturn run execute as @a run say done",
        );
        assert_eq!(
            sites
                .iter()
                .map(|site| (site.line, site.operation.as_str(), site.selector.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (3, "as", "@e[type=minecraft:pig]"),
                (3, "at", "@s"),
                (3, "if", "@a"),
                (4, "as", "@a")
            ]
        );
    }
    /// Entity data/items, both score operands, and stores retain actual selectors.
    #[test]
    fn nested_entity_conditions_and_stores_keep_selector_operands() {
        let sites = selector_sites(concat!(
            "execute if data entity @s Health run say data\n",
            "execute unless items entity @p weapon.mainhand minecraft:stick run say items\n",
            "execute if score @s flag = @p flag run say score\n",
            "execute store result entity @s Health float 1 run say store\n",
            "execute store success score @a flag run say store\n",
            "execute if data storage demo:state value run say @e",
        ));
        assert_eq!(
            sites
                .iter()
                .map(|site| (site.line, site.operation.as_str(), site.selector.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (1, "if", "@s"),
                (2, "unless", "@p"),
                (3, "if", "@s"),
                (3, "if", "@p"),
                (4, "store", "@s"),
                (5, "store", "@a")
            ]
        );
    }
    /// Caller propagation must not multiply arbitrarily large diagnostic payloads.
    #[test]
    fn unresolved_diagnostics_are_bounded_without_claiming_known_costs() {
        let commands = (0..40)
            .map(|index| format!("function other:missing_{index}_{}", "x".repeat(4096)))
            .collect();
        let records = vec![
            ComponentRecord::function("demo", "root", commands),
            ComponentRecord::function("demo", "caller", vec!["function demo:root".into()]),
        ];
        let report = RuntimeReport::collect(&records, &RuntimePlan::default());
        for id in ["demo:root", "demo:caller"] {
            let cost = &report.functions[id];
            assert_eq!(cost.evaluations.maximum, None);
            assert!(serde_json::to_vec(cost).unwrap().len() < 4096, "{id}");
            assert!(
                cost.evaluations
                    .unknown
                    .iter()
                    .any(|reason| reason.contains("additional unknown causes omitted"))
            );
        }
        assert_eq!(report.functions["demo:root"].evaluations.minimum, 40);
        assert_eq!(report.functions["demo:caller"].evaluations.minimum, 41);
    }
    /// Host-added world functions and lifecycle tags participate in final costs.
    #[test]
    fn final_resources_recompute_calls_and_load_entries_without_losing_owners() {
        let records = vec![ComponentRecord::function(
            "demo",
            "tick",
            vec!["function demo:__sand_world_init".into()],
        )];
        let mut plan = RuntimePlan::default();
        plan.own_function("demo:tick".into(), "demo:module".into());
        plan.contribute_tag(
            "minecraft:tick".into(),
            "demo:tick".into(),
            "demo:module".into(),
        );
        let report = RuntimeReport::collect(&records, &plan);
        assert_eq!(report.functions["demo:tick"].evaluations.maximum, None);
        let decoded: RuntimeReport =
            serde_json::from_value(serde_json::to_value(report).unwrap()).unwrap();
        let resources = BTreeMap::from([
            (
                "data/demo/function/tick.mcfunction".into(),
                b"function demo:__sand_world_init".to_vec(),
            ),
            (
                "data/demo/function/__sand_world_init.mcfunction".into(),
                b"say world init".to_vec(),
            ),
            (
                "data/minecraft/tags/function/load.json".into(),
                br#"{"values":["demo:__sand_world_init"]}"#.to_vec(),
            ),
            (
                "data/minecraft/tags/function/tick.json".into(),
                br#"{"values":["demo:tick"]}"#.to_vec(),
            ),
        ]);
        let final_report = decoded.reanalyze(&resources);
        assert_eq!(
            final_report.functions["demo:tick"].evaluations.maximum,
            Some(2)
        );
        assert_eq!(
            final_report.functions["demo:tick"].owners,
            BTreeSet::from(["demo:module".into()])
        );
        assert_eq!(final_report.load[0].reference, "demo:__sand_world_init");
        assert_eq!(final_report.load[0].evaluations.maximum, Some(1));
        assert_eq!(
            final_report.tick[0].contributors,
            BTreeSet::from(["demo:module".into()])
        );
        let mut wrong_version = serde_json::to_value(&final_report).unwrap();
        wrong_version["schema_version"] = 2.into();
        assert!(serde_json::from_value::<RuntimeReport>(wrong_version).is_err());
    }
}
