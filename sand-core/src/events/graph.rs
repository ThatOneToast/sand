//! Event dependency graph for same-cycle chained `SandEvent` dispatch (#240).
//!
//! Builds a deterministic graph of `SandEvent` nodes from direct `#[event]`
//! handler descriptors plus recursively discovered same-cycle and persistent
//! dependencies. A parent referenced only by an occurrence child still gets a
//! node because its detector/setup must be generated. Single-parent `after`
//! edges retain their immediate fan-out path; multi-parent clauses are lowered
//! through a staged coordinator so every relevant per-subject occurrence mark
//! is established before dependent evaluation.
//!
//! Every player-scoped custom tick detector goes through this graph as a
//! [`NodeOrigin::Root`] node — both `SandEventDispatch::tick()` and the
//! legacy `SandEventDispatch::TickCondition` compatibility constructor
//! normalize into the same [`TickEventDispatch`] shape and are discovered
//! identically, so a concrete `SandEvent` type resolves to exactly one node
//! (and one generated detector) regardless of which constructor its
//! `dispatch()` used. Advancement-backed `SandEvent`s are dispatched through
//! their own pre-existing reward-function codegen path, are never added to
//! this graph, and are explicitly rejected as a chain parent (see
//! [`discover`]). The unrelated bare `EventDispatch::TickPoll` used by
//! built-ins like `HoldingItemEvent`/`CurrentlyWearingEvent` (which have no
//! `SandEvent`/chain-parent concept) is also not part of this graph.

use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet};

use crate::condition::Condition;
use crate::events::{
    EventSetup, NormalizedEventDispatch, SameCycleEventDependency, SameCycleEventRequirement,
    TickEventDispatch, TickExecutionPlans, TickScope,
};

/// A resolved persistent-state requirement. This is a dependency for graph
/// validation and condition lowering, never an occurrence-producing edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistentDependency {
    pub type_id: TypeId,
    pub type_name: &'static str,
    pub scope: TickScope,
    pub condition: Condition,
}

/// One resolved same-cycle parent identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccurrenceParent {
    pub type_id: TypeId,
    pub type_name: &'static str,
}

/// One explicit same-cycle occurrence clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OccurrenceDependency {
    After(OccurrenceParent),
    AfterAny(Vec<OccurrenceParent>),
    AfterAll(Vec<OccurrenceParent>),
}

fn compare_occurrence_dependencies(
    left: &OccurrenceDependency,
    right: &OccurrenceDependency,
) -> std::cmp::Ordering {
    fn rank(dependency: &OccurrenceDependency) -> u8 {
        match dependency {
            OccurrenceDependency::After(_) => 0,
            OccurrenceDependency::AfterAny(_) => 1,
            OccurrenceDependency::AfterAll(_) => 2,
        }
    }

    rank(left).cmp(&rank(right)).then_with(|| {
        left.parents()
            .iter()
            .map(|parent| parent.type_name)
            .cmp(right.parents().iter().map(|parent| parent.type_name))
    })
}

impl OccurrenceDependency {
    pub fn parents(&self) -> &[OccurrenceParent] {
        match self {
            Self::After(parent) => std::slice::from_ref(parent),
            Self::AfterAny(parents) | Self::AfterAll(parents) => parents,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::After(_) => "after",
            Self::AfterAny(_) => "after_any",
            Self::AfterAll(_) => "after_all",
        }
    }
}

/// A node's own detection mechanism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeOrigin {
    /// Detected independently via `SandEventDispatch::tick()` (or a
    /// normalized legacy `TickCondition`) — registered to `minecraft:tick`.
    Root(TickEventDispatch),
    /// Evaluated from one or more same-cycle occurrence clauses, never
    /// independently polled.
    Chained {
        /// Explicit occurrence clauses. Every clause must hold; `AfterAny`
        /// is disjunctive only within its own group.
        occurrence: Vec<OccurrenceDependency>,
        /// Explicit persistent state requirements, sorted and deduplicated by
        /// canonical concrete type name.
        persistent: Vec<PersistentDependency>,
        /// Positive conditions — all must hold (ANDed).
        when: Vec<Condition>,
        /// Negative conditions — none may hold.
        unless: Vec<Condition>,
    },
}

/// One concrete `SandEvent` type in the graph.
#[derive(Debug, Clone)]
pub struct EventNode {
    /// In-process grouping identity — not stable across builds.
    pub type_id: TypeId,
    /// Canonical concrete type name (`std::any::type_name::<T>()`), also
    /// used as the graph identity and as input to the deterministic
    /// generated-resource-key derivation.
    pub type_name: &'static str,
    pub origin: NodeOrigin,
    pub setup: EventSetup,
    /// Direct `#[event]` handler function paths, sorted.
    pub handlers: Vec<&'static str>,
}

/// A parent → child same-cycle chain relationship, derived from a child
/// node's [`NodeOrigin::Chained`].
#[derive(Debug, Clone)]
pub struct EventEdge {
    pub parent: String,
    pub child: String,
    pub persistent: Vec<PersistentDependency>,
    pub when: Vec<Condition>,
    pub unless: Vec<Condition>,
}

/// A child that requires staged same-cycle occurrence resolution.
#[derive(Debug, Clone)]
pub struct StagedEvent {
    pub child: String,
    pub occurrence: Vec<OccurrenceDependency>,
    pub persistent: Vec<PersistentDependency>,
    pub when: Vec<Condition>,
    pub unless: Vec<Condition>,
}

