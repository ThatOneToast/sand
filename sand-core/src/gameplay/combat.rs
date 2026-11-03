//! Authoritative State combat; native health remains an explicit integration.
use crate::{
    IntoCommands,
    condition::Condition,
    entity::{EntityCooldown, EntityScore, NumericStateField, StatCurve},
    ir::{Actions, Cmd},
    state::Ticks,
};

/// Reusable combat rules over an author's canonical health and shield State.
///
/// Call actions as a target whose State is already attached. Damage checks
/// invulnerability, applies a resistance fraction and optional critical
/// multiplier, consumes shields, then reduces health. Healing caps at the
/// configured maximum and does not revive zero health. All mutations commit
/// before callbacks; nested hits therefore see committed State.
///
/// This models custom health for any entity, including players. It neither
/// intercepts vanilla damage nor writes player Health NBT. Use an archetype
/// HealthBinding for mob synchronization, or implement a deliberate custom
/// player-health presentation. Native attacks use LivingEntity::damage.
/// Shield and health must use the same storage scale and admit zero. Arithmetic
/// rounds to that scale; missing State or invalid runtime inputs reject the
/// operation without changing health, shields or cooldown. Callbacks run only
/// for positive accepted changes; they are not global Minecraft damage events.
#[derive(Clone, Debug)]
#[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Combat", module = "sand::systems",
        summary = "Reusable damage and healing rules over canonical State.",
        context = "Combat operates on attached canonical State in the current entity context. It owns no parallel health storage and does not intercept native Minecraft damage. Resolve all rules and commit State before invoking isolated callbacks.",
        minecraft = "Uses checked numeric expressions, bounded scoreboard arithmetic, State dirty tracking and isolated function return frames.",
        use_when = ["Building custom combat on State health"], avoid_when = ["Applying native health damage; use LivingEntity::damage"],
        example = "use sand::systems::Combat;",
    )]
