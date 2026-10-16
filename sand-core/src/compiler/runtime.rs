//! Owned runtime provenance retained while the compiler still knows its owners.
//!
//! Resource names are output identities, not a substitute for semantic owners.
//! This input belongs to one compilation and never enters the datapack as a
//! synthetic component or process-global side channel.

mod estimate;
mod report;

pub use report::RuntimeReport;

use std::collections::{BTreeMap, BTreeSet};

/// Canonical owner lookup for a function-tag membership.
type TagContributors = BTreeMap<(String, String), BTreeSet<String>>;

/// Semantic inputs for runtime reporting, before final resource publication.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct RuntimePlan {
    /// Registration-time helper dependencies, consumed before publishing provenance.
    #[serde(skip)]
    generated_callers: BTreeMap<String, BTreeSet<String>>,
    /// Logical owners associated with a generated function's full resource id.
    pub(crate) function_owners: BTreeMap<String, BTreeSet<String>>,
    /// Owners of tag-membership contributions; contributing an invocation does
    /// not imply ownership of the referenced function's implementation.
    #[serde(
        serialize_with = "serialize_tag_contributors",
        deserialize_with = "deserialize_tag_contributors"
    )]
    pub(crate) tag_contributors: TagContributors,
    /// State-system groups after the canonical adjacent-scan fusion pass.
    pub(crate) systems: Vec<StateSystemRuntime>,
    /// Canonical archetype reports retained instead of reconstructed from paths.
    pub(crate) archetypes: Vec<crate::entity::archetype::EntityRuntimeReport>,
    /// Command spans and logical registration/module owners, keyed
    /// by the lifecycle coordinator's full function resource id.
    pub(crate) lifecycle_owners: BTreeMap<String, Vec<CommandOrigin>>,
}

/// Logical ownership of a contribution in its emitted function text.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CommandOrigin {
    /// Original registration or portable module id.
    pub(crate) owner: String,
    /// One-based first line in the generated function.
    pub(crate) first_line: usize,
    /// Number of source lines, including comments and internal blank lines.
    pub(crate) line_count: usize,
}

/// A lowered State-system group, including every owner sharing its scan.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct StateSystemRuntime {
    /// Stable authored system ids, in execution order.
    pub(crate) owners: Vec<String>,
    /// Generated body function; excludes the coordinator's clock maintenance.
    pub(crate) function: String,
    /// Interval in game ticks. Current counters reset to zero on pack load.
    pub(crate) every: u32,
    /// Canonical fused selector, if this group is exactly one shared scan.
    pub(crate) selector: Option<String>,
}

impl RuntimePlan {
    /// Retain all owners of a function without making ownership imply execution.
    pub(crate) fn own_function(&mut self, function: String, owner: String) {
        self.function_owners
            .entry(function)
            .or_default()
            .insert(owner);
    }

    /// Retain exact registration origins, including late replays of shared helpers.
    pub(crate) fn generated_function(
        &mut self,
        function: String,
        origins: impl IntoIterator<Item = crate::function::GeneratedOrigin>,
    ) {
        for origin in origins {
            match origin {
                crate::function::GeneratedOrigin::Logical(owner) => {
                    self.own_function(function.clone(), owner)
                }
                crate::function::GeneratedOrigin::Function(caller) => {
                    self.generated_callers
                        .entry(function.clone())
                        .or_default()
                        .insert(caller);
                }
            }
        }
    }

    /// Propagate owners through recorded helper registrations to a finite fixpoint.
    pub(crate) fn resolve_generated_owners(&mut self) {
        let mut children: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (child, callers) in std::mem::take(&mut self.generated_callers) {
            for caller in callers {
                children.entry(caller).or_default().insert(child.clone());
            }
        }
        let mut queue: std::collections::VecDeque<_> =
            self.function_owners.keys().cloned().collect();
        while let Some(parent) = queue.pop_front() {
            let owners = self
                .function_owners
                .get(&parent)
                .cloned()
                .unwrap_or_default();
            for child in children.get(&parent).into_iter().flatten() {
                let target = self.function_owners.entry(child.clone()).or_default();
                let before = target.len();
                target.extend(owners.iter().cloned());
                if target.len() != before {
                    queue.push_back(child.clone());
                }
            }
        }
    }

    /// Retain every contributor to a deduplicated tag membership.
    pub(crate) fn contribute_tag(&mut self, tag: String, function: String, owner: String) {
        self.tag_contributors
            .entry((tag, function))
            .or_default()
            .insert(owner);
    }
}

/// JSON object keys cannot represent the canonical (tag, member) lookup key.
/// Publish ordered records instead of stringifying that compound identity.
fn serialize_tag_contributors<S: serde::Serializer>(
    entries: &TagContributors,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    #[derive(Serialize)]
    struct Entry<'a> {
        tag: &'a str,
        member: &'a str,
        owners: &'a BTreeSet<String>,
    }
    entries
        .iter()
        .map(|((tag, member), owners)| Entry {
            tag,
            member,
            owners,
        })
        .collect::<Vec<_>>()
        .serialize(serializer)
}

/// Restore the canonical compound-key lookup from ordered wire records.
fn deserialize_tag_contributors<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<TagContributors, D::Error> {
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Entry {
        tag: String,
        member: String,
        owners: BTreeSet<String>,
    }
    let mut entries = BTreeMap::new();
    for entry in Vec::<Entry>::deserialize(deserializer)? {
        entries
            .entry((entry.tag, entry.member))
            .or_insert_with(BTreeSet::new)
            .extend(entry.owners);
    }
    Ok(entries)
}
