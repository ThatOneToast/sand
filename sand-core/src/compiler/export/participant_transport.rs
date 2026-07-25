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
//! This is deliberately conservative rather than clever: `.within(...)`
//! bounded correlation, advancement bridges reached transitively, and
//! transitive inherit-of-inherit chains are all rejected outright here
//! rather than guessed at.
//!
//! # `after_any`/`after_all` multi-parent composition (#271)
//!
//! Because [`EventParticipantPlan::inherit_entity`]/[`inherit_item`](crate::participant::EventParticipantPlan::inherit_item)
//! always name one concrete `Source` type (never an inferred "whichever
//! parent fired"), a same-cycle child reached through `after_any`/
//! `after_all` fan-in can validly inherit from *any one* of its listed
//! occurrence parents, directly, with no ambiguity to resolve:
//!
//! - The reference generated for an inherited role addresses `Source`'s own
//!   generated tag/storage key by *type identity*, never by position in the
//!   parent list — `after_any::<(A, B)>()` and `after_any::<(B, A)>()`
//!   resolve to the exact same graph shape (`EventGraph::discover` sorts
//!   every occurrence-dependency parent list by canonical type name before
//!   this validator ever runs), so registration order can never affect
//!   which binding is selected or what is generated.
//! - `Source`'s own setup/cleanup lifecycle is entirely unaffected by being
//!   composed into an `after_any`/`after_all` group — it still only runs
//!   (and only produces a live tag) exactly when `Source` itself fires this
//!   tick, same as the sole-parent case. If the named `Source` is one
//!   `after_any` alternative among several and a *different* alternative
//!   supplied this tick's occurrence instead, `Source` did not fire, so its
//!   tag is absent and the generated selector legitimately matches nothing
//!   — this is "does not apply this tick", never a wrong/stale entity: the
//!   coordinator defers every occurrence-marked parent's cleanup until
//!   after all of its synchronous descendants (including every staged
//!   `after_any`/`after_all` child) have run (see `pipeline.rs`'s
//!   `deferred_root_post_observation`/`deferred_post_refs`), so a `Source`
//!   that *did* fire this tick keeps a valid tag for the child's entire
//!   execution regardless of how many sibling parents are in its group.
//! - For `after_all`, every listed parent is guaranteed to have fired
//!   before the child ever dispatches (that is what `after_all` means), so
//!   naming any one of them is exactly as sound as the sole-parent case —
//!   no gating is even needed.
//! - Only a *direct* one-hop membership check is performed here — an
//!   `after_any`/`after_all` boundary is never walked past to reach a
//!   grandparent, matching the existing "transitive inheritance is not
//!   supported" rule for the sole-parent case.
//! - Two different `after_all`/`after_any` parents may both directly
//!   declare the same role without conflict, as long as the child names
//!   exactly one of them: the other parent's declaration is simply
//!   irrelevant to this child's plan. A single plan can never declare two
//!   competing bindings for the same role at all — [`EventParticipantPlan::validate`]
//!   already rejects a duplicate role within one plan
//!   ([`DuplicateParticipantRole`]) regardless of how many distinct sources
//!   would otherwise be reachable, which is what actually prevents a
//!   silent arbitrary pick when two parents could both plausibly supply a
//!   role.
//!
//! This module is the *only*
//! parallel capability-bookkeeping mechanism
//! (`EventContextCapabilities::for_event_with_participants`,
//! `capabilities::full`) computed a similar-looking "could honestly
//! promise" value with zero export-pipeline call sites and was removed by
//! #274; see `sand-core/src/participant/capabilities.rs`'s module doc and
//! `docs/testing/participant-role-evidence.md` for that history. Do not
//! reintroduce a second, Rust-level-only propagation mechanism here — this
//! validator against the real [`EventGraph`] is the one source of truth.

use std::collections::{BTreeMap, BTreeSet};

