//! Typed trigger builders for Minecraft advancement triggers.
//!
//! Each trigger type has a builder with typed methods. All builders implement
//! `Into<AdvancementTrigger>` so they work directly with
//! [`crate::event::AdvancementEvent`]'s `Trigger` associated type.
//!
//! # Typed predicates
//!
//! Use [`ItemPredicate`] and [`EntityPredicate`] from the prelude for type-safe
//! trigger filters.  To pass raw JSON as an escape hatch, wrap it with
//! `ItemPredicate::raw(RawJson::new(json!({...})))`.
//!
//! ```rust,ignore
//! use sand_core::event::trigger::ConsumeItemTrigger;
//! use sand_core::{ItemPredicate, RawJson};
//! use serde_json::json;
//!
//! // Typed (preferred):
//! let trigger = ConsumeItemTrigger::new()
//!     .item(ItemPredicate::id(sand_core::generated::Item::GoldenApple))
//!     .build();
//!
//! // Raw JSON escape hatch:
//! let trigger = ConsumeItemTrigger::new()
//!     .item(ItemPredicate::raw(RawJson::new(json!({"items": "minecraft:golden_apple"}))))
//!     .build();
//! ```

use crate::AdvancementTrigger;
use sand_components::ItemId;
use sand_components::advancement::InventorySlotsPredicate;
use sand_components::predicates::{DamagePredicate, EntityPredicate, IntRange, ItemPredicate};

// ── TickTrigger ─────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::TickTrigger",
    aliases = ["sand::prelude::TickTrigger"],
    module = "sand::event",
    summary = "Fires every tick (20 times per second). Commonly used for join detection (with revoke) or per-tick checks.",
    context = "Fires every tick (20 times per second). Commonly used for join detection (with revoke) or per-tick checks. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "The criterion is checked by Minecraft every tick and normally needs an explicit guard to avoid unconditional dispatch.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::TickTrigger;",
)]
/// Fires every tick (20 times per second).
///
/// Commonly used for join detection (with revoke) or per-tick checks.
#[derive(Clone, Debug, Default)]
pub struct TickTrigger;

impl TickTrigger {
    /// Starts an unconstrained tick trigger builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::TickTrigger::new",
        aliases = ["sand::prelude::TickTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Starts an unconstrained tick trigger builder.",
        context = "Starts an unconstrained tick trigger builder. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The resulting criterion matches each player tick.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "A `TickTrigger` initialized to an unconstrained tick trigger builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let tick_trigger = sand::event::trigger::TickTrigger::new();\n}",
    )]
    pub fn new() -> Self {
        Self
    }

    /// Converts this tick builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::TickTrigger::build",
        aliases = ["sand::prelude::TickTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts this tick builder into an advancement criterion.",
        context = "Converts this tick builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes as Minecraft's tick trigger.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert this tick builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tick_trigger_value: sand::event::trigger::TickTrigger)  {\n    let build = tick_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::Tick
    }
}

impl From<TickTrigger> for AdvancementTrigger {
    fn from(_: TickTrigger) -> Self {
        AdvancementTrigger::Tick
    }
}

// ── ImpossibleTrigger ───────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::ImpossibleTrigger",
    aliases = ["sand::prelude::ImpossibleTrigger"],
    module = "sand::event",
    summary = "Never fires. Useful for placeholder or parent-only advancements.",
    context = "Never fires. Useful for placeholder or parent-only advancements. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "The criterion never fires without explicit advancement grant control.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::ImpossibleTrigger;",
)]
/// Never fires. Useful for placeholder or parent-only advancements.
#[derive(Clone, Debug, Default)]
pub struct ImpossibleTrigger;

impl ImpossibleTrigger {
    /// Starts a never-matching trigger builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::ImpossibleTrigger::new",
        aliases = ["sand::prelude::ImpossibleTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Starts a never-matching trigger builder.",
        context = "Starts a never-matching trigger builder. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It has no matching vanilla action.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "An `ImpossibleTrigger` initialized to a never-matching trigger builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let impossible_trigger = sand::event::trigger::ImpossibleTrigger::new();\n}",
    )]
    pub fn new() -> Self {
        Self
    }

    /// Converts this never-matching builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::ImpossibleTrigger::build",
        aliases = ["sand::prelude::ImpossibleTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts this never-matching builder into an advancement criterion.",
        context = "Converts this never-matching builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes as Minecraft's impossible trigger.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert this never-matching builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(impossible_trigger_value: sand::event::trigger::ImpossibleTrigger)  {\n    let build = impossible_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::Impossible
    }
}

impl From<ImpossibleTrigger> for AdvancementTrigger {
    fn from(_: ImpossibleTrigger) -> Self {
        AdvancementTrigger::Impossible
    }
}

// ── ConsumeItemTrigger ──────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::ConsumeItemTrigger",
    aliases = ["sand::prelude::ConsumeItemTrigger"],
    module = "sand::event",
    summary = "Fires when the player consumes an item (food, potion, honey bottle, etc.).",
    context = "Fires when the player consumes an item (food, potion, honey bottle, etc.). This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "Uses minecraft:consume_item and can constrain the consumed item with an item predicate.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::ConsumeItemTrigger;",
)]
/// Fires when the player consumes an item (food, potion, honey bottle, etc.).
#[derive(Clone, Debug, Default)]
pub struct ConsumeItemTrigger {
    item: Option<ItemPredicate>,
}