impl StagedEvent {
    pub fn condition_edge(&self) -> EventEdge {
        EventEdge {
            parent: String::new(),
            child: self.child.clone(),
            persistent: self.persistent.clone(),
            when: self.when.clone(),
            unless: self.unless.clone(),
        }
    }
}

impl EventEdge {
    /// Expand this edge's conditions into explicit [`TickExecutionPlans`],
    /// same semantics as [`TickEventDispatch::execution_plans`] /
    /// `ChainEventDispatch::execution_plans`.
    pub fn execution_plans(&self) -> TickExecutionPlans {
        if self.persistent.is_empty() && self.when.is_empty() && self.unless.is_empty() {
            return TickExecutionPlans::Unconditional;
        }
        let mut positive: Vec<Condition> = self
            .persistent
            .iter()
            .map(|dependency| dependency.condition.clone())
            .collect();
        positive.extend(self.when.clone());
        let mut combined = if positive.is_empty() {
            Condition::all([])
        } else {
            Condition::all(positive)
        };
        for u in &self.unless {
            combined = combined.and_not(u.clone());
        }
        TickExecutionPlans::Plans(combined.to_execute_plans(false))
    }
}

/// The full discovered event dependency graph.
#[derive(Debug, Default)]
pub struct EventGraph {
    /// Nodes keyed by canonical type name — iterating this map is
    /// deterministic (alphabetical by canonical name) regardless of
    /// `#[event]` registration/inventory order.
    pub nodes: BTreeMap<String, EventNode>,
}

impl EventGraph {
    /// Root nodes (independent detectors), in deterministic order.
    pub fn roots(&self) -> impl Iterator<Item = &EventNode> {
        self.nodes
            .values()
            .filter(|n| matches!(n.origin, NodeOrigin::Root(_)))
    }

    /// Direct children chained from `parent`, sorted by canonical child
    /// name — deterministic regardless of registration order.
    pub fn children_of(&self, parent: &str) -> Vec<EventEdge> {
        let mut out: Vec<EventEdge> = self
            .nodes
            .values()
            .filter_map(|n| match &n.origin {
                NodeOrigin::Chained {
                    occurrence,
                    persistent,
                    when,
                    unless,
                } if matches!(occurrence.as_slice(), [OccurrenceDependency::After(p)] if p.type_name == parent) => Some(EventEdge {
                    parent: parent.to_string(),
                    child: n.type_name.to_string(),
                    persistent: persistent.clone(),
                    when: when.clone(),
                    unless: unless.clone(),
                }),
                _ => None,
            })
            .collect();
        out.sort_by(|a, b| a.child.cmp(&b.child));
        out
    }

    /// Whether this graph requires tick-local occurrence staging.
    pub fn has_staged_composition(&self) -> bool {
        self.nodes.values().any(|node| {
            matches!(&node.origin, NodeOrigin::Chained { occurrence, .. }
                if !matches!(occurrence.as_slice(), [OccurrenceDependency::After(_)]))
        })
    }

    /// Multi-clause/multi-parent children in deterministic occurrence
    /// topological order.
    pub fn staged_events(&self) -> Result<Vec<StagedEvent>, GraphError> {
        let order = self.occurrence_topological_order()?;
        let mut rank = BTreeMap::new();
        for (index, name) in order.into_iter().enumerate() {
            rank.insert(name, index);
        }
        let mut staged: Vec<StagedEvent> = self
            .nodes
            .values()
            .filter_map(|node| match &node.origin {
                NodeOrigin::Chained {
                    occurrence,
                    persistent,
                    when,
                    unless,
                } if !matches!(occurrence.as_slice(), [OccurrenceDependency::After(_)]) => {
                    Some(StagedEvent {
                        child: node.type_name.to_string(),
                        occurrence: occurrence.clone(),
                        persistent: persistent.clone(),
                        when: when.clone(),
                        unless: unless.clone(),
                    })
                }
                _ => None,
            })
            .collect();
        staged.sort_by(|left, right| {
            rank[&left.child]
                .cmp(&rank[&right.child])
                .then_with(|| left.child.cmp(&right.child))
        });
        Ok(staged)
    }

    /// Nodes whose per-subject occurrence is consumed by a staged clause.
    /// Leaf staged children are omitted until another staged dependency names
    /// them as a parent.
    pub fn occurrence_marked_nodes(&self) -> BTreeSet<String> {
        let mut marked = BTreeSet::new();
        for node in self.nodes.values() {
            let NodeOrigin::Chained { occurrence, .. } = &node.origin else {
                continue;
            };
            if matches!(occurrence.as_slice(), [OccurrenceDependency::After(_)]) {
                continue;
            }
            for clause in occurrence {
                marked.extend(
                    clause
                        .parents()
                        .iter()
                        .map(|parent| parent.type_name.to_string()),
                );
            }
        }
        marked
    }

    /// Every graph node in canonical same-cycle occurrence order.
    pub fn occurrence_topological_nodes(&self) -> Result<Vec<String>, GraphError> {
        self.occurrence_topological_order()
    }

