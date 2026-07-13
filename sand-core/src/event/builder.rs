#![allow(clippy::result_large_err)]
//! Fluent event builder — alternative to the `AdvancementEvent` trait.
//!
//! [`EventBuilder`] lets you describe an advancement-backed event as a value
//! rather than as a trait impl. The result is an [`EventConfig`] that can:
//!
//! - build the Minecraft [`Advancement`](crate::Advancement) component
//! - generate the reward-function prologue (revoke + guard) as command strings
//! - report the `define()` commands for all state variables declared for the event
//!
//! # When to use each API
//!
//! | Scenario | Use |
//! |---|---|
//! | `#[event]` handler with typed trigger | [`AdvancementEvent`] trait |
//! | Programmatic advancement in a `#[component]` fn | [`EventBuilder`] |
//! | Bridging: pull trait impls into value form | [`AdvancementEvent::into_config()`] |
//!
//! # Example
//!
//! ```rust,ignore
//! use sand_core::event::builder::{EventBuilder, EventConfig};
//! use sand_core::{AdvancementTrigger, ItemPredicate};
//! use sand_core::event::{EventReset, EventVisibility};
//! use sand_macros::component;
//!
//! static MANA: ScoreVar<i32> = ScoreVar::new("mana");
//!
//! fn eat_apple_config() -> EventConfig {
//!     EventBuilder::new()
//!         .trigger(AdvancementTrigger::ConsumeItem {
//!             item: Some(ItemPredicate::id("minecraft:apple")),
//!         })
//!         .guard(MANA.of("@s").lt(100))
//!         .score(&MANA)
//!         .build()
//! }
//!
//! #[component]
//! fn eat_apple_advancement() -> sand_core::Advancement {
//!     eat_apple_config().advancement("my_pack:eat_apple", "my_pack:on_eat_apple")
//! }
//! ```

use crate::AdvancementTrigger;
use crate::condition::Condition;
use crate::event::{EventId, EventReset, EventVisibility, IntoEventId};
use crate::function::IntoFunctionRef;
use crate::state::{Cooldown, Flag, ScoreVar, StorageField, StorageVar, Timer};

// ── EventConfig ───────────────────────────────────────────────────────────────

/// A complete event configuration produced by [`EventBuilder`].
///
/// `EventConfig` is a plain value — it holds everything needed to generate
/// the advancement JSON and reward-function prologue for one event.
///
/// Obtain one via [`EventBuilder::build`] or [`AdvancementEvent::into_config`].
pub struct EventConfig {
    /// The advancement trigger that Minecraft watches.
    pub trigger: AdvancementTrigger,
    /// How the advancement ID is resolved.
    pub id: EventId,
    /// When to re-arm after firing.
    pub reset: EventReset,
    /// Toast / chat visibility.
    pub visibility: EventVisibility,
    /// Optional extra condition; if false, the handler short-circuits.
    pub guard: Option<Condition>,
    /// `define()` commands for state variables this event declared.
    state_defs: Vec<String>,
}

impl EventConfig {
    // ── Advancement generation ────────────────────────────────────────────────

    /// Build the [`Advancement`](crate::Advancement) component for this event.
    ///
    /// - `advancement_id` — the advancement's resource location. Accepts a
    ///   typed [`ResourceLocation`](crate::ResourceLocation) (preferred,
    ///   pre-validated) or a raw `&str`/`String`, which is parsed and
    ///   validated here — invalid input panics with an actionable diagnostic
    ///   instead of silently producing a malformed advancement/revoke command.
    /// - `reward_fn` — the mcfunction to call. Accepts a
    ///   [`FunctionRef`](crate::resource_ref::FunctionRef) (preferred), a
    ///   [`ResourceLocation`](crate::ResourceLocation), or a raw `&str`/`String`
    ///   via [`IntoFunctionRef`].
    pub fn advancement(
        &self,
        advancement_id: impl IntoEventId,
        reward_fn: impl IntoFunctionRef,
    ) -> crate::Advancement {
        let rl = advancement_id.into_event_resource_location();

        // Visibility controls are reserved for future display attachment;
        // currently all event advancements are hidden (no display block).
        crate::Advancement::new(rl)
            .criterion("event", crate::Criterion::new(self.trigger_clone()))
            .rewards(crate::AdvancementRewards::new().function(reward_fn.into_function_id()))
    }

    // ── Reward function prologue ──────────────────────────────────────────────

