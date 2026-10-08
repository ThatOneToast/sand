use sand::prelude::*;
#[derive(State)]
#[state(namespace = "test", scope = entity)]
struct Health { value: Score }
#[derive(Archetype)]
#[archetype(id = "test:bad", entity = Marker, configure = Self::configure)]
struct Bad { health: Health }
impl Bad {
    fn configure(base: EntityArchetype<MarkerKind>) -> EntityArchetype<MarkerKind> {
        base.health(HealthBinding::new(Health::value))
    }
}
fn main() {}
