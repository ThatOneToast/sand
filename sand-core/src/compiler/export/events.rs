//! Event lowering phase of the export pipeline.
//!
//! Owns the generated-command builders for the SandEvent dependency graph
//! (dispatch/edge/staged-occurrence functions), the custom-`SandEvent`
//! dispatch backend resolution (#121), version-aware advancement trigger
//! validation, and the XP level-up observation command sequences.
#![allow(clippy::result_large_err)]

use crate::events::graph::tick_event_resource_key;

use super::ExportCtx;
use super::records::{ComponentRecord, ExportResult};
use crate::component::ComponentExportError;

pub(crate) fn xp_score_commands() -> Vec<String> {
    vec![
        "execute as @a store result score @s __sand_xp_lvl run experience query @s levels"
            .to_string(),
        "execute as @a unless score @s __sand_xp_seen matches 1 \
         run scoreboard players operation @s __sand_xp_prev = @s __sand_xp_lvl"
            .to_string(),
        "scoreboard players set @a __sand_xp_seen 1".to_string(),
        "execute as @a run scoreboard players operation @s __sand_xp_delta = @s __sand_xp_lvl"
            .to_string(),
        "execute as @a run scoreboard players operation @s __sand_xp_delta -= @s __sand_xp_prev"
            .to_string(),
    ]
}

pub(crate) fn xp_advance_command() -> String {
    "execute as @a run scoreboard players operation @s __sand_xp_prev = @s __sand_xp_lvl"
        .to_string()
}

/// Human-readable description of which part of two `SandEvent` definitions
/// for the same event type differ, for the conflicting-descriptor export
/// error.
pub(crate) fn tick_event_export_error(message: impl Into<String>) -> ComponentExportError {
    ComponentExportError::ComponentValidation {
        location: sand_components::ResourceLocation::new("sand", "events")
            .expect("fixed events resource location is valid"),
        kind: "sand_event".to_string(),
        field: "dispatch".to_string(),
        message: message.into(),
    }
}

fn command_declares_objective(command: &str, objective: &str) -> bool {
    let mut parts = command.split_whitespace();
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some("scoreboard"), Some("objectives"), Some("add"), Some(name)) if name == objective
    )
}

