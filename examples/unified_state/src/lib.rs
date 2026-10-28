//! Complete unified-State tutorial: independent components, nesting, runtime
//! presence, queries, tick/event systems, archetype attachment, migrations,
//! player progression, and a global typed-data resource.

use sand::events::{EventSetup, OnJoinEvent, SandEvent, SandEventDispatch};
use sand::prelude::*;
// These objectives observe vanilla statistics for the custom event below;
// gameplay-owned fields use the derived State schemas in this example.
use sand::advanced::state::ScoreVar;

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
    /// Template text keeps dollar-parenthesis sequences as ordinary data.
    #[state(default_snbt = r#""$(player)""#)]
    pub template: Data<String>,
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
        vec![format!(
            "say migrated attack {} -> {}",
            ctx.from(),
            ctx.to()
        )]
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
    /// Literal text can also resemble a generated UUID macro argument.
    #[state(default_snbt = r#""$(u0)""#)]
    pub template: Data<String>,
}

#[derive(State)]
#[state(namespace = "rpg", scope = living)]
pub struct Dead;

#[derive(State)]
#[state(namespace = "rpg", scope = living)]
pub struct Health {
    #[state(default = 20, min = 0, max = 100)]
    pub current: Score,
}

#[derive(StateBundle)]
pub struct Combat {
    pub attack: Attack,
    pub defense: Defense,
}

#[derive(StateBundle)]
pub struct Character {
    pub combat: Combat,
    pub status: Status,
    pub health: Health,
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

#[system(tick, every = 20)]
fn regenerate_health(query: Health) {
    query.each(|health| health.current.add(1));
}

#[system]
#[allow(dead_code, unused_must_use)]
impl CombatSystems {
    #[tick(every = 20)]
    fn regenerate(query: Combatants) {
        query.each(|entity| {
            mcfunction![
                entity.combat.defense.armor.add(1);
                entity.status(|status| status.poison_time.tick());
            ]
        });
    }

    #[tick(every = 20)]
    fn train_attack(query: Combatants) {
        query.each(|entity| entity.combat.attack.damage.add(1));
    }

    #[event(AttackPulse)]
    fn attack(_event: AttackPulse, query: Combatants) {
        query.current(|entity| {
            mcfunction![
                entity.combat.attack.damage.add(1);
                entity.combat.attack.cooldown.start(Ticks::new(20));
            ]
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
    /// Global data uses the same literal-value semantics as keyed data.
    #[state(default_snbt = r#""$(global)""#)]
    pub template: Data<String>,
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

#[derive(Archetype)]
#[archetype(id = "rpg:armored_zombie", entity = ZombieKind, configure = Self::configure)]
#[allow(dead_code)]
pub struct ArmoredZombie {
    pub character: Character,
}

impl ArmoredZombie {
    fn configure(archetype: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
        archetype.adopt(Adoption::natural_and_external().every(Ticks::new(20)))
    }
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

const LITERAL_TEMPLATE: &str = r#"$(missing) $(u0) "quoted" \ path"#;

fn literal_payload() -> sand::command::NbtValue {
    use sand::command::{NbtCompound, NbtValue};
    let mut nested = NbtCompound::new();
    nested.insert("literal", "$(u0)");
    nested.insert("quoted", r#""quoted" \ path"#);
    let mut payload = NbtCompound::new();
    payload.insert("text", LITERAL_TEMPLATE);
    payload.insert("nested", NbtValue::Compound(nested));
    NbtValue::Compound(payload)
}

/// Replace only the current player's text and nested preferences with literal data.
#[function("set_player_literals", context = player)]
pub fn set_player_literals(player: EntityContext<PlayerKind>) {
    let state = Progression::on(player);
    state.template.set(LITERAL_TEMPLATE);
    state.preferences.set(literal_payload());
}

/// Replace only the current zombie's text and nested component data.
#[function]
pub fn set_entity_literals() {
    let state = Status::on(EntityContext::<ZombieKind>::default());
    state.template.set(LITERAL_TEMPLATE);
    state.details.set(literal_payload());
}

/// Native regression entry points for the actual keyed lowering.
#[function]
pub fn get_entity_literal() {
    Status::on(EntityContext::<ZombieKind>::default()).template.get();
}

#[function]
pub fn remove_entity_literal() {
    Status::on(EntityContext::<ZombieKind>::default()).template.remove();
}

#[function]
pub fn present_entity_literal() {
    Status::on(EntityContext::<ZombieKind>::default()).template.if_present(|| {
        let state = Status::on(EntityContext::<ZombieKind>::default());
        let mut commands = state.template.get().into_iter().map(|command|
            format!("execute as @e[tag=keyed_literal_b,limit=1] run {command}")
        ).collect::<Vec<_>>();
        commands.extend(state.template.get());
        commands.push("say SAND_KEYED_PRESENT".into());
        commands
    });
}

/// Store the same literal text without entity-keyed ownership.
#[function]
pub fn set_global_literal() {
    World::global().template.set(LITERAL_TEMPLATE);
}

/// Export hook used by the isolated tutorial workspace.
#[doc(hidden)]
pub fn __sand_export(namespace: &str, mc_version: &str) {
    let json = sand::advanced::try_export_pack_json(namespace, mc_version)
        .unwrap_or_else(|error| panic!("unified State export failed: {error}"));
    println!("{json}");
}

/// Current inventory snapshot, independent of persistent progression experience.
#[derive(State)]
#[state(namespace = "rpg", scope = player)]
pub struct Supplies {
    pub diamonds: Score,
}

/// Player entry demonstrating a semantic inventory read into canonical State.
#[function("count_diamonds", context = player)]
pub fn count_diamonds(player: EntityContext<PlayerKind>) {
    Supplies::on(player)
        .diamonds
        .set(player.inventory().count(vanilla::Item::Diamond));
}

/// Events use the same handle and typed call path as player queries/functions.
#[on_event]
pub fn welcome_player(event: Event<OnJoinEvent>) {
    event.player().call(count_diamonds);
}
