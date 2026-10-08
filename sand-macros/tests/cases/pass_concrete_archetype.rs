#![deny(warnings)]
use sand::prelude::*;
#[derive(State)]
#[state(namespace = "test", scope = living)]
pub struct Combat { pub health: Score }
#[derive(StateBundle)]
pub struct Nested { pub combat: Combat }
#[derive(Archetype)]
#[archetype(id = "test:seeker", entity = Zombie)]
pub struct Seeker { pub nested: Nested }
#[derive(Archetype)]
#[archetype(id = "test:marker", entity = Marker)]
pub struct Waypoint;
#[system(tick, every = 5)]
fn update(query: Seeker) {
    query.each(|seeker| seeker.nested.combat.health.add(1));
}
#[derive(Archetype)]
#[archetype(id = "test:hero", entity = Player)]
pub struct Hero;
fn main() {
    let _ = Hero::adopt(Target::players());
    let _ = Hero::attach(Default::default());
    let seeker = Seeker::on(EntityContext::<ZombieKind>::default());
    let _ = seeker.entity();
    let _ = Seeker::summon(Vec3::here());
    let _ = Seeker::adopt(Target::entities());
    let _ = Seeker::attach(Default::default());
    let _ = Seeker::detach(Default::default());
    let _ = Seeker::is_attached(Default::default());
    let _ = Waypoint::summon(Vec3::here());
}

#[derive(Archetype)]
#[archetype(id = "test:raw", entity = Zombie)]
pub struct RawNamed { pub r#type: Combat }
