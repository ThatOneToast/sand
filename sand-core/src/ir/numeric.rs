//! Destination-aware lowering for semantic numeric assignments.

use super::{Actions, Cmd, ScorePlayersOp};
use crate::entity::curve::scoreboard::{
    NumericContext, append_scale_conversion, render_lowered_curve,
};
use crate::entity::{FixedPoint, StatCurve};
use sand_commands::{
    CommandError, CommandProfile, CommandResult, ObjectiveName, ScoreHolder, Validate,
};
use sand_components::ResourceLocation;
use std::collections::BTreeSet;

/// Compiler-owned numeric assignment retained until function export.
///
/// Gameplay State accessors supply the destination metadata. The canonical
/// curve engine owns arithmetic, scale conversion, and rounding; this node
/// only connects that engine to action composition and export resources.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct NumericWrite {
    pub(crate) value: StatCurve,
    pub(crate) holder: ScoreHolder,
    pub(crate) objective: ObjectiveName,
    pub(crate) scale: i32,
    pub(crate) fixed: FixedPoint,
    pub(crate) bounds: Option<(i32, i32)>,
    pub(crate) dirty: Vec<ObjectiveName>,
}

impl NumericWrite {
    // Removing constant clamps must not expose a terminal command's result
    // where single-command composition previously called a sequence function.
    pub(super) fn requires_sequence_boundary(&self) -> bool {
        self.value.constant_value().is_some()
            && self.dirty.is_empty()
            && sand_commands::__private::score_holder_selector(&self.holder)
                .is_none_or(|selector| selector.to_string() == "@s")
            && self.bounds.is_some_and(|(min, max)| {
                self.holder
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err()
                    || min != i32::MIN
                    || max != i32::MAX
            })
    }