pub(crate) fn setup_objective_owner<'a>(
    graph: &'a crate::events::graph::EventGraph,
    objective: &str,
) -> Option<&'a str> {
    graph.nodes.values().find_map(|node| {
        node.setup
            .objectives
            .iter()
            .any(|command| command_declares_objective(command, objective))
            .then_some(node.type_name)
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChildPostObservation {
    Inline,
    DeferredByOccurrence,
    DeferredByAttempt,
}

/// Build (or reuse the sole handler as) `name`'s dispatch function: direct
/// `#[event]` handler calls (sorted), then child edges (sorted by canonical
/// child name, recursing into each child's own dispatch function first so
/// nested references always resolve), inheriting the current `@s`/position —
/// never re-issuing `execute as @a`.
///
/// This function never contains lifecycle (`pre_observation`/
/// `post_observation`) commands, for either roots or chained nodes. Root
/// lifecycle stays in the tick-check wrapper (run unconditionally every tick
/// regardless of detection, established by the caller). A chained node's own
/// lifecycle is wrapped around *its* condition test by its parent — see
/// [`build_child_edge`] — so it always observes before testing and always
/// advances after testing, whether or not its condition matched.
///
/// The caller invokes this once for each root or staged child. Recursion only
/// follows immediate single-parent edges; multi-parent nodes are separate
/// staged roots, which prevents shared-parent traversal from duplicating a
/// detector or dispatch resource.
pub(crate) fn build_dispatch_function(
    name: &str,
    graph: &crate::events::graph::EventGraph,
    namespace: &str,
    self_guard: Option<&str>,
    occurrence_marked: &std::collections::BTreeSet<String>,
    guarded_children: &mut std::collections::BTreeSet<String>,
    records: &mut Vec<ComponentRecord>,
) -> String {
    let node = &graph.nodes[name];
    let key = tick_event_resource_key(node.type_name);
    let children = graph.children_of(name);

    let records_occurrence = occurrence_marked.contains(name);
    let needs_wrapper = node.handlers.len() != 1
        || !children.is_empty()
        || self_guard.is_some()
        || records_occurrence;

    if !needs_wrapper {
        return format!("{namespace}:{}", node.handlers[0]);
    }

    let mut cmds: Vec<String> = Vec::new();
    if records_occurrence {
        cmds.push(format!("scoreboard players set @s se_{key}_o 1"));
    }
    if let Some(guard) = self_guard {
        cmds.push(format!("scoreboard players set @s {guard} 1"));
    }
    for handler in &node.handlers {
        cmds.push(format!("function {namespace}:{handler}"));
    }
    for edge in &children {
        let child_node = &graph.nodes[&edge.child];
        let post_observation = if occurrence_marked.contains(&edge.child)
            && !child_node.setup.post_observation.is_empty()
        {
            ChildPostObservation::DeferredByAttempt
        } else {
            ChildPostObservation::Inline
        };
        cmds.push(build_child_edge(
            edge,
            graph,
            namespace,
            occurrence_marked,
            post_observation,
            guarded_children,
            records,
        ));
    }

    let dispatch_path = format!("__sand_event_dispatch/{key}");
    records.push(ComponentRecord {
        namespace: namespace.to_string(),
        dir: "function".to_string(),
        path: dispatch_path.clone(),
        ext: "mcfunction".to_string(),
        content_type: "text".to_string(),
        content: cmds.join("\n"),
    });
    format!("{namespace}:{dispatch_path}")
}

/// Build the call line a parent uses to reach one child edge, and — when the
/// child owns lifecycle commands — the dedicated `__sand_event_observe/<child>`
/// function that wraps its condition test between `pre_observation` and
/// `post_observation`.
///
/// The required per-invocation order for a chained child is always:
/// `pre_observation` → condition test → handler/descendant dispatch if
/// matched → `post_observation`, with `post_observation` reached whether or
/// not the condition matched (mirroring the tick-lifecycle contract for
/// roots). A child with no lifecycle commands has no such ordering concern,
/// so it keeps the direct (no-wrapper) call shape.
pub(crate) fn build_child_edge(
    edge: &crate::events::graph::EventEdge,
    graph: &crate::events::graph::EventGraph,
    namespace: &str,
    occurrence_marked: &std::collections::BTreeSet<String>,
    post_observation: ChildPostObservation,
    guarded_children: &mut std::collections::BTreeSet<String>,
    records: &mut Vec<ComponentRecord>,
) -> String {
    let child_node = &graph.nodes[&edge.child];
    let child_ref = build_dispatch_function(
        &edge.child,
        graph,
        namespace,
        None,
        occurrence_marked,
        guarded_children,
        records,
    );
    let has_lifecycle = !child_node.setup.pre_observation.is_empty()
        || !child_node.setup.post_observation.is_empty();

    // Build the condition-gated dispatch line(s) that reach `dispatch_ref`,
    // shared by both the no-lifecycle (emitted directly into the caller's
    // command list) and lifecycle (emitted into the observe function body)
    // shapes.
    let conditional_dispatch_lines = |dispatch_ref: &str,
                                      guarded_children: &mut std::collections::BTreeSet<String>,
                                      records: &mut Vec<ComponentRecord>|
     -> Vec<String> {
        match edge.execution_ir_plans() {
            crate::events::TickExecutionIrPlans::Unconditional => {
                vec![format!("function {dispatch_ref}")]
            }
            crate::events::TickExecutionIrPlans::Plans(plans) if plans.len() <= 1 => plans
                .into_iter()
                .next()
                .map(|plan| {
                    if plan.is_empty() {
                        format!("function {dispatch_ref}")
                    } else {
                        plan.into_iter()
                            .fold(sand_commands::Execute::new(), |execute, clause| {
                                execute.with_operation(clause.into_operation())
                            })
                            .run_fn(dispatch_ref)
                    }
                })
                .into_iter()
                .collect(),
            // else: an explicit `Any([])`-shaped edge condition can never
            // hold — no dead wiring emitted for an unreachable edge.
            crate::events::TickExecutionIrPlans::Plans(plans) => {
                // More than one OR-alternative plan means this child could be
                // reached more than once from the same parent invocation.
                // Coalesce via a per-child, per-player guard reset right
                // before evaluating the plans, mirroring the root
                // self-detection guard.
                guarded_children.insert(edge.child.clone());
                let child_key = tick_event_resource_key(&edge.child);
                let guard = format!("se_{child_key}_g");

                let edge_path = format!("__sand_event_edge/{child_key}");
                let edge_ref = format!("{namespace}:{edge_path}");
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: edge_path,
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: [
                        format!("scoreboard players set @s {guard} 1"),
                        format!("function {dispatch_ref}"),
                    ]
                    .join("\n"),
                });

                let mut lines = vec![format!("scoreboard players set @s {guard} 0")];
                for plan in &plans {
                    let execute = sand_commands::Execute::new().with_operation(
                        sand_commands::ExecuteOp::Unless(
                            sand_commands::ConditionIr::ScoreMatches {
                                holder: sand_commands::ScoreHolder::self_(),
                                objective: guard.clone(),
                                range: "1".to_string(),
                            },
                        ),
                    );
                    lines.push(
                        plan.iter()
                            .cloned()
                            .fold(execute, |execute, clause| {
                                execute.with_operation(clause.into_operation())
                            })
                            .run_fn(&edge_ref),
                    );
                }
                lines
            }
        }
    };

    if !has_lifecycle {
        // No ordering concern — the condition test (and any multi-plan
        // guard) can be emitted directly into the parent's own command list,
        // exactly as before.
        return conditional_dispatch_lines(&child_ref, guarded_children, records).join("\n");
    }

    // The child owns pre_observation/post_observation: wrap the condition
    // test in a dedicated observe function so post_observation is always
    // structurally reached after a condition attempt, matched or not — never
    // only inside a function reached solely on success.
    let child_key = tick_event_resource_key(&edge.child);
    let mut observe_cmds: Vec<String> = Vec::new();
    observe_cmds.extend(child_node.setup.pre_observation.iter().cloned());
    if post_observation == ChildPostObservation::DeferredByAttempt {
        observe_cmds.push(format!("scoreboard players set @s se_{child_key}_c 1"));
    }
    observe_cmds.extend(conditional_dispatch_lines(
        &child_ref,
        guarded_children,
        records,
    ));
    if post_observation == ChildPostObservation::Inline {
        observe_cmds.extend(child_node.setup.post_observation.iter().cloned());
    }

    let observe_path = format!("__sand_event_observe/{child_key}");
    records.push(ComponentRecord {
        namespace: namespace.to_string(),
        dir: "function".to_string(),
        path: observe_path.clone(),
        ext: "mcfunction".to_string(),
        content_type: "text".to_string(),
        content: observe_cmds.join("\n"),
    });
    format!("function {namespace}:{observe_path}")
}