    /// Generate the lines that must appear at the **start** of the reward function.
    ///
    /// The returned commands, in order:
    /// 1. `advancement revoke @s only <id>` — if `reset == AfterFire`
    /// 2. `execute unless <guard> run return 0` — if a guard was declared
    ///    (one command per OR branch — see [`Condition::execute_commands`])
    ///
    /// Prepend these to your handler function's command list.
    pub fn reward_prologue(&self, advancement_id: &str) -> Vec<String> {
        let mut cmds = Vec::new();

        if self.reset.should_revoke() {
            cmds.push(format!("advancement revoke @s only {advancement_id}"));
        }

        if let Some(ref guard) = self.guard {
            // `negated = true` → `execute unless <guard> run return 0`
            // Short-circuits (returns 0) when the guard condition is NOT met.
            for cmd in guard.execute_commands(true, "return 0") {
                cmds.push(cmd);
            }
        }

        cmds
    }

    // ── State variable management ─────────────────────────────────────────────

    /// `scoreboard objectives add …` commands for every state variable
    /// declared via [`EventBuilder::score`], [`EventBuilder::flag`], etc.
    ///
    /// Call these in your `#[component(Load)]` function to ensure all
    /// objectives / storage paths exist before the event fires.
    pub fn state_defines(&self) -> &[String] {
        &self.state_defs
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Clone-by-JSON-roundtrip for `AdvancementTrigger` (no Clone derive yet).
    fn trigger_clone(&self) -> AdvancementTrigger {
        // Serialise the trigger to JSON and reconstruct as Custom so the JSON
        // is preserved exactly.  Once AdvancementTrigger derives Clone we can
        // replace this with a direct clone.
        let v = serde_json::to_value(&self.trigger).expect("trigger serialisation failed");
        let trigger_id = v["trigger"]
            .as_str()
            .expect("trigger has no 'trigger' key")
            .to_string();
        let conditions = v
            .get("conditions")
            .cloned()
            .map(sand_components::RawJson::new);

        AdvancementTrigger::Custom {
            trigger: trigger_id,
            conditions,
        }
    }
}

// ── EventBuilder ──────────────────────────────────────────────────────────────

/// Fluent builder for [`EventConfig`].
///
/// Start with [`EventBuilder::new()`], chain configuration methods, and call
/// [`build()`](EventBuilder::build) to produce an [`EventConfig`].
///
/// # Panics
///
/// [`build`](EventBuilder::build) panics if no trigger was set.
#[derive(Default)]
pub struct EventBuilder {
    trigger: Option<AdvancementTrigger>,
    id: Option<EventId>,
    reset: Option<EventReset>,
    visibility: Option<EventVisibility>,
    guard: Option<Condition>,
    state_defs: Vec<String>,
}

impl EventBuilder {
    /// Create a new builder with all fields unset (defaults applied at `build`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the advancement trigger.
    ///
    /// Accepts any value implementing `Into<AdvancementTrigger>`, including
    /// the trigger builder types from [`crate::event::trigger`] and the
    /// `AdvancementTrigger` enum variants directly.
    pub fn trigger(mut self, trigger: impl Into<AdvancementTrigger>) -> Self {
        self.trigger = Some(trigger.into());
        self
    }

    /// Set an explicit advancement resource location.
    ///
    /// If not set, the ID defaults to [`EventId::Auto`], which generates
    /// `namespace:path` from the event handler function name.
    ///
    /// Accepts a typed [`ResourceLocation`](crate::ResourceLocation)
    /// (preferred, pre-validated) or a raw `&str`/`String`, which is parsed
    /// and validated immediately — invalid input panics here rather than
    /// producing a malformed ID later. Use [`EventBuilder::try_id`] for a
    /// fallible alternative.
    pub fn id(mut self, id: impl IntoEventId) -> Self {
        self.id = Some(EventId::explicit(id));
        self
    }

    /// Fallible alternative to [`EventBuilder::id`].
    ///
    /// Returns `Err` instead of panicking when `id` is not a valid
    /// `namespace:path` resource location.
    pub fn try_id(mut self, id: impl AsRef<str>) -> Result<Self, sand_components::SandError> {
        self.id = Some(EventId::try_explicit(id)?);
        Ok(self)
    }

    /// Control when the advancement re-arms.
    ///
    /// Default: [`EventReset::AfterFire`].
    pub fn reset(mut self, reset: EventReset) -> Self {
        self.reset = Some(reset);
        self
    }

    /// Control toast / chat announcement visibility.
    ///
    /// Default: [`EventVisibility::Hidden`].
    pub fn visibility(mut self, v: EventVisibility) -> Self {
        self.visibility = Some(v);
        self
    }

    /// Add an extra condition that must be true for the handler to run.
    ///
    /// When the guard is false at reward-function execution time, the handler
    /// emits `execute unless <guard> run return 0` before user logic.
    pub fn guard(mut self, cond: Condition) -> Self {
        self.guard = Some(cond);
        self
    }