    pub(crate) fn lower(&self, owner: &ResourceLocation) -> CommandResult<Vec<Cmd>> {
        let profile = CommandProfile::unprofiled();
        self.holder.validate(&profile)?;
        self.objective.validate(&profile)?;
        for objective in &self.dirty {
            objective.validate(&profile)?;
        }
        if self.scale <= 0 || self.bounds.is_some_and(|(min, max)| min > max) {
            return Err(CommandError::new(
                "numeric assignment",
                "destination",
                "destination scale and bounds must be valid",
            ));
        }

        // A selector such as @r must be evaluated once for the write, bounds,
        // and dirty marking. Evaluate inputs in the caller's context first,
        // then bind only the destination commit to the selected entity.
        let selector = sand_commands::__private::score_holder_selector(&self.holder)
            .filter(|selector| selector.to_string() != "@s")
            .cloned();
        let destination = if selector.is_some() {
            ScoreHolder::self_()
        } else {
            self.holder.clone()
        };
        let working = if selector.is_some() || self.holder.validate_single(&profile).is_err() {
            ScoreHolder::fake("#value")
        } else {
            self.holder.clone()
        };

        if let Some(value) = self.value.constant_value() {
            if value.is_nan() {
                return Err(numeric_error("numeric value must not be NaN", owner));
            }
            let mut commands = vec![Cmd::ScorePlayers(ScorePlayersOp::Set {
                selector: destination.to_string(),
                objective: self.objective.to_string(),
                value: crate::entity::state::encode_fixed(value, self.scale, self.bounds),
            })];
            // Encoding already rounds, saturates to i32, and applies destination
            // bounds. Only dependency invalidation remains at runtime; wildcard
            // constants likewise need no generated bound objectives.
            self.append_dirty(&mut commands, &destination);
            return Ok(bind_destination(commands, selector));
        }

        // Use the same identity allocator as structured dynamic helpers. A
        // conflicting generated body remains visible to export collision checks.
        let namespace_key =
            ObjectiveName::logical(format!("numeric namespace:{}", owner.namespace()));
        let path = crate::function::generated_function_path(
            &format!("sand/numeric/{namespace_key}"),
            &Actions(vec![Cmd::NumericWrite(Box::new(self.clone()))]),
        );
        let resource = ResourceLocation::new(crate::function::SAND_LOCAL_NS, &path)
            .map_err(|error| numeric_error(error, owner))?;
        let caller = ScoreHolder::self_();
        let context = NumericContext::new(&resource, &working)
            .and_then(|context| context.with_input_holder(&caller))
            .map_err(|error| numeric_error(error, owner))?;
        let result = ObjectiveName::logical(format!("{resource}.result"));
        // Direct reads already have an exact stored representation. Retain it
        // until destination conversion instead of multiplying an i32 score by
        // the default expression precision and introducing avoidable overflow.
        let fixed = if let Some(scale) = self.value.direct_source_scale() {
            FixedPoint::new(scale, self.fixed.rounding(), self.fixed.overflow())
                .map_err(|error| numeric_error(error, owner))?
        } else {
            self.fixed
        };
        let lowered = self
            .value
            .lower_scoreboard(result.as_str(), &resource.to_string(), fixed)
            .map_err(|error| numeric_error(error, owner))?;
        let rendered = render_lowered_curve(context, &path, &lowered)
            .map_err(|error| numeric_error(error, owner))?;
        let mut objectives: BTreeSet<ObjectiveName> =
            lowered.scratch_objectives().iter().cloned().collect();
        objectives.extend(rendered.objectives);
        objectives.insert(result.clone());
        let mut commands = rendered.commands;
        append_scale_conversion(
            context,
            &mut objectives,
            &mut commands,
            result.as_str(),
            fixed.scale(),
            i64::from(self.scale),
            self.fixed.rounding(),
            lowered.operations().len(),
        )
        .map_err(|error| numeric_error(error, owner))?;

        // Preserve arithmetic precision until the final destination conversion.
        let mut body = crate::IntoCommands::into_commands(commands);
        let mut commit = vec![Cmd::ScorePlayers(ScorePlayersOp::Operation {
            target: destination.to_string(),
            target_obj: self.objective.to_string(),
            op: super::ScoreOpKind::Assign,
            source: working.to_string(),
            source_obj: result.to_string(),
        })];
        self.append_bounds_and_dirty(&mut commit, &destination, &resource, &mut objectives)?;
        self.value
            .validate_generated_objectives(&objectives)
            .map_err(|error| numeric_error(error, owner))?;
        body.0.extend(bind_destination(commit, selector));
        for record in rendered.records {
            crate::function::register_dyn_fn(record.path, record.content);
        }
        crate::function::request_numeric_objectives(objectives)?;
        crate::function::register_dyn_fn(path.clone(), body);
        Ok(vec![Cmd::Function(format!(
            "{}:{path}",
            crate::function::SAND_LOCAL_NS
        ))])
    }

    fn append_bounds_and_dirty(
        &self,
        commands: &mut Vec<Cmd>,
        holder: &ScoreHolder,
        owner: &ResourceLocation,
        objectives: &mut BTreeSet<ObjectiveName>,
    ) -> CommandResult<()> {
        if let Some((min, max)) = self.bounds {
            if holder
                .validate_single(&CommandProfile::unprofiled())
                .is_err()
            {
                // Wildcard score holders cannot appear in score conditions.
                // Scoreboard min/max operations apply the bound to each target.
                for (value, op) in [
                    (min, super::ScoreOpKind::Max),
                    (max, super::ScoreOpKind::Min),
                ] {
                    let bound = ObjectiveName::logical(format!(
                        "{owner}.numeric_bound.{}.{value}",
                        self.objective
                    ));
                    objectives.insert(bound.clone());
                    commands.push(Cmd::ScorePlayers(ScorePlayersOp::Set {
                        selector: "#value".into(),
                        objective: bound.to_string(),
                        value,
                    }));
                    commands.push(Cmd::ScorePlayers(ScorePlayersOp::Operation {
                        target: holder.to_string(),
                        target_obj: self.objective.to_string(),
                        op,
                        source: "#value".into(),
                        source_obj: bound.to_string(),
                    }));
                }
            } else {
                commands.extend(score_bounds(holder, self.objective.as_str(), (min, max)));
            }
        }
        self.append_dirty(commands, holder);
        Ok(())
    }

