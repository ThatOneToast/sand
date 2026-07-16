//! Typed interactable entity builders.
//!
//! An `interaction` entity (added in Minecraft 1.20.2) is a zero-size invisible entity
//! that fires `PlayerInteractedWithEntity` when right-clicked. This makes it ideal for
//! custom clickable objects — doors, NPCs, UI panels — without any visible hitbox.
//!
//! # Example
//! ```rust,ignore
//! use sand_core::systems::entities::{Interactable, InteractSize};
//! use sand_core::cmd::Vec3;
//! use sand_core::ResourceLocation;
//!
//! // Build a 1×2 interactable door-sized hitbox at a fixed position:
//! let interact = Interactable::new(ResourceLocation::parse("my_pack:door_trigger").unwrap())
//!     .size(InteractSize { width: 1.0, height: 2.0 })
//!     .response("my_pack:functions/on_door_open");
//!
//! // Summon command:
//! let summon_cmd = interact.summon_at(Vec3::relative(0.5, 0.0, 0.5));
//!
//! // Advancement that fires when a player right-clicks it:
//! let adv = interact.advancement();
//! ```

use sand_commands::Vec3;

use crate::function::IntoFunctionRef;
use crate::{Advancement, AdvancementRewards, AdvancementTrigger, Criterion, ResourceLocation};
use sand_components::predicates::EntityPredicate;

/// Width and height of the interaction entity's hitbox in blocks.
#[derive(Debug, Clone, Copy)]
pub struct InteractSize {
    pub width: f32,
    pub height: f32,
}

impl Default for InteractSize {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
        }
    }
}

/// Builder for an `interaction` entity with optional size and a typed reward function.
///
/// Produces:
/// - A `summon` command (via `summon_at` / `summon_here`)
/// - An `Advancement` that fires on right-click via `PlayerInteractedWithEntity`
#[derive(Debug, Clone)]
pub struct Interactable {
    advancement_location: ResourceLocation,
    size: InteractSize,
    response: Option<String>,
    entity_filter: Option<EntityPredicate>,
    fixed_tag: Option<String>,
}

impl Interactable {
    /// Create a new `Interactable` builder.
    ///
    /// `advancement_location` is the resource location registered via `#[component]`
    /// for the resulting advancement.
    pub fn new(advancement_location: ResourceLocation) -> Self {
        Self {
            advancement_location,
            size: InteractSize::default(),
            response: None,
            entity_filter: None,
            fixed_tag: None,
        }
    }

    /// Set the hitbox dimensions (default: 1×1 blocks).
    pub fn size(mut self, size: InteractSize) -> Self {
        self.size = size;
        self
    }

    /// Set a typed function ref to call when the player interacts.
    pub fn response(mut self, handler: impl IntoFunctionRef) -> Self {
        self.response = Some(handler.into_function_id());
        self
    }

