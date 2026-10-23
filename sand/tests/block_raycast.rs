//! Maintained Sand-only authoring fixture, also exported by the native harness.
use sand::command::Anchor;
use sand::prelude::*;
use sand::registration::{DatapackRegistration, LifecycleContribution};
use sand::systems::raycast::BlockRaycast;
use sand_core::serde_json::{self, Value};

const HITS: Objective = Objective::new("ray_hits");
const MISSES: Objective = Objective::new("ray_misses");
const OTHER: Objective = Objective::new("ray_other");
const PLAYER: Objective = Objective::new("ray_player");

fn local(path: &str) -> ResourceLocation {
    ResourceLocation::new("__sand_local", path).unwrap()
}
fn block(id: &str) -> BlockPredicate {
    BlockPredicate::new().blocks(vec![BlockId::minecraft(id).unwrap()])
}
fn definition(name: &str) -> BlockRaycast {
    BlockRaycast::new(local(name))
        .origin(Anchor::Feet)
        .max_distance(3.0)
        .step(1.0)
        .scan_for(block("gold_block"))
        .on_hit(hit)
        .on_miss(miss)
}
#[datapack_component]
fn primary() -> BlockRaycast {
    definition("primary")
}
#[datapack_component]
fn eyes() -> BlockRaycast {
    definition("eyes").origin(Anchor::Eyes)
}
#[datapack_component]
fn secondary() -> BlockRaycast {
    definition("secondary")
        .scan_for(block("iron_block"))
        .on_hit(other_hit)
}
#[datapack_component]
fn collision() -> BlockRaycast {
    definition("collision")
        .first_collision()
        .pass_through(block("air"))
}
#[datapack_component]
fn tagged() -> BlockRaycast {
    definition("tagged").scan_for(BlockPredicate::new().tag(TagId::minecraft("logs").unwrap()))
}
#[datapack_component]
fn short() -> BlockRaycast {
    definition("short").max_distance(0.5).step(2.0)
}
#[datapack_component]
fn remainder() -> BlockRaycast {
    definition("remainder").max_distance(2.5).step(1.0)
}
#[datapack_component]
fn decimal_multiple() -> BlockRaycast {
    definition("decimal_multiple").max_distance(0.9).step(0.03)
}
#[datapack_component]
fn decimal_fraction() -> BlockRaycast {
    definition("decimal_fraction")
        .max_distance(0.900_000_000_001)
        .step(0.03)
}
#[datapack_component]
fn player_callbacks() -> BlockRaycast {
    definition("player_callbacks")
        .on_hit(player_hit)
        .on_miss(player_hit)
}
#[datapack_component]
fn hit_only() -> BlockRaycast {
    BlockRaycast::new(local("hit_only"))
        .origin(Anchor::Feet)
        .max_distance(3.0)
        .step(1.0)
        .scan_for(block("gold_block"))
        .on_hit(hit)
}
#[function]
fn cast_hit_only() {
    hit_only().cast();
}
#[datapack_component]
fn evidence_objectives() -> DatapackRegistration {
    [HITS, MISSES, OTHER, PLAYER].iter().fold(
        DatapackRegistration::new(),
        |registration, objective| {
            registration.lifecycle(LifecycleContribution::load(objective.create("dummy")))
        },
    )
}

#[function]
fn hit() {
    HITS.add(ScoreHolder::self_(), 1);
    WorldContext::current()
        .block(BlockPos::here())
        .replace(vanilla::Block::DiamondBlock);
    when(Condition::entity(Target::self_().tag("ray_nested"))).then_one(cmd::function(nested));
    when(Condition::entity(Target::self_().tag("ray_other_nested")))
        .then_one(cmd::function(nested_other));
}
#[function]
fn nested() {
    cmd::tag_remove(Target::self_(), "ray_nested");
    primary().cast();
    WorldContext::current()
        .block(BlockPos::here())
        .replace(vanilla::Block::EmeraldBlock);
}
#[function]
fn nested_other() {
    cmd::tag_remove(Target::self_(), "ray_other_nested");
    secondary().cast();
    WorldContext::current()
        .block(BlockPos::here())
        .replace(vanilla::Block::EmeraldBlock);
}
#[function]
fn miss() {
    MISSES.add(ScoreHolder::self_(), 1);
    WorldContext::current()
        .block(BlockPos::here())
        .replace(vanilla::Block::RedstoneBlock);
}
#[function]
fn other_hit() {
    OTHER.add(ScoreHolder::self_(), 1);
    WorldContext::current()
        .block(BlockPos::here())
        .replace(vanilla::Block::LapisBlock);
}
#[function("player_hit", context = player)]
fn player_hit() {
    PLAYER.add(ScoreHolder::self_(), 1);
}
#[function]
fn cast_primary() {
    primary().cast();
}
#[function]
fn cast_eyes() {
    eyes().cast();
}
#[function]
fn cast_secondary() {
    secondary().cast();
}
#[function]
fn cast_collision() {
    collision().cast();
}
#[function]
fn cast_tagged() {
    tagged().cast();
}
#[function]
fn cast_short() {
    short().cast();
}
#[function]
fn cast_remainder() {
    remainder().cast();
}
#[function]
fn cast_player_callbacks() {
    player_callbacks().cast();
}