    fn append_dirty(&self, commands: &mut Vec<Cmd>, holder: &ScoreHolder) {
        for objective in &self.dirty {
            commands.push(Cmd::ScorePlayers(ScorePlayersOp::Set {
                selector: holder.to_string(),
                objective: objective.to_string(),
                value: 1,
            }));
        }
    }
}

fn bind_destination(commands: Vec<Cmd>, selector: Option<sand_commands::Selector>) -> Vec<Cmd> {
    match selector {
        Some(selector) => vec![Cmd::Execute {
            operations: vec![sand_commands::ExecuteOp::As(selector)],
            run: Box::new(Cmd::AnonymousFunction {
                prefix: "sand/numeric_commit".into(),
                body: Actions(commands),
            }),
        }],
        None => commands,
    }
}

fn numeric_error(error: impl std::fmt::Display, owner: &ResourceLocation) -> CommandError {
    CommandError::new("numeric assignment", "value", error.to_string())
        .with_code("SAND-NUMERIC-ASSIGNMENT")
        .with_context(owner.to_string())
}

/// Lower a signed State mutation to Minecraft's nonnegative command amounts.
/// Splitting the magnitude of i32::MIN preserves wrapping arithmetic; callers
/// apply bounds and dirty marking only after the entire delta is committed.
pub(crate) fn score_delta(
    holder: &ScoreHolder,
    objective: &str,
    amount: i32,
    subtract: bool,
) -> Vec<Cmd> {
    let delta = if subtract {
        -i64::from(amount)
    } else {
        i64::from(amount)
    };
    let magnitude = delta.abs();
    let parts = if magnitude > i64::from(i32::MAX) {
        vec![i32::MAX, 1]
    } else {
        vec![magnitude as i32]
    };
    parts
        .into_iter()
        .map(|amount| {
            Cmd::ScorePlayers(if delta < 0 {
                ScorePlayersOp::Remove {
                    selector: holder.to_string(),
                    objective: objective.to_owned(),
                    amount,
                }
            } else {
                ScorePlayersOp::Add {
                    selector: holder.to_string(),
                    objective: objective.to_owned(),
                    amount,
                }
            })
        })
        .collect()
}

