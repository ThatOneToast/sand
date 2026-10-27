//! Public gameplay fixture for the dedicated two-player native furnace proof.
use sand::prelude::*;
use sand_core::serde_json;

#[derive(State)]
#[state(namespace = "furnace", name = "probe", scope = player)]
struct Probe {
    elapsed: Score,
    total: Score,
    burn: Score,
    hits: Score,
    ready: Score,
    after: Score,
}
#[derive(State)]
#[state(namespace = "furnace", name = "world", scope = global)]
struct WorldProbe {
    burn: Score,
}

#[function("snapshot", context = player)]
fn snapshot() {
    let state = Probe::on(EntityContext::<PlayerKind>::default());
    WorldContext::current().block(BlockPos::here()).when_furnace(|f| mcfunction![
        state.hits.add(1);
        state.elapsed.set(f.cooking_elapsed());
        state.total.set(f.cooking_total());
        state.burn.set(f.burn_remaining());
        state.ready.set(0);
        when(f.burn_remaining().gt(0).and(f.input().matches(vanilla::Item::RawIron).unwrap()))
            .then_all([state.ready.set(1)]);
    ]);
    state.after.add(1);
}

#[function]
fn global_snapshot() {
    WorldContext::current()
        .block(BlockPos::here())
        .when_furnace(|f| WorldProbe::global().burn.set(f.burn_remaining()));
}

#[function("seed_iron", context = player)]
fn seed_iron() {
    WorldContext::current()
        .block(BlockPos::here())
        .when_furnace(|f| {
            mcfunction![
                f.input().set(ItemStack::new(vanilla::Item::RawIron.into())).unwrap();
                f.fuel().set(ItemStack::new(vanilla::Item::Coal.into())).unwrap();
                f.output().clear().unwrap();
            ]
        });
}
#[function("seed_food", context = player)]
fn seed_food() {
    WorldContext::current()
        .block(BlockPos::here())
        .when_furnace(|f| {
            mcfunction![
                f.input().set(ItemStack::new(vanilla::Item::Beef.into())).unwrap();
                f.fuel().set(ItemStack::new(vanilla::Item::Coal.into())).unwrap();
                f.output().clear().unwrap();
            ]
        });
}
#[function("mutate_once", context = player)]
fn mutate_once() {
    let state = Probe::on(EntityContext::<PlayerKind>::default());
    let block = WorldContext::current().block(BlockPos::here());
    block.when_furnace(|_| {
        mcfunction![
            state.hits.add(1);
            block.replace(vanilla::Block::Smoker);
            cmd::return_fail();
            state.hits.add(100);
        ]
    });
    state.after.add(1);
}
#[function("fail_after_valid", context = player)]
fn fail_after_valid() {
    let state = Probe::on(EntityContext::<PlayerKind>::default());
    let block = WorldContext::current().block(BlockPos::here());
    block.when_furnace(|f| {
        mcfunction![
            state.burn.set(f.burn_remaining());
            block.replace(vanilla::Block::Air);
            state.burn.set(f.burn_remaining());
            state.after.add(1);
        ]
    });
}
#[function("nested", context = player)]
fn nested() {
    let state = Probe::on(EntityContext::<PlayerKind>::default());
    let world = WorldContext::current();
    world.block(BlockPos::here()).when_furnace(|outer| mcfunction![
        state.burn.set(outer.burn_remaining());
        world.block(BlockPos::above(1)).when_furnace(|inner| state.total.set(inner.burn_remaining()));
        state.elapsed.set(outer.burn_remaining());
    ]);
}
#[function("in_nether", context = player)]
fn in_nether() {
    WorldContext::current().in_dimension(
        sand::registry::DimensionId::minecraft("the_nether").unwrap(),
        |_| cmd::function(snapshot),
    );
    Probe::on(EntityContext::<PlayerKind>::default())
        .after
        .add(10);
}
#[function("local_snapshot", context = player)]
fn local_snapshot() {
    let state = Probe::on(EntityContext::<PlayerKind>::default());
    WorldContext::current()
        .block(BlockPos::new(
            Coord::local(),
            Coord::local(),
            Coord::local_n(1),
        ))
        .when_furnace(|f| state.burn.set(f.burn_remaining()));
}

fn pack() -> serde_json::Value {
    serde_json::from_str(&sand::advanced::try_export_pack_json("furnace", "26.2").unwrap()).unwrap()
}
#[test]
fn public_furnace_fixture_exports_state_and_conditions() {
    let pack = pack();
    let text = pack.to_string();
    for key in [
        "cooking_time_spent",
        "cooking_total_time",
        "lit_time_remaining",
        "container.0",
        "container.1",
        "container.2",
    ] {
        assert!(text.contains(key), "{key}");
    }
    assert_eq!(pack, self::pack());
}
#[test]
#[ignore = "dedicated native furnace fixture export"]
fn dump_furnace_pack() {
    let state = Probe::on(EntityContext::<PlayerKind>::default());
    let evidence = serde_json::json!({
        "components": pack()["components"],
        "objectives": {"elapsed": state.elapsed.get().objective(), "total": state.total.get().objective(),
            "burn": state.burn.get().objective(), "hits": state.hits.get().objective(),
            "ready": state.ready.get().objective(), "after": state.after.get().objective()},
        "global_burn_query": WorldProbe::global().burn.get().command(),
    });
    std::fs::write(
        std::env::var("SAND_FURNACE_DUMP_PATH").unwrap(),
        evidence.to_string(),
    )
    .unwrap();
}
