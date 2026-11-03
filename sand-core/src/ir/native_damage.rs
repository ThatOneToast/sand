//! Runtime native damage shares the canonical numeric assignment backend.
use super::{Actions, Cmd, NumericWrite};
use crate::{
    IntoCommands,
    entity::{FixedPoint, StatCurve},
};
use sand_commands::{
    CommandError, CommandProfile, CommandResult, DamageAmount, DamageKind, ObjectiveName,
    ScoreHolder, Selector, Validate,
};
use sand_components::ResourceLocation;

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct NativeDamage {
    pub(crate) target: Selector,
    pub(crate) amount: StatCurve,
    pub(crate) kind: DamageKind,
    pub(crate) source: Option<Selector>,
}
impl NativeDamage {
    pub(crate) fn lower(&self, owner: &ResourceLocation) -> CommandResult<Vec<Cmd>> {
        self.target.validate(&CommandProfile::unprofiled())?;
        if let Some(source) = &self.source {
            source.validate(&CommandProfile::unprofiled())?;
        }
        let suffix = self
            .source
            .as_ref()
            .map(|source| format!(" by {source}"))
            .unwrap_or_default();
        if let Some(value) = self.amount.constant_value() {
            let command = sand_commands::__private::damage_one(
                self.target.clone(),
                DamageAmount::try_fixed(value)?,
                self.kind,
            )
            .try_run()?
            .join("\n");
            return Ok(vec![Cmd::Raw(format!("{command}{suffix}"))]);
        }
        let path = crate::function::generated_function_path(
            "sand/combat/damage",
            &Actions(vec![Cmd::NativeDamage(Box::new(self.clone()))]),
        );
        let storage = ResourceLocation::new(owner.namespace(), &path)
            .map_err(|error| CommandError::new("native damage", "identity", error.to_string()))?;
        let score = ObjectiveName::logical(format!("{storage}.amount"));
        crate::function::request_numeric_objectives([score.clone()])?;
        let holder = ScoreHolder::fake("#damage");
        // Reset first: missing sources or failed arithmetic must never replay an old hit.
        let mut body = format!("scoreboard players reset {holder} {score}").into_commands();
        body.0.push(Cmd::NumericWrite(Box::new(NumericWrite {
            value: self.amount.clone(),
            holder: holder.clone(),
            objective: score.clone(),
            scale: 1000,
            fixed: FixedPoint::default(),
            bounds: None,
            dirty: vec![],
        })));
        let reject = crate::function::register_dyn_fn_dedup(
            "sand/combat/damage_rejected",
            vec![
                format!("scoreboard players reset {holder} {score}"),
                "return 0".into(),
            ],
        );
        body.extend([format!(
            "execute unless score {holder} {score} matches 0.. run return run function __sand_local:{reject}"
        )]);
        body.extend([format!("execute store result storage {storage} amount double 0.001 run scoreboard players get {holder} {score}")]);
        body.extend([format!("scoreboard players reset {holder} {score}")]);
        let apply = crate::function::register_dyn_fn_dedup(
            "sand/combat/damage_apply",
            vec![
                format!("data remove storage {storage} amount"),
                format!(
                    "$return run damage {} $(amount) {}{suffix}",
                    self.target, self.kind
                ),
            ],
        );
        body.extend([format!(
            "return run function __sand_local:{apply} with storage {storage}"
        )]);
        crate::function::register_dyn_fn(path.clone(), body);
        Ok(vec![Cmd::Function(format!("__sand_local:{path}"))])
    }
}
