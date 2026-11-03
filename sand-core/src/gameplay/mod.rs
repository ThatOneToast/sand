//! Reusable gameplay built from State, typed actions and ordinary registration.
pub(crate) mod identity;
mod lifetime;
pub use lifetime::{Lifetime, TaskContext};
mod ability;
pub use ability::Ability;
mod projectile;
pub use projectile::Projectile;
mod status;
pub use status::StatusEffect;
mod testing;
pub use testing::GameplayTest;

mod job;
mod runtime;
pub use job::Job;
mod scene;
pub use scene::{Scene, Transform};

mod sequence;
pub use sequence::Sequence;

pub(crate) mod combat;
mod combat_lower;
pub use combat::Combat;

mod encounter;
pub use encounter::Encounter;

mod menu;
pub use menu::Menu;

mod interactable;
pub use interactable::Interactable;

mod purchase;
pub use purchase::Purchase;
