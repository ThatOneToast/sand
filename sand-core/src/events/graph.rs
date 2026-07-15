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
//! Not part of the graph: `TickPoll`/`AdvancementTrigger`-backed events,
//! which are dispatched through their own pre-existing codegen paths and
//! never chained from.

use std::any::TypeId;
use std::collections::BTreeMap;

use crate::condition::Condition;
use crate::events::{EventSetup, NormalizedEventDispatch, TickEventDispatch, TickExecutionPlans};

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
    pub when: Vec<Condition>,
    pub unless: Vec<Condition>,
}

impl EventEdge {
    /// Expand this edge's conditions into explicit [`TickExecutionPlans`],
    /// same semantics as [`TickEventDispatch::execution_plans`] /
    /// `ChainEventDispatch::execution_plans`.
    pub fn execution_plans(&self) -> TickExecutionPlans {
        if self.when.is_empty() && self.unless.is_empty() {
            return TickExecutionPlans::Unconditional;
        }
        let mut combined = if self.when.is_empty() {
            Condition::all([])
        } else {
            Condition::all(self.when.clone())
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
                    when,
                    unless,
                } if p == parent => Some(EventEdge {
                    parent: parent.to_string(),
                    child: n.type_name.to_string(),
                    when: when.clone(),
                    unless: unless.clone(),
                }),
                _ => None,
            })
            .collect();
        out.sort_by(|a, b| a.child.cmp(&b.child));
        out
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

            NodeOrigin::Chained {
                parent: parent_type_name.to_string(),
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
                when,
                unless,
            },
            NormalizedEventDispatch::Chain(c),
        ) => parent.as_str() == (c.parent_type_name)() && when == &c.when && unless == &c.unless,
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
    use crate::events::{ChainEventDispatch, SandEvent, SandEventDispatch};

    struct A;
    struct B;
    struct C;

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
}