    /// Filter by a tag that must be present on the interaction entity.
    ///
    /// Use this when multiple interaction entities are in the world and you need
    /// a unique identifier (combine with the entity filter on the advancement).
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        let t = tag.into();
        self.fixed_tag = Some(t.clone());
        self.entity_filter = Some(EntityPredicate::new().nbt(format!("{{Tags:[\"{t}\"]}}")));
        self
    }

    /// The raw NBT compound appended to the `summon` command for the interaction entity.
    fn nbt(&self) -> String {
        let mut parts = vec![
            format!("width:{:.1}f", self.size.width),
            format!("height:{:.1}f", self.size.height),
            "response:1b".into(),
        ];
        if let Some(ref tag) = self.fixed_tag {
            parts.push(format!("Tags:[\"{tag}\"]"));
        }
        format!("{{{}}}", parts.join(","))
    }

    /// `summon minecraft:interaction <pos> {width:W,height:H,response:1b[,Tags:[...]]}`
    pub fn summon_at(&self, pos: Vec3) -> String {
        format!("summon minecraft:interaction {} {}", pos, self.nbt())
    }

    /// `summon minecraft:interaction ~ ~ ~ {width:W,height:H,response:1b[,Tags:[...]]}`
    pub fn summon_here(&self) -> String {
        self.summon_at(Vec3::here())
    }

    /// Build the advancement that fires when any player right-clicks this entity.
    ///
    /// Panics if no `response` function was set (no reward would be registered).
    pub fn advancement(&self) -> Advancement {
        let reward_fn = self.response.clone().expect(
            "Interactable::advancement() requires a response function — call .response(fn) first",
        );

        Advancement::new(self.advancement_location.clone())
            .criterion(
                "interacted",
                Criterion::new(AdvancementTrigger::PlayerInteractedWithEntity {
                    item: None,
                    entity: self.entity_filter.clone(),
                }),
            )
            .rewards(AdvancementRewards::new().function(reward_fn))
    }

    /// Build the advancement, providing the response function now (overrides any set earlier).
    pub fn advancement_with(self, handler: impl IntoFunctionRef) -> Advancement {
        self.response(handler).advancement()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sand_commands::Vec3;
    use sand_components::component::DatapackComponent;

    fn loc(ns: &str, path: &str) -> ResourceLocation {
        ResourceLocation::new(ns, path).unwrap()
    }

    #[test]
    fn summon_default_size() {
        let interactable =
            Interactable::new(loc("my_pack", "triggers/door")).response("my_pack:on_door");
        let cmd = interactable.summon_here();
        assert_eq!(
            cmd,
            "summon minecraft:interaction ~ ~ ~ {width:1.0f,height:1.0f,response:1b}"
        );
    }

    #[test]
    fn summon_custom_size() {
        let interactable = Interactable::new(loc("my_pack", "triggers/door"))
            .size(InteractSize {
                width: 1.0,
                height: 2.0,
            })
            .response("my_pack:on_door");
        let cmd = interactable.summon_at(Vec3::absolute(10.0, 64.0, -5.0));
        assert_eq!(
            cmd,
            "summon minecraft:interaction 10 64 -5 {width:1.0f,height:2.0f,response:1b}"
        );
    }

    #[test]
    fn summon_with_tag() {
        let interactable = Interactable::new(loc("my_pack", "triggers/door"))
            .tag("my_pack_door")
            .response("my_pack:on_door");
        let cmd = interactable.summon_here();
        assert_eq!(
            cmd,
            "summon minecraft:interaction ~ ~ ~ {width:1.0f,height:1.0f,response:1b,Tags:[\"my_pack_door\"]}"
        );
    }

    #[test]
    fn advancement_has_reward() {
        let interactable = Interactable::new(loc("my_pack", "triggers/door"))
            .response("my_pack:functions/on_door");
        let adv = interactable.advancement();
        let v = adv.to_json();
        assert_eq!(v["rewards"]["function"], "my_pack:functions/on_door");
    }

    #[test]
    fn advancement_trigger_is_interacted() {
        let interactable = Interactable::new(loc("my_pack", "triggers/door"))
            .response("my_pack:functions/on_door");
        let adv = interactable.advancement();
        let v = adv.to_json();
        assert_eq!(
            v["criteria"]["interacted"]["trigger"],
            "minecraft:player_interacted_with_entity"
        );
    }

    #[test]
    fn advancement_with_tag_filter() {
        let interactable = Interactable::new(loc("my_pack", "triggers/door"))
            .tag("my_pack_door")
            .response("my_pack:on_door");
        let adv = interactable.advancement();
        let v = adv.to_json();
        // Entity predicate should filter by nbt tag
        let entity_cond = &v["criteria"]["interacted"]["conditions"]["entity"];
        assert_eq!(
            entity_cond[0]["condition"], "minecraft:entity_properties",
            "entity condition should use the advancement entity consumer"
        );
        assert_eq!(entity_cond[0]["entity"], "this");
        assert!(entity_cond[0]["predicate"]["minecraft:nbt"].is_string());
    }

    #[test]
    fn advancement_with_fn() {
        let interactable = Interactable::new(loc("my_pack", "triggers/btn"));
        let adv = interactable.advancement_with("my_pack:functions/on_btn");
        let v = adv.to_json();
        assert_eq!(v["rewards"]["function"], "my_pack:functions/on_btn");
    }

    #[test]
    #[should_panic(expected = "requires a response function")]
    fn advancement_without_response_panics() {
        let interactable = Interactable::new(loc("my_pack", "triggers/door"));
        let _ = interactable.advancement();
    }
}