    // ── State variable declarations ───────────────────────────────────────────

    /// Declare a [`ScoreVar`] used by this event.
    ///
    /// Adds the variable's `define()` command to [`EventConfig::state_defines()`].
    pub fn score<T>(mut self, var: &ScoreVar<T>) -> Self {
        self.state_defs.push(var.define());
        self
    }

    /// Declare a [`Flag`] used by this event.
    pub fn flag(mut self, f: &Flag) -> Self {
        self.state_defs.push(f.define());
        self
    }

    /// Declare a [`Cooldown`] used by this event.
    pub fn cooldown(mut self, cd: &Cooldown) -> Self {
        self.state_defs.push(cd.define());
        self
    }

    /// Declare a [`Timer`] used by this event.
    pub fn timer(mut self, t: &Timer) -> Self {
        self.state_defs.push(t.define());
        self
    }

    /// Declare a [`StorageVar`] used by this event.
    ///
    /// `StorageVar` has no `define()` (NBT storage is created on first write),
    /// so this is a no-op for the define list — it exists for documentation.
    pub fn storage<T>(self, _var: &StorageVar<T>) -> Self {
        // NBT storage doesn't need an explicit define command.
        self
    }

    /// Declare a typed [`StorageField`] used by this event.
    ///
    /// Storage-backed fields do not need explicit scoreboard-style definition,
    /// so this is a no-op for [`EventConfig::state_defines()`]. It exists to
    /// keep event state declarations complete and typed.
    pub fn storage_field<Schema, T>(self, _field: StorageField<Schema, T>) -> Self {
        self
    }

    /// Finalise the builder into an [`EventConfig`].
    ///
    /// # Panics
    ///
    /// Panics if no trigger was set via [`trigger`](EventBuilder::trigger).
    pub fn build(self) -> EventConfig {
        EventConfig {
            trigger: self
                .trigger
                .expect("EventBuilder::build: no trigger set — call .trigger() before .build()"),
            id: self.id.unwrap_or(EventId::Auto),
            reset: self.reset.unwrap_or(EventReset::AfterFire),
            visibility: self.visibility.unwrap_or(EventVisibility::Hidden),
            guard: self.guard,
            state_defs: self.state_defs,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatapackComponent;
    use crate::state::{Flag, ScoreVar, StorageField, StorageSchema};
    use sand_components::predicates::{EntityPredicate, ItemPredicate};

    static MANA: ScoreVar<i32> = ScoreVar::new("mana");
    static CASTING: Flag = Flag::new("casting");
    #[derive(Debug)]
    struct MagicState;
    static MAGIC: StorageSchema<MagicState> = StorageSchema::new("test:players", "player.magic");
    static MAGIC_MANA: StorageField<MagicState, i32> = MAGIC.field("mana");
    static MAGIC_SCHOOL: StorageField<MagicState, String> = MAGIC.field("school");

    // ── Advancement generation ────────────────────────────────────────────────

    #[test]
    fn advancement_has_correct_trigger() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::ConsumeItem {
                item: Some(ItemPredicate::id("minecraft:golden_apple")),
            })
            .build();

        let adv = config.advancement("test:eat_apple", "test:on_eat_apple");
        let json = adv.to_json();

        assert_eq!(
            json["criteria"]["event"]["trigger"],
            "minecraft:consume_item"
        );
        assert_eq!(
            json["criteria"]["event"]["conditions"]["item"]["items"],
            serde_json::json!(["minecraft:golden_apple"])
        );
        assert_eq!(
            json["rewards"]["function"].as_str().unwrap(),
            "test:on_eat_apple"
        );
    }

    #[test]
    fn advancement_entity_kill_trigger() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::PlayerKilledEntity {
                entity: Some(EntityPredicate::type_("minecraft:ender_dragon")),
                killing_blow: None,
            })
            .build();

        let adv = config.advancement("test:slay_dragon", "test:on_slay");
        let json = adv.to_json();