impl ConsumeItemTrigger {
    /// Starts an unconstrained consume-item criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::ConsumeItemTrigger::new",
        aliases = ["sand::prelude::ConsumeItemTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Starts an unconstrained consume-item criterion.",
        context = "Starts an unconstrained consume-item criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Matches any item consumption until narrowed with item.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "A `ConsumeItemTrigger` initialized to an unconstrained consume-item criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let consume_item_trigger = sand::event::trigger::ConsumeItemTrigger::new();\n}",
    )]
    pub fn new() -> Self {
        Self { item: None }
    }

    /// Filter by the consumed item.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::ConsumeItemTrigger::item",
        aliases = ["sand::prelude::ConsumeItemTrigger::item"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by the consumed item.",
        context = "Filter by the consumed item. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the predicate against the consumed stack.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to filter by the consumed item."),
        returns = "The `ConsumeItemTrigger` value with the documented change applied to filter by the consumed item.",
        example = "use sand::prelude::*;\n\nfn demonstrate(consume_item_trigger_value: sand::event::trigger::ConsumeItemTrigger, predicate: sand::predicate::ItemPredicate)  {\n    let updated_consume_item_trigger = consume_item_trigger_value.item(predicate);\n}",
    )]
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    /// Converts the consume-item builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::ConsumeItemTrigger::build",
        aliases = ["sand::prelude::ConsumeItemTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts the consume-item builder into an advancement criterion.",
        context = "Converts the consume-item builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes the selected consume-item conditions into advancement JSON.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert the consume-item builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(consume_item_trigger_value: sand::event::trigger::ConsumeItemTrigger)  {\n    let build = consume_item_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::ConsumeItem { item: self.item }
    }
}

impl From<ConsumeItemTrigger> for AdvancementTrigger {
    fn from(t: ConsumeItemTrigger) -> Self {
        t.build()
    }
}

// ── PlayerKilledEntityTrigger ───────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::PlayerKilledEntityTrigger",
    aliases = ["sand::prelude::PlayerKilledEntityTrigger"],
    module = "sand::event",
    summary = "Fires when the player kills any entity.",
    context = "Fires when the player kills any entity. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "Uses minecraft:player_killed_entity with optional victim and killing-blow predicates.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::PlayerKilledEntityTrigger;",
)]
/// Fires when the player kills any entity.
#[derive(Clone, Debug, Default)]
pub struct PlayerKilledEntityTrigger {
    entity: Option<EntityPredicate>,
    killing_blow: Option<DamagePredicate>,
}

impl PlayerKilledEntityTrigger {
    /// Starts an unconstrained player-killed-entity criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::PlayerKilledEntityTrigger::new",
        aliases = ["sand::prelude::PlayerKilledEntityTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Starts an unconstrained player-killed-entity criterion.",
        context = "Starts an unconstrained player-killed-entity criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Matches any entity killed by the triggering player.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "A `PlayerKilledEntityTrigger` initialized to an unconstrained player-killed-entity criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let player_killed_entity_trigger = sand::event::trigger::PlayerKilledEntityTrigger::new();\n}",
    )]
    pub fn new() -> Self {
        Self {
            entity: None,
            killing_blow: None,
        }
    }

    /// Filter by the killed entity's properties.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::PlayerKilledEntityTrigger::entity",
        aliases = ["sand::prelude::PlayerKilledEntityTrigger::entity"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by the killed entity's properties.",
        context = "Filter by the killed entity's properties. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the predicate against the killed entity.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to filter by the killed entity's properties."),
        returns = "The `PlayerKilledEntityTrigger` value with the documented change applied to filter by the killed entity's properties.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_killed_entity_trigger_value: sand::event::trigger::PlayerKilledEntityTrigger, predicate: sand::predicate::EntityPredicate)  {\n    let updated_player_killed_entity_trigger = player_killed_entity_trigger_value.entity(predicate);\n}",
    )]
    pub fn entity(mut self, predicate: EntityPredicate) -> Self {
        self.entity = Some(predicate);
        self
    }

    /// Filter by how the entity was killed (damage type, etc.).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::PlayerKilledEntityTrigger::killing_blow",
        aliases = ["sand::prelude::PlayerKilledEntityTrigger::killing_blow"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by how the entity was killed (damage type, etc.).",
        context = "Filter by how the entity was killed (damage type, etc.). This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the killing blow predicate when the victim dies.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to filter by how the entity was killed (damage type, etc.)."),
        returns = "The `PlayerKilledEntityTrigger` value with the documented change applied to filter by how the entity was killed (damage type, etc.).",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_killed_entity_trigger_value: sand::event::trigger::PlayerKilledEntityTrigger, predicate: sand::predicate::DamagePredicate)  {\n    let updated_player_killed_entity_trigger = player_killed_entity_trigger_value.killing_blow(predicate);\n}",
    )]
    pub fn killing_blow(mut self, predicate: DamagePredicate) -> Self {
        self.killing_blow = Some(predicate);
        self
    }

    /// Converts the player-kill builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::PlayerKilledEntityTrigger::build",
        aliases = ["sand::prelude::PlayerKilledEntityTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts the player-kill builder into an advancement criterion.",
        context = "Converts the player-kill builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes player_killed_entity conditions into advancement JSON.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert the player-kill builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_killed_entity_trigger_value: sand::event::trigger::PlayerKilledEntityTrigger)  {\n    let build = player_killed_entity_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::PlayerKilledEntity {
            entity: self.entity,
            killing_blow: self.killing_blow,
        }
    }
}

impl From<PlayerKilledEntityTrigger> for AdvancementTrigger {
    fn from(t: PlayerKilledEntityTrigger) -> Self {
        t.build()
    }
}

