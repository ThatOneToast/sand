//! Runnable RPG Zombie demonstrating issue #295's complete authoring path.
//!
//! The exporter adopts loaded, previously-unmarked Zombies. State persists on
//! each entity in scoreboards; unloaded chunks are not scanned, and refresh
//! resumes after the Zombie becomes loaded again. No persistent Rust entity
//! reference or user-managed scratch storage is involved.

use sand::prelude::*;

/// Stable rarity wire values stored as entity scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EntityStateEnum)]
pub enum Rarity {
    Common = 0,
    Rare = 1,
    Legendary = 2,
}

/// All persistent state owned by one adopted Zombie.
#[derive(State)]
#[state(namespace = "rpg", scope = living, name = "zombie", version = 2)]
#[allow(dead_code)]
pub struct ZombieState {
    #[state(default = 1, min = 1, max = 100)]
    level: EntityScore<i32>,
    #[state(default = 20, min = 0, max = 2000)]
    health: EntityScore<i32>,
    #[state(default = 20, min = 1, max = 2000)]
    max_health: EntityScore<i32>,
    #[state(default = 3, min = 0, max = 1000)]
    attack_damage: EntityScore<i32>,
    #[state(default = false)]
    sick: EntityFlag,
    #[state(default = "Rarity::Common")]
    rarity: EntityEnum<Rarity>,
    #[state(default = 0)]
    age: EntityTimer,
    #[state(default = 0)]
    ability: EntityCooldown,
    #[state(default = 2, kind = "version")]
    schema_version: EntityScore<i32>,
    #[state(kind = "dirty")]
    stats_dirty: EntityScore<i32>,
}

fn whole_numbers() -> FixedPoint {
    FixedPoint::new(1, RoundingPolicy::TowardZero, OverflowPolicy::Error)
        .expect("scale one is a valid fixed-point configuration")
}

fn hundredths() -> FixedPoint {
    FixedPoint::new(100, RoundingPolicy::TowardZero, OverflowPolicy::Error)
        .expect("scale one hundred is a valid fixed-point configuration")
}

/// Initialization callback referenced by a canonical typed function ID.
#[function]
pub fn initialized() {
    ZombieState::ability.bind().start(Ticks::seconds(5));
}

/// Contiguous version-one to version-two migration.
#[function]
pub fn migrate_v1_v2() {
    ZombieState::schema_version.bind().set(2);
}

/// Transition action used when sickness becomes enabled.
#[function]
pub fn sickness_started() {
    ZombieState::stats_dirty.bind().set(1);
}

/// Runtime validation entry: mutate only this Zombie's level.
#[function]
pub fn level_up() {
    ZombieState::level.bind().add(1);
}

/// Runtime validation entry: enable sickness for this Zombie.
#[function]
pub fn infect() {
    ZombieState::sick.bind().enable();
}

/// Natural/external Zombie archetype registered into the export.
#[entity_archetype]
pub fn rpg_zombie() -> EntityArchetype<ZombieKind, ZombieState> {
    let health_curve = StatCurve::multiply([
        StatCurve::linear(StatCurve::state(ZombieState::level), 2.0, 18.0),
        StatCurve::enum_mapping(
            ZombieState::rarity,
            [
                (Rarity::Common, 1.0),
                (Rarity::Rare, 2.0),
                (Rarity::Legendary, 3.0),
            ],
            1.0,
        ),
        StatCurve::flag_mapping(ZombieState::sick, 1.0, 0.75),
    ]);
    let name = EntityText::new()
        .literal("Lv. ")
        .color_last(ChatColor::Gold)
        .score(ZombieState::level)
        .color_last(ChatColor::Yellow)
        .literal(" Plagued Zombie")
        .color_last(ChatColor::DarkGreen);

    EntityArchetype::new(ResourceLocation::new("rpg", "plagued_zombie").unwrap())
        .version(2)
        .adopt(
            Adoption::natural_and_external()
                .every(Ticks::new(5))
                .special_entities(SpecialEntityPolicy::Preserve),
        )
        .reconcile(ReconcilePolicy::WhenDirty)
        .migration(Migration::new(
            1,
            2,
            "rpg:migrate_v1_v2".parse::<FunctionId>().unwrap(),
        ))
        .initialize_with("rpg:initialized".parse::<FunctionId>().unwrap())
        .derive(
            EntityDerivation::new("max_health", ZombieState::max_health, health_curve)
                .fixed_point(hundredths()),
        )
        .derive(
            EntityDerivation::new(
                "attack_damage",
                ZombieState::attack_damage,
                StatCurve::clamped_linear(
                    StatCurve::state(ZombieState::level),
                    1.0,
                    2.0,
                    3.0,
                    100.0,
                ),
            )
            .fixed_point(whole_numbers()),
        )
        .health(
            HealthBinding::new(ZombieState::max_health)
                .current_health(ZombieState::health, CurrentHealthSync::Bidirectional)
                .resize(HealthResizePolicy::PreserveRatio)
                .observe_native_every(Ticks::new(20)),
        )
        .attribute(AttributeBinding::new(
            AttributeType::AttackDamage,
            NumericPropertySource::state(ZombieState::attack_damage),
        ))
        .name(NameBinding::new(name))
        .effect_when(
            ZombieState::sick,
            EffectBinding::new(
                StatusEffectId::minecraft("weakness").unwrap(),
                Ticks::seconds(10),
            ),
        )
        .on(
            EntityTransition::flag_enabled(ZombieState::sick),
            EntityAction::Run("rpg:sickness_started".parse::<FunctionId>().unwrap()),
        )
}

/// Export hook used by the standard `sand_export` binary.
#[doc(hidden)]
pub fn __sand_export(namespace: &str, mc_version: &str) {
    let resolved = sand::version::resolve_export_caps(mc_version)
        .unwrap_or_else(|error| panic!("invalid export profile: {error}"));
    let json = sand::advanced::try_export_components_json_for_version(
        namespace,
        &resolved.caps,
        &resolved.version,
        resolved.is_fallback,
    )
    .unwrap_or_else(|error| panic!("entity archetype export failed: {error}"));
    println!("{json}");
}