        assert_eq!(
            json["criteria"]["event"]["trigger"],
            "minecraft:player_killed_entity"
        );
        assert_eq!(
            json["criteria"]["event"]["conditions"]["entity"]["type"],
            "minecraft:ender_dragon"
        );
    }

    // ── Reward prologue — revoke ──────────────────────────────────────────────

    #[test]
    fn reward_prologue_revokes_by_default() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .build();

        let cmds = config.reward_prologue("test:my_event");
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "advancement revoke @s only test:my_event");
    }

    #[test]
    fn reward_prologue_no_revoke_when_once_per_player() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .reset(EventReset::OncePerPlayer)
            .build();

        let cmds = config.reward_prologue("test:my_event");
        assert!(cmds.is_empty(), "OncePerPlayer should not revoke: {cmds:?}");
    }

    #[test]
    fn reward_prologue_no_revoke_when_manual() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .reset(EventReset::Manual)
            .build();

        let cmds = config.reward_prologue("test:my_event");
        assert!(cmds.is_empty(), "Manual should not revoke: {cmds:?}");
    }

    // ── Reward prologue — guard ───────────────────────────────────────────────

    #[test]
    fn reward_prologue_with_guard_emits_unless() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .guard(MANA.of("@s").lt(100))
            .build();

        let cmds = config.reward_prologue("test:my_event");
        // [0] = revoke, [1] = guard
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], "advancement revoke @s only test:my_event");
        assert!(
            cmds[1].contains("unless"),
            "guard must use 'unless': {}",
            cmds[1]
        );
        assert!(
            cmds[1].contains("return 0"),
            "guard must return 0: {}",
            cmds[1]
        );
        assert!(
            cmds[1].contains("mana"),
            "guard must reference mana obj: {}",
            cmds[1]
        );
    }

    #[test]
    fn reward_prologue_only_guard_no_revoke() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .reset(EventReset::Manual)
            .guard(MANA.of("@s").gte(25))
            .build();

        let cmds = config.reward_prologue("test:my_event");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("unless"), "{}", cmds[0]);
        assert!(cmds[0].contains("return 0"), "{}", cmds[0]);
    }

    // ── State defines ─────────────────────────────────────────────────────────

    #[test]
    fn state_defines_collects_score_and_flag() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .score(&MANA)
            .flag(&CASTING)
            .build();

        let defs = config.state_defines();
        assert_eq!(defs.len(), 2);
        assert!(
            defs.contains(&MANA.define()),
            "missing mana define: {defs:?}"
        );
        assert!(
            defs.contains(&CASTING.define()),
            "missing casting define: {defs:?}"
        );
    }

    #[test]
    fn state_defines_empty_by_default() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .build();

        assert!(config.state_defines().is_empty());
    }

    #[test]
    fn state_defines_accepts_typed_storage_fields() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .score(&MANA)
            .storage_field(MAGIC_MANA)
            .storage_field(MAGIC_SCHOOL)
            .build();

        assert_eq!(config.state_defines(), &[MANA.define()]);
    }

    #[test]
    fn state_defines_with_cooldown() {
        use crate::state::Ticks;
        static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));

        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .cooldown(&DASH)
            .build();

        let defs = config.state_defines();
        assert_eq!(defs.len(), 1);
        assert!(
            defs[0].contains("dash"),
            "expected dash define: {}",
            defs[0]
        );
    }

    // ── EventId and reset fields ──────────────────────────────────────────────

    #[test]
    fn explicit_id_stored() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .id("my_pack:special_event")
            .build();

        match config.id {
            EventId::Explicit(rl) => assert_eq!(rl.to_string(), "my_pack:special_event"),
            EventId::Auto => panic!("expected Explicit, got Auto"),
        }
    }

    #[test]
    fn explicit_id_accepts_typed_resource_location() {
        let rl: crate::ResourceLocation = "my_pack:special_event".parse().unwrap();
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .id(rl)
            .build();

        match config.id {
            EventId::Explicit(rl) => assert_eq!(rl.to_string(), "my_pack:special_event"),
            EventId::Auto => panic!("expected Explicit, got Auto"),
        }
    }

    #[test]
    fn try_id_rejects_invalid_id() {
        let result = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .try_id("not a valid id!");
        assert!(result.is_err());
    }

    #[test]
    fn default_id_is_auto() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .build();

        assert!(matches!(config.id, EventId::Auto));
    }

    #[test]
    fn default_reset_is_after_fire() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .build();

        assert!(config.reset.should_revoke());
    }

    #[test]
    fn default_visibility_is_hidden() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::Tick)
            .build();

        assert!(matches!(config.visibility, EventVisibility::Hidden));
    }

    // ── Trigger clone round-trip ──────────────────────────────────────────────

    #[test]
    fn trigger_clone_preserves_consume_item() {
        let config = EventBuilder::new()
            .trigger(AdvancementTrigger::ConsumeItem {
                item: Some(ItemPredicate::id("minecraft:apple")),
            })
            .build();

        // trigger_clone() must produce a trigger that serialises identically
        let original = serde_json::to_value(&config.trigger).unwrap();
        let cloned = serde_json::to_value(config.trigger_clone()).unwrap();
        assert_eq!(original, cloned, "trigger_clone must round-trip");
    }
}
