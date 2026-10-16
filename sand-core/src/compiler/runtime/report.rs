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
#[derive(Debug, Clone, serde::Serialize)]
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
    schema_version: u32,
    /// Compiler-owned origins and cadence before publication erases them.
    provenance: RuntimePlan,
    /// Ordered direct entries in this pack's minecraft:tick tag.
    tick: Vec<TickEntry>,
    /// Function definitions, sorted by full resource id; shared definitions are
    /// listed once even when several owners or invocations refer to them.
    functions: BTreeMap<String, FunctionCost>,
    /// Conditions that prevent interpreting this report as a complete cost bound.
    limitations: BTreeSet<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TickEntry {
    reference: String,
    required: bool,
    contributors: BTreeSet<String>,
    evaluations: Work,
}

#[derive(Debug, Clone, serde::Serialize)]
struct FunctionCost {
    owners: BTreeSet<String>,
    evaluations: Work,
    selector_sites: Vec<SelectorSite>,
}

/// A lexical execution-context or entity-condition selector site. A site can
/// execute zero, one, or many times; this is not a runtime scan count.
#[derive(Debug, Clone, serde::Serialize)]
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

impl RuntimeReport {
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
        let costs = functions
            .keys()
            .map(|id| {
                (
                    id.clone(),
                    FunctionCost {
                        owners: plan.function_owners.get(id).cloned().unwrap_or_default(),
                        evaluations: estimator.function(id),
                        selector_sites: selector_sites(&functions[id]),
                    },
                )
            })
            .collect();
        let mut tick = Vec::new();
        let mut limitations = BTreeSet::from([
            "Counts are explicit command evaluations, not selector search cost, terminal execution multiplicity, MSPT, or Minecraft command-budget units.".into(),
            "Estimates assume successfully loaded functions and uninterrupted execution; runtime aborts, server budget exhaustion, external pack contributions, and implicit resource-triggered callbacks are not fully analyzed.".into(),
            "Selector sites cover execute contexts, entity/score conditions, and stores, not selectors in terminal commands; sites are not runtime scan counts.".into(),
            "An empty owner set means provenance is unavailable; resource paths are not inferred semantic owners.".into(),
        ]);
        for record in records.iter().filter(|record| {
            record.namespace == "minecraft"
                && record.dir == "tags/function"
                && record.path == "tick"
                && record.ext == "json"
        }) {
            let values = if record.content_type == "text" {
                serde_json::from_str::<serde_json::Value>(&record.content)
                    .ok()
                    .and_then(|value| value.get("values").and_then(|v| v.as_array()).cloned())
            } else {
                None
            };
            let Some(values) = values else {
                limitations.insert("minecraft:tick contents are unavailable or unsupported".into());
                continue;
            };
            for value in values {
                let Some(reference) = value
                    .as_str()
                    .or_else(|| value.get("id").and_then(|id| id.as_str()))
                else {
                    limitations.insert("minecraft:tick contains an unsupported entry".into());
                    continue;
                };
                let required = value
                    .get("required")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                tick.push(TickEntry {
                    reference: reference.into(),
                    required,
                    contributors: plan
                        .tag_contributors
                        .get(&("minecraft:tick".into(), reference.into()))
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
            functions: costs,
            limitations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
