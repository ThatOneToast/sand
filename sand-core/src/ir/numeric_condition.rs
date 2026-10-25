//! Numeric predicates using the canonical curve evaluator and scoreboard backend.

use super::{Actions, Cmd};
use crate::condition::ScoreCompareOp;
use crate::entity::curve::scoreboard::{NumericContext, render_lowered_curve};
use crate::entity::{FixedPoint, StatCurve};
use sand_commands::{
    CommandError, CommandResult, ConditionIr, ExecuteOp, ObjectiveName, ScoreCmp, ScoreHolder,
};
use sand_components::ResourceLocation;
use std::collections::BTreeSet;

/// Compiler-owned operands for a runtime numeric condition.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct NumericComparison {
    pub(crate) left: StatCurve,
    pub(crate) op: ScoreCompareOp,
    pub(crate) right: StatCurve,
}

impl NumericComparison {
    pub(crate) fn register(&self) -> String {
        self.reference(true)
    }

    /// Compute the same owned identity without registering discarded Actions.
    pub(crate) fn reference(&self, register: bool) -> String {
        let body = Actions(vec![Cmd::NumericComparison(Box::new(self.clone()))]);
        let prefix = "sand/conditions/numeric";
        let path = if register {
            crate::function::register_dyn_fn_dedup(prefix, body)
        } else {
            crate::function::generated_function_path(prefix, &body)
        };
        format!("{}:{path}", crate::function::SAND_LOCAL_NS)
    }

    pub(super) fn lower(&self, owner: &ResourceLocation) -> CommandResult<Vec<Cmd>> {
        let error = |detail| {
            CommandError::new(
                "numeric comparison",
                "operands",
                format!("{owner}: {detail}"),
            )
        };
        let mut fixed = FixedPoint::default();
        let constant_fits = |value: &StatCurve, scale: i64| {
            value.constant_value().is_some_and(|value| {
                let Ok(precision) = FixedPoint::new(scale, fixed.rounding(), fixed.overflow())
                else {
                    return false;
                };
                precision
                    .encode(value, &owner.to_string(), "comparison constant")
                    .is_ok_and(|stored| {
                        i32::try_from(stored.units()).is_ok() && stored.as_f64(precision) == value
                    })
            })
        };
        let exact_scale = match (
            self.left.direct_source_scale(),
            self.right.direct_source_scale(),
        ) {
            (Some(left), Some(right)) if left == right => Some(left),
            (Some(scale), _) if constant_fits(&self.right, scale) => Some(scale),
            (_, Some(scale)) if constant_fits(&self.left, scale) => Some(scale),
            _ if constant_fits(&self.left, 1) && constant_fits(&self.right, 1) => Some(1),
            _ => None,
        };
        if let Some(scale) = exact_scale {
            fixed = FixedPoint::new(scale, fixed.rounding(), fixed.overflow())
                .map_err(|detail| error(detail.to_string()))?;
        }
        let holder = ScoreHolder::fake("#comparison");
        let caller = ScoreHolder::self_();
        let mut commands = Actions::default();
        let mut results = Vec::new();
        let mut objectives = BTreeSet::new();
        let mut records = Vec::new();
        for (side, value) in [("left", &self.left), ("right", &self.right)] {
            let path = format!("{}/{side}", owner.path());
            let resource = ResourceLocation::new(owner.namespace(), &path)
                .map_err(|detail| error(detail.to_string()))?;
            let result = ObjectiveName::logical(format!("{resource}.result"));
            let context = NumericContext::new(&resource, &holder)
                .and_then(|context| context.with_input_holder(&caller))
                .map_err(|detail| error(detail.to_string()))?;
            let lowered = value
                .lower_scoreboard(result.as_str(), &resource.to_string(), fixed)
                .map_err(|detail| error(detail.to_string()))?;
            let rendered = render_lowered_curve(context, &path, &lowered)
                .map_err(|detail| error(detail.to_string()))?;
            objectives.extend(lowered.scratch_objectives().iter().cloned());
            objectives.extend(rendered.objectives);
            objectives.insert(result.clone());
            commands.extend(rendered.commands);
            records.extend(rendered.records);
            results.push(result);
        }
        // Either operand may name a raw objective used by the other side's
        // generated storage. Check the complete allocation before registration.
        self.left
            .validate_generated_objectives(&objectives)
            .map_err(|detail| error(detail.to_string()))?;
        self.right
            .validate_generated_objectives(&objectives)
            .map_err(|detail| error(detail.to_string()))?;
        crate::function::request_numeric_objectives(objectives)?;
        for record in records {
            crate::function::register_dyn_fn(record.path, record.content);
        }
        let op = match self.op {
            ScoreCompareOp::Eq => ScoreCmp::Eq,
            ScoreCompareOp::Gt => ScoreCmp::Gt,
            ScoreCompareOp::Gte => ScoreCmp::Ge,
            ScoreCompareOp::Lt => ScoreCmp::Lt,
            ScoreCompareOp::Lte => ScoreCmp::Le,
        };
        commands.0.push(Cmd::Execute {
            operations: vec![ExecuteOp::If(ConditionIr::ScoreCompare {
                left: holder.clone(),
                left_objective: results[0].to_string(),
                op,
                right: holder,
                right_objective: results[1].to_string(),
            })],
            run: Box::new(Cmd::Return(1)),
        });
        commands.0.push(Cmd::Return(0));
        Ok(commands.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_operand_cannot_alias_the_other_operands_generated_storage() {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        let owner: ResourceLocation = "test:comparison_collision".parse().unwrap();
        let right_result = ObjectiveName::logical("test:comparison_collision/right.result");
        let comparison = NumericComparison {
            left: StatCurve::input_raw(right_result.as_str()),
            op: ScoreCompareOp::Eq,
            right: StatCurve::constant(2.0),
        };
        let error = comparison.lower(&owner).unwrap_err().to_string();
        assert!(error.contains("numeric input"), "{error}");
        assert!(error.contains(right_result.as_str()), "{error}");
    }
}
