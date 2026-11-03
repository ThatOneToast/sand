#[allow(dead_code)]
mod feature;
use sand::prelude::*;
use sand_ability_linking_consumer as schema;
#[datapack_component]
fn maintenance() -> sand::systems::ability::ResourceMaintenance {
    feature::maintenance()
}
fn main() {
    schema::export();
}