use crate::events::TickWindow;
use crate::events::graph::{EventGraph, NodeOrigin, OccurrenceDependency};
use crate::participant::plan::EventParticipantPlan;
use crate::participant::role::{EntityParticipantRole, ItemParticipantRole, ParticipantHand};

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
    /// `(role, hand)` pairs directly captured by this event's own plan —
    /// #272's bounded-item-transport validator needs the hand, not just the
    /// role, since [`EventParticipantPlan::inherit_item_within`] names both.
    direct_item_hands: Vec<(ItemParticipantRole, ParticipantHand)>,
    /// `(role, source_event, hand, window)` bounded item declarations this
    /// event made via [`EventParticipantPlan::inherit_item_within`] (#272).
    pub(crate) bounded_item_roles: Vec<(
        ItemParticipantRole,
        &'static str,
        ParticipantHand,
        TickWindow,
    )>,
}

impl ParticipantDeclarations {
    pub(crate) fn from_plan(plan: &EventParticipantPlan) -> Self {
        Self {
            direct_entity_roles: plan.direct_entity_roles(),
            inherited_entity_roles: plan.inherited_entity_roles(),
            direct_item_roles: plan.direct_item_roles(),
            inherited_item_roles: plan.inherited_item_roles(),
            direct_item_hands: plan.direct_item_hands(),
            bounded_item_roles: plan.bounded_item_roles(),
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
                "`{current}` is not a plain same-cycle graph node and is not `{source}` either — \
                 if it is an advancement-bridge parent (#240 Phase 6), its own plan can only supply \
                 a role to its *direct* bridge children, not transitively beyond it (#269)"
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
                        // An advancement-bridge parent (#240 Phase 6) is not
                        // a plain graph node — `graph.nodes.get(current)`
                        // above never returns one — but as of #269 its own
                        // `participants()` plan *is* applied directly around
                        // its synthesized bridge entry, so it can be a valid
                        // inherit source. Walking through it here (rather
                        // than rejecting) lets the loop's next iteration
                        // either terminate at `source == parent.type_name`
                        // or continue past it if `parent` is itself not the
                        // requested source.
                        current = parent.type_name;
                    }
                    // #271: a direct, one-hop membership check — `source`
                    // must be one of the group's own listed parents, never
                    // a grandparent reached transitively through one of
                    // them (see this module's doc for why naming one
                    // specific member is always sound, regardless of which
                    // sibling alternative actually supplied a given tick's
                    // occurrence).
                    [OccurrenceDependency::AfterAny(parents)]
                        if parents.iter().any(|parent| parent.type_name == source) =>
                    {
                        return Ok(());
                    }
                    [OccurrenceDependency::AfterAll(parents)]
                        if parents.iter().any(|parent| parent.type_name == source) =>
                    {
                        return Ok(());
                    }
                    [OccurrenceDependency::AfterAny(parents)] => {
                        return Err(format!(
                            "`{current}` is reached through `after_any` fan-in over [{}], and `{source}` is not one of those listed parents — name one of them directly (no further ancestry is walked past an after_any/after_all boundary)",
                            parents
                                .iter()
                                .map(|parent| parent.type_name)
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                    [OccurrenceDependency::AfterAll(parents)] => {
                        return Err(format!(
                            "`{current}` is reached through `after_all` fan-in over [{}], and `{source}` is not one of those listed parents — name one of them directly (no further ancestry is walked past an after_any/after_all boundary)",
                            parents
                                .iter()
                                .map(|parent| parent.type_name)
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
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
                            "`{current}` declares multiple simultaneous occurrence clauses — same-cycle borrowing requires exactly one `.after(...)`, one `after_any` group, or one `after_all` group"
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

// ── Bounded item transport (#272) ───────────────────────────────────────────
//
// `inherit_item_within` is deliberately validated separately from
// `validate_one` above rather than folded into it: same-cycle borrowing
// (`inherit_entity`/`inherit_item`) and bounded copying
// (`inherit_item_within`) have opposite soundness conditions — the former
// requires *not* crossing a `.within(...)` edge
// (`find_borrowable_ancestor_path` rejects exactly that), the latter
// requires the child to be reached through *exactly* that kind of edge, with
// a matching window. Reusing one validator for both would mean threading a
// "but accept bounded edges this time" flag through
// `find_borrowable_ancestor_path`'s same-cycle-only walk, which is more
// confusing than two small, single-purpose functions.

/// One validated bounded item transport declaration: `child_event` reads
/// `source_event`'s own direct `(role, hand)` capture through a genuine
/// `.within::<source_event>(window)` bounded dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoundedItemTransport {
    pub(crate) source_event: &'static str,
    pub(crate) role: ItemParticipantRole,
    pub(crate) hand: ParticipantHand,
    pub(crate) window: TickWindow,
}

/// Validate every [`EventParticipantPlan::inherit_item_within`] declaration
/// recorded in `declarations` against `graph`'s actual resolved shape.
/// Returns the validated transports (deterministic child-then-declaration
/// order) so the caller can generate persist/expire codegen from them,
/// or the first violation found.
pub(crate) fn validate_bounded_item_transport(
    graph: &EventGraph,
    declarations: &BTreeMap<&'static str, ParticipantDeclarations>,
) -> Result<Vec<BoundedItemTransport>, ParticipantTransportDiagnostic> {
    let mut transports = Vec::new();
    for (child_event, decl) in declarations {
        for &(role, source_event, hand, window) in &decl.bounded_item_roles {
            let Some(source_decl) = declarations.get(source_event) else {
                return Err(diagnostic(
                    child_event,
                    source_event,
                    ParticipantTransportKind::Item,
                    role,
                    format!("`{source_event}` declares no participant plan at all"),
                    "inherit from the event whose own participants() plan actually captures this role/hand directly".to_string(),
                ));
            };
            if !source_decl.direct_item_hands.contains(&(role, hand)) {
                let reason = if source_decl.direct_item_roles.contains(&role) {
                    format!(
                        "`{source_event}` captures this role from a different hand than `{hand:?}`"
                    )
                } else {
                    format!("`{source_event}` does not directly capture this role at all")
                };
                return Err(diagnostic(
                    child_event,
                    source_event,
                    ParticipantTransportKind::Item,
                    role,
                    reason,
                    "name the exact (role, hand) pair the source's own observe_held_item/observe_weapon declaration captures".to_string(),
                ));
            }

            let Some(node) = graph.nodes.get(*child_event) else {
                return Err(diagnostic(
                    child_event,
                    source_event,
                    ParticipantTransportKind::Item,
                    role,
                    format!("`{child_event}` is not a plain same-cycle graph node"),
                    "inherit_item_within is only valid on an ordinary chained SandEvent"
                        .to_string(),
                ));
            };
            let NodeOrigin::Chained { bounded, .. } = &node.origin else {
                return Err(diagnostic(
                    child_event,
                    source_event,
                    ParticipantTransportKind::Item,
                    role,
                    format!("`{child_event}` is a root event with no `.within(...)` dependency"),
                    "declare `.within::<Source>(window)` on this event's own dispatch()"
                        .to_string(),
                ));
            };
            let matched = bounded
                .iter()
                .any(|b| b.type_name == source_event && b.window == window);
            if !matched {
                return Err(diagnostic(
                    child_event,
                    source_event,
                    ParticipantTransportKind::Item,
                    role,
                    format!(
                        "`{child_event}` is not reached from `{source_event}` through a `.within::<{source_event}>({:?})` bounded dependency with this exact window",
                        window.ticks()
                    ),
                    "declare a `.within::<Source>(window)` dependency on this event's own dispatch() whose window matches the one passed to inherit_item_within exactly".to_string(),
                ));
            }

            transports.push(BoundedItemTransport {
                source_event,
                role,
                hand,
                window,
            });
        }
    }
    Ok(transports)
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
    fn advancement_bridge_parent_is_a_valid_direct_source() {
        // #269: a bridge parent's own plan is applied around its
        // synthesized entry, so a direct bridge child can inherit from it.
        let mut node = chained_after("Child", "Bridge");
        let NodeOrigin::Chained { occurrence, .. } = &mut node.origin else {
            unreachable!()
        };
        let OccurrenceDependency::After(parent) = &mut occurrence[0] else {
            unreachable!()
        };
        parent.is_advancement = true;
        let graph = graph_with(vec![node]);
        assert_eq!(
            find_borrowable_ancestor_path(&graph, "Child", "Bridge"),
            Ok(())
        );
    }

    #[test]
    fn advancement_bridge_parent_does_not_extend_the_chain_transitively() {
        // A bridge parent is never a plain graph node, so it cannot itself
        // have a same-cycle ancestor to keep walking toward.
        let mut node = chained_after("Child", "Bridge");
        let NodeOrigin::Chained { occurrence, .. } = &mut node.origin else {
            unreachable!()
        };
        let OccurrenceDependency::After(parent) = &mut occurrence[0] else {
            unreachable!()
        };
        parent.is_advancement = true;
        let graph = graph_with(vec![node]);
        let err = find_borrowable_ancestor_path(&graph, "Child", "Grandparent").unwrap_err();
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

    fn chained_after_any(name: &'static str, parents: &[&'static str]) -> EventNode {
        let mut sorted: Vec<&'static str> = parents.to_vec();
        sorted.sort_unstable();
        EventNode {
            type_id: TypeId::of::<()>(),
            type_name: name,
            origin: NodeOrigin::Chained {
                occurrence: vec![OccurrenceDependency::AfterAny(
                    sorted
                        .into_iter()
                        .map(|parent| OccurrenceParent {
                            type_id: TypeId::of::<()>(),
                            type_name: parent,
                            is_advancement: false,
                        })
                        .collect(),
                )],
                persistent: vec![],
                bounded: vec![],
                when: vec![],
                unless: vec![],
            },
            setup: EventSetup::none(),
            handlers: vec!["h"],
        }
    }

    fn chained_after_all(name: &'static str, parents: &[&'static str]) -> EventNode {
        let mut sorted: Vec<&'static str> = parents.to_vec();
        sorted.sort_unstable();
        EventNode {
            type_id: TypeId::of::<()>(),
            type_name: name,
            origin: NodeOrigin::Chained {
                occurrence: vec![OccurrenceDependency::AfterAll(
                    sorted
                        .into_iter()
                        .map(|parent| OccurrenceParent {
                            type_id: TypeId::of::<()>(),
                            type_name: parent,
                            is_advancement: false,
                        })
                        .collect(),
                )],
                persistent: vec![],
                bounded: vec![],
                when: vec![],
                unless: vec![],
            },
            setup: EventSetup::none(),
            handlers: vec!["h"],
        }
    }

    #[test]
    fn after_any_named_member_resolves_directly() {
        let graph = graph_with(vec![
            root_node("A"),
            root_node("B"),
            chained_after_any("Child", &["A", "B"]),
        ]);
        assert_eq!(find_borrowable_ancestor_path(&graph, "Child", "A"), Ok(()));
        assert_eq!(find_borrowable_ancestor_path(&graph, "Child", "B"), Ok(()));
    }

    #[test]
    fn after_any_non_member_is_rejected_with_actionable_message() {
        let graph = graph_with(vec![
            root_node("A"),
            root_node("B"),
            root_node("C"),
            chained_after_any("Child", &["A", "B"]),
        ]);
        let err = find_borrowable_ancestor_path(&graph, "Child", "C").unwrap_err();
        assert!(err.contains("after_any"), "{err}");
        assert!(err.contains("A, B"), "{err}");
        assert!(
            err.contains("`C` is not one of those listed parents"),
            "{err}"
        );
    }

    #[test]
    fn after_any_does_not_walk_past_the_group_transitively() {
        // Grandparent is A's own parent, not a direct after_any member —
        // only a one-hop membership check is performed.
        let graph = graph_with(vec![
            root_node("Grandparent"),
            chained_after("A", "Grandparent"),
            root_node("B"),
            chained_after_any("Child", &["A", "B"]),
        ]);
        assert!(find_borrowable_ancestor_path(&graph, "Child", "Grandparent").is_err());
    }

    #[test]
    fn after_all_named_member_resolves_directly() {
        let graph = graph_with(vec![
            root_node("A"),
            root_node("B"),
            chained_after_all("Child", &["A", "B"]),
        ]);
        assert_eq!(find_borrowable_ancestor_path(&graph, "Child", "A"), Ok(()));
        assert_eq!(find_borrowable_ancestor_path(&graph, "Child", "B"), Ok(()));
    }

    #[test]
    fn after_all_non_member_is_rejected_with_actionable_message() {
        let graph = graph_with(vec![
            root_node("A"),
            root_node("B"),
            root_node("C"),
            chained_after_all("Child", &["A", "B"]),
        ]);
        let err = find_borrowable_ancestor_path(&graph, "Child", "C").unwrap_err();
        assert!(err.contains("after_all"), "{err}");
        assert!(err.contains("A, B"), "{err}");
        assert!(
            err.contains("`C` is not one of those listed parents"),
            "{err}"
        );
    }

    #[test]
    fn after_any_membership_check_is_independent_of_declared_parent_order() {
        // The graph builder canonically sorts occurrence parents by type
        // name (see `EventGraph::discover`), so a node built with parents
        // given in either order produces an identical resolved shape —
        // this test constructs both orders directly (bypassing `discover`)
        // to prove the validator itself doesn't care about order either.
        let forward = graph_with(vec![
            root_node("A"),
            root_node("B"),
            chained_after_any("Child", &["A", "B"]),
        ]);
        let backward = graph_with(vec![
            root_node("A"),
            root_node("B"),
            chained_after_any("Child", &["B", "A"]),
        ]);
        assert_eq!(
            find_borrowable_ancestor_path(&forward, "Child", "A"),
            find_borrowable_ancestor_path(&backward, "Child", "A"),
        );
        assert_eq!(
            find_borrowable_ancestor_path(&forward, "Child", "B"),
            find_borrowable_ancestor_path(&backward, "Child", "B"),
        );
    }

    #[test]
    fn validate_accepts_named_inherit_through_after_any() {
        let graph = graph_with(vec![
            root_node("A"),
            root_node("B"),
            chained_after_any("Child", &["A", "B"]),
        ]);
        let mut declarations = BTreeMap::new();
        declarations.insert(
            "A",
            ParticipantDeclarations {
                direct_entity_roles: vec![EntityParticipantRole::Killer],
                ..Default::default()
            },
        );
        declarations.insert(
            "Child",
            ParticipantDeclarations {
                inherited_entity_roles: vec![(EntityParticipantRole::Killer, "A")],
                ..Default::default()
            },
        );
        assert_eq!(
            validate_participant_transport(&graph, &declarations),
            Ok(())
        );
    }

    #[test]
    fn validate_accepts_two_after_all_parents_supplying_distinct_roles() {
        let graph = graph_with(vec![
            root_node("A"),
            root_node("B"),
            chained_after_all("Child", &["A", "B"]),
        ]);
        let mut declarations = BTreeMap::new();
        declarations.insert(
            "A",
            ParticipantDeclarations {
                direct_entity_roles: vec![EntityParticipantRole::Killer],
                ..Default::default()
            },
        );
        declarations.insert(
            "B",
            ParticipantDeclarations {
                direct_entity_roles: vec![EntityParticipantRole::Victim],
                ..Default::default()
            },
        );
        declarations.insert(
            "Child",
            ParticipantDeclarations {
                inherited_entity_roles: vec![
                    (EntityParticipantRole::Killer, "A"),
                    (EntityParticipantRole::Victim, "B"),
                ],
                ..Default::default()
            },
        );
        assert_eq!(
            validate_participant_transport(&graph, &declarations),
            Ok(())
        );
    }

    #[test]
    fn validate_accepts_naming_one_of_two_parents_that_both_declare_the_same_role() {
        // A second after_all parent independently declaring the identical
        // role does not block the child's own explicit, unambiguous choice
        // of source — this is the "compatible" case, not a conflict, since
        // the child only ever names one concrete source.
        let graph = graph_with(vec![
            root_node("A"),
            root_node("B"),
            chained_after_all("Child", &["A", "B"]),
        ]);
        let mut declarations = BTreeMap::new();
        declarations.insert(
            "A",
            ParticipantDeclarations {
                direct_entity_roles: vec![EntityParticipantRole::Killer],
                ..Default::default()
            },
        );
        declarations.insert(
            "B",
            ParticipantDeclarations {
                direct_entity_roles: vec![EntityParticipantRole::Killer],
                ..Default::default()
            },
        );
        declarations.insert(
            "Child",
            ParticipantDeclarations {
                inherited_entity_roles: vec![(EntityParticipantRole::Killer, "A")],
                ..Default::default()
            },
        );
        assert_eq!(
            validate_participant_transport(&graph, &declarations),
            Ok(())
        );
    }

    #[test]
    fn validate_rejects_after_any_inherit_from_a_non_member() {
        let graph = graph_with(vec![
            root_node("A"),
            root_node("B"),
            root_node("C"),
            chained_after_any("Child", &["A", "B"]),
        ]);
        let mut declarations = BTreeMap::new();
        declarations.insert(
            "C",
            ParticipantDeclarations {
                direct_entity_roles: vec![EntityParticipantRole::Killer],
                ..Default::default()
            },
        );
        declarations.insert(
            "Child",
            ParticipantDeclarations {
                inherited_entity_roles: vec![(EntityParticipantRole::Killer, "C")],
                ..Default::default()
            },
        );
        let err = validate_participant_transport(&graph, &declarations).unwrap_err();
        assert!(err.reason.contains("after_any"), "{}", err.reason);
    }

    fn chained_within(
        name: &'static str,
        parent: &'static str,
        window: crate::events::TickWindow,
    ) -> EventNode {
        use crate::events::graph::BoundedDependency;
        EventNode {
            type_id: TypeId::of::<()>(),
            type_name: name,
            origin: NodeOrigin::Chained {
                occurrence: vec![],
                persistent: vec![],
                bounded: vec![BoundedDependency {
                    type_id: TypeId::of::<()>(),
                    type_name: parent,
                    window,
                    condition: crate::condition::Condition::raw("dummy"),
                }],
                when: vec![],
                unless: vec![],
            },
            setup: EventSetup::none(),
            handlers: vec!["h"],
        }
    }

    #[test]
    fn bounded_item_transport_accepts_matching_source_and_window() {
        let window = crate::events::TickWindow::new(20).unwrap();
        let graph = graph_with(vec![
            root_node("Source"),
            chained_within("Child", "Source", window),
        ]);
        let mut declarations = BTreeMap::new();
        declarations.insert(
            "Source",
            ParticipantDeclarations {
                direct_item_hands: vec![(ItemParticipantRole::Weapon, ParticipantHand::MainHand)],
                direct_item_roles: vec![ItemParticipantRole::Weapon],
                ..Default::default()
            },
        );
        declarations.insert(
            "Child",
            ParticipantDeclarations {
                bounded_item_roles: vec![(
                    ItemParticipantRole::Weapon,
                    "Source",
                    ParticipantHand::MainHand,
                    window,
                )],
                ..Default::default()
            },
        );
        let transports = validate_bounded_item_transport(&graph, &declarations).unwrap();
        assert_eq!(transports.len(), 1);
        assert_eq!(transports[0].source_event, "Source");
        assert_eq!(transports[0].role, ItemParticipantRole::Weapon);
        assert_eq!(transports[0].hand, ParticipantHand::MainHand);
        assert_eq!(transports[0].window, window);
    }

    #[test]
    fn bounded_item_transport_rejects_source_that_does_not_capture_directly() {
        let window = crate::events::TickWindow::new(20).unwrap();
        let graph = graph_with(vec![
            root_node("Source"),
            chained_within("Child", "Source", window),
        ]);
        let mut declarations = BTreeMap::new();
        declarations.insert("Source", ParticipantDeclarations::default());
        declarations.insert(
            "Child",
            ParticipantDeclarations {
                bounded_item_roles: vec![(
                    ItemParticipantRole::Weapon,
                    "Source",
                    ParticipantHand::MainHand,
                    window,
                )],
                ..Default::default()
            },
        );
        let err = validate_bounded_item_transport(&graph, &declarations).unwrap_err();
        assert!(
            err.reason.contains("does not directly capture"),
            "{}",
            err.reason
        );
    }

    #[test]
    fn bounded_item_transport_rejects_mismatched_hand() {
        let window = crate::events::TickWindow::new(20).unwrap();
        let graph = graph_with(vec![
            root_node("Source"),
            chained_within("Child", "Source", window),
        ]);
        let mut declarations = BTreeMap::new();
        declarations.insert(
            "Source",
            ParticipantDeclarations {
                direct_item_hands: vec![(ItemParticipantRole::Weapon, ParticipantHand::OffHand)],
                direct_item_roles: vec![ItemParticipantRole::Weapon],
                ..Default::default()
            },
        );
        declarations.insert(
            "Child",
            ParticipantDeclarations {
                bounded_item_roles: vec![(
                    ItemParticipantRole::Weapon,
                    "Source",
                    ParticipantHand::MainHand,
                    window,
                )],
                ..Default::default()
            },
        );
        let err = validate_bounded_item_transport(&graph, &declarations).unwrap_err();
        assert!(err.reason.contains("different hand"), "{}", err.reason);
    }

    #[test]
    fn bounded_item_transport_rejects_mismatched_window() {
        let declared_window = crate::events::TickWindow::new(20).unwrap();
        let graph_window = crate::events::TickWindow::new(40).unwrap();
        let graph = graph_with(vec![
            root_node("Source"),
            chained_within("Child", "Source", graph_window),
        ]);
        let mut declarations = BTreeMap::new();
        declarations.insert(
            "Source",
            ParticipantDeclarations {
                direct_item_hands: vec![(ItemParticipantRole::Weapon, ParticipantHand::MainHand)],
                direct_item_roles: vec![ItemParticipantRole::Weapon],
                ..Default::default()
            },
        );
        declarations.insert(
            "Child",
            ParticipantDeclarations {
                bounded_item_roles: vec![(
                    ItemParticipantRole::Weapon,
                    "Source",
                    ParticipantHand::MainHand,
                    declared_window,
                )],
                ..Default::default()
            },
        );
        let err = validate_bounded_item_transport(&graph, &declarations).unwrap_err();
        assert!(err.reason.contains("not reached"), "{}", err.reason);
    }

    #[test]
    fn bounded_item_transport_rejects_a_child_with_no_within_dependency_at_all() {
        let window = crate::events::TickWindow::new(20).unwrap();
        let graph = graph_with(vec![root_node("Source"), chained_after("Child", "Source")]);
        let mut declarations = BTreeMap::new();
        declarations.insert(
            "Source",
            ParticipantDeclarations {
                direct_item_hands: vec![(ItemParticipantRole::Weapon, ParticipantHand::MainHand)],
                direct_item_roles: vec![ItemParticipantRole::Weapon],
                ..Default::default()
            },
        );
        declarations.insert(
            "Child",
            ParticipantDeclarations {
                bounded_item_roles: vec![(
                    ItemParticipantRole::Weapon,
                    "Source",
                    ParticipantHand::MainHand,
                    window,
                )],
                ..Default::default()
            },
        );
        let err = validate_bounded_item_transport(&graph, &declarations).unwrap_err();
        assert!(err.reason.contains("not reached"), "{}", err.reason);
    }
}