// ── EntityKilledPlayerTrigger ───────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::EntityKilledPlayerTrigger",
    aliases = ["sand::prelude::EntityKilledPlayerTrigger"],
    module = "sand::event",
    summary = "Fires when any entity kills the player.",
    context = "Fires when any entity kills the player. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "Uses minecraft:entity_killed_player with optional killer and damage predicates.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::EntityKilledPlayerTrigger;",
)]
/// Fires when any entity kills the player.
#[derive(Clone, Debug, Default)]
pub struct EntityKilledPlayerTrigger {
    entity: Option<EntityPredicate>,
    killing_blow: Option<DamagePredicate>,
}

impl EntityKilledPlayerTrigger {
    /// Starts an unconstrained entity-killed-player criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::EntityKilledPlayerTrigger::new",
        aliases = ["sand::prelude::EntityKilledPlayerTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Starts an unconstrained entity-killed-player criterion.",
        context = "Starts an unconstrained entity-killed-player criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Matches any death of the triggering player caused by an entity.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "An `EntityKilledPlayerTrigger` initialized to an unconstrained entity-killed-player criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let entity_killed_player_trigger = sand::event::trigger::EntityKilledPlayerTrigger::new();\n}",
    )]
    pub fn new() -> Self {
        Self {
            entity: None,
            killing_blow: None,
        }
    }

    /// Filter by the attacking entity's properties.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::EntityKilledPlayerTrigger::entity",
        aliases = ["sand::prelude::EntityKilledPlayerTrigger::entity"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by the attacking entity's properties.",
        context = "Filter by the attacking entity's properties. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the predicate against the killing entity.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to filter by the attacking entity's properties."),
        returns = "The `EntityKilledPlayerTrigger` value with the documented change applied to filter by the attacking entity's properties.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_killed_player_trigger_value: sand::event::trigger::EntityKilledPlayerTrigger, predicate: sand::predicate::EntityPredicate)  {\n    let updated_entity_killed_player_trigger = entity_killed_player_trigger_value.entity(predicate);\n}",
    )]
    pub fn entity(mut self, predicate: EntityPredicate) -> Self {
        self.entity = Some(predicate);
        self
    }

    /// Filter by the killing blow (damage type, etc.).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::EntityKilledPlayerTrigger::killing_blow",
        aliases = ["sand::prelude::EntityKilledPlayerTrigger::killing_blow"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by the killing blow (damage type, etc.).",
        context = "Filter by the killing blow (damage type, etc.). This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the supplied damage predicate at death.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to filter by the killing blow (damage type, etc.)."),
        returns = "The `EntityKilledPlayerTrigger` value with the documented change applied to filter by the killing blow (damage type, etc.).",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_killed_player_trigger_value: sand::event::trigger::EntityKilledPlayerTrigger, predicate: sand::predicate::DamagePredicate)  {\n    let updated_entity_killed_player_trigger = entity_killed_player_trigger_value.killing_blow(predicate);\n}",
    )]
    pub fn killing_blow(mut self, predicate: DamagePredicate) -> Self {
        self.killing_blow = Some(predicate);
        self
    }

    /// Converts the entity-killed-player builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::EntityKilledPlayerTrigger::build",
        aliases = ["sand::prelude::EntityKilledPlayerTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts the entity-killed-player builder into an advancement criterion.",
        context = "Converts the entity-killed-player builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes entity_killed_player conditions into advancement JSON.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert the entity-killed-player builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_killed_player_trigger_value: sand::event::trigger::EntityKilledPlayerTrigger)  {\n    let build = entity_killed_player_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::EntityKilledPlayer {
            entity: self.entity,
            killing_blow: self.killing_blow,
        }
    }
}

impl From<EntityKilledPlayerTrigger> for AdvancementTrigger {
    fn from(t: EntityKilledPlayerTrigger) -> Self {
        t.build()
    }
}

// ── RecipeUnlockedTrigger ───────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::RecipeUnlockedTrigger",
    aliases = ["sand::prelude::RecipeUnlockedTrigger"],
    module = "sand::event",
    summary = "Fires when the player unlocks a specific recipe.",
    context = "Fires when the player unlocks a specific recipe. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "Uses minecraft:recipe_unlocked for the specified recipe resource location.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::RecipeUnlockedTrigger;",
)]
/// Fires when the player unlocks a specific recipe.
#[derive(Clone, Debug)]
pub struct RecipeUnlockedTrigger {
    recipe: String,
}