pub(crate) fn build_staged_post_observation_line(
    staged: &crate::events::graph::StagedEvent,
    post_ref: &str,
) -> String {
    if staged.occurrence.iter().any(|dependency| {
        matches!(
            dependency,
            crate::events::graph::OccurrenceDependency::AfterAny(_)
        )
    }) {
        let child_key = tick_event_resource_key(&staged.child);
        return format!(
            "execute as @a at @s if score @s se_{child_key}_m matches 1 run function {post_ref}"
        );
    }

    let mut clauses = Vec::new();
    for dependency in &staged.occurrence {
        match dependency {
            crate::events::graph::OccurrenceDependency::After(parent) => {
                let key = tick_event_resource_key(parent.type_name);
                clauses.push(format!("if score @s se_{key}_o matches 1"));
            }
            crate::events::graph::OccurrenceDependency::AfterAll(parents) => {
                clauses.extend(parents.iter().map(|parent| {
                    let key = tick_event_resource_key(parent.type_name);
                    format!("if score @s se_{key}_o matches 1")
                }));
            }
            crate::events::graph::OccurrenceDependency::AfterAny(_) => unreachable!(),
        }
    }
    format!(
        "execute as @a at @s {} run function {post_ref}",
        clauses.join(" ")
    )
}

pub(crate) fn build_staged_occurrence_lines(
    staged: &crate::events::graph::StagedEvent,
    evaluation_ref: &str,
    namespace: &str,
    records: &mut Vec<ComponentRecord>,
) -> Vec<String> {
    let mut required = Vec::new();
    let mut any_parents = Vec::new();
    for dependency in &staged.occurrence {
        match dependency {
            crate::events::graph::OccurrenceDependency::After(parent) => {
                let key = tick_event_resource_key(parent.type_name);
                required.push(format!("if score @s se_{key}_o matches 1"));
            }
            crate::events::graph::OccurrenceDependency::AfterAll(parents) => {
                for parent in parents {
                    let key = tick_event_resource_key(parent.type_name);
                    required.push(format!("if score @s se_{key}_o matches 1"));
                }
            }
            crate::events::graph::OccurrenceDependency::AfterAny(parents) => {
                any_parents.extend(parents.iter());
            }
        }
    }

    if any_parents.is_empty() {
        return vec![format!(
            "execute as @a at @s {} run function {evaluation_ref}",
            required.join(" ")
        )];
    }

    let child_key = tick_event_resource_key(&staged.child);
    let guard = format!("se_{child_key}_m");
    let gate_path = format!("__sand_event_multi_gate/{child_key}");
    let gate_ref = format!("{namespace}:{gate_path}");
    records.push(ComponentRecord {
        namespace: namespace.to_string(),
        dir: "function".to_string(),
        path: gate_path,
        ext: "mcfunction".to_string(),
        content_type: "text".to_string(),
        content: [
            format!("scoreboard players set @s {guard} 1"),
            format!("function {evaluation_ref}"),
        ]
        .join("\n"),
    });

    any_parents
        .into_iter()
        .map(|parent| {
            let parent_key = tick_event_resource_key(parent.type_name);
            let mut clauses = vec![format!("unless score @s {guard} matches 1")];
            clauses.extend(required.clone());
            clauses.push(format!("if score @s se_{parent_key}_o matches 1"));
            format!(
                "execute as @a at @s {} run function {gate_ref}",
                clauses.join(" ")
            )
        })
        .collect()
}