/// Clamp a single score holder using the canonical inclusive State bounds.
/// Callers validate bounds and bind selectors once before composing a write.
pub(crate) fn score_bounds(
    holder: &ScoreHolder,
    objective: &str,
    (min, max): (i32, i32),
) -> Vec<Cmd> {
    [
        (min.checked_sub(1).map(|limit| format!("..{limit}")), min),
        (max.checked_add(1).map(|limit| format!("{limit}..")), max),
    ]
    .into_iter()
    .filter_map(|(range, value)| {
        range.map(|range| Cmd::Execute {
            operations: vec![sand_commands::ExecuteOp::If(
                sand_commands::ConditionIr::ScoreMatches {
                    holder: holder.clone(),
                    objective: objective.to_owned(),
                    range,
                },
            )],
            run: Box::new(Cmd::ScorePlayers(ScorePlayersOp::Set {
                selector: holder.to_string(),
                objective: objective.to_owned(),
                value,
            })),
        })
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{EntityStateField, FixedScore};
    use crate::function::ExportFunctionRegistryScope;

    fn assignment(value: StatCurve) -> Actions {
        Actions(vec![Cmd::NumericWrite(Box::new(NumericWrite {
            value,
            holder: ScoreHolder::self_(),
            objective: ObjectiveName::minecraft("result"),
            scale: 100,
            fixed: FixedPoint::default(),
            bounds: Some((0, 500)),
            dirty: vec![ObjectiveName::minecraft("result_dirty")],
        }))])
    }

    #[test]
    fn constants_encode_once_without_bounds_storage_and_keep_dirty_order() {
        // Expected stored values are explicit, independent of encode_fixed.
        let cases = [
            (4.0, 1, Some((0, 10)), 4),
            (-1.0, 1, Some((0, 10)), 0),
            (11.0, 1, Some((0, 10)), 10),
            (1.5, 1, None, 2),
            (-1.5, 1, None, -2),
            (0.875, 100, Some((-200, 500)), 88),
            (-0.875, 100, Some((-200, 500)), -88),
            (-9.0, 100, Some((-200, 500)), -200),
            (9.0, 100, Some((-200, 500)), 500),
            (f64::from(i32::MAX), 1, None, i32::MAX),
            (f64::from(i32::MIN), 1, None, i32::MIN),
            (f64::from(i32::MAX) + 1.0, 1, None, i32::MAX),
            (f64::from(i32::MIN) - 1.0, 1, None, i32::MIN),
            (f64::MAX, 100, None, i32::MAX),
            (-f64::MAX, 100, None, i32::MIN),
            (f64::INFINITY, 100, Some((-200, 500)), 500),
            (f64::NEG_INFINITY, 100, Some((-200, 500)), -200),
            (42.0, 100, Some((7, 7)), 7),
            (f64::from(i32::MAX), 1, Some((i32::MIN, i32::MAX)), i32::MAX),
        ];
        for holder in ["@s", "#fake", "*"] {
            for (value, scale, bounds, expected) in cases {
                for dirty in [
                    vec![],
                    vec![
                        ObjectiveName::minecraft("field_dirty"),
                        ObjectiveName::minecraft("component_dirty"),
                    ],
                ] {
                    let _scope = ExportFunctionRegistryScope::enter();
                    let write = NumericWrite {
                        value: StatCurve::constant(value),
                        holder: sand_commands::__private::score_holder_compat(holder.into()),
                        objective: ObjectiveName::minecraft("result"),
                        scale,
                        fixed: FixedPoint::default(),
                        bounds,
                        dirty: dirty.clone(),
                    };
                    let body = Actions(vec![Cmd::NumericWrite(Box::new(write))])
                        .lower(&"test:constant".parse().unwrap())
                        .unwrap();
                    let mut expected_lines =
                        vec![format!("scoreboard players set {holder} result {expected}")];
                    expected_lines.extend(
                        dirty.iter().map(|objective| {
                            format!("scoreboard players set {holder} {objective} 1")
                        }),
                    );
                    assert_eq!(body, expected_lines, "{holder}: {value} at scale {scale}");
                    assert!(crate::function::generated_objective_owners().is_empty());
                    assert!(crate::state::score::drain_internal_score_setup().is_empty());
                    assert!(crate::function::drain_dyn_fns().is_empty());
                }
            }
        }
    }

    #[test]
    fn constant_destination_validation_precedes_emission() {
        let valid = NumericWrite {
            value: StatCurve::constant(1.0),
            holder: ScoreHolder::self_(),
            objective: ObjectiveName::minecraft("result"),
            scale: 1,
            fixed: FixedPoint::default(),
            bounds: Some((0, 10)),
            dirty: vec![],
        };
        let invalid = [
            NumericWrite {
                scale: 0,
                ..valid.clone()
            },
            NumericWrite {
                bounds: Some((10, 0)),
                ..valid.clone()
            },
            NumericWrite {
                objective: ObjectiveName::minecraft("bad objective"),
                ..valid.clone()
            },
            NumericWrite {
                dirty: vec![ObjectiveName::minecraft("bad objective")],
                ..valid.clone()
            },
            NumericWrite {
                holder: ScoreHolder::fake("bad holder"),
                ..valid.clone()
            },
            NumericWrite {
                value: StatCurve::constant(f64::NAN),
                ..valid
            },
        ];
        for write in invalid {
            let _scope = ExportFunctionRegistryScope::enter();
            assert!(write.lower(&"test:invalid".parse().unwrap()).is_err());
            assert!(crate::function::generated_objective_owners().is_empty());
            assert!(crate::state::score::drain_internal_score_setup().is_empty());
            assert!(crate::function::drain_dyn_fns().is_empty());
        }
    }

    #[test]
    fn nested_numeric_assignment_provisions_late_helpers_and_preserves_precision() {
        fn export() -> (Vec<crate::component::ComponentRecord>, Vec<String>) {
            let _scope = ExportFunctionRegistryScope::enter();
            let input = FixedScore::__new("test", "combat", "power", 1000, 0, None);
            let body = assignment(StatCurve::from(input.bind()));
            crate::function::register_dyn_fn("outer".into(), body);
            // The numeric operation is not encountered until a dynamic helper
            // is lowered. Initialization must be collected after this phase.
            let mut records = Vec::new();
            crate::compiler::export::functions::drain_generated_resources_into(
                &mut records,
                "test",
                None,
                &mut Default::default(),
            )
            .unwrap();
            let setup = crate::state::score::drain_internal_score_setup();
            assert!(!setup.is_empty());
            let numeric = records
                .iter()
                .find(|record| record.path.starts_with("sand/numeric/"))
                .unwrap();
            assert!(numeric.content.contains(&format!(
                "run scoreboard players get @s {}",
                input.objective()
            )));
            let write = numeric
                .content
                .find("scoreboard players operation @s result =")
                .unwrap();
            let clamp = numeric
                .content
                .find("execute if score @s result matches 501..")
                .unwrap();
            let dirty = numeric
                .content
                .find("scoreboard players set @s result_dirty 1")
                .unwrap();
            let failure = numeric.content.find("run return fail").unwrap();
            assert!(failure < write && write < clamp && clamp < dirty);
            assert!(
                numeric.content[..write].contains("/="),
                "scale conversion must precede the destination write"
            );
            assert!(
                records
                    .iter()
                    .find(|record| record.path == "outer")
                    .unwrap()
                    .content
                    .starts_with("function __sand_local:sand/numeric/")
            );
            (records, setup)
        }
        assert_eq!(export(), export());
    }

    #[test]
    fn selected_destination_is_bound_once_without_rebinding_source_reads() {
        for constant in [false, true] {
            let _scope = ExportFunctionRegistryScope::enter();
            let source = FixedScore::__new("test", "combat", "source", 100, 0, None);
            let target = FixedScore::__new("test", "combat", "target", 100, 0, Some((0, 500)));
            let destination = target.bind_to("@r", true);
            let body = if constant {
                destination.set(1.25)
            } else {
                destination.set(source.bind())
            };
            crate::function::register_dyn_fn("outer".into(), body);
            let mut records = Vec::new();
            crate::compiler::export::functions::drain_generated_resources_into(
                &mut records,
                "test",
                None,
                &mut Default::default(),
            )
            .unwrap();
            let all = records
                .iter()
                .map(|record| record.content.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            assert_eq!(
                all.matches("@r").count(),
                1,
                "the destination selector must run only once"
            );
            assert!(all.contains("execute as @r run function"));
            let commit = records
                .iter()
                .find(|record| record.path.starts_with("sand/numeric_commit/"))
                .unwrap();
            assert!(
                commit
                    .content
                    .contains(&format!("@s {}", target.objective()))
            );
            assert!(commit.content.contains(&format!(
                "scoreboard players set @s {} 1",
                target.dirty_objective()
            )));
            if !constant {
                let numeric = records
                    .iter()
                    .find(|record| record.path.starts_with("sand/numeric/"))
                    .unwrap();
                assert!(
                    numeric.content.contains(&format!(
                        "run scoreboard players get @s {}",
                        source.objective()
                    )),
                    "source reads must retain the original executor before destination binding"
                );
                assert!(
                    commit.content.contains("= #value "),
                    "the bound commit reads stable scratch"
                );
            }
        }
    }

    #[test]
    fn multi_entity_and_wildcard_destinations_preserve_assignment_support() {
        for holder in ["@a", "@e", "*"] {
            for constant in [false, true] {
                let _scope = ExportFunctionRegistryScope::enter();
                let field = FixedScore::__new("test", "combat", "target", 100, 0, Some((0, 500)));
                let destination = field.bind_to(holder, true);
                let actions = if constant {
                    destination.set(10)
                } else {
                    destination.set(field.bind())
                };
                crate::function::register_dyn_fn("outer".into(), actions);
                let mut records = Vec::new();
                crate::compiler::export::functions::drain_generated_resources_into(
                    &mut records,
                    "test",
                    None,
                    &mut Default::default(),
                )
                .unwrap();
                let all = records
                    .iter()
                    .map(|record| record.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                if holder == "*" {
                    assert!(
                        !all.contains("score *"),
                        "wildcards are not legal score conditions"
                    );
                    assert_eq!(
                        all.contains(&format!(
                            "scoreboard players operation * {} > #value ",
                            field.objective()
                        )),
                        !constant
                    );
                    assert_eq!(
                        all.contains(&format!(
                            "scoreboard players operation * {} < #value ",
                            field.objective()
                        )),
                        !constant
                    );
                    assert!(all.contains(&format!(
                        "scoreboard players set * {} 1",
                        field.dirty_objective()
                    )));
                } else {
                    assert_eq!(
                        all.matches(&format!("execute as {holder} run function"))
                            .count(),
                        1
                    );
                    let commit = records
                        .iter()
                        .find(|record| record.path.starts_with("sand/numeric_commit/"))
                        .unwrap();
                    assert_eq!(
                        commit
                            .content
                            .contains(&format!("if score @s {} matches 501..", field.objective())),
                        !constant
                    );
                    assert!(commit.content.contains(&format!(
                        "scoreboard players set @s {} 1",
                        field.dirty_objective()
                    )));
                }
            }
        }
    }

    #[test]
    fn bounded_wildcard_assignment_reuses_one_helper_across_callers() {
        let _scope = ExportFunctionRegistryScope::enter();
        let field = FixedScore::__new("test", "combat", "shared_target", 100, 0, Some((0, 500)));
        let action = field.bind_to("*", true).set(field.bind());
        for path in ["first", "second"] {
            crate::function::register_dyn_fn(path.into(), action.clone());
        }
        let mut records = Vec::new();
        crate::compiler::export::functions::drain_generated_resources_into(
            &mut records,
            "test",
            None,
            &mut Default::default(),
        )
        .unwrap();
        crate::compiler::export::records::validate_unique_output_identities(&records).unwrap();
        let caller = |path| {
            &records
                .iter()
                .find(|record| record.path == path)
                .unwrap()
                .content
        };
        assert_eq!(caller("first"), caller("second"));
        assert_eq!(
            records
                .iter()
                .filter(|record| record.path.starts_with("sand/numeric/"))
                .count(),
            1
        );
    }

    #[test]
    fn raw_numeric_inputs_are_validated_before_helper_emission() {
        for input in ["", "bad objective", &"x".repeat(32_768), "bad\nobjective"] {
            for curve in [
                StatCurve::input_raw(input),
                StatCurve::lookup_raw(input, [(1, 2.0)], 0.0),
                StatCurve::flag_mapping_raw(input, 0.0, 1.0),
            ] {
                let _scope = ExportFunctionRegistryScope::enter();
                let error = assignment(curve)
                    .lower(&"game:raw_input".parse().unwrap())
                    .unwrap_err()
                    .to_string();
                assert!(error.contains("invalid numeric input objective"), "{error}");
                assert!(error.contains("game:raw_input"), "{error}");
                assert!(crate::function::drain_dyn_fns().is_empty());
                assert!(crate::function::generated_objective_owners().is_empty());
            }
        }
    }

    #[test]
    fn generated_storage_cannot_alias_undeclared_curve_inputs() {
        let scratch = ObjectiveName::logical("game:raw_input.curve.0");
        let generated = BTreeSet::from([scratch.clone()]);
        for curve in [
            StatCurve::input_raw(scratch.as_str()),
            StatCurve::lookup_raw(scratch.as_str(), [(1, 2.0)], 0.0),
            StatCurve::flag_mapping_raw(scratch.as_str(), 0.0, 1.0),
            StatCurve::add([
                StatCurve::constant(1.0),
                StatCurve::input_raw(scratch.as_str()),
            ]),
        ] {
            let lowering_error = curve
                .lower_scoreboard("result", "game:raw_input", FixedPoint::default())
                .unwrap_err()
                .to_string();
            assert!(
                lowering_error.contains(scratch.as_str()),
                "{lowering_error}"
            );
            let error = curve
                .validate_generated_objectives(&generated)
                .unwrap_err()
                .to_string();
            assert!(error.contains(scratch.as_str()), "{error}");
            assert!(error.contains(scratch.logical_name()), "{error}");
            assert!(error.contains("numeric input"), "{error}");
        }
        StatCurve::input_raw("external")
            .validate_generated_objectives(&generated)
            .unwrap();
    }

    #[test]
    fn logical_objective_collisions_are_rejected_and_export_scoped() {
        let emitted;
        {
            let _scope = ExportFunctionRegistryScope::enter();
            let first = ObjectiveName::logical("test:a deliberately long numeric scratch identity");
            emitted = first.to_string();
            let second = ObjectiveName::logical(emitted.clone());
            assert_ne!(first.logical_name(), second.logical_name());
            assert_eq!(first.as_str(), second.as_str());
            crate::function::request_numeric_objectives([first]).unwrap();
            let error = crate::function::request_numeric_objectives([second]).unwrap_err();
            assert!(error.to_string().contains("collides between"));
        }
        let _scope = ExportFunctionRegistryScope::enter();
        assert!(crate::function::generated_objective_owners().is_empty());
        crate::function::request_numeric_objectives([ObjectiveName::logical(emitted)]).unwrap();
    }

    #[test]
    fn numeric_validation_is_deferred_and_retains_gameplay_owner() {
        let _scope = ExportFunctionRegistryScope::enter();
        let body = assignment(StatCurve::constant(f64::NAN));
        let error = body
            .lower(&"game:calculate".parse().unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.contains("game:calculate"));
        assert!(error.contains("numeric assignment"));
        assert!(error.contains("actions[0].value"));
        assert!(crate::function::drain_dyn_fns().is_empty());
        assert!(crate::state::score::drain_internal_score_setup().is_empty());
    }

    #[test]
    fn numeric_scratch_isolated_between_pack_namespaces() {
        fn objectives(namespace: &str) -> BTreeSet<String> {
            let _scope = ExportFunctionRegistryScope::enter();
            assignment(StatCurve::from(
                FixedScore::__new("test", "combat", "power", 100, 0, None).bind(),
            ))
            .lower(&ResourceLocation::new(namespace, "calculate").unwrap())
            .unwrap();
            crate::function::take_numeric_objectives()
        }
        let first = objectives("first");
        let second = objectives("second");
        assert!(!first.is_empty());
        assert!(first.is_disjoint(&second));
    }

    #[test]
    fn numeric_initialization_does_not_escape_an_export_scope() {
        {
            let _scope = ExportFunctionRegistryScope::enter();
            assignment(StatCurve::from(
                FixedScore::__new("test", "combat", "power", 100, 0, None).bind(),
            ))
            .lower(&"game:calculate".parse().unwrap())
            .unwrap();
            // Simulate a later export failure before either registry is drained.
        }
        let _scope = ExportFunctionRegistryScope::enter();
        assert!(crate::function::drain_dyn_fns().is_empty());
        assert!(crate::state::score::drain_internal_score_setup().is_empty());
    }
}

#[cfg(test)]
#[path = "numeric/result_tests.rs"]
mod result_tests;