impl RecipeUnlockedTrigger {
    /// Legacy string compatibility constructor.
    ///
    /// Prefer [`Self::from_id`] for new code so malformed IDs fail before a
    /// trigger value is constructed.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::RecipeUnlockedTrigger::new",
        aliases = ["sand::prelude::RecipeUnlockedTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Legacy string compatibility constructor. Prefer [`Self::from_id`] for new code so malformed IDs fail before a trigger value is constructed.",
        context = "Legacy string compatibility constructor. Prefer [`Self::from_id`] for new code so malformed IDs fail before a trigger value is constructed. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The recipe identifier is written into the vanilla advancement condition.",
        use_when = ["Prefer [`Self::from_id`] for new code so malformed IDs fail before a trigger value is constructed."],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(recipe = "`recipe` sets the recipe for legacy string compatibility constructor. Prefer [`Self::from_id`] for new code so malformed IDs fail before a trigger value is constructed."),
        returns = "A `RecipeUnlockedTrigger` configured for legacy string compatibility constructor. Prefer [`Self::from_id`] for new code so malformed IDs fail before a trigger value is constructed.",
        example = "use sand::prelude::*;\n\nfn demonstrate(recipe: impl Into < String >)  {\n    let recipe_unlocked_trigger = sand::event::trigger::RecipeUnlockedTrigger::new(recipe);\n}",
    )]
    pub fn new(recipe: impl Into<String>) -> Self {
        Self {
            recipe: recipe.into(),
        }
    }

    /// Create a recipe-unlocked trigger builder from a validated recipe ID.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::RecipeUnlockedTrigger::from_id",
        aliases = ["sand::prelude::RecipeUnlockedTrigger::from_id"],
        module = "sand::event",
        kind = "method",
        summary = "Create a recipe-unlocked trigger builder from a validated recipe ID.",
        context = "Create a recipe-unlocked trigger builder from a validated recipe ID. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Uses the exact namespace:path recipe ID in advancement JSON.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(recipe = "`recipe` provides the typed Minecraft resource identifier used to create a recipe-unlocked trigger builder from a validated recipe ID."),
        returns = "A `RecipeUnlockedTrigger` representing a recipe-unlocked trigger builder from a validated recipe ID.",
        example = "use sand::prelude::*;\n\nfn demonstrate(recipe: sand::ResourceLocation)  {\n    let recipe_unlocked_trigger = sand::event::trigger::RecipeUnlockedTrigger::from_id(recipe);\n}",
    )]
    pub fn from_id(recipe: crate::ResourceLocation) -> Self {
        Self {
            recipe: recipe.to_string(),
        }
    }

    /// Converts the recipe-unlocked builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::RecipeUnlockedTrigger::build",
        aliases = ["sand::prelude::RecipeUnlockedTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts the recipe-unlocked builder into an advancement criterion.",
        context = "Converts the recipe-unlocked builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes minecraft:recipe_unlocked with its recipe ID.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert the recipe-unlocked builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(recipe_unlocked_trigger_value: sand::event::trigger::RecipeUnlockedTrigger)  {\n    let build = recipe_unlocked_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        // `from_id` validates at construction; `new` remains a raw compatibility
        // path and is protected by Advancement's fallible export validation.
        AdvancementTrigger::RecipeUnlocked {
            recipe: self.recipe,
        }
    }
}

impl From<RecipeUnlockedTrigger> for AdvancementTrigger {
    fn from(t: RecipeUnlockedTrigger) -> Self {
        t.build()
    }
}

// ─── InventoryChangedTrigger ────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::InventoryChangedTrigger",
    aliases = ["sand::prelude::InventoryChangedTrigger"],
    module = "sand::event",
    summary = "Fires when the player's inventory changes.",
    context = "Fires when the player's inventory changes. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "Uses minecraft:inventory_changed and can constrain occupied slots or a matching stack.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::InventoryChangedTrigger;",
)]
/// Fires when the player's inventory changes.
#[derive(Clone, Debug, Default)]
pub struct InventoryChangedTrigger {
    slots: Option<InventorySlotsPredicate>,
    items: Vec<ItemPredicate>,
}

impl InventoryChangedTrigger {
    /// Starts an unconstrained inventory-change criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::InventoryChangedTrigger::new",
        aliases = ["sand::prelude::InventoryChangedTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Starts an unconstrained inventory-change criterion.",
        context = "Starts an unconstrained inventory-change criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Matches inventory changes until predicates narrow it.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "An `InventoryChangedTrigger` initialized to an unconstrained inventory-change criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let inventory_changed_trigger = sand::event::trigger::InventoryChangedTrigger::new();\n}",
    )]
    pub fn new() -> Self {
        Self {
            slots: None,
            items: Vec::new(),
        }
    }

    /// Filter by occupied/empty slot ranges.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::InventoryChangedTrigger::slots",
        aliases = ["sand::prelude::InventoryChangedTrigger::slots"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by occupied/empty slot ranges.",
        context = "Filter by occupied/empty slot ranges. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the supplied slots predicate when inventory state changes.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(slots = "`slots` provides the typed predicate that must match used to filter by occupied/empty slot ranges."),
        returns = "The `InventoryChangedTrigger` value with the documented change applied to filter by occupied/empty slot ranges.",
        example = "use sand::prelude::*;\n\nfn demonstrate(inventory_changed_trigger_value: sand::event::trigger::InventoryChangedTrigger, slots: sand::component::InventorySlotsPredicate)  {\n    let updated_inventory_changed_trigger = inventory_changed_trigger_value.slots(slots);\n}",
    )]
    pub fn slots(mut self, slots: InventorySlotsPredicate) -> Self {
        self.slots = Some(slots);
        self
    }

    /// Add an item filter. Can be called multiple times.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::InventoryChangedTrigger::item",
        aliases = ["sand::prelude::InventoryChangedTrigger::item"],
        module = "sand::event",
        kind = "method",
        summary = "Add an item filter. Can be called multiple times.",
        context = "Add an item filter. Can be called multiple times. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the predicate against changed inventory contents.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to add an item filter. Can be called multiple times."),
        returns = "The `InventoryChangedTrigger` value with the documented change applied to add an item filter. Can be called multiple times.",
        example = "use sand::prelude::*;\n\nfn demonstrate(inventory_changed_trigger_value: sand::event::trigger::InventoryChangedTrigger, predicate: sand::predicate::ItemPredicate)  {\n    let updated_inventory_changed_trigger = inventory_changed_trigger_value.item(predicate);\n}",
    )]
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.items.push(predicate);
        self
    }

    /// Converts the inventory-change builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::InventoryChangedTrigger::build",
        aliases = ["sand::prelude::InventoryChangedTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts the inventory-change builder into an advancement criterion.",
        context = "Converts the inventory-change builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes inventory_changed conditions into advancement JSON.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert the inventory-change builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(inventory_changed_trigger_value: sand::event::trigger::InventoryChangedTrigger)  {\n    let build = inventory_changed_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::InventoryChanged {
            slots: self.slots,
            items: self.items,
        }
    }
}