    fn occurrence_topological_order(&self) -> Result<Vec<String>, GraphError> {
        let mut indegree: BTreeMap<String, usize> =
            self.nodes.keys().map(|name| (name.clone(), 0)).collect();
        let mut outgoing: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for node in self.nodes.values() {
            let NodeOrigin::Chained { occurrence, .. } = &node.origin else {
                continue;
            };
            let mut unique = BTreeSet::new();
            for clause in occurrence {
                for parent in clause.parents() {
                    unique.insert(parent.type_name.to_string());
                }
            }
            *indegree
                .get_mut(node.type_name)
                .expect("all graph nodes have indegree entries") += unique.len();
            for parent in unique {
                outgoing
                    .entry(parent)
                    .or_default()
                    .insert(node.type_name.to_string());
            }
        }

        let mut ready: BTreeSet<String> = indegree
            .iter()
            .filter_map(|(name, degree)| (*degree == 0).then_some(name.clone()))
            .collect();
        let mut order = Vec::with_capacity(indegree.len());
        while let Some(name) = ready.pop_first() {
            order.push(name.clone());
            if let Some(children) = outgoing.get(&name) {
                for child in children {
                    let degree = indegree
                        .get_mut(child)
                        .expect("outgoing children are graph nodes");
                    *degree -= 1;
                    if *degree == 0 {
                        ready.insert(child.clone());
                    }
                }
            }
        }
        if order.len() != indegree.len() {
            return Err(GraphError(
                "SandEvent occurrence dependency graph contains a cycle".to_string(),
            ));
        }
        Ok(order)
    }