/// The resolved dispatch backend for a custom [`crate::events::SandEvent`],
/// after enforcing that exactly one of `make_trigger()` / `make_condition()` /
/// `make_tick()` returned `Some` (see #121).
#[allow(clippy::large_enum_variant)]
pub(crate) enum CustomDispatchBackend {
    Advancement(crate::AdvancementTrigger),
    /// Legacy single-fragment `SandEventDispatch::TickCondition` string.
    ///
    /// Normalized into the same structured `TickEventDispatch` shape as
    /// [`TickLifecycle`](Self::TickLifecycle) (matching
    /// `SandEventDispatch::normalize()`) and fed into the same event graph
    /// discovery, so a legacy parent shares exactly one generated detector
    /// with any structured-`tick()` sibling handlers and same-cycle chain
    /// children — never a second, independent detector.
    TickPoll(String),
    /// Structured, typed tick dispatch with lifecycle/setup support.
    TickLifecycle(crate::events::TickEventDispatch),
    /// Structured, same-cycle chained dispatch (#240).
    Chain(crate::events::ChainEventDispatch),
    /// Reusable tracked-transition dispatch (#49) — shares the same
    /// generated provider backend as built-in `EventDispatch::Tracked`
    /// handlers, but reachable for arbitrary/generic `SandEvent` types.
    Tracked(crate::TrackedTransition),
}

/// Resolve which dispatch backend a custom `SandEvent` uses, enforcing the
/// documented `EventDispatch::Custom` contract: exactly one of `make_trigger()`
/// / `make_condition()` / `make_tick()` / `make_chain()` / `make_tracked()`
/// must return `Some`.
///
/// All five factories are evaluated by the caller *before* this function
/// runs, so this is a pure decision function — panicking here (rather than
/// returning a `Result`) matches the existing "both `None`" precedent: this is
/// a Rust-level authoring bug in the `SandEvent` impl, detected at
/// export/codegen time, not a runtime datapack-validity issue.
pub(crate) fn resolve_custom_dispatch_backend(
    trigger: Option<crate::AdvancementTrigger>,
    condition: Option<String>,
    tick: Option<crate::events::TickEventDispatch>,
    chain: Option<crate::events::ChainEventDispatch>,
    tracked: Option<crate::TrackedTransition>,
    handler_path: &str,
) -> CustomDispatchBackend {
    let some_count = [
        trigger.is_some(),
        condition.is_some(),
        tick.is_some(),
        chain.is_some(),
        tracked.is_some(),
    ]
    .iter()
    .filter(|b| **b)
    .count();
    match (trigger, condition, tick, chain, tracked, some_count) {
        (Some(trigger), None, None, None, None, 1) => CustomDispatchBackend::Advancement(trigger),
        (None, Some(condition), None, None, None, 1) => CustomDispatchBackend::TickPoll(condition),
        (None, None, Some(tick), None, None, 1) => CustomDispatchBackend::TickLifecycle(tick),
        (None, None, None, Some(chain), None, 1) => CustomDispatchBackend::Chain(chain),
        (None, None, None, None, Some(tracked), 1) => CustomDispatchBackend::Tracked(tracked),
        (_, _, _, _, _, 0) => {
            panic!(
                "Custom SandEvent for handler `{handler_path}` returned None from \
                 make_trigger(), make_condition(), make_tick(), make_chain(), and \
                 make_tracked() — implement exactly one dispatch strategy from \
                 SandEvent::dispatch()"
            );
        }
        _ => {
            panic!(
                "Custom SandEvent for handler `{handler_path}` returned more than one dispatch \
                 strategy (make_trigger/make_condition/make_tick/make_chain/make_tracked) — \
                 implement exactly one"
            );
        }
    }
}

