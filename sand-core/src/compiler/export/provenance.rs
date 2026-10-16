//! Semantic origins for event resources, captured from the validated event graph.
use std::collections::{BTreeMap, BTreeSet};

use super::records::ComponentRecord;
use crate::{
    compiler::runtime::RuntimePlan,
    events::graph::{EventGraph, NodeOrigin},
};

/// Event-specific owner sets and the compilation-local destination for them.
pub(crate) struct GraphRuntime<'a> {
    plan: &'a mut RuntimePlan,
    owners: BTreeMap<String, BTreeSet<String>>,
}

impl<'a> GraphRuntime<'a> {
    /// A detector belongs to its direct subscribers and consumers of its occurrences.
    pub(crate) fn new(graph: &EventGraph, namespace: &str, plan: &'a mut RuntimePlan) -> Self {
        let mut owners: BTreeMap<String, BTreeSet<String>> = graph
            .nodes
            .values()
            .map(|node| {
                let direct = node
                    .handlers
                    .iter()
                    .flat_map(|handler| plan.function_owners.get(&format!("{namespace}:{handler}")))
                    .flatten()
                    .cloned()
                    .collect();
                (node.type_name.to_owned(), direct)
            })
            .collect();
        // Persistent queries do not subscribe to provider occurrences. Only
        // occurrence and bounded dependencies require their source detectors.
        loop {
            let mut changed = false;
            for node in graph.nodes.values() {
                let NodeOrigin::Chained {
                    occurrence,
                    bounded,
                    ..
                } = &node.origin
                else {
                    continue;
                };
                let consumers = owners.get(node.type_name).cloned().unwrap_or_default();
                for parent in occurrence
                    .iter()
                    .flat_map(|clause| clause.parents().iter().map(|parent| parent.type_name))
                    .chain(bounded.iter().map(|dependency| dependency.type_name))
                {
                    let target = owners.entry(parent.to_owned()).or_default();
                    let before = target.len();
                    target.extend(consumers.iter().cloned());
                    changed |= target.len() != before;
                }
            }
            if !changed {
                break;
            }
        }
        Self { plan, owners }
    }

    /// Attribute one emitted function at the point its event identity is known.
    pub(crate) fn function(&mut self, event: &str, namespace: &str, path: &str) {
        for owner in self.owners.get(event).into_iter().flatten() {
            self.plan
                .own_function(format!("{namespace}:{path}"), owner.clone());
        }
    }

    /// Attribute a newly emitted batch to exactly the events that required it.
    pub(crate) fn records<'b>(
        &mut self,
        events: impl IntoIterator<Item = &'b str>,
        records: &[ComponentRecord],
    ) {
        let owners: BTreeSet<_> = events
            .into_iter()
            .flat_map(|event| self.owners.get(event))
            .flatten()
            .cloned()
            .collect();
        for record in records
            .iter()
            .filter(|record| record.dir == "function" && record.ext == "mcfunction")
        {
            for owner in &owners {
                self.plan.own_function(
                    format!("{}:{}", record.namespace, record.path),
                    owner.clone(),
                );
            }
        }
    }

    /// Retain lifecycle contributors for graph resources emitted in this batch.
    pub(crate) fn tags(
        &mut self,
        records: &[ComponentRecord],
        tags: &BTreeMap<String, Vec<String>>,
    ) {
        let functions: BTreeSet<_> = records
            .iter()
            .filter(|record| record.dir == "function")
            .map(|record| format!("{}:{}", record.namespace, record.path))
            .collect();
        for (tag, members) in tags {
            for member in members.iter().filter(|member| functions.contains(*member)) {
                for owner in self
                    .plan
                    .function_owners
                    .get(member)
                    .cloned()
                    .unwrap_or_default()
                {
                    self.plan.contribute_tag(tag.clone(), member.clone(), owner);
                }
            }
        }
    }
}

/// Attribute each validated typed function-tag member while its factory is known.
pub(crate) fn function_tag(
    record: &ComponentRecord,
    namespace: &str,
    origins: BTreeSet<crate::function::GeneratedOrigin>,
    runtime: &mut RuntimePlan,
) {
    if record.dir != "tags/function" || record.ext != "json" || record.content_type != "text" {
        return;
    }
    let Ok(tag) = serde_json::from_str::<serde_json::Value>(&record.content) else {
        return;
    };
    let Some(values) = tag["values"].as_array() else {
        return;
    };
    let tag_id = super::functions::resolve_local_refs(
        &format!("{}:{}", record.namespace, record.path),
        namespace,
    );
    for value in values {
        if let Some(member) = value
            .as_str()
            .or_else(|| value.get("id").and_then(|id| id.as_str()))
        {
            runtime.contribute_generated_tag(
                tag_id.clone(),
                super::functions::resolve_local_refs(member, namespace),
                super::functions::resolve_origins(origins.clone(), namespace),
            );
        }
    }
}
