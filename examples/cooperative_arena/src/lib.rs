//! Two independent, explicitly registered cooperative arenas on loaded platforms.
use sand::component::IntoCommands;
use sand::prelude::*;
use sand::systems::{Combat, Hud, Lifetime, Menu, Progression, Roster, Scene, Session, Transform};
use sand::world::WorldContext;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EntityStateEnum)]
enum Role {
    Spectator = 0,
    Fighter = 1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, EntityStateEnum)]
enum Phase {
    Results = 0,
    Lobby = 1,
    Active = 2,
}
#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "cooperative_arena", scope = player)]
struct Player {
    arena: Score,
    role: EntityEnum<Role>,
    #[state(default = 20, min = 0, max = 20)]
    health: Score,
    #[state(auto_tick)]
    attack: Cooldown,
    wins: Score,
}
#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "cooperative_arena", scope = entity)]
struct Boss {
    #[state(default = 100, min = 0, max = 100)]
    health: Score,
    #[state(min = 0, max = 1)]
    objective: Score,
}
#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "cooperative_arena", scope = global)]
struct Arenas {
    a: EntityEnum<Phase>,
    b: EntityEnum<Phase>,
}
fn phase(arena: usize) -> sand::entity::EntityEnumAccessor<Phase> {
    if arena == 0 {
        Arenas::global().a
    } else {
        Arenas::global().b
    }
}
fn tag(arena: usize) -> &'static str {
    if arena == 0 {
        "cooperative_arena.host_a"
    } else {
        "cooperative_arena.host_b"
    }
}
fn id(arena: usize, suffix: &str) -> ResourceLocation {
    format!(
        "cooperative_arena:{}_{suffix}",
        if arena == 0 { "a" } else { "b" }
    )
    .parse()
    .unwrap()
}
fn roster(arena: usize) -> Roster<Role> {
    Roster::new(id(arena, "roster"), Player::arena, Player::role, 4)
}
#[datapack_component]
fn roster_a() -> Roster<Role> {
    roster(0)
}
#[datapack_component]
fn roster_b() -> Roster<Role> {
    roster(1)
}
fn hosts(arena: usize) -> sand::command::Target {
    Target::entities().tag(tag(arena)).in_current_dimension()
}
fn boss_scene(arena: usize) -> Scene<AnyEntity> {
    Scene::new(
        Lifetime::new(id(arena, "scene"), Ticks::seconds(60))
            .capacity(1)
            .requires::<Boss>(),
    )
    .part(
        "core",
        BlockId::minecraft("amethyst_block").unwrap(),
        Transform::default()
            .translated([-1.0, 0.0, -1.0])
            .scaled([2.0; 3]),
    )
}
#[datapack_component]
fn scene_a() -> Scene<AnyEntity> {
    boss_scene(0)
}
#[datapack_component]
fn scene_b() -> Scene<AnyEntity> {
    boss_scene(1)
}
fn round(arena: usize) -> Session<AnyEntity> {
    Session::new(
        Lifetime::new(id(arena, "round"), Ticks::seconds(60))
            .capacity(1)
            .requires::<Boss>()
            .every(Ticks::seconds(1))
            .on_start(move |ctx| {
                let mut actions = phase(arena).set(Phase::Active).into_commands();
                actions.extend([
                    ctx.owner(|owner| owner.state(Boss::health).set(100)),
                    ctx.start_child(boss_scene(arena)),
                    roster(arena).each(None, |player| {
                        let mut ready = player.state(Player::health).set(20);
                        ready.extend([
                            hud().cancel(),
                            hud().start(),
                            cmd::tellraw(
                                Target::self_(),
                                Text::new("Sneak to strike the core. You have 60 seconds!"),
                            )
                            .into_commands(),
                        ]);
                        ready
                    }),
                ]);
                actions
            })
            .on_tick(move |ctx| {
                let mut actions = roster(arena).each(Some(Role::Fighter), |_| {
                    Combat::new(Player::health, 20).damage(1)
                });
                actions.extend([
                    unless(roster(arena).has_members(Some(Role::Fighter))).then(ctx.stop())
                ]);
                actions
            })
            .on_stop(move |_| {
                let mut actions = phase(arena).set(Phase::Results).into_commands();
                actions.extend([
                    roster(arena).each(None, |_| {
                        let mut actions = hud().cancel();
                        actions.extend([cmd::tellraw(
                            Target::self_(),
                            Text::new("Round ended. Sneak for the lobby menu."),
                        )]);
                        actions
                    }),
                    roster(arena).close(),
                ]);
                actions
            }),
        Progression::new(id(arena, "objectives"), Boss::objective).objective(
            |host| host.state(Boss::health).matches(..=0).unwrap(),
            move |_| {
                roster(arena).each(Some(Role::Fighter), |player| {
                    let mut actions = player.state(Player::wins).add(1);
                    actions.extend([cmd::tellraw(
                        Target::self_(),
                        Text::new("Victory! One saved win awarded."),
                    )]);
                    actions
                })
            },
        ),
    )
}
#[datapack_component]
fn round_a() -> Session<AnyEntity> {
    round(0)
}
#[datapack_component]
fn round_b() -> Session<AnyEntity> {
    round(1)
}
#[datapack_component]
fn hud() -> Hud {
    Hud::new(
        Lifetime::new("cooperative_arena:hud".parse().unwrap(), Ticks::seconds(65))
            .capacity(8)
            .requires::<Player>()
            .on_tick(|ctx| {
                unless(ctx.owner_matches(|_| {
                    Condition::any([roster(0).contains(), roster(1).contains()])
                }))
                .then(ctx.stop())
            }),
    )
    .bossbar(
        Text::new("Arena health"),
        StatCurve::state(Player::health),
        20,
    )
}
fn admission(arena: usize, role: Role) -> Actions {
    let mut actions = roster(arena).join(role);
    actions.extend([when(roster(arena).is(role)).then(
        Execute::new().in_("minecraft:overworld").run(
            EntityContext::<PlayerKind>::default()
                .transform()
                .teleport(Vec3::new(arena as f64 * 32.0, 101.0, 4.0))
                .unwrap(),
        ),
    )]);
    actions
}
fn reopen(arena: usize) -> Actions {
    let mut actions = roster(arena).open();
    actions.extend([phase(arena).set(Phase::Lobby)]);
    actions
}
#[datapack_component]
fn lobby() -> Menu {
    let mut menu = Menu::new(
        "cooperative_arena:lobby".parse().unwrap(),
        Text::new("Cooperative arenas"),
        Ticks::seconds(30),
    );
    for arena in 0..2 {
        let label = if arena == 0 { "A" } else { "B" };
        menu = menu
            .choice(
                Text::new(format!("Open lobby {label}")),
                phase(arena).is(Phase::Results),
                move || reopen(arena),
            )
            .choice(
                Text::new(format!("Join {label}")),
                phase(arena).is(Phase::Lobby),
                move || admission(arena, Role::Fighter),
            )
            .choice(
                Text::new(format!("Watch {label}")),
                Condition::any([
                    phase(arena).is(Phase::Lobby),
                    phase(arena).is(Phase::Active),
                ]),
                move || admission(arena, Role::Spectator),
            )
            .choice(
                Text::new(format!("Start {label}")),
                Condition::all([
                    roster(arena).is(Role::Fighter),
                    phase(arena).is(Phase::Lobby),
                ]),
                move || hosts(arena).each(|_| round(arena).restart()),
            );
    }
    menu.choice(Text::new("Leave"), Condition::all([]), || {
        let mut actions = hud().cancel();
        actions.extend([roster(0).leave(), roster(1).leave()]);
        actions
    })
}
#[function("cooperative_arena:menu", context = player)]
pub fn menu() {
    lobby().open();
}
#[on_event]
fn sneak(_: Event<sand::events::PlayerStartSneakingEvent>) {
    sneak_actions();
}
fn sneak_actions() -> Actions {
    let active = Condition::any((0..2).map(|arena| {
        Condition::all([
            roster(arena).is(Role::Fighter),
            phase(arena).is(Phase::Active),
        ])
    }));
    let mut actions = unless(active).then(lobby().open());
    for arena in 0..2 {
        actions.extend([when(Condition::all([
            roster(arena).is(Role::Fighter),
            phase(arena).is(Phase::Active),
            Player::health.matches(1..).unwrap().condition(),
            Player::attack.ready().condition(),
        ]))
        .then(run_fn!({
            Player::attack.bind().start(Ticks::new(10));
            hosts(arena)
                .within_blocks(8.0)
                .each(|_| Combat::new(Boss::health, 100).damage(5));
        }))]);
    }
    actions
}
#[system(tick)]
fn defeated(players: Player) {
    players.each(|player| {
        let mut actions = Actions::default();
        for arena in 0..2 {
            actions.extend([when(Condition::all([
                roster(arena).is(Role::Fighter),
                player.health.matches(..=0).unwrap(),
            ]))
            .then(roster(arena).join(Role::Spectator))]);
        }
        actions
    });
}
#[on_event]
fn death(_: Event<sand::events::OnDeathEvent>) {
    hud().cancel();
    roster(0).leave();
    roster(1).leave();
}
#[on_event]
fn reconnect(_: Event<sand::events::OnJoinEvent>) {
    hud().cancel();
    roster(0).leave();
    roster(1).leave();
}