fn snapshot(namespace: &str) -> Value {
    serde_json::from_str(&sand::advanced::try_export_pack_json(namespace, "26.2").unwrap()).unwrap()
}
fn resource<'a>(pack: &'a Value, namespace: &str, dir: &str, path: &str) -> &'a str {
    pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["namespace"] == namespace && r["dir"] == dir && r["path"] == path)
        .unwrap()["content"]
        .as_str()
        .unwrap()
}
#[test]
fn raycast_export_preserves_terminal_calls_guards_and_predicates() {
    let pack = snapshot("rays");
    let path = |name: &str, part: &str| format!("__sand_system/raycast/{name}/{part}");
    let start = resource(&pack, "rays", "function", &path("eyes", "start"));
    assert!(
        start.starts_with("execute unless entity @s run return 0\n"),
        "{start}"
    );
    assert!(
        start.contains(
            "return run execute anchored eyes positioned ^ ^ ^ anchored feet run function rays:"
        ),
        "{start}"
    );
    let sample = resource(&pack, "rays", "function", &path("primary", "sample"));
    assert!(sample.contains("execute unless loaded ~ ~ ~ run return run function rays:__sand_system/raycast/primary/miss"), "{sample}");
    assert!(sample.contains("execute if predicate rays:__sand_system/raycast/primary/criterion run return run function rays:__sand_system/raycast/primary/hit"), "{sample}");
    assert!(sample.contains("execute unless predicate rays:__sand_system/raycast/primary/valid_sample run return run function rays:__sand_system/raycast/primary/miss"), "{sample}");
    assert!(sample.ends_with("return run function rays:__sand_system/raycast/primary/advance"));
    let silent_miss = resource(&pack, "rays", "function", &path("hit_only", "miss"));
    assert!(
        silent_miss.starts_with("scoreboard players reset @s "),
        "{silent_miss}"
    );
    assert!(silent_miss.ends_with("\nreturn 0"), "{silent_miss}");
    let player = resource(&pack, "rays", "function", &path("player_callbacks", "hit"));
    assert!(
        player.contains(
            "return run execute if entity @s[type=minecraft:player] run function rays:player_hit"
        ),
        "{player}"
    );
    let collision = resource(&pack, "rays", "function", &path("collision", "sample"));
    assert!(collision.contains("unless predicate rays:__sand_system/raycast/collision/criterion"));
    let tag: Value = serde_json::from_str(resource(
        &pack,
        "rays",
        "predicate",
        &path("tagged", "criterion"),
    ))
    .unwrap();
    assert_eq!(tag["predicate"]["block"]["blocks"], "#minecraft:logs");
    for name in ["short", "remainder"] {
        let advance = resource(&pack, "rays", "function", &path(name, "advance"));
        assert!(advance.contains("positioned ^ ^ ^0.5"), "{advance}");
    }
    for (name, distance, count) in [
        ("decimal_multiple", 0.9, 30),
        ("decimal_fraction", 0.900_000_000_001, 31),
    ] {
        let start = resource(&pack, "rays", "function", &path(name, "start"));
        assert!(
            start
                .lines()
                .nth(1)
                .unwrap()
                .ends_with(&format!(" {count}"))
        );
        let advance = resource(&pack, "rays", "function", &path(name, "advance"));
        let final_step: f64 = advance
            .lines()
            .nth(1)
            .unwrap()
            .split("positioned ^ ^ ^")
            .nth(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!(final_step > 0.0);
        assert_eq!(0.03 * (count - 1) as f64 + final_step, distance);
        if name == "decimal_multiple" {
            assert!((final_step - 0.03).abs() < 1e-15, "{advance}");
        } else {
            assert!(final_step < 1e-10, "{advance}");
        }
    }
    assert!(!pack.to_string().contains("__sand_local"));
    assert_eq!(pack, snapshot("rays"));
    std::thread::scope(|scope| {
        let same = scope.spawn(|| snapshot("rays"));
        let other = scope.spawn(|| snapshot("other"));
        assert_eq!(pack, same.join().unwrap());
        let other = other.join().unwrap();
        assert!(!other.to_string().contains("rays:"));
        assert_ne!(
            resource(&pack, "rays", "function", &path("primary", "load")),
            resource(&other, "other", "function", &path("primary", "load"))
        );
    });
}

#[test]
fn sample_count_has_finite_bounded_endpoint_semantics() {
    for (distance, step, count) in [
        (3.0, 1.0, 3),
        (2.5, 1.0, 3),
        (0.5, 2.0, 1),
        (256.0, 1.0, 256),
        (0.9, 0.03, 30),
        (0.3, 0.1, 3),
        (2.55, 0.01, 255),
        (7.68, 0.03, 256),
        (7.68_f64.next_up(), 0.03, 256),
        (0.900_000_000_001, 0.03, 31),
        (f64::MIN_POSITIVE, f64::MAX, 1),
    ] {
        assert_eq!(
            definition("count")
                .max_distance(distance)
                .step(step)
                .sample_count()
                .unwrap(),
            count
        );
    }
    for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            definition("count")
                .max_distance(invalid)
                .sample_count()
                .is_err()
        );
        assert!(definition("count").step(invalid).sample_count().is_err());
    }
    assert!(
        definition("count")
            .max_distance(257.0)
            .sample_count()
            .is_err()
    );
    assert!(
        definition("count")
            .max_distance(7.680_000_000_001)
            .step(0.03)
            .sample_count()
            .is_err()
    );
}

#[test]
#[ignore = "maintained native-server fixture export"]
fn dump_raycast_pack() {
    std::fs::write(
        std::env::var("SAND_RAYCAST_DUMP_PATH").unwrap(),
        serde_json::to_string(&snapshot("raycast")["components"]).unwrap(),
    )
    .unwrap();
}
