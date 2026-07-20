//! Export-time validation for cross-event participant propagation (#264).
//!
//! [`EventParticipantPlan::inherit_entity`]/[`inherit_item`](crate::participant::EventParticipantPlan::inherit_item)
//! let a same-cycle child event borrow a role its ancestor already
//! captured, with zero extra setup/cleanup commands — see their own docs
//! for exactly why that is sound *when the edge shape permits it*. This
//! module is the gate that makes it sound in every other case: it walks
//! the fully-resolved [`EventGraph`] and rejects, with an actionable
//! [`ParticipantTransportDiagnostic`], any inherited declaration whose
//! ancestor chain is not a genuine unbroken run of single-parent,
//! unbounded, non-advancement-bridge `.after(...)` edges reaching a source
//! event that directly captures the requested role.
//!
//! This is deliberately conservative rather than clever: `after_any`/
//! `after_all` fan-in, `.within(...)` bounded correlation, advancement
//! bridges, and transitive inherit-of-inherit chains are all rejected
//! outright here rather than guessed at — see the module doc on
//! `sand-core/src/participant/capabilities.rs` for why "capability
//! bookkeeping that doesn't match generated command access" is exactly the
//! defect this PR closes, not something to reproduce for a new mechanism.

use std::collections::{BTreeMap, BTreeSet};

use crate::events::graph::{EventGraph, NodeOrigin, OccurrenceDependency};
use crate::participant::plan::EventParticipantPlan;
use crate::participant::role::{EntityParticipantRole, ItemParticipantRole};

/// What a plan declared, stripped down to just the role/source-label pairs
/// [`validate_participant_transport`] needs — recorded once per graph node
/// while the export pipeline still has each node's owned
/// [`EventParticipantPlan`] in hand (the graph itself only stores rendered
/// [`crate::events::EventSetup`] commands, not the structured plan).
#[derive(Debug, Default, Clone)]
pub(crate) struct ParticipantDeclarations {
    direct_entity_roles: Vec<EntityParticipantRole>,
    inherited_entity_roles: Vec<(EntityParticipantRole, &'static str)>,
    direct_item_roles: Vec<ItemParticipantRole>,
    inherited_item_roles: Vec<(ItemParticipantRole, &'static str)>,
}

impl ParticipantDeclarations {
    pub(crate) fn from_plan(plan: &EventParticipantPlan) -> Self {
        Self {
            direct_entity_roles: plan.direct_entity_roles(),
            inherited_entity_roles: plan.inherited_entity_roles(),
            direct_item_roles: plan.direct_item_roles(),
            inherited_item_roles: plan.inherited_item_roles(),
        }
    }
}

/// The kind of participant a transport diagnostic concerns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipantTransportKind {
    Entity,
    Item,
}

impl std::fmt::Display for ParticipantTransportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Entity => write!(f, "entity"),
            Self::Item => write!(f, "item"),
        }
    }
}

/// An actionable diagnostic for an `inherit_entity`/`inherit_item`
/// declaration the export pipeline could not validate — see the module doc
/// for the full list of rejection reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantTransportDiagnostic {
    pub child_event: String,
    pub source_event: String,
    pub kind: ParticipantTransportKind,
    pub role: String,
    pub reason: String,
    pub suggestion: String,
}

impl std::fmt::Display for ParticipantTransportDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cannot propagate {} participant `{}` from `{}` to `{}`: {} {}",
            self.kind, self.role, self.source_event, self.child_event, self.reason, self.suggestion
        )
    }
}

impl std::error::Error for ParticipantTransportDiagnostic {}

fn diagnostic(
    child_event: &str,
    source_event: &str,
    kind: ParticipantTransportKind,
    role: impl std::fmt::Debug,
    reason: impl Into<String>,
    suggestion: impl Into<String>,
) -> ParticipantTransportDiagnostic {
    ParticipantTransportDiagnostic {
        child_event: child_event.to_string(),
        source_event: source_event.to_string(),
        kind,
        role: format!("{role:?}"),
        reason: reason.into(),
        suggestion: suggestion.into(),
    }
}