/// Validate an advancement trigger for the target version, returning a
/// fallible error instead of panicking.
///
/// Delegates to the same target-aware validator used by ordinary advancement
/// export so event wrappers cannot drift from component behavior.
pub(crate) fn check_event_trigger(
    trigger: &crate::AdvancementTrigger,
    advancement_id: &str,
    handler_path: &str,
    ctx: Option<&ExportCtx>,
) -> ExportResult<()> {
    trigger
        .validate_for_caps(ctx.map(|context| context.caps))
        .map_err(
            |diagnostic| sand_components::error::SandError::ComponentValidation {
                location: advancement_id.parse().unwrap_or_else(|_| {
                    sand_components::ResourceLocation::new("sand", "error").unwrap()
                }),
                kind: "advancement_event".to_string(),
                field: "trigger".to_string(),
                message: format!("cannot export advancement event `{handler_path}`: {diagnostic}"),
            },
        )
}

/// Resolve the [`crate::version::VersionProfile`] a participant plan's
/// version gating (`Relation::check_supported`) should run against, from
/// the export's own already-resolved [`ExportCtx`].
///
/// `ctx: None` is the unprofiled compatibility export path
/// ([`crate::try_export_components`]) — resolved against
/// [`crate::version::LATEST_KNOWN`], the same permissive default
/// `check_event_trigger` effectively uses for that path (no caps to gate
/// against).
pub(crate) fn resolve_participant_profile(
    ctx: Option<&ExportCtx>,
) -> crate::version::VersionProfile {
    let requested = ctx
        .map(|context| context.requested_version)
        .unwrap_or(crate::version::LATEST_KNOWN);
    let version = crate::version::MinecraftVersion::parse(requested).unwrap_or_else(|_| {
        crate::version::MinecraftVersion::parse(crate::version::LATEST_KNOWN).unwrap()
    });
    crate::version::VersionProfile::resolve(&version).unwrap_or_else(|_| {
        crate::version::VersionProfile::resolve(
            &crate::version::MinecraftVersion::parse(crate::version::LATEST_KNOWN).unwrap(),
        )
        .expect("LATEST_KNOWN always resolves")
    })
}

/// Merge `plan`'s generated commands into `setup`'s pre/post-observation
/// (#230 automatic tick-dispatch integration) — a no-op returning `setup`
/// unchanged when `plan.is_empty()`. `event_label` must be the event type's
/// canonical name (`event_type_name()`), the same key
/// [`crate::event::Event::entity`] reconstructs from, so a handler body can
/// address what this merge bound.
pub(crate) fn apply_participants_to_setup(
    setup: crate::events::EventSetup,
    plan: crate::participant::EventParticipantPlan,
    event_label: &str,
    ctx: Option<&ExportCtx>,
    handler_path: &str,
) -> ExportResult<crate::events::EventSetup> {
    if plan.is_empty() {
        return Ok(setup);
    }
    let profile = resolve_participant_profile(ctx);
    let (setup_commands, cleanup_commands) = plan
        .build(event_label, &profile)
        .map_err(|err| participant_plan_export_error(handler_path, err))?;
    let mut setup = setup;
    setup.pre_observation.extend(setup_commands);
    setup.post_observation.extend(cleanup_commands);
    Ok(setup)
}

