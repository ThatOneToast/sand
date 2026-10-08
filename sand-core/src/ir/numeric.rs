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
    pub(super) fn lower(&self, owner: &ResourceLocation) -> CommandResult<Vec<Cmd>> {
        let profile = CommandProfile::unprofiled();
        self.holder.validate_single(&profile)?;
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
        let working = if selector.is_some() {
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
            self.append_bounds_and_dirty(&mut commands, &destination);
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
        let mut objectives: BTreeSet<String> =
            lowered.scratch_objectives().iter().cloned().collect();
        objectives.extend(rendered.objectives);
        objectives.insert(result.as_str().to_owned());
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
        self.append_bounds_and_dirty(&mut commit, &destination);
        body.0.extend(bind_destination(commit, selector));
        for record in rendered.records {
            crate::function::register_dyn_fn(record.path, record.content);
        }
        crate::function::request_numeric_objectives(objectives);
        crate::function::register_dyn_fn(path.clone(), body);
        Ok(vec![Cmd::Function(format!(
            "{}:{path}",
            crate::function::SAND_LOCAL_NS
        ))])
    }

    fn append_bounds_and_dirty(&self, commands: &mut Vec<Cmd>, holder: &ScoreHolder) {
        if let Some((min, max)) = self.bounds {
            for (range, value) in [
                (min.checked_sub(1).map(|limit| format!("..{limit}")), min),
                (max.checked_add(1).map(|limit| format!("{limit}..")), max),
            ] {
                if let Some(range) = range {
                    commands.push(Cmd::Execute {
                        operations: vec![sand_commands::ExecuteOp::If(
                            sand_commands::ConditionIr::ScoreMatches {
                                holder: holder.clone(),
                                objective: self.objective.to_string(),
                                range,
                            },
                        )],
                        run: Box::new(Cmd::ScorePlayers(ScorePlayersOp::Set {
                            selector: holder.to_string(),
                            objective: self.objective.to_string(),
                            value,
                        })),
                    });
                }
            }
        }
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
    fn nested_numeric_assignment_provisions_late_helpers_and_preserves_precision() {
        fn export() -> (Vec<crate::component::ComponentRecord>, Vec<String>) {
            let _scope = ExportFunctionRegistryScope::enter();
            let input = FixedScore::__new("test", "combat", "power", 1000, 0, None);
            let body = assignment(StatCurve::from(input.bind()));
            crate::function::register_dyn_fn("outer".into(), body);
            // The numeric operation is not encountered until a dynamic helper
            // is lowered. Initialization must be collected after this phase.
            let mut records = Vec::new();
            crate::compiler::export::functions::drain_dynamic_functions_into(&mut records, "test")
                .unwrap();
            let setup = crate::state::score::drain_internal_score_setup();
            assert!(!setup.is_empty());
            let numeric = records
                .iter()
                .find(|record| record.path.starts_with("sand/numeric/"))
                .unwrap();
            assert!(
                numeric
                    .content
                    .contains(&format!("= @s {}", input.objective()))
            );
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
            assert!(write < clamp && clamp < dirty);
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
            crate::compiler::export::functions::drain_dynamic_functions_into(&mut records, "test")
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
                    numeric
                        .content
                        .contains(&format!("= @s {}", source.objective())),
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