/// Walk `graph` from `from` up its same-cycle occurrence ancestry looking
/// for `source` through an unbroken run of single-parent, unbounded,
/// non-advancement-bridge `.after(...)` edges. `Ok(())` if found;
/// otherwise a human-readable reason naming exactly which edge broke the
/// chain and why.
fn find_borrowable_ancestor_path(
    graph: &EventGraph,
    from: &str,
    source: &str,
) -> Result<(), String> {
    let mut current = from;
    let mut visited: BTreeSet<&str> = BTreeSet::new();
    loop {
        if current == source {
            return Ok(());
        }
        if !visited.insert(current) {
            // The graph's own cycle validation already runs before this
            // check (see `EventGraph::validate_dependencies`); this is an
            // unreachable defensive fallback, not a case real input reaches.
            return Err(format!(
                "same-cycle ancestry from `{from}` revisits `{current}` without reaching `{source}`"
            ));
        }
        let Some(node) = graph.nodes.get(current) else {
            return Err(format!(
                "`{current}` is not a plain same-cycle graph node (it may be an advancement-bridge parent, which cannot own a participant plan — see #240 Phase 6)"
            ));
        };
        match &node.origin {
            NodeOrigin::Root(_) => {
                return Err(format!(
                    "`{current}` is an independently-detected root event with no same-cycle parent — the ancestry ends here without reaching `{source}`"
                ));
            }
            NodeOrigin::Chained {
                occurrence,
                bounded,
                persistent,
                ..
            } => {
                if !bounded.is_empty() {
                    return Err(format!(
                        "`{current}` reaches its parent through a bounded `.within(...)` window, which cannot carry a same-cycle borrowed entity/live reference across a tick boundary"
                    ));
                }
                match occurrence.as_slice() {
                    [OccurrenceDependency::After(parent)] => {
                        if parent.is_advancement {
                            return Err(format!(
                                "`{current}`'s parent `{}` is an advancement-bridge parent, which cannot own a participant plan (#240 Phase 6 requires an empty EventSetup for bridge parents)",
                                parent.type_name
                            ));
                        }
                        current = parent.type_name;
                    }
                    [] if !persistent.is_empty() => {
                        return Err(format!(
                            "`{current}` reaches its parent only through a persistent `.while_(...)` condition, which contributes truth, not occurrence-scoped participant state"
                        ));
                    }
                    [] => {
                        return Err(format!(
                            "`{current}` has no same-cycle occurrence parent at all"
                        ));
                    }
                    _ => {
                        return Err(format!(
                            "`{current}` is reached through `after_any`/`after_all`/multiple occurrence clauses — #264 does not propagate a same-cycle borrow through multi-parent fan-in (see the after_any/after_all diagnostics instead)"
                        ));
                    }
                }
            }
        }
    }
}

