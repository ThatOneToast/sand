use sand::prelude::*;

#[derive(State)]
#[state(namespace = "test", scope = entity, name = "marker", version = 1)]
struct MarkerState {
    #[state(default = 20, min = 1, max = 100)]
    health: EntityScore<i32>,
}

fn main() {
    let archetype = EntityArchetype::<MarkerKind, MarkerState>::new(
        ResourceLocation::new("test", "marker").unwrap(),
    );
    let _ = archetype.health(HealthBinding::new(MarkerState::health));
}