impl From<InventoryChangedTrigger> for AdvancementTrigger {
    fn from(t: InventoryChangedTrigger) -> Self {
        t.build()
    }
}

// ─── ItemObtainedTrigger (legacy crafted-result filter) ─────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::ItemObtainedTrigger",
    aliases = ["sand::prelude::ItemObtainedTrigger"],
    module = "sand::event",
    summary = "Source-compatibility builder for the removed `minecraft:crafted_item` trigger. Both filtered and unfiltered forms fail target-aware export on verified current profiles. Use [`AdvancementTrigger::RecipeCrafted`] with a concrete recipe ID for current vanilla.",
    context = "Source-compatibility builder for the removed `minecraft:crafted_item` trigger. Both filtered and unfiltered forms fail target-aware export on verified current profiles. Use [`AdvancementTrigger::RecipeCrafted`] with a concrete recipe ID for current vanilla. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "Uses minecraft:inventory_changed with a matching item condition.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::ItemObtainedTrigger;",
)]
/// Source-compatibility builder for the removed `minecraft:crafted_item`
/// trigger. Both filtered and unfiltered forms fail target-aware export on
/// verified current profiles. Use [`AdvancementTrigger::RecipeCrafted`] with
/// a concrete recipe ID for current vanilla.
#[derive(Clone, Debug, Default)]
pub struct ItemObtainedTrigger {
    item: Option<ItemPredicate>,
}

impl ItemObtainedTrigger {
    /// Starts an unconstrained item-obtained criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::ItemObtainedTrigger::new",
        aliases = ["sand::prelude::ItemObtainedTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Starts an unconstrained item-obtained criterion.",
        context = "Starts an unconstrained item-obtained criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Matches inventory observations until narrowed with item.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "An `ItemObtainedTrigger` initialized to an unconstrained item-obtained criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let item_obtained_trigger = sand::event::trigger::ItemObtainedTrigger::new();\n}",
    )]
    pub fn new() -> Self {
        Self { item: None }
    }

    /// Filter by the crafted item.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::ItemObtainedTrigger::item",
        aliases = ["sand::prelude::ItemObtainedTrigger::item"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by the crafted item.",
        context = "Filter by the crafted item. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The predicate is emitted as an inventory_changed item condition.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to filter by the crafted item."),
        returns = "The `ItemObtainedTrigger` value with the documented change applied to filter by the crafted item.",
        example = "use sand::prelude::*;\n\nfn demonstrate(item_obtained_trigger_value: sand::event::trigger::ItemObtainedTrigger, predicate: sand::predicate::ItemPredicate)  {\n    let updated_item_obtained_trigger = item_obtained_trigger_value.item(predicate);\n}",
    )]
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    /// Converts the item-obtained builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::ItemObtainedTrigger::build",
        aliases = ["sand::prelude::ItemObtainedTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts the item-obtained builder into an advancement criterion.",
        context = "Converts the item-obtained builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes the inventory_changed criterion used for item acquisition.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert the item-obtained builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(item_obtained_trigger_value: sand::event::trigger::ItemObtainedTrigger)  {\n    let build = item_obtained_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        match self.item {
            None => AdvancementTrigger::CraftedItem { item: None },
            Some(item) => AdvancementTrigger::CraftedItem { item: Some(item) },
        }
    }
}

impl From<ItemObtainedTrigger> for AdvancementTrigger {
    fn from(t: ItemObtainedTrigger) -> Self {
        t.build()
    }
}

// ─── ItemEnchantTrigger ─────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::ItemEnchantTrigger",
    aliases = ["sand::prelude::ItemEnchantTrigger"],
    module = "sand::event",
    summary = "Fires when the player enchants an item.",
    context = "Fires when the player enchants an item. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "Uses minecraft:enchanted_item with optional item and experience-level constraints.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::ItemEnchantTrigger;",
)]
/// Fires when the player enchants an item.
#[derive(Clone, Debug, Default)]
pub struct ItemEnchantTrigger {
    item: Option<ItemPredicate>,
    levels: Option<IntRange>,
}

impl ItemEnchantTrigger {
    /// Starts an unconstrained item-enchantment criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::ItemEnchantTrigger::new",
        aliases = ["sand::prelude::ItemEnchantTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Starts an unconstrained item-enchantment criterion.",
        context = "Starts an unconstrained item-enchantment criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Matches any successful enchantment until predicates narrow it.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "An `ItemEnchantTrigger` initialized to an unconstrained item-enchantment criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let item_enchant_trigger = sand::event::trigger::ItemEnchantTrigger::new();\n}",
    )]
    pub fn new() -> Self {
        Self {
            item: None,
            levels: None,
        }
    }

    /// Filter by the enchanted item.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::ItemEnchantTrigger::item",
        aliases = ["sand::prelude::ItemEnchantTrigger::item"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by the enchanted item.",
        context = "Filter by the enchanted item. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the stack being enchanted.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to filter by the enchanted item."),
        returns = "The `ItemEnchantTrigger` value with the documented change applied to filter by the enchanted item.",
        example = "use sand::prelude::*;\n\nfn demonstrate(item_enchant_trigger_value: sand::event::trigger::ItemEnchantTrigger, predicate: sand::predicate::ItemPredicate)  {\n    let updated_item_enchant_trigger = item_enchant_trigger_value.item(predicate);\n}",
    )]
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    /// Filter by experience levels spent.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::ItemEnchantTrigger::levels",
        aliases = ["sand::prelude::ItemEnchantTrigger::levels"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by experience levels spent.",
        context = "Filter by experience levels spent. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the advancement levels range for the enchantment.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(levels = "`levels` provides the accepted numeric range used to filter by experience levels spent."),
        returns = "The `ItemEnchantTrigger` value with the documented change applied to filter by experience levels spent.",
        example = "use sand::prelude::*;\n\nfn demonstrate(item_enchant_trigger_value: sand::event::trigger::ItemEnchantTrigger, levels: sand::predicate::IntRange)  {\n    let updated_item_enchant_trigger = item_enchant_trigger_value.levels(levels);\n}",
    )]
    pub fn levels(mut self, levels: IntRange) -> Self {
        self.levels = Some(levels);
        self
    }

    /// Converts the enchantment builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::ItemEnchantTrigger::build",
        aliases = ["sand::prelude::ItemEnchantTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts the enchantment builder into an advancement criterion.",
        context = "Converts the enchantment builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes enchanted_item conditions into advancement JSON.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert the enchantment builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(item_enchant_trigger_value: sand::event::trigger::ItemEnchantTrigger)  {\n    let build = item_enchant_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::EnchantedItem {
            item: self.item,
            levels: self.levels,
        }
    }
}

