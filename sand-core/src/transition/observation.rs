//! Owned observation planning shared by tracked-event consumers.
//!
//! This first backend samples in the existing per-player tick context. Its
//! first successful sample initializes a baseline without dispatch; unavailable
//! numeric samples neither dispatch nor overwrite that baseline. Conditions
//! and numbers retain the same semantics/resources as ordinary gameplay IR.

use super::{GeneratedTransitionFunction, TransitionHandler, TransitionPlan};
use crate::condition::{Condition, ScoreRange};
use crate::entity::{FixedPoint, StatCurve};
use crate::ir::{Actions, Cmd, NumericWrite, ScorePlayersOp};
use crate::{ScoreThresholdComparator, TrackedSource, TransitionKind};
use sand_commands::{ObjectiveName, ScoreHolder};
use sand_components::ResourceLocation;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ObservationSource {
    Boolean(Condition),
    Numeric(StatCurve),
}

/// A sample definition, independent of the event names consuming it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Observation {
    pub source: ObservationSource,
    pub declarations: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ObservationHandler {
    pub tracker_id: String,
    pub description: String,
    pub observation: Observation,
    pub kind: TransitionKind,
    pub path: String,
}

impl From<crate::TrackedTransition> for crate::events::OwnedObservationDispatch {
    fn from(transition: crate::TrackedTransition) -> Self {
        let mut declarations = BTreeMap::new();
        let source = match transition.source {
            TrackedSource::BooleanCondition { condition, .. } => {
                // The static generated-event metadata is the legacy boundary.
                ObservationSource::Boolean(Condition::raw(condition))
            }
            TrackedSource::Score {
                objective,
                criterion,
                ..
            } => {
                declarations.insert(objective.into(), criterion.into());
                ObservationSource::Numeric(StatCurve::input_raw(objective))
            }
            TrackedSource::ScoreThreshold {
                objective,
                criterion,
                comparator,
                ..
            } => {
                declarations.insert(objective.into(), criterion.into());
                let range = match comparator {
                    ScoreThresholdComparator::AtOrBelow(value) => ScoreRange::Lte(value),
                    ScoreThresholdComparator::AtOrAbove(value) => ScoreRange::Gte(value),
                };
                ObservationSource::Boolean(Condition::score("@s".into(), objective.into(), range))
            }
        };
        Self {
            tracker_id: transition.tracker_id.into(),
            description: transition.source.description().into(),
            observation: Observation {
                source,
                declarations,
            },
            kind: transition.kind,
        }
    }
}

impl From<&TransitionHandler> for ObservationHandler {
    fn from(handler: &TransitionHandler) -> Self {
        Self {
            tracker_id: handler.transition.tracker_id.clone(),
            description: handler.transition.description.clone(),
            observation: handler.transition.observation.clone(),
            kind: handler.transition.kind,
            path: handler.path.clone(),
        }
    }
}

#[derive(Debug)]
struct Group<'a> {
    observation: &'a Observation,
    ids: BTreeSet<&'a str>,
    descriptions: BTreeSet<&'a str>,
    handlers: BTreeSet<(TransitionKind, &'a str)>,
}

