//! Deterministic generated backend for per-player tracked transitions.

use std::collections::{BTreeMap, BTreeSet};

use crate::{TrackedTransition, TransitionKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TransitionHandler {
    pub path: String,
    pub transition: TrackedTransition,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct TransitionPlan {
    pub load_commands: Vec<String>,
    pub tick_commands: Vec<String>,
    pub global_tick_commands: Vec<String>,
    pub functions: Vec<GeneratedTransitionFunction>,
    pub private_objectives: BTreeMap<String, (String, String)>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct GeneratedTransitionFunction {
    pub tracker_id: String,
    pub source: String,
    pub path: String,
    pub commands: Vec<String>,
}

pub(crate) mod observation;

pub(crate) fn resolve_transition_plan(
    namespace: &str,
    handlers: &[TransitionHandler],
) -> Result<TransitionPlan, String> {
    let observations = handlers
        .iter()
        .map(observation::ObservationHandler::from)
        .collect::<Vec<_>>();
    observation::resolve(namespace, &observations)
}

/// Resolve tracked events and typed state flows through one transition backend.
///
/// Both families share lifecycle merging, generated-function collision checks,
/// private objective diagnostics, deterministic ordering, and the exporter
/// wiring below this boundary.
pub(crate) fn resolve_unified_transition_plan(
    namespace: &str,
    handlers: &[TransitionHandler],
) -> Result<TransitionPlan, String> {
    let mut plan = resolve_transition_plan(namespace, handlers)?;
    let flows = crate::state::flow::drain_flows();
    let flow_plan = crate::state::flow::resolve_state_flow_plan(namespace, flows)?;

    let mut paths: BTreeSet<String> = plan
        .functions
        .iter()
        .map(|function| function.path.clone())
        .collect();
    for function in flow_plan.functions {
        if !paths.insert(function.path.clone()) {
            return Err(format!(
                "error[SAND-STATE-FLOW]: generated transition helper collision\n\nPath: {}\nFlow: {}\n\nCorrection: give the state flow a distinct `.named(...)` label",
                function.path, function.tracker_id
            ));
        }
        plan.functions.push(function);
    }
    for (objective, owner) in flow_plan.private_objectives {
        if let Some(existing) = plan
            .private_objectives
            .insert(objective.clone(), owner.clone())
        {
            return Err(format!(
                "error[SAND-STATE-FLOW]: generated objective collision\n\nObjective: {objective}\nExisting owner: {}\nFlow owner: {}\n\nCorrection: give the state flow a distinct `.named(...)` label",
                existing.0, owner.0
            ));
        }
    }
    plan.load_commands.extend(flow_plan.load_commands);
    plan.tick_commands.extend(flow_plan.tick_commands);
    plan.global_tick_commands
        .extend(flow_plan.global_tick_commands);
    Ok(plan)
}

fn boolean_commands(
    handlers: &BTreeSet<(TransitionKind, &str)>,
    mut commands: Vec<String>,
    previous: &str,
    current: &str,
    seen: &str,
    namespace: &str,
) -> Vec<String> {
    for (kind, path) in handlers {
        let comparison = match kind {
            TransitionKind::BecameTrue => {
                format!("if score @s {current} matches 1 unless score @s {previous} matches 1")
            }
            TransitionKind::BecameFalse => {
                format!("unless score @s {current} matches 1 if score @s {previous} matches 1")
            }
            _ => unreachable!("validated boolean transition kind"),
        };
        commands.push(format!(
            "execute if score @s {seen} matches 1 {comparison} at @s run function {namespace}:{path}"
        ));
    }
    // Every handler observes the old baseline. State advances only afterward.
    commands.push(format!(
        "scoreboard players operation @s {previous} = @s {current}"
    ));
    commands.push(format!("scoreboard players set @s {seen} 1"));
    commands
}

fn score_commands(
    handlers: &BTreeSet<(TransitionKind, &str)>,
    mut commands: Vec<String>,
    previous: &str,
    current: &str,
    seen: &str,
    available: &str,
    namespace: &str,
) -> Vec<String> {
    for (kind, path) in handlers {
        let comparison = match kind {
            TransitionKind::ScoreChanged => {
                format!("unless score @s {current} = @s {previous}")
            }
            TransitionKind::ScoreIncreased => {
                format!("if score @s {current} > @s {previous}")
            }
            TransitionKind::ScoreDecreased => {
                format!("if score @s {current} < @s {previous}")
            }
            _ => unreachable!("validated score transition kind"),
        };
        commands.push(format!(
            "execute if score @s {available} matches 1 if score @s {seen} matches 1 {comparison} at @s run function {namespace}:{path}"
        ));
    }
    commands.push(format!(
        "execute if score @s {available} matches 1 run scoreboard players operation @s {previous} = @s {current}"
    ));
    commands.push(format!(
        "execute if score @s {available} matches 1 run scoreboard players set @s {seen} 1"
    ));
    commands
}

fn tracker_key(id: &str) -> String {
    format!("{:010x}", fnv1a(id) & 0xFF_FFFF_FFFF)
}

fn fnv1a(value: &str) -> u64 {
    let mut hash = 14_695_981_039_346_656_037;
    for byte in value.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScoreThresholdComparator, TrackedSource};

    // Canonical numeric sampling owns helpers/objectives just like ordinary
    // gameplay lowering. Keep each planner-only test inside an export scope.
    fn resolve_transition_plan(
        namespace: &str,
        handlers: &[TransitionHandler],
    ) -> Result<TransitionPlan, String> {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        super::resolve_transition_plan(namespace, handlers)
    }

    const BOOL: TrackedSource = TrackedSource::BooleanCondition {
        description: "test boolean",
        condition: "predicate test:flag",
    };
    const SCORE: TrackedSource = TrackedSource::Score {
        description: "test score",
        objective: "points",
        criterion: "dummy",
    };

    fn handler(
        path: &str,
        id: &'static str,
        source: TrackedSource,
        kind: TransitionKind,
    ) -> TransitionHandler {
        TransitionHandler {
            path: path.to_string(),
            transition: TrackedTransition::new(id, source, kind),
        }
    }

    #[test]
    fn boolean_transition_truth_table_and_first_observation() {
        assert!(!TransitionKind::BecameTrue.matches(0, 1, false));
        assert!(TransitionKind::BecameTrue.matches(0, 1, true));
        assert!(!TransitionKind::BecameTrue.matches(1, 1, true));
        assert!(TransitionKind::BecameFalse.matches(1, 0, true));
        assert!(!TransitionKind::BecameFalse.matches(0, 0, true));
    }

    #[test]
    fn score_transition_truth_table() {
        assert!(TransitionKind::ScoreChanged.matches(4, 5, true));
        assert!(!TransitionKind::ScoreChanged.matches(5, 5, true));
        assert!(TransitionKind::ScoreIncreased.matches(4, 5, true));
        assert!(TransitionKind::ScoreDecreased.matches(5, 4, true));
        assert!(!TransitionKind::ScoreChanged.matches(4, 5, false));
    }

    #[test]
    fn shared_boolean_tracker_dispatches_before_one_state_update() {
        let plan = resolve_transition_plan(
            "pack",
            &[
                handler("z_stop", "sneak", BOOL, TransitionKind::BecameFalse),
                handler("b_start", "sneak", BOOL, TransitionKind::BecameTrue),
                handler("a_start", "sneak", BOOL, TransitionKind::BecameTrue),
            ],
        )
        .unwrap();
        assert_eq!(plan.functions.len(), 1);
        let commands = &plan.functions[0].commands;
        assert_eq!(
            commands
                .iter()
                .filter(|line| line.contains("a_start"))
                .count(),
            1
        );
        assert_eq!(
            commands
                .iter()
                .filter(|line| line.contains("b_start"))
                .count(),
            1
        );
        assert!(commands[0].contains("players set") && commands[0].ends_with(" 0"));
        assert!(commands[1].contains("predicate test:flag"));
        assert!(commands[2].contains("a_start"));
        let first_update = commands
            .iter()
            .position(|line| line.contains("players operation"))
            .unwrap();
        assert_eq!(
            first_update, 5,
            "all handlers must run after sampling and before state updates"
        );
    }

    #[test]
    fn score_tracker_generates_all_comparisons_and_initializes_after_dispatch() {
        let plan = resolve_transition_plan(
            "pack",
            &[
                handler("changed", "points", SCORE, TransitionKind::ScoreChanged),
                handler("up", "points", SCORE, TransitionKind::ScoreIncreased),
                handler("down", "points", SCORE, TransitionKind::ScoreDecreased),
            ],
        )
        .unwrap();
        let commands = &plan.functions[0].commands;
        assert!(
            commands
                .iter()
                .any(|line| line.contains("unless score @s __st_") && line.contains(" = @s __st_"))
        );
        assert!(
            commands
                .iter()
                .any(|line| line.contains("if score @s __st_") && line.contains(" > @s __st_"))
        );
        assert!(
            commands
                .iter()
                .any(|line| line.contains("if score @s __st_") && line.contains(" < @s __st_"))
        );
        assert!(commands[commands.len() - 2].contains("players operation"));
        assert!(commands.last().unwrap().contains("players set"));
    }

    #[test]
    fn identical_handlers_dedupe_and_conflicting_sources_fail_contextually() {
        let duplicate = handler("same", "shared", BOOL, TransitionKind::BecameTrue);
        let plan = resolve_transition_plan("pack", &[duplicate.clone(), duplicate]).unwrap();
        assert_eq!(
            plan.functions[0]
                .commands
                .iter()
                .filter(|line| line.contains("function pack:same"))
                .count(),
            1
        );

        let err = resolve_transition_plan(
            "pack",
            &[
                handler("boolean", "shared", BOOL, TransitionKind::BecameTrue),
                handler("score", "shared", SCORE, TransitionKind::ScoreChanged),
            ],
        )
        .unwrap_err();
        assert!(err.contains("conflicting transition tracker `shared`"));
        assert!(err.contains("test boolean"));
        assert!(err.contains("test score"));
    }

    #[test]
    fn generated_names_are_stable_and_objectives_fit_vanilla_limit() {
        let first = resolve_transition_plan(
            "pack",
            &[handler(
                "start",
                "sneaking",
                BOOL,
                TransitionKind::BecameTrue,
            )],
        )
        .unwrap();
        let second = resolve_transition_plan(
            "pack",
            &[handler(
                "start",
                "sneaking",
                BOOL,
                TransitionKind::BecameTrue,
            )],
        )
        .unwrap();
        assert_eq!(first, second);
        for command in &first.load_commands {
            let objective = command.split_whitespace().nth(3).unwrap();
            assert!(objective.len() <= 16);
        }
    }

    #[test]
    fn score_threshold_renders_comparator_and_supports_boolean_kinds() {
        const LOW_HEALTH: TrackedSource = TrackedSource::ScoreThreshold {
            description: "low health threshold",
            objective: "sand_health",
            criterion: "health",
            comparator: ScoreThresholdComparator::AtOrBelow(6),
        };
        let plan = resolve_transition_plan(
            "pack",
            &[
                handler(
                    "low",
                    "player_low_health",
                    LOW_HEALTH,
                    TransitionKind::BecameTrue,
                ),
                handler(
                    "recovered",
                    "player_low_health",
                    LOW_HEALTH,
                    TransitionKind::BecameFalse,
                ),
            ],
        )
        .unwrap();
        assert_eq!(plan.functions.len(), 1);
        let commands = &plan.functions[0].commands;
        assert!(
            commands
                .iter()
                .any(|line| line.contains("score @s sand_health matches ..6"))
        );
        assert!(
            commands
                .iter()
                .any(|line| line.contains("function pack:low"))
        );
        assert!(
            commands
                .iter()
                .any(|line| line.contains("function pack:recovered"))
        );
    }

    #[test]
    fn score_threshold_at_or_above_renders_open_lower_bound() {
        let plan = resolve_transition_plan(
            "pack",
            &[handler(
                "high",
                "high_health",
                TrackedSource::ScoreThreshold {
                    description: "high health",
                    objective: "health",
                    criterion: "health",
                    comparator: ScoreThresholdComparator::AtOrAbove(10),
                },
                TransitionKind::BecameTrue,
            )],
        )
        .unwrap();
        assert!(
            plan.functions[0]
                .commands
                .iter()
                .any(|line| line.contains("score @s health matches 10.."))
        );
    }

    #[test]
    fn distinct_score_thresholds_under_one_tracker_id_conflict() {
        const LOW_6: TrackedSource = TrackedSource::ScoreThreshold {
            description: "low health (6)",
            objective: "sand_health",
            criterion: "health",
            comparator: ScoreThresholdComparator::AtOrBelow(6),
        };
        const LOW_10: TrackedSource = TrackedSource::ScoreThreshold {
            description: "low health (10)",
            objective: "sand_health",
            criterion: "health",
            comparator: ScoreThresholdComparator::AtOrBelow(10),
        };
        let err = resolve_transition_plan(
            "pack",
            &[
                handler(
                    "low6",
                    "player_low_health",
                    LOW_6,
                    TransitionKind::BecameTrue,
                ),
                handler(
                    "low10",
                    "player_low_health",
                    LOW_10,
                    TransitionKind::BecameTrue,
                ),
            ],
        )
        .unwrap_err();
        assert!(err.contains("conflicting transition tracker `player_low_health`"));
    }

    #[test]
    fn score_and_score_threshold_auto_declare_shared_criterion_once() {
        const CHANGED: TrackedSource = TrackedSource::Score {
            description: "health changed",
            objective: "sand_health",
            criterion: "health",
        };
        const LOW: TrackedSource = TrackedSource::ScoreThreshold {
            description: "low health",
            objective: "sand_health",
            criterion: "health",
            comparator: ScoreThresholdComparator::AtOrBelow(6),
        };
        let plan = resolve_transition_plan(
            "pack",
            &[
                handler(
                    "changed",
                    "player_health",
                    CHANGED,
                    TransitionKind::ScoreChanged,
                ),
                handler("low", "player_low_health", LOW, TransitionKind::BecameTrue),
            ],
        )
        .unwrap();
        assert_eq!(
            plan.load_commands
                .iter()
                .filter(|line| *line == "scoreboard objectives add sand_health health")
                .count(),
            1,
            "the shared sand_health/health criterion must be declared exactly once: {:?}",
            plan.load_commands
        );
    }

    #[test]
    fn conflicting_criteria_on_the_same_objective_are_rejected() {
        const A: TrackedSource = TrackedSource::Score {
            description: "a",
            objective: "shared_obj",
            criterion: "health",
        };
        const B: TrackedSource = TrackedSource::Score {
            description: "b",
            objective: "shared_obj",
            criterion: "xp",
        };
        let err = resolve_transition_plan(
            "pack",
            &[
                handler("a", "tracker_a", A, TransitionKind::ScoreChanged),
                handler("b", "tracker_b", B, TransitionKind::ScoreChanged),
            ],
        )
        .unwrap_err();
        assert!(err.contains("shared_obj"));
        assert!(err.contains("conflicts with a previously declared criterion"));
    }
}