/// Validate every inherited participant declaration recorded in
/// `declarations` against `graph`'s actual resolved shape. Returns the
/// first violation found, in deterministic (event name, role) order, so
/// export failures are reproducible rather than order-dependent.
pub(crate) fn validate_participant_transport(
    graph: &EventGraph,
    declarations: &BTreeMap<&'static str, ParticipantDeclarations>,
) -> Result<(), ParticipantTransportDiagnostic> {
    for (child_event, decl) in declarations {
        for (role, source_event) in &decl.inherited_entity_roles {
            validate_one(
                graph,
                declarations,
                child_event,
                source_event,
                ParticipantTransportKind::Entity,
                *role,
                |d| d.direct_entity_roles.contains(role),
                |d| d.inherited_entity_roles.iter().any(|(r, _)| r == role),
            )?;
        }
        for (role, source_event) in &decl.inherited_item_roles {
            validate_one(
                graph,
                declarations,
                child_event,
                source_event,
                ParticipantTransportKind::Item,
                *role,
                |d| d.direct_item_roles.contains(role),
                |d| d.inherited_item_roles.iter().any(|(r, _)| r == role),
            )?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_one(
    graph: &EventGraph,
    declarations: &BTreeMap<&'static str, ParticipantDeclarations>,
    child_event: &str,
    source_event: &str,
    kind: ParticipantTransportKind,
    role: impl std::fmt::Debug + Copy,
    source_declares_directly: impl Fn(&ParticipantDeclarations) -> bool,
    source_declares_inherited: impl Fn(&ParticipantDeclarations) -> bool,
) -> Result<(), ParticipantTransportDiagnostic> {
    let Some(source_decl) = declarations.get(source_event) else {
        return Err(diagnostic(
            child_event,
            source_event,
            kind,
            role,
            format!("`{source_event}` declares no participant plan at all"),
            "inherit from the event whose own participants() plan actually captures this role"
                .to_string(),
        ));
    };
    if !source_declares_directly(source_decl) {
        let reason = if source_declares_inherited(source_decl) {
            format!(
                "`{source_event}` only inherits this role itself (transitive inheritance is not supported)"
            )
        } else {
            format!("`{source_event}` does not declare this role at all")
        };
        return Err(diagnostic(
            child_event,
            source_event,
            kind,
            role,
            reason,
            "name the actual capturing ancestor directly in inherit_entity::<...>/inherit_item::<...>, not an intermediate event that only re-borrows it".to_string(),
        ));
    }
    if let Err(reason) = find_borrowable_ancestor_path(graph, child_event, source_event) {
        return Err(diagnostic(
            child_event,
            source_event,
            kind,
            role,
            reason,
            "same-cycle borrowing is only sound through an unbroken chain of single-parent, unbounded `.after(...)`/chain edges — use a copied/bounded transport instead, or restructure the composition".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::graph::{EventNode, OccurrenceParent};
    use crate::events::{EventSetup, TickEventDispatch};
    use std::any::TypeId;

    fn root_node(name: &'static str) -> EventNode {
        EventNode {
            type_id: TypeId::of::<()>(),
            type_name: name,
            origin: NodeOrigin::Root(TickEventDispatch::default()),
            setup: EventSetup::none(),
            handlers: vec!["h"],
        }
    }

    fn chained_after(name: &'static str, parent: &'static str) -> EventNode {
        EventNode {
            type_id: TypeId::of::<()>(),
            type_name: name,
            origin: NodeOrigin::Chained {
                occurrence: vec![OccurrenceDependency::After(OccurrenceParent {
                    type_id: TypeId::of::<()>(),
                    type_name: parent,
                    is_advancement: false,
                })],
                persistent: vec![],
                bounded: vec![],
                when: vec![],
                unless: vec![],
            },
            setup: EventSetup::none(),
            handlers: vec!["h"],
        }
    }

    fn graph_with(nodes: Vec<EventNode>) -> EventGraph {
        EventGraph {
            nodes: nodes
                .into_iter()
                .map(|n| (n.type_name.to_string(), n))
                .collect(),
            advancement_bridges: Default::default(),
        }
    }

    #[test]
    fn direct_parent_chain_resolves() {
        let graph = graph_with(vec![root_node("Root"), chained_after("Child", "Root")]);
        assert_eq!(
            find_borrowable_ancestor_path(&graph, "Child", "Root"),
            Ok(())
        );
    }

    #[test]
    fn grandparent_chain_resolves_through_two_hops() {
        let graph = graph_with(vec![
            root_node("Root"),
            chained_after("Mid", "Root"),
            chained_after("Grandchild", "Mid"),
        ]);
        assert_eq!(
            find_borrowable_ancestor_path(&graph, "Grandchild", "Root"),
            Ok(())
        );
    }

    #[test]
    fn multi_parent_edge_breaks_the_chain() {
        let mut node = chained_after("Child", "A");
        let NodeOrigin::Chained { occurrence, .. } = &mut node.origin else {
            unreachable!()
        };
        occurrence.push(OccurrenceDependency::After(OccurrenceParent {
            type_id: TypeId::of::<()>(),
            type_name: "B",
            is_advancement: false,
        }));
        let graph = graph_with(vec![root_node("A"), root_node("B"), node]);
        assert!(find_borrowable_ancestor_path(&graph, "Child", "A").is_err());
    }

    #[test]
    fn bounded_edge_breaks_the_chain() {
        use crate::events::graph::BoundedDependency;
        let mut node = chained_after("Child", "Root");
        let NodeOrigin::Chained {
            occurrence,
            bounded,
            ..
        } = &mut node.origin
        else {
            unreachable!()
        };
        occurrence.clear();
        bounded.push(BoundedDependency {
            type_id: TypeId::of::<()>(),
            type_name: "Root",
            window: crate::events::TickWindow::new(40).unwrap(),
            condition: crate::condition::Condition::raw("dummy"),
        });
        let graph = graph_with(vec![root_node("Root"), node]);
        assert!(find_borrowable_ancestor_path(&graph, "Child", "Root").is_err());
    }

    #[test]
    fn advancement_bridge_parent_breaks_the_chain() {
        let mut node = chained_after("Child", "Bridge");
        let NodeOrigin::Chained { occurrence, .. } = &mut node.origin else {
            unreachable!()
        };
        let OccurrenceDependency::After(parent) = &mut occurrence[0] else {
            unreachable!()
        };
        parent.is_advancement = true;
        let graph = graph_with(vec![node]);
        let err = find_borrowable_ancestor_path(&graph, "Child", "Bridge").unwrap_err();
        assert!(err.contains("advancement-bridge"), "{err}");
    }

    #[test]
    fn validate_rejects_source_with_no_declarations() {
        let graph = graph_with(vec![root_node("Root"), chained_after("Child", "Root")]);
        let mut declarations = BTreeMap::new();
        declarations.insert(
            "Child",
            ParticipantDeclarations {
                inherited_entity_roles: vec![(EntityParticipantRole::Attacker, "Root")],
                ..Default::default()
            },
        );
        let err = validate_participant_transport(&graph, &declarations).unwrap_err();
        assert_eq!(err.source_event, "Root");
        assert!(err.reason.contains("no participant plan"), "{}", err.reason);
    }

    #[test]
    fn validate_rejects_transitive_inheritance() {
        let graph = graph_with(vec![
            root_node("Root"),
            chained_after("Mid", "Root"),
            chained_after("Grandchild", "Mid"),
        ]);
        let mut declarations = BTreeMap::new();
        declarations.insert(
            "Mid",
            ParticipantDeclarations {
                inherited_entity_roles: vec![(EntityParticipantRole::Attacker, "Root")],
                ..Default::default()
            },
        );
        declarations.insert(
            "Grandchild",
            ParticipantDeclarations {
                inherited_entity_roles: vec![(EntityParticipantRole::Attacker, "Mid")],
                ..Default::default()
            },
        );
        let err = validate_participant_transport(&graph, &declarations).unwrap_err();
        assert!(err.reason.contains("transitive"), "{}", err.reason);
    }

    #[test]
    fn validate_accepts_direct_ancestor_capture() {
        let graph = graph_with(vec![root_node("Root"), chained_after("Child", "Root")]);
        let mut declarations = BTreeMap::new();
        declarations.insert(
            "Root",
            ParticipantDeclarations {
                direct_entity_roles: vec![EntityParticipantRole::Attacker],
                ..Default::default()
            },
        );
        declarations.insert(
            "Child",
            ParticipantDeclarations {
                inherited_entity_roles: vec![(EntityParticipantRole::Attacker, "Root")],
                ..Default::default()
            },
        );
        assert_eq!(
            validate_participant_transport(&graph, &declarations),
            Ok(())
        );
    }
}