    /// Validate the combined occurrence and persistent dependency topology.
    ///
    /// Occurrence edges establish deterministic evaluation topology.
    /// Persistent edges are read-only constraints, but cycles involving them
    /// are rejected so a type cannot recursively define its current state
    /// through its consumers.
    pub fn validate_dependencies(&self) -> Result<(), GraphError> {
        let mut persistent_registry: BTreeMap<
            &'static str,
            (TypeId, TickScope, Condition, &'static str),
        > = BTreeMap::new();
        for node in self.nodes.values() {
            let NodeOrigin::Chained { persistent, .. } = &node.origin else {
                continue;
            };
            for dependency in persistent {
                match persistent_registry.get(dependency.type_name) {
                    Some((type_id, scope, condition, first_child))
                        if *type_id != dependency.type_id
                            || *scope != dependency.scope
                            || *condition != dependency.condition =>
                    {
                        return Err(GraphError(format!(
                            "persistent event identity collision for `{}`: children `{first_child}` and `{}` resolved the same canonical name to incompatible type identities or conditions",
                            dependency.type_name, node.type_name
                        )));
                    }
                    Some(_) => {}
                    None => {
                        persistent_registry.insert(
                            dependency.type_name,
                            (
                                dependency.type_id,
                                dependency.scope,
                                dependency.condition.clone(),
                                node.type_name,
                            ),
                        );
                    }
                }
            }
        }

        self.occurrence_topological_order()?;
        Ok(())
    }
}

/// Graph construction/validation failure — a cycle, an unsupported parent
/// scope, or conflicting definitions for the same concrete event type.
/// Always surfaced as an export error, never a panic.
#[derive(Debug, Clone)]
pub struct GraphError(pub String);

/// Discover (or reuse) the node for `type_id`/`type_name`, recursively
/// resolving any chain parent, and record `handler_path` against it.
///
/// Idempotent: calling this once per `#[event]` handler descriptor is safe —
/// handlers of the same concrete type share one node, and repeat calls for an
/// already-resolved parent are cheap cache hits validated for consistency.
pub fn discover_node(
    type_id: TypeId,
    type_name: &'static str,
    dispatch: NormalizedEventDispatch,
    setup: EventSetup,
    handler_path: &'static str,
    resolved: &mut BTreeMap<TypeId, EventNode>,
) -> Result<(), GraphError> {
    let mut visiting: Vec<&'static str> = Vec::new();
    discover(
        type_id,
        type_name,
        dispatch,
        setup,
        handler_path,
        resolved,
        &mut visiting,
    )?;
    let node = resolved
        .get_mut(&type_id)
        .expect("discover() always inserts or returns Err");
    if !node.handlers.contains(&handler_path) {
        node.handlers.push(handler_path);
    }
    Ok(())
}

fn discover(
    type_id: TypeId,
    type_name: &'static str,
    dispatch: NormalizedEventDispatch,
    setup: EventSetup,
    handler_path: &'static str,
    resolved: &mut BTreeMap<TypeId, EventNode>,
    visiting: &mut Vec<&'static str>,
) -> Result<(), GraphError> {
    if let Some(existing) = resolved.get(&type_id) {
        return validate_consistent(type_name, &dispatch, &setup, handler_path, existing);
    }
    if visiting.contains(&type_name) {
        return Err(cycle_error(visiting, type_name));
    }
    visiting.push(type_name);

    let origin = match dispatch {
        NormalizedEventDispatch::Advancement(_) => {
            visiting.pop();
            return Err(GraphError(format!(
                "SandEvent `{type_name}` cannot participate in same-cycle chained dispatch: \
                 advancement-backed SandEvent parents are not yet supported by chained dispatch \
                 (#240) — chain from a tick-lifecycle SandEvent instead"
            )));
        }
        NormalizedEventDispatch::Tick(t) => NodeOrigin::Root(t),
        NormalizedEventDispatch::Chain(c) => {
            if c.occurrence.is_empty() {
                visiting.pop();
                return Err(GraphError(format!(
                    "SandEvent `{type_name}` returned an empty same-cycle composition; add `after::<E>()`, `after_any::<(A, B)>()`, or `after_all::<(A, B)>()`"
                )));
            }

            let occurrence = resolve_occurrence_dependencies(
                type_id,
                type_name,
                &c.occurrence,
                handler_path,
                resolved,
                visiting,
            )?;

            let mut persistent_by_name: BTreeMap<&'static str, PersistentDependency> =
                BTreeMap::new();
            for dependency in c.persistent {
                let dependency_type_id = (dependency.event_type_id)();
                let dependency_type_name = (dependency.event_type_name)();
                let resolved_condition = (dependency.make_condition)();
                if dependency_type_id == type_id || dependency_type_name == type_name {
                    visiting.pop();
                    return Err(GraphError(format!(
                        "SandEvent `{type_name}` has an invalid persistent self-dependency through `while_::<{dependency_type_name}>()`"
                    )));
                }
                if resolved_condition.scope != TickScope::Players {
                    visiting.pop();
                    return Err(GraphError(format!(
                        "SandEvent `{type_name}` cannot evaluate persistent state `{dependency_type_name}`: the child inherits player scope but the persistent condition requires {:?}",
                        resolved_condition.scope
                    )));
                }
                let dependency_setup = (dependency.event_setup)();
                if dependency_setup != EventSetup::none() {
                    visiting.pop();
                    return Err(GraphError(format!(
                        "SandEvent `{type_name}` cannot evaluate persistent state `{dependency_type_name}`: its SandEvent::setup() is non-empty, but `while_` requires a directly queryable condition and never runs provider detector lifecycle; provision shared resources through typed state lifecycle or return an independently valid condition"
                    )));
                }
                let mut topology = vec![(type_id, type_name, "root")];
                validate_definition_topology(
                    dependency_type_id,
                    dependency_type_name,
                    (dependency.event_dispatch)().normalize(),
                    "while",
                    &mut topology,
                )?;
                let resolved = PersistentDependency {
                    type_id: dependency_type_id,
                    type_name: dependency_type_name,
                    scope: resolved_condition.scope,
                    condition: resolved_condition.condition,
                };
                match persistent_by_name.get(dependency_type_name) {
                    Some(existing) if existing != &resolved => {
                        visiting.pop();
                        return Err(GraphError(format!(
                            "SandEvent `{type_name}` received conflicting persistent definitions for `{dependency_type_name}`"
                        )));
                    }
                    Some(_) => {}
                    None => {
                        persistent_by_name.insert(dependency_type_name, resolved);
                    }
                }
            }

            NodeOrigin::Chained {
                occurrence,
                persistent: persistent_by_name.into_values().collect(),
                when: c.when,
                unless: c.unless,
            }
        }
    };

    visiting.pop();
    resolved.insert(
        type_id,
        EventNode {
            type_id,
            type_name,
            origin,
            setup,
            handlers: Vec::new(),
        },
    );
    Ok(())
}

fn resolve_occurrence_dependencies(
    child_type_id: TypeId,
    child_type_name: &'static str,
    requirements: &[SameCycleEventRequirement],
    handler_path: &'static str,
    resolved: &mut BTreeMap<TypeId, EventNode>,
    visiting: &mut Vec<&'static str>,
) -> Result<Vec<OccurrenceDependency>, GraphError> {
    let mut resolved_requirements = Vec::with_capacity(requirements.len());
    let mut any_groups = 0usize;
    let mut all_groups = 0usize;

    for requirement in requirements {
        let (label, factories): (&'static str, &[SameCycleEventDependency]) = match requirement {
            SameCycleEventRequirement::After(parent) => ("after", std::slice::from_ref(parent)),
            SameCycleEventRequirement::AfterAny(parents) => {
                any_groups += 1;
                ("after_any", parents)
            }
            SameCycleEventRequirement::AfterAll(parents) => {
                all_groups += 1;
                ("after_all", parents)
            }
        };
        if factories.is_empty() {
            return Err(GraphError(format!(
                "SandEvent `{child_type_name}` declared an empty `{label}` group; use a typed tuple of 2 through 8 concrete SandEvent parents"
            )));
        }

        let mut parents = Vec::with_capacity(factories.len());
        let mut seen_types = BTreeSet::new();
        let mut seen_names: BTreeMap<&'static str, TypeId> = BTreeMap::new();
        for factory in factories {
            let parent_type_id = (factory.event_type_id)();
            let parent_type_name = (factory.event_type_name)();
            if parent_type_id == child_type_id || parent_type_name == child_type_name {
                return Err(GraphError(format!(
                    "SandEvent `{child_type_name}` has an invalid `{label}` self-dependency through parent `{parent_type_name}`"
                )));
            }
            if !seen_types.insert(parent_type_id) {
                return Err(GraphError(format!(
                    "SandEvent `{child_type_name}` declared duplicate parent `{parent_type_name}` inside `{label}`; every group member must be a distinct concrete event type"
                )));
            }
            if let Some(existing_id) = seen_names.insert(parent_type_name, parent_type_id)
                && existing_id != parent_type_id
            {
                return Err(GraphError(format!(
                    "canonical event identity collision in `{label}` for child `{child_type_name}`: distinct Rust event types resolve to `{parent_type_name}`"
                )));
            }

            let mut topology = vec![(child_type_id, child_type_name, "root")];
            validate_definition_topology(
                parent_type_id,
                parent_type_name,
                (factory.event_dispatch)().normalize(),
                label,
                &mut topology,
            )?;

            let parent_dispatch = (factory.event_dispatch)().normalize();
            if matches!(parent_dispatch, NormalizedEventDispatch::Advancement(_)) {
                return Err(GraphError(format!(
                    "SandEvent `{child_type_name}` cannot use `{parent_type_name}` in `{label}`: the advancement-backed parent cannot inherit the required player execution context (see #240)"
                )));
            }
            if let NormalizedEventDispatch::Tick(tick) = &parent_dispatch
                && tick.scope != TickScope::Players
            {
                return Err(GraphError(format!(
                    "SandEvent `{child_type_name}` cannot use `{parent_type_name}` in `{label}`: the child requires player scope but the parent dispatch uses {:?}",
                    tick.scope
                )));
            }
            discover(
                parent_type_id,
                parent_type_name,
                parent_dispatch,
                (factory.event_setup)(),
                handler_path,
                resolved,
                visiting,
            )?;
            parents.push(OccurrenceParent {
                type_id: parent_type_id,
                type_name: parent_type_name,
            });
        }
        parents.sort_by_key(|parent| parent.type_name);
        let resolved = match requirement {
            SameCycleEventRequirement::After(_) => OccurrenceDependency::After(parents.remove(0)),
            SameCycleEventRequirement::AfterAny(_) => OccurrenceDependency::AfterAny(parents),
            SameCycleEventRequirement::AfterAll(_) => OccurrenceDependency::AfterAll(parents),
        };
        resolved_requirements.push(resolved);
    }

    if any_groups > 1 {
        return Err(GraphError(format!(
            "SandEvent `{child_type_name}` declared multiple `after_any` groups; combine the parents into one typed tuple so the at-most-once coalescing boundary is explicit"
        )));
    }
    if all_groups > 1 {
        return Err(GraphError(format!(
            "SandEvent `{child_type_name}` declared multiple `after_all` groups; combine the parents into one typed tuple"
        )));
    }
    resolved_requirements.sort_by(compare_occurrence_dependencies);
    for duplicate in resolved_requirements.windows(2) {
        if duplicate[0] == duplicate[1] {
            return Err(GraphError(format!(
                "SandEvent `{child_type_name}` declared the same `{}` occurrence group more than once",
                duplicate[0].label()
            )));
        }
    }
    Ok(resolved_requirements)
}

/// Validate a referenced event definition's dependency topology without
/// subscribing it or generating its detector. This keeps graph validity
/// independent of whether the persistent provider happens to have a handler.
fn validate_definition_topology(
    type_id: TypeId,
    type_name: &'static str,
    dispatch: NormalizedEventDispatch,
    incoming_kind: &'static str,
    path: &mut Vec<(TypeId, &'static str, &'static str)>,
) -> Result<(), GraphError> {
    if path
        .iter()
        .any(|(existing_id, _, _)| *existing_id == type_id)
    {
        let start = path
            .iter()
            .position(|(existing_id, _, _)| *existing_id == type_id)
            .expect("the repeated type was just found");
        let mut rendered = path[start].1.to_string();
        for (_, name, kind) in path.iter().skip(start + 1) {
            rendered.push_str(&format!(" -[{kind}]-> {name}"));
        }
        rendered.push_str(&format!(" -[{incoming_kind}]-> {type_name}"));
        return Err(GraphError(format!(
            "SandEvent dependency cycle:\n{rendered}"
        )));
    }

    if let Some((_, _, first_kind)) = path.iter().find(|(existing_id, existing_name, _)| {
        *existing_id != type_id && *existing_name == type_name
    }) {
        return Err(GraphError(format!(
            "canonical event identity collision for `{type_name}` while validating persistent topology: the `{first_kind}` and `{incoming_kind}` dependencies resolve that name to distinct concrete event types"
        )));
    }

    path.push((type_id, type_name, incoming_kind));
    if let NormalizedEventDispatch::Chain(chain) = dispatch {
        for requirement in chain.occurrence {
            let (label, parents): (&'static str, Vec<SameCycleEventDependency>) = match requirement
            {
                SameCycleEventRequirement::After(parent) => ("after", vec![parent]),
                SameCycleEventRequirement::AfterAny(parents) => ("after_any", parents),
                SameCycleEventRequirement::AfterAll(parents) => ("after_all", parents),
            };
            for parent in parents {
                validate_definition_topology(
                    (parent.event_type_id)(),
                    (parent.event_type_name)(),
                    (parent.event_dispatch)().normalize(),
                    label,
                    path,
                )?;
            }
        }
        for persistent in chain.persistent {
            validate_definition_topology(
                (persistent.event_type_id)(),
                (persistent.event_type_name)(),
                (persistent.event_dispatch)().normalize(),
                "while",
                path,
            )?;
        }
    }
    path.pop();
    Ok(())
}

fn validate_consistent(
    type_name: &'static str,
    dispatch: &NormalizedEventDispatch,
    setup: &EventSetup,
    handler_path: &'static str,
    existing: &EventNode,
) -> Result<(), GraphError> {
    if &existing.setup != setup {
        return Err(GraphError(format!(
            "conflicting SandEvent definitions for `{type_name}`: handler(s) {:?} and handler \
             `{handler_path}` returned different setup() results for the same event type — every \
             handler subscribing to one SandEvent must observe identical dispatch()/setup() \
             output",
            existing.handlers,
        )));
    }
    let consistent = match (&existing.origin, dispatch) {
        (NodeOrigin::Root(t1), NormalizedEventDispatch::Tick(t2)) => t1 == t2,
        (
            NodeOrigin::Chained {
                occurrence,
                persistent,
                when,
                unless,
            },
            NormalizedEventDispatch::Chain(c),
        ) => {
            let mut incoming: Vec<PersistentDependency> = c
                .persistent
                .iter()
                .map(|dependency| {
                    let condition = (dependency.make_condition)();
                    PersistentDependency {
                        type_id: (dependency.event_type_id)(),
                        type_name: (dependency.event_type_name)(),
                        scope: condition.scope,
                        condition: condition.condition,
                    }
                })
                .collect();
            incoming.sort_by_key(|dependency| dependency.type_name);
            incoming.dedup_by(|left, right| left == right);
            occurrence == &occurrence_identity(&c.occurrence)
                && persistent == &incoming
                && when == &c.when
                && unless == &c.unless
        }
        _ => false,
    };
    if !consistent {
        return Err(GraphError(format!(
            "conflicting SandEvent definitions for `{type_name}`: handler(s) {:?} and handler \
             `{handler_path}` returned different dispatch() results for the same event type — \
             every handler subscribing to one SandEvent must observe identical dispatch()/setup() \
             output",
            existing.handlers,
        )));
    }
    Ok(())
}

fn occurrence_identity(requirements: &[SameCycleEventRequirement]) -> Vec<OccurrenceDependency> {
    let mut resolved = requirements
        .iter()
        .map(|requirement| {
            let mut parents = match requirement {
                SameCycleEventRequirement::After(parent) => vec![OccurrenceParent {
                    type_id: (parent.event_type_id)(),
                    type_name: (parent.event_type_name)(),
                }],
                SameCycleEventRequirement::AfterAny(parents)
                | SameCycleEventRequirement::AfterAll(parents) => parents
                    .iter()
                    .map(|parent| OccurrenceParent {
                        type_id: (parent.event_type_id)(),
                        type_name: (parent.event_type_name)(),
                    })
                    .collect(),
            };
            parents.sort_by_key(|parent| parent.type_name);
            match requirement {
                SameCycleEventRequirement::After(_) => {
                    OccurrenceDependency::After(parents.remove(0))
                }
                SameCycleEventRequirement::AfterAny(_) => OccurrenceDependency::AfterAny(parents),
                SameCycleEventRequirement::AfterAll(_) => OccurrenceDependency::AfterAll(parents),
            }
        })
        .collect::<Vec<_>>();
    resolved.sort_by(compare_occurrence_dependencies);
    resolved
}

/// Build the readable canonical cycle path, e.g.
/// `SandEvent dependency cycle:\nA -> B -> C -> A`.
fn cycle_error(visiting: &[&'static str], repeated: &'static str) -> GraphError {
    let mut path: Vec<&str> = visiting.to_vec();
    path.push(repeated);
    let start = path.iter().position(|n| *n == repeated).unwrap_or(0);
    let cycle = &path[start..];
    GraphError(format!(
        "SandEvent dependency cycle:\n{}",
        cycle.join(" -> ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{
        PersistentEventCondition, PersistentSandEvent, SandEvent, SandEventDispatch,
    };

    struct A;
    struct B;
    struct C;
    struct PersistentA;
    struct PersistentB;
    struct PersistentLeaf;
    struct SetupPersistent;
    struct CollisionA;
    struct CollisionB;
    struct D;
    struct MultiAny;
    struct MultiAll;
    struct MixedCycleA;
    struct MixedCycleB;
    struct GenericParent<T>(std::marker::PhantomData<T>);
    struct GenericOne;
    struct GenericTwo;

    impl SandEvent for A {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::tick().as_players()
        }
    }
    impl SandEvent for B {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::chain::<A>()
        }
    }
    impl SandEvent for C {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::chain::<B>()
        }
    }
    impl SandEvent for D {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::tick().as_players()
        }
    }
    impl SandEvent for MultiAny {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::after_any::<(D, A)>()
        }
    }
    impl SandEvent for MultiAll {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::after_all::<(MultiAny, A)>()
        }
    }
    impl SandEvent for MixedCycleA {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::after_any::<(MixedCycleB, D)>()
        }
    }
    impl SandEvent for MixedCycleB {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::chain::<MixedCycleA>()
        }
    }
    impl<T> SandEvent for GenericParent<T> {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::tick().as_players()
        }
    }
    impl SandEvent for PersistentA {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::chain::<A>().while_::<PersistentB>()
        }
    }
    impl PersistentSandEvent for PersistentA {
        fn persistent_condition() -> PersistentEventCondition {
            PersistentEventCondition::players(Condition::entity("@s[tag=a]"))
        }
    }
    impl SandEvent for PersistentB {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::chain::<PersistentA>()
        }
    }
    impl PersistentSandEvent for PersistentB {
        fn persistent_condition() -> PersistentEventCondition {
            PersistentEventCondition::players(Condition::entity("@s[tag=b]"))
        }
    }
    impl SandEvent for PersistentLeaf {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::tick().as_players()
        }
    }
    impl PersistentSandEvent for PersistentLeaf {
        fn persistent_condition() -> PersistentEventCondition {
            PersistentEventCondition::players(Condition::entity("@s[tag=leaf]"))
        }
    }
    impl SandEvent for SetupPersistent {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::tick().as_players()
        }

        fn setup() -> EventSetup {
            EventSetup {
                objectives: vec!["scoreboard objectives add needs_setup dummy".into()],
                pre_observation: vec![],
                post_observation: vec![],
            }
        }
    }
    impl PersistentSandEvent for SetupPersistent {
        fn persistent_condition() -> PersistentEventCondition {
            PersistentEventCondition::players(Condition::Score {
                selector: "@s".into(),
                objective: "needs_setup".into(),
                range: crate::condition::ScoreRange::Eq(1),
            })
        }
    }

    fn tick_root() -> NormalizedEventDispatch {
        NormalizedEventDispatch::Tick(TickEventDispatch::default().as_players())
    }

    fn as_graph(resolved: BTreeMap<TypeId, EventNode>) -> EventGraph {
        EventGraph {
            nodes: resolved
                .into_values()
                .map(|n| (n.type_name.to_string(), n))
                .collect(),
        }
    }

    #[test]
    fn one_parent_one_child() {
        let mut resolved = BTreeMap::new();
        discover_node(
            TypeId::of::<A>(),
            std::any::type_name::<A>(),
            tick_root(),
            EventSetup::none(),
            "on_a",
            &mut resolved,
        )
        .unwrap();
        discover_node(
            TypeId::of::<B>(),
            std::any::type_name::<B>(),
            NormalizedEventDispatch::Chain(SandEventDispatch::chain::<A>()),
            EventSetup::none(),
            "on_b",
            &mut resolved,
        )
        .unwrap();

        let graph = as_graph(resolved);
        assert_eq!(graph.roots().count(), 1);
        let children = graph.children_of(std::any::type_name::<A>());
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].child, std::any::type_name::<B>());
    }

    #[test]
    fn parent_with_several_children() {
        let mut resolved = BTreeMap::new();
        discover_node(
            TypeId::of::<A>(),
            std::any::type_name::<A>(),
            tick_root(),
            EventSetup::none(),
            "on_a",
            &mut resolved,
        )
        .unwrap();
        for (id, name, path) in [
            (TypeId::of::<B>(), std::any::type_name::<B>(), "on_b"),
            (TypeId::of::<C>(), std::any::type_name::<C>(), "on_c"),
        ] {
            discover_node(
                id,
                name,
                NormalizedEventDispatch::Chain(SandEventDispatch::chain::<A>()),
                EventSetup::none(),
                path,
                &mut resolved,
            )
            .unwrap();
        }

        let graph = as_graph(resolved);
        let children = graph.children_of(std::any::type_name::<A>());
        assert_eq!(children.len(), 2);
        // Deterministic: sorted by canonical child name, not registration order.
        assert!(children[0].child < children[1].child);
    }

    #[test]
    fn direct_self_cycle_rejected() {
        let mut resolved = BTreeMap::new();
        let err = discover_node(
            TypeId::of::<A>(),
            std::any::type_name::<A>(),
            NormalizedEventDispatch::Chain(SandEventDispatch::chain::<A>()),
            EventSetup::none(),
            "on_a",
            &mut resolved,
        )
        .unwrap_err();
        assert!(err.0.contains("self-dependency"));
    }

    #[test]
    fn indirect_cycle_rejected() {
        // A chains from B, B chains from A: A -> B -> A.
        let chain_a = SandEventDispatch::chain::<B>();
        let mut resolved = BTreeMap::new();
        let err = discover_node(
            TypeId::of::<A>(),
            std::any::type_name::<A>(),
            NormalizedEventDispatch::Chain(chain_a),
            EventSetup::none(),
            "on_a",
            &mut resolved,
        )
        .unwrap_err();
        assert!(err.0.contains("dependency cycle"));
    }

    #[test]
    fn deep_chain_a_b_c() {
        let mut resolved = BTreeMap::new();
        discover_node(
            TypeId::of::<A>(),
            std::any::type_name::<A>(),
            tick_root(),
            EventSetup::none(),
            "on_a",
            &mut resolved,
        )
        .unwrap();
        discover_node(
            TypeId::of::<B>(),
            std::any::type_name::<B>(),
            NormalizedEventDispatch::Chain(SandEventDispatch::chain::<A>()),
            EventSetup::none(),
            "on_b",
            &mut resolved,
        )
        .unwrap();
        discover_node(
            TypeId::of::<C>(),
            std::any::type_name::<C>(),
            NormalizedEventDispatch::Chain(SandEventDispatch::chain::<B>()),
            EventSetup::none(),
            "on_c",
            &mut resolved,
        )
        .unwrap();

        let graph = as_graph(resolved);
        let b_children = graph.children_of(std::any::type_name::<B>());
        assert_eq!(b_children.len(), 1);
        assert_eq!(b_children[0].child, std::any::type_name::<C>());
        let a_children = graph.children_of(std::any::type_name::<A>());
        assert_eq!(a_children.len(), 1);
        assert_eq!(a_children[0].child, std::any::type_name::<B>());
    }

    #[test]
    fn persistent_edge_is_distinct_and_deduplicated() {
        let mut resolved = BTreeMap::new();
        let dispatch = SandEventDispatch::chain::<A>()
            .while_::<PersistentLeaf>()
            .while_::<PersistentLeaf>();
        discover_node(
            TypeId::of::<B>(),
            std::any::type_name::<B>(),
            NormalizedEventDispatch::Chain(dispatch),
            EventSetup::none(),
            "on_b",
            &mut resolved,
        )
        .unwrap();
        let graph = as_graph(resolved);
        let edge = graph
            .children_of(std::any::type_name::<A>())
            .pop()
            .expect("edge exists");
        assert_eq!(edge.persistent.len(), 1);
        assert_eq!(
            edge.persistent[0].type_name,
            std::any::type_name::<PersistentLeaf>()
        );
    }

    #[test]
    fn direct_persistent_self_dependency_is_rejected() {
        let mut resolved = BTreeMap::new();
        let err = discover_node(
            TypeId::of::<PersistentA>(),
            std::any::type_name::<PersistentA>(),
            NormalizedEventDispatch::Chain(SandEventDispatch::chain::<A>().while_::<PersistentA>()),
            EventSetup::none(),
            "on_a",
            &mut resolved,
        )
        .unwrap_err();
        assert!(err.0.contains("persistent self-dependency"));
        assert!(err.0.contains("PersistentA"));
    }

    #[test]
    fn indirect_persistent_cycle_is_rejected() {
        let mut resolved = BTreeMap::new();
        let err = discover_node(
            TypeId::of::<PersistentA>(),
            std::any::type_name::<PersistentA>(),
            PersistentA::dispatch().into().normalize(),
            EventSetup::none(),
            "on_a",
            &mut resolved,
        )
        .unwrap_err();
        assert!(err.0.contains("dependency cycle"));
        assert!(err.0.contains("PersistentA") && err.0.contains("PersistentB"));
        assert!(err.0.contains("-[while]->") && err.0.contains("-[after]->"));
    }

    #[test]
    fn persistent_provider_with_detector_setup_is_rejected() {
        let mut resolved = BTreeMap::new();
        let err = discover_node(
            TypeId::of::<B>(),
            std::any::type_name::<B>(),
            NormalizedEventDispatch::Chain(
                SandEventDispatch::chain::<A>().while_::<SetupPersistent>(),
            ),
            EventSetup::none(),
            "on_b",
            &mut resolved,
        )
        .unwrap_err();
        assert!(err.0.contains(std::any::type_name::<B>()));
        assert!(err.0.contains(std::any::type_name::<SetupPersistent>()));
        assert!(err.0.contains("setup() is non-empty"));
    }

    #[test]
    fn provider_only_topology_reports_canonical_identity_collisions() {
        let colliding_name = "test::SamePersistentProvider";
        let mut path = vec![(TypeId::of::<CollisionA>(), colliding_name, "root")];
        let err = validate_definition_topology(
            TypeId::of::<CollisionB>(),
            colliding_name,
            tick_root(),
            "while",
            &mut path,
        )
        .unwrap_err();

        assert!(err.0.contains("canonical event identity collision"));
        assert!(err.0.contains(colliding_name));
        assert!(!err.0.contains("dependency cycle"));
    }

    #[test]
    fn multi_parent_groups_are_explicit_and_topologically_ordered() {
        let mut resolved = BTreeMap::new();
        discover_node(
            TypeId::of::<MultiAll>(),
            std::any::type_name::<MultiAll>(),
            MultiAll::dispatch().into().normalize(),
            EventSetup::none(),
            "on_multi_all",
            &mut resolved,
        )
        .unwrap();
        let graph = as_graph(resolved);
        let staged = graph.staged_events().unwrap();
        assert_eq!(staged.len(), 2);
        assert_eq!(staged[0].child, std::any::type_name::<MultiAny>());
        assert_eq!(staged[1].child, std::any::type_name::<MultiAll>());
        assert!(matches!(
            staged[0].occurrence.as_slice(),
            [OccurrenceDependency::AfterAny(_)]
        ));
        assert!(matches!(
            staged[1].occurrence.as_slice(),
            [OccurrenceDependency::AfterAll(_)]
        ));
    }

    #[test]
    fn duplicate_parent_inside_group_is_rejected() {
        let mut resolved = BTreeMap::new();
        let dispatch = SandEventDispatch::from(SandEventDispatch::after_any::<(A, A)>());
        let err = discover_node(
            TypeId::of::<MultiAny>(),
            std::any::type_name::<MultiAny>(),
            dispatch.normalize(),
            EventSetup::none(),
            "on_duplicate",
            &mut resolved,
        )
        .unwrap_err();
        assert!(err.0.contains("duplicate parent"));
        assert!(err.0.contains(std::any::type_name::<A>()));
    }

    #[test]
    fn repeated_any_group_is_rejected() {
        let mut resolved = BTreeMap::new();
        let dispatch = SandEventDispatch::from(
            SandEventDispatch::compose()
                .after_any::<(A, D)>()
                .after_any::<(B, D)>(),
        );
        let err = discover_node(
            TypeId::of::<MultiAny>(),
            std::any::type_name::<MultiAny>(),
            dispatch.normalize(),
            EventSetup::none(),
            "on_repeated_any",
            &mut resolved,
        )
        .unwrap_err();
        assert!(err.0.contains("multiple `after_any` groups"));
    }

    #[test]
    fn group_order_is_canonical_and_generic_parents_remain_distinct() {
        let left = SandEventDispatch::from(SandEventDispatch::after_any::<(
            GenericParent<GenericTwo>,
            GenericParent<GenericOne>,
        )>())
        .normalize();
        let right = SandEventDispatch::from(SandEventDispatch::after_any::<(
            GenericParent<GenericOne>,
            GenericParent<GenericTwo>,
        )>())
        .normalize();

        let NormalizedEventDispatch::Chain(left) = left else {
            unreachable!();
        };
        let NormalizedEventDispatch::Chain(right) = right else {
            unreachable!();
        };
        assert_eq!(
            occurrence_identity(&left.occurrence),
            occurrence_identity(&right.occurrence)
        );
        let identity = occurrence_identity(&left.occurrence);
        assert_eq!(identity.len(), 1);
        let OccurrenceDependency::AfterAny(parents) = &identity[0] else {
            panic!("expected one any-parent group");
        };
        assert_eq!(parents.len(), 2);
        assert_ne!(parents[0].type_id, parents[1].type_id);
        assert_ne!(parents[0].type_name, parents[1].type_name);
    }

    #[test]
    fn mixed_any_after_cycle_has_labeled_path() {
        let mut resolved = BTreeMap::new();
        let err = discover_node(
            TypeId::of::<MixedCycleA>(),
            std::any::type_name::<MixedCycleA>(),
            MixedCycleA::dispatch().into().normalize(),
            EventSetup::none(),
            "on_mixed_cycle",
            &mut resolved,
        )
        .unwrap_err();
        assert!(err.0.contains("dependency cycle"));
        assert!(err.0.contains("-[after_any]->"));
        assert!(err.0.contains("-[after]->"));
    }
}