/// Map an [`crate::participant::EventParticipantPlanError`] into a fallible
/// export diagnostic naming the handler that declared the plan.
pub(crate) fn participant_plan_export_error(
    handler_path: &str,
    err: crate::participant::EventParticipantPlanError,
) -> ComponentExportError {
    sand_components::error::SandError::ComponentValidation {
        location: sand_components::ResourceLocation::new("sand", "participants")
            .expect("fixed participants resource location is valid"),
        kind: "event_participant_plan".to_string(),
        field: "participants".to_string(),
        message: format!("cannot apply participant plan for handler `{handler_path}`: {err}"),
    }
}

/// Convert a caught [`crate::participant::diagnostic::MissingParticipantPanic`]
/// (#280 item 2) into the structured `SAND-EVENT-PARTICIPANT` export
/// diagnostic — see `invoke_event_handler_body` in `pipeline.rs`, the sole
/// call site, for the panic-hook/`catch_unwind` boundary that produces it.
pub(crate) fn participant_accessor_panic_export_error(
    handler_path: &str,
    panic: &crate::participant::diagnostic::MissingParticipantPanic,
) -> ComponentExportError {
    sand_components::error::SandError::ComponentValidation {
        location: sand_components::ResourceLocation::new("sand", "participants")
            .expect("fixed participants resource location is valid"),
        kind: "event_participant_accessor".to_string(),
        field: "participants".to_string(),
        message: panic.render(handler_path),
    }
}

#[cfg(test)]
mod tests {
    use crate::AdvancementTrigger;
    use sand_version::VersionCaps;

    use super::super::ExportCtx;

    #[test]
    fn xp_score_operations_are_lowered_per_player() {
        let commands = super::xp_score_commands();
        assert_eq!(commands, super::xp_score_commands());
        assert_eq!(
            commands,
            vec![
                "execute as @a store result score @s __sand_xp_lvl run experience query @s levels"
                    .to_string(),
                "execute as @a unless score @s __sand_xp_seen matches 1 run scoreboard players operation @s __sand_xp_prev = @s __sand_xp_lvl"
                    .to_string(),
                "scoreboard players set @a __sand_xp_seen 1".to_string(),
                "execute as @a run scoreboard players operation @s __sand_xp_delta = @s __sand_xp_lvl"
                    .to_string(),
                "execute as @a run scoreboard players operation @s __sand_xp_delta -= @s __sand_xp_prev"
                    .to_string(),
            ]
        );
        assert_eq!(
            super::xp_advance_command(),
            "execute as @a run scoreboard players operation @s __sand_xp_prev = @s __sand_xp_lvl"
        );
    }

    #[test]
    fn custom_sand_event_advancement_trigger_validation_accepts_supported_trigger() {
        let result = super::check_event_trigger(
            &AdvancementTrigger::Tick,
            "test:legacy_valid_event",
            "legacy_valid_event",
            None,
        );
        assert!(
            result.is_ok(),
            "supported trigger should pass: {:?}",
            result.err()
        );
    }

    // ── Custom SandEvent dispatch backend validation (#121) ────────────────────

    #[test]
    fn custom_dispatch_backend_accepts_trigger_only() {
        let backend = super::resolve_custom_dispatch_backend(
            Some(AdvancementTrigger::Tick),
            None,
            None,
            None,
            None,
            "my_pack:on_thing",
        );
        assert!(matches!(
            backend,
            super::CustomDispatchBackend::Advancement(_)
        ));
    }

    #[test]
    fn custom_dispatch_backend_accepts_condition_only() {
        let backend = super::resolve_custom_dispatch_backend(
            None,
            Some("score @s foo matches 1..".to_string()),
            None,
            None,
            None,
            "my_pack:on_thing",
        );
        assert!(matches!(backend, super::CustomDispatchBackend::TickPoll(_)));
    }

    #[test]
    fn custom_dispatch_backend_accepts_tick_only() {
        let backend = super::resolve_custom_dispatch_backend(
            None,
            None,
            Some(crate::events::TickEventDispatch::default()),
            None,
            None,
            "my_pack:on_thing",
        );
        assert!(matches!(
            backend,
            super::CustomDispatchBackend::TickLifecycle(_)
        ));
    }