impl From<ItemEnchantTrigger> for AdvancementTrigger {
    fn from(t: ItemEnchantTrigger) -> Self {
        t.build()
    }
}

// ─── UsingItemTrigger ─────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::UsingItemTrigger",
    aliases = ["sand::prelude::UsingItemTrigger"],
    module = "sand::event",
    summary = "Fires when the player is actively using (holding right-click) an item.",
    context = "Fires when the player is actively using (holding right-click) an item. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "Uses minecraft:using_item with an optional item predicate.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::UsingItemTrigger;",
)]
/// Fires when the player is actively using (holding right-click) an item.
#[derive(Clone, Debug, Default)]
pub struct UsingItemTrigger {
    item: Option<ItemPredicate>,
}

impl UsingItemTrigger {
    /// Starts an unconstrained using-item criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::UsingItemTrigger::new",
        aliases = ["sand::prelude::UsingItemTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Starts an unconstrained using-item criterion.",
        context = "Starts an unconstrained using-item criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Matches item-use observations until narrowed with item.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "An `UsingItemTrigger` initialized to an unconstrained using-item criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let using_item_trigger = sand::event::trigger::UsingItemTrigger::new();\n}",
    )]
    pub fn new() -> Self {
        Self { item: None }
    }

    /// Filter by the item being used.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::UsingItemTrigger::item",
        aliases = ["sand::prelude::UsingItemTrigger::item"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by the item being used.",
        context = "Filter by the item being used. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the item currently being used.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to filter by the item being used."),
        returns = "The `UsingItemTrigger` value with the documented change applied to filter by the item being used.",
        example = "use sand::prelude::*;\n\nfn demonstrate(using_item_trigger_value: sand::event::trigger::UsingItemTrigger, predicate: sand::predicate::ItemPredicate)  {\n    let updated_using_item_trigger = using_item_trigger_value.item(predicate);\n}",
    )]
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    /// Converts the using-item builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::UsingItemTrigger::build",
        aliases = ["sand::prelude::UsingItemTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts the using-item builder into an advancement criterion.",
        context = "Converts the using-item builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes using_item conditions into advancement JSON.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert the using-item builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(using_item_trigger_value: sand::event::trigger::UsingItemTrigger)  {\n    let build = using_item_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::UsingItem { item: self.item }
    }
}

impl From<UsingItemTrigger> for AdvancementTrigger {
    fn from(t: UsingItemTrigger) -> Self {
        t.build()
    }
}

// ─── MultiKillTrigger (KilledByArrow) ───────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::MultiKillTrigger",
    aliases = ["sand::prelude::MultiKillTrigger"],
    module = "sand::event",
    summary = "Fires when the player kills multiple unique entity types with a crossbow.",
    context = "Fires when the player kills multiple unique entity types with a crossbow. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "Uses minecraft:player_killed_entity with unique-entity-type and victim constraints.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::MultiKillTrigger;",
)]
/// Fires when the player kills multiple unique entity types with a crossbow.
#[derive(Clone, Debug, Default)]
pub struct MultiKillTrigger {
    unique_entity_types: Option<IntRange>,
    victims: Option<Vec<EntityPredicate>>,
}

