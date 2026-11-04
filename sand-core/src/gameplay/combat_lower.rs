//! Transactional lowering for canonical State combat.
use super::{combat::CombatAction, runtime};
use crate::{
    IntoCommands,
    entity::{EntityStateField, FixedPoint, NumericStateField, StatCurve},
    execute_when::when,
    ir::{Actions, Cmd, NumericWrite},
};
use sand_commands::{CommandError, CommandResult, ObjectiveName, ScoreHolder};
use sand_components::ResourceLocation;

impl CombatAction {
    pub(crate) fn lower(&self, owner: &ResourceLocation) -> CommandResult<Vec<Cmd>> {
        let policy = &self.policy;
        let error = |field, message| CommandError::new("combat", field, message);
        if policy
            .maximum
            .constant_value()
            .is_some_and(|maximum| !maximum.is_finite() || maximum <= 0.0)
        {
            return Err(error(
                "maximum",
                "maximum health must be finite and positive",
            ));
        }
        let scale = policy.health.numeric_scale();
        for field in std::iter::once(policy.health).chain(policy.shield) {
            if field.numeric_scale() != scale
                || field
                    .descriptor()
                    .bounds
                    .is_some_and(|(min, max)| min > 0 || max <= 0)
            {
                return Err(error(
                    "state",
                    "health and shield must share a storage scale and bounds admitting zero and positive values",
                ));
            }
        }
        if policy
            .shield
            .is_some_and(|shield| shield.objective() == policy.health.objective())
        {
            return Err(error(
                "shield",
                "shield and health must be distinct State fields",
            ));
        }
        if policy
            .immunity
            .is_some_and(|(_, duration)| duration.get() == 0 || duration.get() > i32::MAX as u32)
        {
            return Err(error(
                "invulnerability",
                "duration must be in 1..=2147483647 ticks",
            ));
        }
        if policy
            .critical
            .as_ref()
            .is_some_and(|(_, multiplier)| !multiplier.is_finite() || *multiplier < 1.0)
        {
            return Err(error(
                "critical",
                "critical multiplier must be finite and at least one",
            ));
        }
        if self
            .amount
            .constant_value()
            .is_some_and(|amount| !amount.is_finite() || amount < 0.0)
        {
            return Err(error(
                "amount",
                "damage and healing must be finite and nonnegative",
            ));
        }
        let mut schemas = std::collections::BTreeMap::new();
        for reference in std::iter::once(policy.health.field_reference())
            .chain(policy.shield.map(|field| field.field_reference()))
            .chain(policy.immunity.map(|(field, _)| field.field_reference()))
        {
            let schema = inventory::iter::<crate::state::StateDescriptor>
                .into_iter()
                .find(|schema| schema.id == reference.component)
                .ok_or_else(|| error("state", "combat fields must belong to registered State"))?;
            if matches!(schema.scope, crate::state::StateScope::Global(_)) {
                return Err(error(
                    "state",
                    "combat requires player, entity or living State, not global State",
                ));
            }
            if !schema.fields.iter().any(|field| {
                ObjectiveName::logical(field.objective).as_str() == reference.objective
                    && field.field == reference.descriptor
            }) {
                return Err(error(
                    "state",
                    "combat field metadata must match its registered State schema",
                ));
            }
            schemas.insert(reference.component, schema);
        }
        let tracks_dirty = |component: &str| {
            matches!(
                schemas[component].scope,
                crate::state::StateScope::Entity | crate::state::StateScope::Living
            )
        };
        let path = crate::function::generated_function_path(
            "sand/combat/resolve",
            &Actions(vec![Cmd::Combat(Box::new(self.clone()))]),
        );
        let objective = ObjectiveName::logical(format!("{}:{path}", owner.namespace()));
        crate::function::request_numeric_objectives([objective.clone()])?;
        let holders = [
            "#health",
            "#shield",
            "#amount",
            "#maximum",
            "#absorbed",
            "#critical",
        ];
        let cleanup = holders
            .map(|holder| format!("scoreboard players reset {holder} {objective}"))
            .to_vec()
            .into_commands();
        let mut rejected = cleanup.clone();
        rejected.extend(["return 0"]);
        let reject = crate::function::register_dyn_fn_dedup("sand/combat/rejected", rejected);
        let fail = format!("run return run function __sand_local:{reject}");
        let read = |holder: &str| {
            StatCurve::runtime_score(ScoreHolder::fake(holder), objective.clone(), scale)
        };
        let write = |holder: &str, value: StatCurve| {
            Cmd::NumericWrite(Box::new(NumericWrite {
                value,
                holder: ScoreHolder::fake(holder),
                objective: objective.clone(),
                scale,
                fixed: FixedPoint::default(),
                bounds: None,
                dirty: vec![],
            }))
        };
        let mut body = cleanup.clone();
        for schema in schemas.values() {
            let presence = ObjectiveName::logical(schema.presence_objective);
            body.extend([format!(
                "execute unless score @s {presence} matches {} {fail}",
                schema.version
            )]);
        }
        body.0
            .push(write("#health", StatCurve::state(policy.health)));
        body.extend([format!(
            "execute unless score #health {objective} matches 1.. {fail}"
        )]);
        if !self.healing
            && let Some((cooldown, _)) = policy.immunity
        {
            // A missing cooldown is not ready. Never create missing State.
            body.extend([format!(
                "execute unless score @s {} matches 0 {fail}",
                cooldown.objective()
            )]);
        }
        let mut amount = self.amount.clone();
        if !self.healing {
            amount = StatCurve::multiply([
                amount,
                StatCurve::clamped_linear(policy.resistance.clone(), -1.0, 1.0, 0.0, 1.0),
            ]);
            if let Some((condition, multiplier)) = &policy.critical {
                let mut check = when(condition.clone()).then("return 1").into_commands();
                check.extend(["return 0"]);
                let check = crate::function::register_dyn_fn_dedup("sand/combat/critical", check);
                body.extend([format!("execute store result score #critical {objective} run function __sand_local:{check}")]);
                amount = StatCurve::multiply([
                    amount,
                    StatCurve::linear(
                        StatCurve::runtime_score(
                            ScoreHolder::fake("#critical"),
                            objective.clone(),
                            1,
                        ),
                        multiplier - 1.0,
                        1.0,
                    ),
                ]);
            }
        }
        body.0.push(write("#amount", amount));
        body.extend([format!(
            "execute unless score #amount {objective} matches 1.. {fail}"
        )]);
        if self.healing {
            body.0.push(write("#maximum", policy.maximum.clone()));
            body.extend([format!(
                "execute unless score #maximum {objective} matches 1.. {fail}"
            )]);
            if let Some((_, maximum)) = policy
                .health
                .descriptor()
                .bounds
                .filter(|(_, maximum)| *maximum < i32::MAX)
            {
                body.extend([format!("execute if score #maximum {objective} matches {}.. run scoreboard players set #maximum {objective} {maximum}", i64::from(maximum) + 1)]);
            }
            body.extend([
                format!("execute if score #health {objective} >= #maximum {objective} {fail}"),
                format!("scoreboard players operation #maximum {objective} -= #health {objective}"),
                format!("scoreboard players operation #amount {objective} < #maximum {objective}"),
                format!("scoreboard players operation #health {objective} += #amount {objective}"),
            ]);
        } else {
            if let Some(shield) = policy.shield {
                body.0.push(write("#shield", StatCurve::state(shield)));
                body.extend([
                    format!("execute unless score #shield {objective} matches 0.. {fail}"),
                    format!(
                        "scoreboard players operation #absorbed {objective} = #amount {objective}"
                    ),
                    format!(
                        "scoreboard players operation #absorbed {objective} < #shield {objective}"
                    ),
                    format!(
                        "scoreboard players operation #shield {objective} -= #absorbed {objective}"
                    ),
                    format!(
                        "scoreboard players operation #amount {objective} -= #absorbed {objective}"
                    ),
                ]);
            }
            body.extend([
                format!("scoreboard players operation #amount {objective} < #health {objective}"),
                format!("scoreboard players operation #health {objective} -= #amount {objective}"),
            ]);
        }
        // All fallible calculations precede this commit. Direct score reads
        // preserve scale exactly, and all committed values already fit bounds.
        body.extend([policy
            .health
            .bind_to("@s", tracks_dirty(&policy.health.component_id()))
            .set(read("#health"))]);
        if !self.healing {
            if let Some(shield) = policy.shield {
                body.extend([shield
                    .bind_to("@s", tracks_dirty(&shield.component_id()))
                    .set(read("#shield"))]);
            }
            if let Some((cooldown, duration)) = policy.immunity {
                body.extend([cooldown
                    .bind_to("@s", tracks_dirty(&cooldown.component_id()))
                    .start(duration)]);
            }
        }
        let mut complete = cleanup.clone();
        complete.extend([runtime::isolated(if self.healing {
            policy.healed.clone()
        } else {
            policy.hit.clone()
        })]);
        complete.extend(["return 1"]);
        let complete = crate::function::register_dyn_fn_dedup("sand/combat/complete", complete);
        if !self.healing && !policy.defeat.0.is_empty() {
            let mut defeated = cleanup;
            defeated.extend([
                runtime::isolated(policy.hit.clone()),
                runtime::isolated(policy.defeat.clone()),
            ]);
            defeated.extend(["return 1"]);
            let defeated = crate::function::register_dyn_fn_dedup("sand/combat/defeated", defeated);
            body.extend([format!("execute if score #health {objective} matches 0 run return run function __sand_local:{defeated}")]);
        }
        body.extend([format!("return run function __sand_local:{complete}")]);
        crate::function::register_dyn_fn(path.clone(), body);
        Ok(vec![Cmd::Function(format!("__sand_local:{path}"))])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        entity::{EntityCooldown, EntityScore, FixedScore},
        gameplay::Combat,
        state::Ticks,
    };

