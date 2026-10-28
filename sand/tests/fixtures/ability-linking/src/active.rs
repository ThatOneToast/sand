mod feature;
use sand::prelude::*;
use sand_ability_linking_consumer as schema;
#[datapack_component]
fn maintenance() -> sand::systems::ability::ResourceMaintenance {
    feature::maintenance()
}
#[function("activate", context = player)]
fn activate(player: EntityContext<PlayerKind>) {
    feature::ability(player).accept(Condition::all([]), cmd::say("accepted"));
}
fn main() {
    schema::export();
}
