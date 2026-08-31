//! Complete unified-State tutorial: independent components, nesting, runtime
//! presence, queries, tick/event systems, archetype attachment, migrations,
//! player progression, and a global typed-data resource.

use sand::events::{EventSetup, SandEvent, SandEventDispatch};
use sand::prelude::*;

static ATTACKS: ScoreVar<i32> = ScoreVar::new("rpg_attacks");
static SEEN_ATTACKS: ScoreVar<i32> = ScoreVar::new("rpg_seen_atk");

#[derive(State)]
#[state(namespace = "rpg", scope = player)]
pub struct Progression {
    #[state(default = 1, min = 1, max = 100)]
    pub level: Score,
    #[state(default = 0, min = 0, max = 1_000_000)]
    pub experience: Score,
    #[state(default = 1.0, min = 0, max = 10, scale = 100)]
    pub experience_multiplier: FixedScore,
    #[state(default_snbt = "{announcements:1b,title:\"Rookie\"}")]
    pub preferences: Data<serde_json::Value>,
}

#[derive(State)]
#[state(
    namespace = "rpg",
    scope = living,
    version = 2,
    migrate(from = 1, to = 2)
)]
pub struct Attack {
    #[state(default = 2, min = 0, max = 100)]
    pub damage: Score,
    #[state(auto_tick)]
    pub cooldown: Cooldown,
}

#[state_lifecycle]
impl StateLifecycle for Attack {
    fn migrate(ctx: StateMigrate) -> Vec<String> {
        vec![format!("say migrated attack {} -> {}", ctx.from(), ctx.to())]
    }
}

#[derive(State)]
#[state(namespace = "rpg", scope = living)]
pub struct Defense {
    #[state(default = 0, min = 0, max = 100)]
    pub armor: Score,
    #[state(default = false)]
    pub blocking: Flag,
}

#[derive(State)]
#[state(namespace = "rpg", scope = living)]
pub struct Status {
    #[state(default = false)]
    pub poisoned: Flag,
    #[state(auto_tick)]
    pub poison_time: Timer,
    #[state(default_snbt = "{stacks:0,note:\"clean\"}")]
    pub details: Data<serde_json::Value>,
}

#[derive(State)]
#[state(namespace = "rpg", scope = living)]
pub struct Dead;

#[derive(State)]
#[state(namespace = "rpg", scope = living)]
pub struct ManagedZombie;

#[derive(StateBundle)]
pub struct Combat {
    pub attack: Attack,
    pub defense: Defense,
}

#[derive(StateBundle)]
pub struct Character {
    pub combat: Combat,
    pub status: Status,
}

#[derive(StateQuery)]
#[query(scope = living)]
pub struct Combatants {
    #[require]
    pub combat: Combat,
    #[optional]
    pub status: Status,
    #[without]
    pub dead: Dead,
}

pub struct CombatSystems;

#[system]
#[allow(dead_code, unused_must_use)]
impl CombatSystems {
    #[tick(every = 20)]
    fn regenerate(query: Combatants) {
        query.each(|entity| {
            let mut commands = entity.combat.defense.armor.add(1);
            commands.extend(entity.status(|status| status.poison_time.tick()));
            commands
        });
    }

    #[tick(every = 20)]
    fn train_attack(query: Combatants) {
        query.each(|entity| entity.combat.attack.damage.add(1));
    }

    #[event(AttackPulse)]
    fn attack(_event: AttackPulse, query: Combatants) {
        query.current(|entity| {
            let mut commands = entity.combat.attack.damage.add(1);
            commands.extend(entity.combat.attack.cooldown.start(Ticks::new(20)));
            commands
        });
    }
}

pub struct AttackPulse;

impl SandEvent for AttackPulse {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(SEEN_ATTACKS.of("@s").lt_score(ATTACKS.of("@s")))
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec![
                "scoreboard objectives add rpg_attacks minecraft.custom:minecraft.damage_dealt"
                    .into(),
                "scoreboard objectives add rpg_seen_atk dummy".into(),
            ],
            pre_observation: Vec::new(),
            post_observation: vec![
                "execute as @a run scoreboard players operation @s rpg_seen_atk = @s rpg_attacks"
                    .into(),
            ],
        }
    }
}

#[derive(State)]
#[state(namespace = "rpg", scope = global)]
pub struct World {
    #[state(default = 1)]
    pub wave: Score,
    #[state(default_snbt = "{difficulty:1,announcements:1b}")]
    pub settings: Data<serde_json::Value>,
}

#[function]
pub fn attach_zombie_components() {
    Character::attach(EntityContext::<ZombieKind>::default());
}

#[function]
pub fn detach_attack() {
    Attack::detach(EntityContext::<ZombieKind>::default());
}

#[function]
pub fn reattach_attack() {
    Attack::attach(EntityContext::<ZombieKind>::default());
}

#[function]
pub fn mark_status_data() {
    Status::on(EntityContext::<ZombieKind>::default())
        .details
        .set(sand::command::NbtValue::raw("{stacks:3,note:\"kept\"}"));
}

#[function]
pub fn detach_status() {
    Status::detach(EntityContext::<ZombieKind>::default());
}

#[entity_archetype]
pub fn armored_zombie() -> EntityArchetype<ZombieKind, ManagedZombie> {
    EntityArchetype::new(ResourceLocation::new("rpg", "armored_zombie").unwrap())
        .components::<Character>()
        .adopt(Adoption::natural_and_external().every(Ticks::new(20)))
}

#[function]
pub fn advance_world() {
    World::global().wave.add(1);
    Progression::on(EntityContext::<PlayerKind>::default())
        .experience
        .add(10);
    Progression::on(EntityContext::<PlayerKind>::default())
        .experience_multiplier
        .add(0.05);
    Progression::on(EntityContext::<PlayerKind>::default())
        .preferences
        .if_present(|| vec!["say preferences loaded".into()]);
}

/// Export hook used by the isolated tutorial workspace.
#[doc(hidden)]
pub fn __sand_export(namespace: &str, mc_version: &str) {
    let json = sand::advanced::try_export_components_json(namespace, mc_version)
        .unwrap_or_else(|error| panic!("unified State export failed: {error}"));
    println!("{json}");
}