pub struct Combat {
    pub(crate) health: EntityScore,
    pub(crate) maximum: StatCurve,
    pub(crate) shield: Option<EntityScore>,
    pub(crate) resistance: StatCurve,
    pub(crate) immunity: Option<(EntityCooldown, Ticks)>,
    pub(crate) critical: Option<(Condition, f64)>,
    pub(crate) hit: Actions,
    pub(crate) defeat: Actions,
    pub(crate) healed: Actions,
}
impl Combat {
    /// Create rules for an attached health field and runtime maximum.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Combat::new", module = "sand::systems", kind = "method",
        summary = "Create rules for an attached health field and runtime maximum.",
        context = "Combat operates on attached canonical State in the current entity context. It owns no parallel health storage and does not intercept native Minecraft damage. Resolve all rules and commit State before invoking isolated callbacks.",
        minecraft = "Uses checked numeric expressions, bounded scoreboard arithmetic, State dirty tracking and isolated function return frames.",
        use_when = ["Building custom combat on State health"], avoid_when = ["Applying native health damage; use LivingEntity::damage"],
        params(health = "Authoritative Score or FixedScore, with bounds admitting zero.", maximum = "Positive maximum health evaluated in the target context."),
        returns = "A reusable combat policy with this configuration.",
        example = "use sand::systems::Combat;",
    )]
    pub fn new(health: impl NumericStateField, maximum: impl Into<StatCurve>) -> Self {
        Self {
            health: health.erase_numeric(),
            maximum: maximum.into(),
            shield: None,
            resistance: 0.0.into(),
            immunity: None,
            critical: None,
            hit: Actions::default(),
            defeat: Actions::default(),
            healed: Actions::default(),
        }
    }
    /// Consume this State shield before reducing health.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Combat::shield", module = "sand::systems", kind = "method",
        summary = "Consume this State shield before reducing health.",
        context = "Combat operates on attached canonical State in the current entity context. It owns no parallel health storage and does not intercept native Minecraft damage. Resolve all rules and commit State before invoking isolated callbacks.",
        minecraft = "Uses checked numeric expressions, bounded scoreboard arithmetic, State dirty tracking and isolated function return frames.",
        use_when = ["Building custom combat on State health"], avoid_when = ["Applying native health damage; use LivingEntity::damage"],
        params(field = "Nonnegative shield using the health field storage scale."),
        returns = "A reusable combat policy with this configuration.",
        example = "use sand::systems::Combat;",
    )]
    pub fn shield(mut self, field: impl NumericStateField) -> Self {
        self.shield = Some(field.erase_numeric());
        self
    }
    /// Reduce damage by a runtime fraction between zero and one.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Combat::resistance", module = "sand::systems", kind = "method",
        summary = "Reduce damage by a runtime fraction between zero and one.",
        context = "Combat operates on attached canonical State in the current entity context. It owns no parallel health storage and does not intercept native Minecraft damage. Resolve all rules and commit State before invoking isolated callbacks.",
        minecraft = "Uses checked numeric expressions, bounded scoreboard arithmetic, State dirty tracking and isolated function return frames.",
        use_when = ["Building custom combat on State health"], avoid_when = ["Applying native health damage; use LivingEntity::damage"],
        params(fraction = "Resistance fraction; values outside zero through one are clamped."),
        returns = "A reusable combat policy with this configuration.",
        example = "use sand::systems::Combat;",
    )]
    pub fn resistance(mut self, fraction: impl Into<StatCurve>) -> Self {
        self.resistance = fraction.into();
        self
    }
    /// Start a canonical cooldown after a positive accepted hit.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Combat::invulnerability", module = "sand::systems", kind = "method",
        summary = "Start a canonical cooldown after a positive accepted hit.",
        context = "Combat operates on attached canonical State in the current entity context. It owns no parallel health storage and does not intercept native Minecraft damage. Resolve all rules and commit State before invoking isolated callbacks.",
        minecraft = "Uses checked numeric expressions, bounded scoreboard arithmetic, State dirty tracking and isolated function return frames.",
        use_when = ["Building custom combat on State health"], avoid_when = ["Applying native health damage; use LivingEntity::damage"],
        params(field = "Attached cooldown; its State lifecycle must tick it.", duration = "Positive number of loaded ticks."),
        returns = "A reusable combat policy with this configuration.",
        example = "use sand::systems::Combat;",
    )]
    pub fn invulnerability(mut self, field: EntityCooldown, duration: Ticks) -> Self {
        self.immunity = Some((field, duration));
        self
    }
    /// Multiply damage when a target-context condition is true.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Combat::critical", module = "sand::systems", kind = "method",
        summary = "Multiply damage when a target-context condition is true.",
        context = "Combat operates on attached canonical State in the current entity context. It owns no parallel health storage and does not intercept native Minecraft damage. Resolve all rules and commit State before invoking isolated callbacks.",
        minecraft = "Uses checked numeric expressions, bounded scoreboard arithmetic, State dirty tracking and isolated function return frames.",
        use_when = ["Building custom combat on State health"], avoid_when = ["Applying native health damage; use LivingEntity::damage"],
        params(condition = "Critical-hit condition, sampled once before mutations.", multiplier = "Finite multiplier of at least one."),
        returns = "A reusable combat policy with this configuration.",
        example = "use sand::systems::Combat;",
    )]
    pub fn critical(mut self, condition: Condition, multiplier: f64) -> Self {
        self.critical = Some((condition, multiplier));
        self
    }
    /// Append actions after a positive shield or health loss.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Combat::on_hit", module = "sand::systems", kind = "method",
        summary = "Append actions after a positive shield or health loss.",
        context = "Combat operates on attached canonical State in the current entity context. It owns no parallel health storage and does not intercept native Minecraft damage. Resolve all rules and commit State before invoking isolated callbacks.",
        minecraft = "Uses checked numeric expressions, bounded scoreboard arithmetic, State dirty tracking and isolated function return frames.",
        use_when = ["Building custom combat on State health"], avoid_when = ["Applying native health damage; use LivingEntity::damage"],
        params(actions = "Target-context actions; nested hits observe committed State."),
        returns = "A reusable combat policy with this configuration.",
        example = "use sand::systems::Combat;",
    )]
    pub fn on_hit(mut self, actions: impl IntoCommands) -> Self {
        self.hit.extend([actions]);
        self
    }
    /// Append actions when an accepted hit reduces positive health to zero.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Combat::on_defeat", module = "sand::systems", kind = "method",
        summary = "Append actions when an accepted hit reduces positive health to zero.",
        context = "Combat operates on attached canonical State in the current entity context. It owns no parallel health storage and does not intercept native Minecraft damage. Resolve all rules and commit State before invoking isolated callbacks.",
        minecraft = "Uses checked numeric expressions, bounded scoreboard arithmetic, State dirty tracking and isolated function return frames.",
        use_when = ["Building custom combat on State health"], avoid_when = ["Applying native health damage; use LivingEntity::damage"],
        params(actions = "Target-context actions, after on_hit; outcome is selected before callbacks."),
        returns = "A reusable combat policy with this configuration.",
        example = "use sand::systems::Combat;",
    )]
    pub fn on_defeat(mut self, actions: impl IntoCommands) -> Self {
        self.defeat.extend([actions]);
        self
    }
    /// Append actions after a positive health gain.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Combat::on_heal", module = "sand::systems", kind = "method",
        summary = "Append actions after a positive health gain.",
        context = "Combat operates on attached canonical State in the current entity context. It owns no parallel health storage and does not intercept native Minecraft damage. Resolve all rules and commit State before invoking isolated callbacks.",
        minecraft = "Uses checked numeric expressions, bounded scoreboard arithmetic, State dirty tracking and isolated function return frames.",
        use_when = ["Building custom combat on State health"], avoid_when = ["Applying native health damage; use LivingEntity::damage"],
        params(actions = "Target-context actions after health is committed."),
        returns = "A reusable combat policy with this configuration.",
        example = "use sand::systems::Combat;",
    )]
    pub fn on_heal(mut self, actions: impl IntoCommands) -> Self {
        self.healed.extend([actions]);
        self
    }
    /// Resolve one hit against the current entity.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Combat::damage", module = "sand::systems", kind = "method",
        summary = "Resolve one hit against the current entity.",
        context = "Combat operates on attached canonical State in the current entity context. It owns no parallel health storage and does not intercept native Minecraft damage. Resolve all rules and commit State before invoking isolated callbacks.",
        minecraft = "Uses checked numeric expressions, bounded scoreboard arithmetic, State dirty tracking and isolated function return frames.",
        use_when = ["Building custom combat on State health"], avoid_when = ["Applying native health damage; use LivingEntity::damage"],
        params(amount = "Nonnegative logical damage before resistance and critical rules."),
        returns = "Actions returning one for an accepted change and zero for rejection or no change.",
        example = "use sand::systems::Combat;",
    )]
    pub fn damage(&self, amount: impl Into<StatCurve>) -> Actions {
        self.action(amount.into(), false)
    }
    /// Restore positive health up to its runtime maximum without reviving.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Combat::heal", module = "sand::systems", kind = "method",
        summary = "Restore positive health up to its runtime maximum without reviving.",
        context = "Combat operates on attached canonical State in the current entity context. It owns no parallel health storage and does not intercept native Minecraft damage. Resolve all rules and commit State before invoking isolated callbacks.",
        minecraft = "Uses checked numeric expressions, bounded scoreboard arithmetic, State dirty tracking and isolated function return frames.",
        use_when = ["Building custom combat on State health"], avoid_when = ["Applying native health damage; use LivingEntity::damage"],
        params(amount = "Nonnegative logical healing; shields and invulnerability do not affect healing."),
        returns = "Actions returning one for a positive gain and zero for rejection or no change.",
        example = "use sand::systems::Combat;",
    )]
    pub fn heal(&self, amount: impl Into<StatCurve>) -> Actions {
        self.action(amount.into(), true)
    }
    fn action(&self, amount: StatCurve, healing: bool) -> Actions {
        Actions(vec![Cmd::Combat(Box::new(CombatAction {
            policy: self.clone(),
            amount,
            healing,
        }))])
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct CombatAction {
    pub(crate) policy: Combat,
    pub(crate) amount: StatCurve,
    pub(crate) healing: bool,
}