    const HEALTH: EntityScore = EntityScore::new("combat", "fighter", "health", 20, Some((0, 20)));
    fn rejects(policy: Combat, amount: f64, field: &str) {
        let action = CombatAction {
            policy,
            amount: amount.into(),
            healing: false,
        };
        let error = action.lower(&"combat:test".parse().unwrap()).unwrap_err();
        assert_eq!(error.field, field);
    }
    #[test]
    fn rejects_invalid_policies_before_generating_mutations() {
        rejects(Combat::new(HEALTH, 20).shield(HEALTH), 1.0, "shield");
        rejects(
            Combat::new(
                EntityScore::<i32>::new("combat", "fighter", "positive", 20, Some((1, 20))),
                20,
            ),
            1.0,
            "state",
        );
        rejects(
            Combat::new(HEALTH, 20).shield(FixedScore::__new(
                "combat", "fighter", "shield", 100, 0, None,
            )),
            1.0,
            "state",
        );
        rejects(
            Combat::new(HEALTH, 20).critical(crate::condition::Condition::all([]), f64::NAN),
            1.0,
            "critical",
        );
        rejects(
            Combat::new(HEALTH, 20).critical(crate::condition::Condition::all([]), 0.5),
            1.0,
            "critical",
        );
        rejects(
            Combat::new(HEALTH, 20).invulnerability(
                EntityCooldown::new("combat", "fighter", "immune"),
                Ticks::new(0),
            ),
            1.0,
            "invulnerability",
        );
        rejects(Combat::new(HEALTH, 0), 1.0, "maximum");
        for amount in [-1.0, f64::NAN, f64::INFINITY] {
            rejects(Combat::new(HEALTH, 20), amount, "amount");
        }
    }
}