/// Explicit setup for a disposable world, with both platform chunks loaded.
/// Creates floors at y=100 around x=0 and x=32 in the invoking dimension.
#[function("cooperative_arena:setup")]
pub fn setup() {
    Execute::new()
        .in_("minecraft:overworld")
        .run_actions(setup_actions());
}
fn setup_actions() -> Actions {
    let mut actions = Actions::default();
    for arena in 0..2 {
        actions.extend([
            hosts(arena).each(|_| round(arena).cancel()),
            hosts(arena).each(|host| host.identity().kill().unwrap()),
        ]);
        for x in -6..=6 {
            for z in -6..=6 {
                actions.extend([WorldContext::current()
                    .block(BlockPos::new(x + arena as i32 * 32, 100, z))
                    .replace(vanilla::Block::Stone)]);
            }
        }
        actions.extend([
            Execute::new()
                .positioned(Vec3::new(arena as f64 * 32.0, 101.0, 0.0))
                .summon(EntityTypeId::minecraft("marker").unwrap())
                .run_actions(run_fn!({
                    Boss::attach(EntityContext::<AnyEntity>::default());
                    EntityContext::<AnyEntity>::default()
                        .identity()
                        .add_tag(&EntityTag::new(tag(arena)).unwrap())
                        .unwrap();
                })),
            reopen(arena),
        ]);
    }
    actions
}
/// Stop both loaded rounds, invalidate rosters, and cancel all online HUDs.
/// Saved wins and platform blocks are deliberately retained.
#[function("cooperative_arena:reset")]
pub fn reset() {
    Execute::new()
        .in_("minecraft:overworld")
        .run_actions(reset_actions());
}
fn reset_actions() -> Actions {
    let mut actions = Actions::default();
    for arena in 0..2 {
        actions.extend([
            hosts(arena).each(|_| round(arena).reset()),
            roster(arena).close(),
            phase(arena).set(Phase::Results).into_commands(),
        ]);
    }
    actions.extend([Target::players().each(|_| hud().cancel())]);
    actions
}
#[datapack_component(Load)]
fn reload_policy() {
    phase(0).set(Phase::Results);
    phase(1).set(Phase::Results);
}
#[doc(hidden)]
pub fn __sand_export(namespace: &str, version: &str) {
    println!(
        "{}",
        sand::advanced::try_export_pack_json(namespace, version).unwrap()
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    #[function("cooperative_arena:proof_join_a", context = player)]
    fn join_a() {
        when(phase(0).is(Phase::Lobby)).then(admission(0, Role::Fighter));
    }
    #[function("cooperative_arena:proof_join_b", context = player)]
    fn join_b() {
        when(phase(1).is(Phase::Lobby)).then(admission(1, Role::Fighter));
    }
    #[function("cooperative_arena:proof_start_a", context = player)]
    fn start_a() {
        when(Condition::all([
            roster(0).is(Role::Fighter),
            phase(0).is(Phase::Lobby),
        ]))
        .then(hosts(0).each(|_| round(0).restart()));
    }
    #[function("cooperative_arena:proof_start_b", context = player)]
    fn start_b() {
        when(Condition::all([
            roster(1).is(Role::Fighter),
            phase(1).is(Phase::Lobby),
        ]))
        .then(hosts(1).each(|_| round(1).restart()));
    }
    #[function("cooperative_arena:proof_strike", context = player)]
    fn strike() {
        sneak_actions();
    }
    #[function("cooperative_arena:proof_reopen_a")]
    fn reopen_a() {
        when(phase(0).is(Phase::Results)).then(reopen(0));
    }
    #[test]
    fn export_native_arena_fixture_when_requested() {
        let Some(path) = std::env::var_os("SAND_ARENA_PROOF_PACK") else {
            return;
        };
        let root = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&root).unwrap();
        let pack: serde_json::Value = serde_json::from_str(
            &sand::advanced::try_export_pack_json("cooperative_arena", "26.3").unwrap(),
        )
        .unwrap();
        std::fs::write(root.join("pack.mcmeta"),r#"{"pack":{"description":"Cooperative arena proof","min_format":121,"max_format":121}}"#).unwrap();
        for record in pack["components"].as_array().unwrap() {
            let dir = record["dir"].as_str().unwrap();
            let ext = if dir == "function" {
                "mcfunction"
            } else {
                "json"
            };
            let path = root
                .join("data")
                .join(record["namespace"].as_str().unwrap())
                .join(dir)
                .join(format!("{}.{}", record["path"].as_str().unwrap(), ext));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let content = &record["content"];
            std::fs::write(
                path,
                content
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| content.to_string()),
            )
            .unwrap();
        }
        std::fs::write(root.join("fixture.json"),serde_json::json!({"member":Player::arena.objective(),"role":Player::role.objective(),"health":Player::health.objective(),"wins":Player::wins.objective(),"boss_health":Boss::health.objective()}).to_string()).unwrap();
    }

    #[test]
    fn both_arenas_export_deterministically() {
        for version in ["26.1", "26.2", "26.3"] {
            let first = sand::advanced::try_export_pack_json("cooperative_arena", version).unwrap();
            assert_eq!(
                first,
                sand::advanced::try_export_pack_json("cooperative_arena", version).unwrap()
            );
            for function in [
                "roster_contains",
                "session_restart",
                "hud_cleanup",
                "roster_any_1",
            ] {
                assert!(first.contains(function));
            }
        }
    }
}