impl MultiKillTrigger {
    /// Starts an unconstrained multi-kill criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::MultiKillTrigger::new",
        aliases = ["sand::prelude::MultiKillTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Starts an unconstrained multi-kill criterion.",
        context = "Starts an unconstrained multi-kill criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Matches vanilla kill progress until its range or victim predicate is set.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "A `MultiKillTrigger` initialized to an unconstrained multi-kill criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let multi_kill_trigger = sand::event::trigger::MultiKillTrigger::new();\n}",
    )]
    pub fn new() -> Self {
        Self {
            unique_entity_types: None,
            victims: None,
        }
    }

    /// Number of unique entity types that must be killed.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::MultiKillTrigger::unique_entity_types",
        aliases = ["sand::prelude::MultiKillTrigger::unique_entity_types"],
        module = "sand::event",
        kind = "method",
        summary = "Number of unique entity types that must be killed.",
        context = "Number of unique entity types that must be killed. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The range is serialized into the trigger's unique entity type condition.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(count = "`count` provides the requested numeric amount used to number of unique entity types that must be killed."),
        returns = "The `MultiKillTrigger` value with the documented change applied to number of unique entity types that must be killed.",
        example = "use sand::prelude::*;\n\nfn demonstrate(multi_kill_trigger_value: sand::event::trigger::MultiKillTrigger, count: sand::predicate::IntRange)  {\n    let updated_multi_kill_trigger = multi_kill_trigger_value.unique_entity_types(count);\n}",
    )]
    pub fn unique_entity_types(mut self, count: IntRange) -> Self {
        self.unique_entity_types = Some(count);
        self
    }

    /// Filter by victim entity predicates.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::MultiKillTrigger::victim",
        aliases = ["sand::prelude::MultiKillTrigger::victim"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by victim entity predicates.",
        context = "Filter by victim entity predicates. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the predicate for each qualifying kill.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to filter by victim entity predicates."),
        returns = "The `MultiKillTrigger` value with the documented change applied to filter by victim entity predicates.",
        example = "use sand::prelude::*;\n\nfn demonstrate(multi_kill_trigger_value: sand::event::trigger::MultiKillTrigger, predicate: sand::predicate::EntityPredicate)  {\n    let updated_multi_kill_trigger = multi_kill_trigger_value.victim(predicate);\n}",
    )]
    pub fn victim(mut self, predicate: EntityPredicate) -> Self {
        self.victims.get_or_insert_with(Vec::new).push(predicate);
        self
    }

    /// Converts the multi-kill builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::MultiKillTrigger::build",
        aliases = ["sand::prelude::MultiKillTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts the multi-kill builder into an advancement criterion.",
        context = "Converts the multi-kill builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes the kill-progress conditions into advancement JSON.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert the multi-kill builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(multi_kill_trigger_value: sand::event::trigger::MultiKillTrigger)  {\n    let build = multi_kill_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::KilledByArrow {
            unique_entity_types: self.unique_entity_types,
            fired_from_weapon: Some(ItemPredicate::id(
                ItemId::minecraft("crossbow").expect("crossbow is a valid vanilla item ID"),
            )),
            victims: self.victims,
        }
    }
}

impl From<MultiKillTrigger> for AdvancementTrigger {
    fn from(t: MultiKillTrigger) -> Self {
        t.build()
    }
}

// ── PlayerInteractedWithEntityTrigger ─────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::PlayerInteractedWithEntityTrigger",
    aliases = ["sand::prelude::PlayerInteractedWithEntityTrigger"],
    module = "sand::event",
    summary = "Fires when the player right-clicks an entity. Use this with `interaction` entities for custom clickable objects.",
    context = "Fires when the player right-clicks an entity. Use this with `interaction` entities for custom clickable objects. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "Uses minecraft:player_interacted_with_entity with optional held-item and target predicates.",
    use_when = ["Use this with `interaction` entities for custom clickable objects."],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::PlayerInteractedWithEntityTrigger;",
)]
/// Fires when the player right-clicks an entity.
///
/// Use this with `interaction` entities for custom clickable objects.
#[derive(Clone, Debug, Default)]
pub struct PlayerInteractedWithEntityTrigger {
    item: Option<ItemPredicate>,
    entity: Option<EntityPredicate>,
}

impl PlayerInteractedWithEntityTrigger {
    /// Starts an unconstrained player-interaction criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::PlayerInteractedWithEntityTrigger::new",
        aliases = ["sand::prelude::PlayerInteractedWithEntityTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Starts an unconstrained player-interaction criterion.",
        context = "Starts an unconstrained player-interaction criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Matches any interaction with an entity until narrowed.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "A `PlayerInteractedWithEntityTrigger` initialized to an unconstrained player-interaction criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let player_interacted_with_entity_trigger = sand::event::trigger::PlayerInteractedWithEntityTrigger::new();\n}",
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by the item held during the interaction.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::PlayerInteractedWithEntityTrigger::item",
        aliases = ["sand::prelude::PlayerInteractedWithEntityTrigger::item"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by the item held during the interaction.",
        context = "Filter by the item held during the interaction. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the held interaction stack against the predicate.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to filter by the item held during the interaction."),
        returns = "The `PlayerInteractedWithEntityTrigger` value with the documented change applied to filter by the item held during the interaction.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_interacted_with_entity_trigger_value: sand::event::trigger::PlayerInteractedWithEntityTrigger, predicate: sand::predicate::ItemPredicate)  {\n    let updated_player_interacted_with_entity_trigger = player_interacted_with_entity_trigger_value.item(predicate);\n}",
    )]
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    /// Filter by the entity that was interacted with.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::PlayerInteractedWithEntityTrigger::entity",
        aliases = ["sand::prelude::PlayerInteractedWithEntityTrigger::entity"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by the entity that was interacted with.",
        context = "Filter by the entity that was interacted with. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the target entity against the predicate.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to filter by the entity that was interacted with."),
        returns = "The `PlayerInteractedWithEntityTrigger` value with the documented change applied to filter by the entity that was interacted with.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_interacted_with_entity_trigger_value: sand::event::trigger::PlayerInteractedWithEntityTrigger, predicate: sand::predicate::EntityPredicate)  {\n    let updated_player_interacted_with_entity_trigger = player_interacted_with_entity_trigger_value.entity(predicate);\n}",
    )]
    pub fn entity(mut self, predicate: EntityPredicate) -> Self {
        self.entity = Some(predicate);
        self
    }

    /// Converts the interaction builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::PlayerInteractedWithEntityTrigger::build",
        aliases = ["sand::prelude::PlayerInteractedWithEntityTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts the interaction builder into an advancement criterion.",
        context = "Converts the interaction builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes player_interacted_with_entity conditions into advancement JSON.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert the interaction builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_interacted_with_entity_trigger_value: sand::event::trigger::PlayerInteractedWithEntityTrigger)  {\n    let build = player_interacted_with_entity_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::PlayerInteractedWithEntity {
            item: self.item,
            entity: self.entity,
        }
    }
}

impl From<PlayerInteractedWithEntityTrigger> for AdvancementTrigger {
    fn from(t: PlayerInteractedWithEntityTrigger) -> Self {
        t.build()
    }
}

