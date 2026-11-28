//! Transactional lowering for canonical State combat.
use super::{
    combat::{CombatAction, CombatField},
    runtime,
};
use crate::{
    IntoCommands,
    entity::{
        EntityCooldown, EntityScore, EntityStateField, FixedPoint, NumericStateField, StatCurve,
    },
    execute_when::when,
    ir::{Actions, Cmd, NumericWrite},
};
use sand_commands::{CommandError, CommandResult, ObjectiveName, ScoreHolder};
use sand_components::ResourceLocation;
use std::collections::{BTreeMap, BTreeSet};

impl<T: EntityStateField> CombatField<T> {
    fn objective(&self) -> String {
        match self {
            Self::Declared(field) => field.objective(),
            Self::Player(field) => field.objective.to_string(),
        }
    }

    fn resolve_schema(
        &self,
        presence: &mut BTreeMap<String, (ObjectiveName, u32)>,
        dirty: &mut BTreeSet<String>,
    ) -> CommandResult<()> {
        let field = match self {
            Self::Declared(field) => field,
            Self::Player(field) => {
                presence.insert(
                    field.presence.to_string(),
                    (field.presence.clone(), field.revision),
                );
                return Ok(());
            }
        };
        let error = |message| CommandError::new("combat", "state", message);
        let reference = field.field_reference();
        let schema = inventory::iter::<crate::state::StateDescriptor>
            .into_iter()
            .find(|schema| schema.id == reference.component)
            .ok_or_else(|| error("combat fields must belong to registered State"))?;
        if matches!(schema.scope, crate::state::StateScope::Global(_)) {
            return Err(error(
                "combat requires player, entity or living State, not global State",
            ));
        }
        if !schema.fields.iter().any(|field| {
            ObjectiveName::logical(field.objective).as_str() == reference.objective
                && field.field == reference.descriptor
        }) {
            return Err(error(
                "combat field metadata must match its registered State schema",
            ));
        }
        if matches!(
            schema.scope,
            crate::state::StateScope::Entity | crate::state::StateScope::Living
        ) {
            dirty.insert(reference.component);
        }
        presence.insert(
            schema.id.into(),
            (
                ObjectiveName::logical(schema.presence_objective),
                schema.version,
            ),
        );
        Ok(())
    }
}

impl CombatField<EntityScore> {
    fn scale(&self) -> i32 {
        match self {
            Self::Declared(field) => field.numeric_scale(),
            Self::Player(_) => 1,
        }
    }

    fn bounds(&self) -> Option<(i32, i32)> {
        match self {
            Self::Declared(field) => field.descriptor().bounds,
            Self::Player(field) => field.bounds,
        }
    }

    fn read(&self) -> StatCurve {
        match self {
            Self::Declared(field) => StatCurve::state(*field),
            Self::Player(field) => {
                StatCurve::runtime_score(ScoreHolder::self_(), field.objective.clone(), 1)
            }
        }
    }

    fn write(&self, value: StatCurve, dirty: &BTreeSet<String>) -> Actions {
        match self {
            Self::Declared(field) => field
                .bind_to("@s", dirty.contains(&field.component_id()))
                .set(value),
            Self::Player(field) => Cmd::NumericWrite(Box::new(NumericWrite {
                value,
                holder: ScoreHolder::self_(),
                objective: field.objective.clone(),
                scale: 1,
                fixed: FixedPoint::default(),
                bounds: field.bounds,
                dirty: vec![],
            }))
            .into_commands(),
        }
    }
}

impl CombatField<EntityCooldown> {
    fn start(&self, duration: crate::state::Ticks, dirty: &BTreeSet<String>) -> Actions {
        match self {
            Self::Declared(field) => field
                .bind_to("@s", dirty.contains(&field.component_id()))
                .start(duration)
                .into_commands(),
            Self::Player(field) => field.set(duration.get() as i32).into_commands(),
        }
    }
}

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
        let scale = policy.health.scale();
        for field in std::iter::once(&policy.health).chain(&policy.shield) {
            if field.scale() != scale
                || field.bounds().is_some_and(|(min, max)| min > 0 || max <= 0)
            {
                return Err(error(
                    "state",
                    "health and shield must share a storage scale and bounds admitting zero and positive values",
                ));
            }
        }
        if policy
            .shield
            .as_ref()
            .is_some_and(|shield| shield.objective() == policy.health.objective())
        {
            return Err(error(
                "shield",
                "shield and health must be distinct State fields",
            ));
        }
        if policy
            .immunity
            .as_ref()
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
        let mut presence = BTreeMap::new();
        let mut dirty = BTreeSet::new();
        for field in std::iter::once(&policy.health).chain(&policy.shield) {
            field.resolve_schema(&mut presence, &mut dirty)?;
            if matches!(field, CombatField::Player(field) if field.timer) {
                return Err(error(
                    "state",
                    "health and shield require non-ticking player scores",
                ));
            }
        }
        if let Some((cooldown, _)) = &policy.immunity {
            cooldown.resolve_schema(&mut presence, &mut dirty)?;
            if matches!(cooldown, CombatField::Player(field) if !field.timer) {
                return Err(error("invulnerability", "requires a player timer"));
            }
        }
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
        for (presence, revision) in presence.values() {
            body.extend([format!(
                "execute unless score @s {presence} matches {revision} {fail}"
            )]);
        }
        body.0.push(write("#health", policy.health.read()));
        body.extend([format!(
            "execute unless score #health {objective} matches 1.. {fail}"
        )]);
        if !self.healing
            && let Some((cooldown, _)) = &policy.immunity
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
                .bounds()
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
            if let Some(shield) = &policy.shield {
                body.0.push(write("#shield", shield.read()));
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
        body.extend([policy.health.write(read("#health"), &dirty)]);
        if !self.healing {
            if let Some(shield) = &policy.shield {
                body.extend([shield.write(read("#shield"), &dirty)]);
            }
            if let Some((cooldown, duration)) = &policy.immunity {
                body.extend([cooldown.start(*duration, &dirty)]);
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
