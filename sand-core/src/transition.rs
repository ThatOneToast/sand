//! Deterministic generated backend for per-player tracked transitions.

use std::collections::{BTreeMap, BTreeSet};

use crate::{TrackedSource, TrackedTransition, TransitionKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TransitionHandler {
    pub path: String,
    pub transition: TrackedTransition,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct TransitionPlan {
    pub load_commands: Vec<String>,
    pub tick_commands: Vec<String>,
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

#[derive(Debug)]
struct Tracker<'a> {
    source: TrackedSource,
    handlers: BTreeSet<(TransitionKind, &'a str)>,
}

pub(crate) fn resolve_transition_plan<'a>(
    namespace: &str,
    handlers: &'a [TransitionHandler],
) -> Result<TransitionPlan, String> {
    let mut trackers: BTreeMap<&str, Tracker<'a>> = BTreeMap::new();

    for handler in handlers {
        validate_kind(handler.transition)?;
        let id = handler.transition.tracker_id;
        if id.is_empty() {
            return Err(format!(
                "transition handler `{}` has an empty tracker ID for source `{}`",
                handler.path,
                handler.transition.source.description()
            ));
        }
        match trackers.get_mut(id) {
            Some(tracker) if tracker.source == handler.transition.source => {
                tracker
                    .handlers
                    .insert((handler.transition.kind, handler.path.as_str()));
            }
            Some(tracker) => {
                return Err(format!(
                    "conflicting transition tracker `{id}`: source `{}` conflicts with `{}` for handler `{}`",
                    tracker.source.description(),
                    handler.transition.source.description(),
                    handler.path
                ));
            }
            None => {
                trackers.insert(
                    id,
                    Tracker {
                        source: handler.transition.source,
                        handlers: BTreeSet::from([(
                            handler.transition.kind,
                            handler.path.as_str(),
                        )]),
                    },
                );
            }
        }
    }

    let mut generated_names: BTreeMap<String, &str> = BTreeMap::new();
    let mut declared_criteria: BTreeMap<&str, &str> = BTreeMap::new();
    let mut plan = TransitionPlan::default();
    for (id, tracker) in trackers {
        let key = tracker_key(id);
        let previous = format!("__st_{key}p");
        let current = format!("__st_{key}c");
        let seen = format!("__st_{key}s");
        let available =
            matches!(tracker.source, TrackedSource::Score { .. }).then(|| format!("__st_{key}a"));
        let function_path = format!("__sand_transition/{key}");

        if let TrackedSource::Score {
            objective,
            criterion,
            ..
        }
        | TrackedSource::ScoreThreshold {
            objective,
            criterion,
            ..
        } = tracker.source
        {
            match declared_criteria.get(objective) {
                Some(existing) if *existing != criterion => {
                    return Err(format!(
                        "transition tracker `{id}` references objective `{objective}` with criterion `{criterion}`, which conflicts with a previously declared criterion `{existing}`"
                    ));
                }
                Some(_) => {}
                None => {
                    declared_criteria.insert(objective, criterion);
                    plan.load_commands
                        .push(format!("scoreboard objectives add {objective} {criterion}"));
                }
            }
        }

        for generated in [&previous, &current, &seen, &function_path]
            .into_iter()
            .chain(available.as_ref())
        {
            if let Some(existing) = generated_names.insert(generated.clone(), id)
                && existing != id
            {
                return Err(format!(
                    "transition tracker name collision: `{existing}` and `{id}` both generate `{generated}`"
                ));
            }
        }

        plan.load_commands
            .push(format!("scoreboard objectives add {previous} dummy"));
        plan.load_commands
            .push(format!("scoreboard objectives add {current} dummy"));
        plan.load_commands
            .push(format!("scoreboard objectives add {seen} dummy"));
        if let Some(available) = &available {
            plan.load_commands
                .push(format!("scoreboard objectives add {available} dummy"));
        }
        for objective in [&previous, &current, &seen]
            .into_iter()
            .chain(available.as_ref())
        {
            plan.private_objectives.insert(
                objective.clone(),
                (id.to_string(), tracker.source.description().to_string()),
            );
        }
        plan.tick_commands
            .push(format!("function {namespace}:{function_path}"));

        let commands = match tracker.source {
            TrackedSource::BooleanCondition { condition, .. } => boolean_commands(
                &tracker.handlers,
                condition,
                &previous,
                &current,
                &seen,
                namespace,
            ),
            TrackedSource::ScoreThreshold {
                objective,
                comparator,
                ..
            } => boolean_commands(
                &tracker.handlers,
                &comparator.render(objective),
                &previous,
                &current,
                &seen,
                namespace,
            ),
            TrackedSource::Score { objective, .. } => score_commands(
                &tracker.handlers,
                objective,
                &previous,
                &current,
                &seen,
                available
                    .as_deref()
                    .expect("score trackers have availability state"),
                namespace,
            ),
        };
        plan.functions.push(GeneratedTransitionFunction {
            tracker_id: id.to_string(),
            source: tracker.source.description().to_string(),
            path: function_path,
            commands,
        });
    }
    Ok(plan)
}

fn validate_kind(transition: TrackedTransition) -> Result<(), String> {
    let valid = matches!(
        (transition.source, transition.kind),
        (
            TrackedSource::BooleanCondition { .. } | TrackedSource::ScoreThreshold { .. },
            TransitionKind::BecameTrue | TransitionKind::BecameFalse
        ) | (
            TrackedSource::Score { .. },
            TransitionKind::ScoreChanged
                | TransitionKind::ScoreIncreased
                | TransitionKind::ScoreDecreased
        )
    );
    if valid {
        Ok(())
    } else {
        Err(format!(
            "transition tracker `{}` uses incompatible kind {:?} for source `{}`",
            transition.tracker_id,
            transition.kind,
            transition.source.description()
        ))
    }
}

fn boolean_commands(
    handlers: &BTreeSet<(TransitionKind, &str)>,
    condition: &str,
    previous: &str,
    current: &str,
    seen: &str,
    namespace: &str,
) -> Vec<String> {
    let mut commands = vec![format!(
        "execute store success score @s {current} if {condition}"
    )];
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
    objective: &str,
    previous: &str,
    current: &str,
    seen: &str,
    available: &str,
    namespace: &str,
) -> Vec<String> {
    let mut commands = vec![
        format!(
            "execute store success score @s {available} if score @s {objective} matches -2147483648.."
        ),
        format!(
            "execute if score @s {available} matches 1 run scoreboard players operation @s {current} = @s {objective}"
        ),
    ];
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
        assert!(commands[0].contains("store success score"));
        assert!(commands[1].contains("a_start"));
        let first_update = commands
            .iter()
            .position(|line| line.contains("players operation"))
            .unwrap();
        assert_eq!(
            first_update, 4,
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
}