// ── SummonedEntityTrigger ─────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::trigger::SummonedEntityTrigger",
    aliases = ["sand::prelude::SummonedEntityTrigger"],
    module = "sand::event",
    summary = "Fires when the player summons an entity (via a spawn egg, totem, etc.).",
    context = "Fires when the player summons an entity (via a spawn egg, totem, etc.). This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "Uses minecraft:summoned_entity with an optional entity predicate.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::trigger::SummonedEntityTrigger;",
)]
/// Fires when the player summons an entity (via a spawn egg, totem, etc.).
#[derive(Clone, Debug, Default)]
pub struct SummonedEntityTrigger {
    entity: Option<EntityPredicate>,
}

impl SummonedEntityTrigger {
    /// Starts an unconstrained summoned-entity criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::SummonedEntityTrigger::new",
        aliases = ["sand::prelude::SummonedEntityTrigger::new"],
        module = "sand::event",
        kind = "method",
        summary = "Starts an unconstrained summoned-entity criterion.",
        context = "Starts an unconstrained summoned-entity criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Matches a player's entity summons until narrowed.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "A `SummonedEntityTrigger` initialized to an unconstrained summoned-entity criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let summoned_entity_trigger = sand::event::trigger::SummonedEntityTrigger::new();\n}",
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by the summoned entity's properties.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::SummonedEntityTrigger::entity",
        aliases = ["sand::prelude::SummonedEntityTrigger::entity"],
        module = "sand::event",
        kind = "method",
        summary = "Filter by the summoned entity's properties.",
        context = "Filter by the summoned entity's properties. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Minecraft evaluates the summoned entity against the predicate.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(predicate = "`predicate` provides the predicate that must match used to filter by the summoned entity's properties."),
        returns = "The `SummonedEntityTrigger` value with the documented change applied to filter by the summoned entity's properties.",
        example = "use sand::prelude::*;\n\nfn demonstrate(summoned_entity_trigger_value: sand::event::trigger::SummonedEntityTrigger, predicate: sand::predicate::EntityPredicate)  {\n    let updated_summoned_entity_trigger = summoned_entity_trigger_value.entity(predicate);\n}",
    )]
    pub fn entity(mut self, predicate: EntityPredicate) -> Self {
        self.entity = Some(predicate);
        self
    }

    /// Converts the summoned-entity builder into an advancement criterion.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::trigger::SummonedEntityTrigger::build",
        aliases = ["sand::prelude::SummonedEntityTrigger::build"],
        module = "sand::event",
        kind = "method",
        summary = "Converts the summoned-entity builder into an advancement criterion.",
        context = "Converts the summoned-entity builder into an advancement criterion. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Serializes summoned_entity conditions into advancement JSON.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `AdvancementTrigger` value produced to convert the summoned-entity builder into an advancement criterion.",
        example = "use sand::prelude::*;\n\nfn demonstrate(summoned_entity_trigger_value: sand::event::trigger::SummonedEntityTrigger)  {\n    let build = summoned_entity_trigger_value.build();\n}",
    )]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::SummonedEntity {
            entity: self.entity,
        }
    }
}

impl From<SummonedEntityTrigger> for AdvancementTrigger {
    fn from(t: SummonedEntityTrigger) -> Self {
        t.build()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_item_typed_predicate() {
        let trigger = ConsumeItemTrigger::new()
            .item(ItemPredicate::id(crate::generated::Item::GoldenApple))
            .build();
        let v = serde_json::to_value(&trigger).unwrap();
        assert_eq!(
            v["conditions"]["item"]["items"],
            serde_json::json!(["minecraft:golden_apple"])
        );
    }

    #[test]
    fn consume_item_raw_json_escape_hatch() {
        use sand_components::RawJson;
        let trigger = ConsumeItemTrigger::new()
            .item(ItemPredicate::raw(RawJson::new(
                serde_json::json!({"items": "minecraft:honey_bottle"}),
            )))
            .build();
        let v = serde_json::to_value(&trigger).unwrap();
        assert_eq!(v["conditions"]["item"]["items"], "minecraft:honey_bottle");
    }

    #[test]
    fn player_killed_entity_typed_predicate() {
        let trigger = PlayerKilledEntityTrigger::new()
            .entity(EntityPredicate::type_(crate::generated::EntityType::Zombie))
            .build();
        let v = serde_json::to_value(&trigger).unwrap();
        assert_eq!(v["conditions"]["entity"]["type"], "minecraft:zombie");
    }

    #[test]
    fn inventory_changed_typed_item_predicate() {
        let trigger = InventoryChangedTrigger::new()
            .item(ItemPredicate::id(crate::generated::Item::Diamond))
            .build();
        let v = serde_json::to_value(&trigger).unwrap();
        let items = &v["conditions"]["items"];
        assert_eq!(items[0]["items"], serde_json::json!(["minecraft:diamond"]));
    }

    #[test]
    fn tick_trigger_builds() {
        let t: AdvancementTrigger = TickTrigger::new().into();
        assert!(matches!(t, AdvancementTrigger::Tick));
    }

    #[test]
    fn impossible_trigger_builds() {
        let t: AdvancementTrigger = ImpossibleTrigger::new().into();
        assert!(matches!(t, AdvancementTrigger::Impossible));
    }

    #[test]
    fn recipe_unlocked_uses_typed_variant() {
        let t = RecipeUnlockedTrigger::from_id("minecraft:crafting_table".parse().unwrap()).build();
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:recipe_unlocked");
        assert_eq!(v["conditions"]["recipe"], "minecraft:crafting_table");
        assert!("bad recipe".parse::<crate::ResourceLocation>().is_err());
    }
}
