//! # Predicates
//!
//! Demonstrates standalone `data/<namespace>/predicate/*.json` authoring
//! through the dedicated `PredicateRoot` typed condition tree — boolean
//! composition, entity/location/weather/time checks, references to other
//! predicate files, and the explicit raw escape hatch. These predicates can
//! be reused by `/execute if predicate`, advancement criteria, loot table
//! conditions, and selectors.

use sand_core::{
    DimensionId, EntityPredicate, EntityPredicateTarget, EntityTypeId, IntRange, LocationPredicate,
    Predicate, PredicateId, PredicateRoot, RawJson, WeatherPredicate,
};
use sand_macros::datapack_component;

// ── Entity predicate reused by `execute if predicate` ────────────────────────
// Matches a baby zombie — see chapter 8 (commands-and-execution) for how to
// gate a command on `execute if predicate my_pack:is_baby_zombie run ...`.

#[datapack_component]
pub fn is_baby_zombie() -> Predicate {
    Predicate::new(
        "my_pack:is_baby_zombie".parse().unwrap(),
        PredicateRoot::entity_properties(
            EntityPredicateTarget::This,
            EntityPredicate::type_(EntityTypeId::minecraft("zombie").unwrap())
                .flags(sand_core::EntityFlags::new().baby(true)),
        ),
    )
}

// ── Location, weather, and time predicates ───────────────────────────────────
// Composed with `any_of`: matches if the player is in the nether, it's
// currently thundering, or it's nighttime.

#[datapack_component]
pub fn dangerous_moment() -> Predicate {
    Predicate::any_of(
        "my_pack:dangerous_moment".parse().unwrap(),
        [
            PredicateRoot::location(
                LocationPredicate::new().dimension(DimensionId::minecraft("the_nether").unwrap()),
            ),
            PredicateRoot::weather(WeatherPredicate::new().thundering(true)),
            PredicateRoot::time(IntRange::between(13000, 23000)),
        ],
    )
}

// ── Reference to another predicate file ───────────────────────────────────────
// `minecraft:reference` points at another standalone predicate by ID.

#[datapack_component]
pub fn references_dangerous_moment() -> Predicate {
    Predicate::new(
        "my_pack:gated_by_danger".parse().unwrap(),
        PredicateRoot::inverted(PredicateRoot::reference(PredicateId::custom(
            "my_pack:dangerous_moment".parse().unwrap(),
        ))),
    )
}

// ── Raw escape hatch for unsupported/modded predicate shapes ────────────────
// Use `PredicateRoot::raw` for condition types Sand does not yet model —
// e.g. a modded condition type, or a vanilla shape added after this release.

#[datapack_component]
pub fn modded_condition() -> Predicate {
    Predicate::new(
        "my_pack:modded_condition".parse().unwrap(),
        PredicateRoot::raw(RawJson::new(serde_json::json!({
            "condition": "mymod:phase_check",
            "phase": 2
        })))
        .unwrap(),
    )
}
