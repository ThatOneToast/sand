//! Event dependency graph for same-cycle chained `SandEvent` dispatch (#240).
//!
//! Builds a forest of `SandEvent` nodes from direct `#[event]` handler
//! descriptors plus recursively-discovered `SandEventDispatch::chain::<P>()`
//! parents. A parent referenced only by a chain child (no direct `#[event]`
//! handler of its own) still gets a node — its detector/setup must still be
//! generated. This phase supports at most one parent per event, so the graph
//! is always a forest (no node has more than one incoming chain edge),
//! which makes cycle detection a simple parent-pointer walk.
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
use std::collections::BTreeMap;

use crate::condition::Condition;
use crate::events::{
    EventSetup, NormalizedEventDispatch, TickEventDispatch, TickExecutionPlans, TickScope,
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

/// A node's own detection mechanism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeOrigin {
    /// Detected independently via `SandEventDispatch::tick()` (or a
    /// normalized legacy `TickCondition`) — registered to `minecraft:tick`.
    Root(TickEventDispatch),
    /// Evaluated only from `parent`'s successful dispatch cycle, under the
    /// same execution subject/position, during the same generated dispatch
    /// path — never independently polled.
    Chained {
        /// Canonical type name of the parent node.
        parent: String,
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
                    parent: p,
                    persistent,
                    when,
                    unless,
                } if p == parent => Some(EventEdge {
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

    /// Validate the combined occurrence and persistent dependency topology.
    ///
    /// Occurrence edges remain the code-generation forest. Persistent edges
    /// are read-only constraints, but cycles involving them are rejected so a
    /// type cannot recursively define its current state through its consumers.
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
            let parent_type_id = (c.parent_type_id)();
            let parent_type_name = (c.parent_type_name)();
            let parent_dispatch = (c.parent_dispatch)().normalize();
            let parent_setup = (c.parent_setup)();

            if matches!(parent_dispatch, NormalizedEventDispatch::Advancement(_)) {
                visiting.pop();
                return Err(GraphError(format!(
                    "SandEvent `{type_name}` cannot chain from `{parent_type_name}`:\n\
                     parent dispatch scope does not provide a player execution context \
                     (advancement-backed SandEvent parents are not yet supported by chained \
                     dispatch — see #240)"
                )));
            }

            if let Err(e) = discover(
                parent_type_id,
                parent_type_name,
                parent_dispatch,
                parent_setup,
                handler_path,
                resolved,
                visiting,
            ) {
                visiting.pop();
                return Err(e);
            }

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
                parent: parent_type_name.to_string(),
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
        validate_definition_topology(
            (chain.parent_type_id)(),
            (chain.parent_type_name)(),
            (chain.parent_dispatch)().normalize(),
            "after",
            path,
        )?;
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
                parent,
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
            parent.as_str() == (c.parent_type_name)()
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
        ChainEventDispatch, PersistentEventCondition, PersistentSandEvent, SandEvent,
        SandEventDispatch,
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
        assert!(err.0.contains("dependency cycle"));
    }

    #[test]
    fn indirect_cycle_rejected() {
        // A chains from B, B chains from A: A -> B -> A.
        fn b_dispatch() -> SandEventDispatch {
            SandEventDispatch::chain::<A>().into()
        }
        let chain_a = ChainEventDispatch {
            parent_type_id: std::any::TypeId::of::<B>,
            parent_type_name: std::any::type_name::<B>,
            parent_dispatch: b_dispatch,
            parent_setup: EventSetup::none,
            persistent: Vec::new(),
            when: Vec::new(),
            unless: Vec::new(),
        };
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
}