    #[test]
    #[should_panic(
        expected = "returned None from make_trigger(), make_condition(), make_tick(), make_chain(), and make_tracked()"
    )]
    fn custom_dispatch_backend_rejects_neither_backend() {
        super::resolve_custom_dispatch_backend(None, None, None, None, None, "my_pack:on_thing");
    }

    #[test]
    #[should_panic(expected = "returned more than one dispatch strategy")]
    fn custom_dispatch_backend_rejects_both_backends() {
        super::resolve_custom_dispatch_backend(
            Some(AdvancementTrigger::Tick),
            Some("score @s foo matches 1..".to_string()),
            None,
            None,
            None,
            "my_pack:on_thing",
        );
    }

    #[test]
    fn custom_dispatch_backend_both_backends_panic_names_handler_path() {
        let result = std::panic::catch_unwind(|| {
            super::resolve_custom_dispatch_backend(
                Some(AdvancementTrigger::Tick),
                Some("score @s foo matches 1..".to_string()),
                None,
                None,
                None,
                "my_pack:on_elevator_placed",
            )
        });
        let err = match result {
            Ok(_) => panic!("expected panic, got Ok"),
            Err(err) => err,
        };
        let message = err
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert_eq!(
            message,
            "Custom SandEvent for handler `my_pack:on_elevator_placed` returned more than one \
             dispatch strategy (make_trigger/make_condition/make_tick/make_chain/make_tracked) — \
             implement exactly one"
        );
    }

    #[test]
    fn custom_sand_event_advancement_trigger_validation_rejects_invalid_trigger() {
        let result = super::check_event_trigger(
            &AdvancementTrigger::LeveledUp { level: None },
            "test:legacy_level_up",
            "legacy_level_up",
            None,
        );
        let err = result.expect_err("invalid trigger should be rejected");
        let msg = err.to_string();
        assert_eq!(
            msg,
            "component `test:legacy_level_up` (advancement_event): cannot export advancement \
             event `legacy_level_up`: advancement trigger `minecraft:leveled_up` is not \
             available for Sand's supported Minecraft targets. use tick polling: `execute \
             store result score @s <objective> run experience query @s levels`, then compare \
             the stored score [field: trigger]"
        );
    }

    #[test]
    fn event_trigger_gating_rejects_unsupported_and_fallback_profiles() {
        let old_caps = crate::version::VersionProfile::resolve(
            &crate::version::MinecraftVersion::parse("1.18.2").unwrap(),
        )
        .unwrap()
        .caps();
        let old_ctx = ExportCtx {
            caps: &old_caps,
            requested_version: "1.18.2",
            is_fallback: false,
        };
        let unsupported = super::check_event_trigger(
            &AdvancementTrigger::AllayDropItemOnBlock {
                item: None,
                location: None,
            },
            "test:allay_event",
            "allay_event",
            Some(&old_ctx),
        )
        .expect_err("allay trigger was introduced after 1.18.2");
        assert!(
            unsupported
                .to_string()
                .contains("minecraft:allay_drop_item_on_block")
        );
        assert!(unsupported.to_string().contains("1.18.2"));

        let fallback_caps = crate::version::VersionProfile::resolve(
            &crate::version::MinecraftVersion::parse("999.0").unwrap(),
        )
        .unwrap()
        .caps();
        let fallback_ctx = ExportCtx {
            caps: &fallback_caps,
            requested_version: "999.0",
            is_fallback: true,
        };
        let fallback = super::check_event_trigger(
            &AdvancementTrigger::Tick,
            "test:fallback_event",
            "fallback_event",
            Some(&fallback_ctx),
        )
        .expect_err("fallback profiles must not claim exact trigger support");
        assert!(fallback.to_string().contains("fallback"));
        assert!(fallback.to_string().contains("minecraft:tick"));
    }

    #[test]
    fn event_trigger_gating_accepts_supported_and_custom_triggers() {
        let caps = VersionCaps::all_enabled();
        let exact_ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.19",
            is_fallback: false,
        };
        super::check_event_trigger(
            &AdvancementTrigger::AllayDropItemOnBlock {
                item: None,
                location: None,
            },
            "test:allay_event",
            "allay_event",
            Some(&exact_ctx),
        )
        .expect("allay trigger should be valid in 1.19");

        let fallback_ctx = ExportCtx {
            caps: &caps,
            requested_version: "999.0",
            is_fallback: true,
        };
        super::check_event_trigger(
            &AdvancementTrigger::Custom {
                trigger: "examplemod:custom_trigger".to_string(),
                conditions: None,
            },
            "test:custom_event",
            "custom_event",
            Some(&fallback_ctx),
        )
        .expect("explicit custom triggers remain a raw/modded escape hatch");
    }
}