pub(crate) fn resolve(
    namespace: &str,
    handlers: &[ObservationHandler],
) -> Result<TransitionPlan, String> {
    let mut declared_criteria = BTreeMap::<String, String>::new();
    let mut ids = BTreeMap::<&str, &ObservationHandler>::new();
    let mut groups = BTreeMap::<String, Group<'_>>::new();
    for handler in handlers {
        if handler.tracker_id.is_empty() {
            return Err(format!(
                "transition handler `{}` has an empty tracker ID for source `{}`",
                handler.path, handler.description
            ));
        }
        let valid = matches!(
            (&handler.observation.source, handler.kind),
            (
                ObservationSource::Boolean(_),
                TransitionKind::BecameTrue | TransitionKind::BecameFalse
            ) | (
                ObservationSource::Numeric(_),
                TransitionKind::ScoreChanged
                    | TransitionKind::ScoreIncreased
                    | TransitionKind::ScoreDecreased
            )
        );
        if !valid {
            return Err(format!(
                "transition tracker `{}` uses incompatible kind {:?} for source `{}`",
                handler.tracker_id, handler.kind, handler.description
            ));
        }
        if let Some(existing) = ids.insert(&handler.tracker_id, handler)
            && existing.observation.source != handler.observation.source
        {
            return Err(format!(
                "conflicting transition tracker `{}`: source `{}` conflicts with `{}` for handler `{}`",
                handler.tracker_id, existing.description, handler.description, handler.path
            ));
        }
        for (objective, criterion) in &handler.observation.declarations {
            if let Some(existing) = declared_criteria.insert(objective.clone(), criterion.clone())
                && existing != *criterion
            {
                return Err(format!(
                    "transition tracker `{}` references objective `{objective}` with criterion `{criterion}`, which conflicts with a previously declared criterion `{existing}`",
                    handler.tracker_id
                ));
            }
        }
        // Canonical owned models also supply the generated-action identity.
        // Keep the full identity as the map key; hashed-name collisions below
        // remain errors instead of silently sharing unrelated sample state.
        let identity = format!("{:?}", handler.observation.source);
        let group = groups.entry(identity).or_insert_with(|| Group {
            observation: &handler.observation,
            ids: BTreeSet::new(),
            descriptions: BTreeSet::new(),
            handlers: BTreeSet::new(),
        });
        group.ids.insert(&handler.tracker_id);
        group.descriptions.insert(&handler.description);
        group.handlers.insert((handler.kind, &handler.path));
    }

    let mut plan = TransitionPlan::default();
    for (objective, criterion) in &declared_criteria {
        let name = ObjectiveName::try_dynamic(objective).map_err(|e| e.to_string())?;
        sand_commands::validate::no_whitespace_or_control(criterion, "observation", "criterion")
            .map_err(|e| e.to_string())?;
        plan.load_commands.push(
            Cmd::ScoreDefine {
                objective: name.to_string(),
                criterion: criterion.clone(),
            }
            .try_render()
            .map_err(|e| e.to_string())?,
        );
    }
    let mut generated_names = BTreeMap::<String, String>::new();
    for (identity, group) in groups {
        let key = super::tracker_key(&format!("{namespace}:{identity}"));
        let previous = format!("__st_{key}p");
        let current = format!("__st_{key}c");
        let seen = format!("__st_{key}s");
        let available = matches!(group.observation.source, ObservationSource::Numeric(_))
            .then(|| format!("__st_{key}a"));
        let path = format!("__sand_transition/{key}");
        let tracker_id = group.ids.into_iter().collect::<Vec<_>>().join(", ");
        let description = group
            .descriptions
            .into_iter()
            .collect::<Vec<_>>()
            .join("; ");
        for name in [&previous, &current, &seen, &path]
            .into_iter()
            .chain(available.as_ref())
        {
            if let Some(existing) = generated_names.insert(name.clone(), identity.clone())
                && existing != identity
            {
                return Err(format!(
                    "transition tracker name collision: `{existing}` and `{identity}` both generate `{name}`"
                ));
            }
        }
        for objective in [&previous, &current, &seen]
            .into_iter()
            .chain(available.as_ref())
        {
            if declared_criteria.contains_key(objective) {
                return Err(format!(
                    "transition tracker `{tracker_id}` private objective `{objective}` collides with a source declaration"
                ));
            }
            plan.load_commands
                .push(format!("scoreboard objectives add {objective} dummy"));
            plan.private_objectives
                .insert(objective.clone(), (tracker_id.clone(), description.clone()));
        }
        let owner = ResourceLocation::new(namespace, &path).map_err(|e| e.to_string())?;
        let commands = match &group.observation.source {
            ObservationSource::Boolean(condition) => {
                let mut sample = Actions(vec![set(&current, 0)]);
                sample.0.extend(
                    condition
                        .guard_actions(false, &[], Actions(vec![set(&current, 1)]))
                        .0,
                );
                let sample = sample.lower(&owner).map_err(|e| e.to_string())?;
                super::boolean_commands(
                    &group.handlers,
                    sample,
                    &previous,
                    &current,
                    &seen,
                    namespace,
                )
            }
            ObservationSource::Numeric(value) => {
                let fixed = FixedPoint::default();
                let scale = i32::try_from(value.direct_source_scale().unwrap_or(fixed.scale()))
                    .map_err(|e| e.to_string())?;
                let sample = Actions(vec![
                    Cmd::ScorePlayers(ScorePlayersOp::Reset {
                        selector: "@s".into(),
                        objective: current.clone(),
                    }),
                    Cmd::NumericWrite(Box::new(NumericWrite {
                        value: value.clone(),
                        holder: ScoreHolder::self_(),
                        objective: ObjectiveName::try_dynamic(&current)
                            .map_err(|e| e.to_string())?,
                        scale,
                        fixed,
                        bounds: None,
                        dirty: Vec::new(),
                    })),
                ]);
                let mut sample = sample.lower(&owner).map_err(|e| e.to_string())?;
                let available = available
                    .as_deref()
                    .expect("numeric samples track availability");
                sample.push(format!("execute store success score @s {available} if score @s {current} matches -2147483648.."));
                super::score_commands(
                    &group.handlers,
                    sample,
                    &previous,
                    &current,
                    &seen,
                    available,
                    namespace,
                )
            }
        };
        plan.tick_commands
            .push(format!("execute at @s run function {namespace}:{path}"));
        plan.functions.push(GeneratedTransitionFunction {
            tracker_id,
            source: description,
            path,
            commands,
        });
    }
    // Sample all observations for one player before advancing to the next.
    // The lifecycle owns the single player scan; this coordinator also fixes
    // dimension, position and rotation before each source, including after a
    // preceding handler teleports the player.
    if plan.tick_commands.len() > 1 {
        let path = "__sand_observation_tick".to_owned();
        plan.functions.push(GeneratedTransitionFunction {
            tracker_id: "shared player observations".into(),
            source: "deterministic per-player observation sequence".into(),
            path: path.clone(),
            commands: std::mem::take(&mut plan.tick_commands),
        });
        plan.tick_commands
            .push(format!("function {namespace}:{path}"));
    }
    let objectives = plan
        .private_objectives
        .keys()
        .map(|name| ObjectiveName::try_dynamic(name).map_err(|e| e.to_string()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    for handler in handlers {
        match &handler.observation.source {
            ObservationSource::Numeric(value) => value
                .validate_generated_objectives(&objectives)
                .map_err(|e| e.to_string())?,
            ObservationSource::Boolean(condition) => {
                for operand in condition.score_operands() {
                    if plan.private_objectives.contains_key(&operand.objective) {
                        return Err(format!(
                            "observation `{}` reads private sample objective `{}`",
                            handler.tracker_id, operand.objective
                        ));
                    }
                }
            }
        }
    }
    Ok(plan)
}

fn set(objective: &str, value: i32) -> Cmd {
    Cmd::ScorePlayers(ScorePlayersOp::Set {
        selector: "@s".into(),
        objective: objective.into(),
        value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handler(
        id: &str,
        path: &str,
        source: ObservationSource,
        kind: TransitionKind,
    ) -> ObservationHandler {
        ObservationHandler {
            tracker_id: id.into(),
            description: format!("consumer {id}"),
            observation: Observation {
                source,
                declarations: BTreeMap::new(),
            },
            kind,
            path: path.into(),
        }
    }

    #[test]
    fn equivalent_sources_share_one_sample_across_named_consumers() {
        let source = ObservationSource::Boolean(Condition::raw("entity @s[tag=ready]"));
        let a = handler(
            "first",
            "entered",
            source.clone(),
            TransitionKind::BecameTrue,
        );
        let b = handler("second", "left", source, TransitionKind::BecameFalse);
        let one = resolve("pack", std::slice::from_ref(&a)).unwrap();
        let both = resolve("pack", &[b.clone(), a.clone()]).unwrap();
        assert_eq!(both.functions.len(), 1);
        assert_eq!(
            one.functions[0].path, both.functions[0].path,
            "adding consumers must not reset the persistent baseline identity"
        );
        assert_eq!(both.tick_commands.len(), 1);
        assert_eq!(both, resolve("pack", &[a, b]).unwrap());
        let body = both.functions[0].commands.join("\n");
        assert_eq!(body.matches("entity @s[tag=ready]").count(), 1);
        assert!(body.contains("function pack:entered"));
        assert!(body.contains("function pack:left"));
        assert!(
            !body.contains("@a") && !body.contains("@e"),
            "the existing lifecycle owns the player scan"
        );
    }

    #[test]
    fn shared_sample_identity_does_not_depend_on_which_consumer_declares_the_source() {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        let source = ObservationSource::Numeric(StatCurve::input_raw("points"));
        let mut a = handler(
            "declares",
            "changed",
            source.clone(),
            TransitionKind::ScoreChanged,
        );
        a.observation
            .declarations
            .insert("points".into(), "dummy".into());
        let b = handler(
            "reuses",
            "increased",
            source,
            TransitionKind::ScoreIncreased,
        );
        let plan = resolve("pack", &[a, b]).unwrap();
        assert_eq!(plan.functions.len(), 1);
        assert_eq!(
            plan.load_commands
                .iter()
                .filter(|line| *line == "scoreboard objectives add points dummy")
                .count(),
            1
        );
    }

    #[test]
    fn private_baselines_are_isolated_between_pack_namespaces() {
        let source = ObservationSource::Boolean(Condition::raw("entity @s[tag=ready]"));
        let handler = handler("ready", "ready", source, TransitionKind::BecameTrue);
        let first = resolve("first", std::slice::from_ref(&handler)).unwrap();
        let second = resolve("second", &[handler]).unwrap();
        assert!(
            first
                .private_objectives
                .keys()
                .all(|name| !second.private_objectives.contains_key(name))
        );
    }

    #[test]
    fn disjunction_samples_every_alternative_before_dispatch_and_baseline_update() {
        let condition =
            Condition::raw("entity @s[tag=one]").or(Condition::raw("entity @s[tag=two]"));
        let plan = resolve(
            "pack",
            &[handler(
                "ready",
                "ready",
                ObservationSource::Boolean(condition),
                TransitionKind::BecameTrue,
            )],
        )
        .unwrap();
        let commands = &plan.functions[0].commands;
        assert!(commands[0].ends_with(" 0"));
        assert!(commands[1].contains("tag=one"));
        assert!(commands[2].contains("tag=two"));
        assert!(commands[3].contains("function pack:ready"));
        assert!(
            commands[3].contains("matches 1"),
            "first sample requires a prior baseline"
        );
        assert!(commands[4].contains("players operation"));
        assert!(commands[5].ends_with(" 1"));
    }

    #[test]
    fn numeric_sample_clears_stale_value_and_guards_dispatch_and_baseline() {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        let plan = resolve(
            "pack",
            &[handler(
                "points",
                "changed",
                ObservationSource::Numeric(StatCurve::input_raw("points")),
                TransitionKind::ScoreChanged,
            )],
        )
        .unwrap();
        let commands = &plan.functions[0].commands;
        assert!(commands[0].starts_with("scoreboard players reset @s "));
        assert!(commands[1].starts_with("function __sand_local:sand/numeric/"));
        assert!(
            commands[2].contains("store success score @s")
                && commands[2].contains("matches -2147483648..")
        );
        assert!(
            commands[3].contains("if score @s") && commands[3].contains("function pack:changed")
        );
        assert!(
            commands[4].starts_with("execute if score @s")
                && commands[4].contains("players operation")
        );
        assert!(commands[5].starts_with("execute if score @s") && commands[5].ends_with(" 1"));
    }
    #[test]
    fn distinct_sources_share_one_player_scan_with_current_player_position() {
        let plan = resolve(
            "pack",
            &[
                handler(
                    "one",
                    "one",
                    ObservationSource::Boolean(Condition::raw("entity @s[tag=one]")),
                    TransitionKind::BecameTrue,
                ),
                handler(
                    "two",
                    "two",
                    ObservationSource::Boolean(Condition::raw("entity @s[tag=two]")),
                    TransitionKind::BecameFalse,
                ),
            ],
        )
        .unwrap();
        assert_eq!(
            plan.tick_commands,
            ["function pack:__sand_observation_tick"]
        );
        let sequence = plan
            .functions
            .iter()
            .find(|f| f.path == "__sand_observation_tick")
            .unwrap();
        let calls = plan
            .functions
            .iter()
            .filter(|f| f.path.starts_with("__sand_transition/"))
            .map(|f| format!("execute at @s run function pack:{}", f.path))
            .collect::<Vec<_>>();
        assert_eq!(sequence.commands, calls);
        assert_eq!(sequence.commands.len(), 2);
        assert!(!sequence.commands.join("\n").contains("@a"));
    }
}
