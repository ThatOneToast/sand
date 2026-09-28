//! Rust frontend of the portable-program acceptance counter.
//!
//! Iteration changes only the executor, never its position. Canonical automatic
//! lifecycle work initializes player State before the tick entry runs.

use sand::prelude::*;
use sand::registration::{DatapackRegistration, FunctionTagContribution};

#[derive(State)]
#[state(namespace = "demo", name = "counter", scope = player, version = 1)]
pub struct Counter {
    #[state(default = 0)]
    pub value: Score,
}

#[function("demo:increment", context = player)]
pub fn increment() {
    Counter::on(EntityContext::<PlayerKind>::default())
        .value
        .add(1);
}

#[function("demo:tick")]
pub fn tick() {
    Execute::new()
        .as_(Target::players())
        .run(cmd::function(increment));
}

#[datapack_component]
fn tick_membership() -> DatapackRegistration {
    DatapackRegistration::new().function_tag(FunctionTagContribution::new(
        "minecraft:tick".parse().unwrap(),
        FunctionId::custom("demo:tick".parse().unwrap()),
    ))
}

/// Export this linked Rust frontend through Sand's supported collection hook.
pub fn export() -> String {
    sand::advanced::try_export_components_json("demo", "26.2")
        .expect("the acceptance counter must compile")
}
