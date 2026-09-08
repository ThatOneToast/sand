//! Focused, capability-gated operations for execution-scoped entities.

use std::marker::PhantomData;

use sand_commands::{DataCommand, DataTarget, NbtPath, NbtRef, NbtValue, Selector, UntypedNbt};
use sand_components::{AttributeType, EffectId, EquipmentSlot};

use crate::cmd::{Anchor, CommandResult, DamageKind, EffectGive, One, Rotation, Target, Vec3};
use crate::entity::kind::{
    EntityKind, EquipmentEntityKind, LivingEntityKind, SafeEntityDataWriteKind,
};
use crate::entity::property::{EntityTag, EntityTeam};
use crate::item::{ItemLocation, ItemLocationError};

/// Identity and lifecycle operations for one execution-scoped entity.
#[derive(Debug, Clone)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityIdentity",
    aliases = ["sand::prelude::EntityIdentity"],
    module = "sand::entity",
    summary = "Identity and lifecycle operations for one execution-scoped entity.",
    context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
    minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
    use_when = ["Discovering legal operations on the current or scoped entity"],
    avoid_when = ["Keeping an entity identity across ticks"],
    example = "use sand::prelude::*;",
)]
pub struct EntityIdentity<K> {
    selector: Selector,
    marker: PhantomData<fn() -> K>,
}

impl<K: EntityKind> EntityIdentity<K> {
    pub(crate) fn new(selector: Selector) -> Self {
        Self {
            selector,
            marker: PhantomData,
        }
    }

    /// Add a validated entity tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityIdentity::add_tag",
        aliases = ["sand::prelude::EntityIdentity::add_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "Adds a validated entity tag.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(tag = "The typed tag used by this operation."),
    )]
    pub fn add_tag(&self, tag: &EntityTag) -> String {
        sand_commands::builtins::tag_add(self.selector.clone(), tag.as_str())
    }

    /// Remove a validated entity tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityIdentity::remove_tag",
        aliases = ["sand::prelude::EntityIdentity::remove_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "Removes a validated entity tag.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(tag = "The typed tag used by this operation."),
    )]
    pub fn remove_tag(&self, tag: &EntityTag) -> String {
        sand_commands::builtins::tag_remove(self.selector.clone(), tag.as_str())
    }

    /// Join a validated scoreboard team.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityIdentity::join_team",
        aliases = ["sand::prelude::EntityIdentity::join_team"],
        module = "sand::entity",
        kind = "method",
        summary = "Joins a validated scoreboard team.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(team = "The typed team used by this operation."),
    )]
    pub fn join_team(&self, team: &EntityTeam) -> String {
        sand_commands::builtins::team_join(team.as_str(), self.selector.clone())
    }

    /// Leave the current scoreboard team.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityIdentity::leave_team",
        aliases = ["sand::prelude::EntityIdentity::leave_team"],
        module = "sand::entity",
        kind = "method",
        summary = "Leaves the current scoreboard team.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn leave_team(&self) -> String {
        sand_commands::builtins::team_leave(self.selector.clone())
    }

    /// Kill the entity. For non-living entities vanilla removes it directly.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityIdentity::kill",
        aliases = ["sand::prelude::EntityIdentity::kill"],
        module = "sand::entity",
        kind = "method",
        summary = "Kills or removes the entity using vanilla kill semantics.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn kill(&self) -> String {
        sand_commands::builtins::kill(self.selector.clone())
    }
}

/// Teleportation, position, rotation, and facing operations.
#[derive(Debug, Clone)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityTransform",
    aliases = ["sand::prelude::EntityTransform"],
    module = "sand::entity",
    summary = "Teleportation, position, rotation, and facing operations for one entity.",
    context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
    minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
    use_when = ["Discovering legal operations on the current or scoped entity"],
    avoid_when = ["Keeping an entity identity across ticks"],
    example = "use sand::prelude::*;",
)]
pub struct EntityTransform<K> {
    selector: Selector,
    marker: PhantomData<fn() -> K>,
}

impl<K: EntityKind> EntityTransform<K> {
    pub(crate) fn new(selector: Selector) -> Self {
        Self {
            selector,
            marker: PhantomData,
        }
    }

    /// Returns a typed view of vanilla's `Pos` list.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransform::position",
        aliases = ["sand::prelude::EntityTransform::position"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns a typed view of the entity's native position.",
        context = "The returned EntityData retains the capability selector. Writes remain capability-gated, and teleport is the recommended movement operation.",
        minecraft = "Reads vanilla's three-double Pos list through Sand's canonical typed data API.",
        use_when = ["Reading or copying the entity's current native position"],
        avoid_when = ["Moving an entity; use teleport instead"],
        returns = "A typed entity-data path for the native Pos list.",
        example = "use sand::prelude::*;",
    )]
    pub fn position(&self) -> EntityData<K, Vec<f64>> {
        EntityDataRoot::<K>::new(self.selector.clone()).field("Pos")
    }

    /// Returns a typed view of vanilla's `Rotation` list.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransform::rotation",
        aliases = ["sand::prelude::EntityTransform::rotation"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns a typed view of the entity's native rotation.",
        context = "The returned EntityData retains the capability selector. Writes remain capability-gated, and rotate or facing operations are recommended for movement.",
        minecraft = "Reads vanilla's yaw-and-pitch Rotation list through Sand's canonical typed data API.",
        use_when = ["Reading or copying the entity's current native rotation"],
        avoid_when = ["Changing rotation; use rotate or a facing operation"],
        returns = "A typed entity-data path for the native Rotation list.",
        example = "use sand::prelude::*;",
    )]
    pub fn rotation(&self) -> EntityData<K, Vec<f32>> {
        EntityDataRoot::<K>::new(self.selector.clone()).field("Rotation")
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransform::teleport",
        aliases = ["sand::prelude::EntityTransform::teleport"],
        module = "sand::entity",
        kind = "method",
        summary = "Teleports the entity to a typed position.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(position = "The typed position used by this operation."),
    )]
    pub fn teleport(&self, position: Vec3) -> String {
        #[cfg(sand_placeholder_codegen)]
        {
            let _ = (&self.selector, position);
            panic!("teleport is unavailable in an explicit placeholder-codegen build");
        }
        #[cfg(not(sand_placeholder_codegen))]
        crate::cmd::teleport_4(self.selector.clone(), position).to_string()
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransform::teleport_to",
        aliases = ["sand::prelude::EntityTransform::teleport_to"],
        module = "sand::entity",
        kind = "method",
        summary = "Teleports the entity to another selected entity.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*; let entity = EntityContext::<ZombieKind>::default(); let command = entity.transform().teleport_to(Target::nearest_player());",
        returns = "The requested capability value or canonical Minecraft command.",
        params(destination = "The typed destination used by this operation."),
    )]
    pub fn teleport_to<TargetKind>(&self, destination: Target<TargetKind, One>) -> String {
        #[cfg(sand_placeholder_codegen)]
        {
            let _ = (&self.selector, destination);
            panic!("teleport is unavailable in an explicit placeholder-codegen build");
        }
        #[cfg(not(sand_placeholder_codegen))]
        crate::cmd::teleport_3(self.selector.clone(), destination).to_string()
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransform::rotate",
        aliases = ["sand::prelude::EntityTransform::rotate"],
        module = "sand::entity",
        kind = "method",
        summary = "Sets the entity's typed yaw and pitch.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(rotation = "The typed rotation used by this operation."),
    )]
    pub fn rotate(&self, rotation: Rotation) -> String {
        #[cfg(sand_placeholder_codegen)]
        {
            let _ = (&self.selector, rotation);
            panic!("rotate is unavailable in an explicit placeholder-codegen build");
        }
        #[cfg(not(sand_placeholder_codegen))]
        crate::cmd::rotate(self.selector.clone(), rotation).to_string()
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransform::face_position",
        aliases = ["sand::prelude::EntityTransform::face_position"],
        module = "sand::entity",
        kind = "method",
        summary = "Rotates the entity to face a typed position.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(position = "The typed position used by this operation."),
    )]
    pub fn face_position(&self, position: Vec3) -> String {
        #[cfg(sand_placeholder_codegen)]
        {
            let _ = (&self.selector, position);
            panic!("rotate is unavailable in an explicit placeholder-codegen build");
        }
        #[cfg(not(sand_placeholder_codegen))]
        crate::cmd::rotate_facing(self.selector.clone(), position).to_string()
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransform::face_entity",
        aliases = ["sand::prelude::EntityTransform::face_entity"],
        module = "sand::entity",
        kind = "method",
        summary = "Rotates the entity to face another entity at the chosen anchor.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*; let entity = EntityContext::<ZombieKind>::default(); let command = entity.transform().face_entity(Target::nearest_player(), Anchor::Eyes);",
        returns = "The requested capability value or canonical Minecraft command.",
        params(target = "The typed target used by this operation.", anchor = "The typed anchor used by this operation."),
    )]
    pub fn face_entity<TargetKind>(
        &self,
        target: Target<TargetKind, One>,
        anchor: Anchor,
    ) -> String {
        #[cfg(sand_placeholder_codegen)]
        {
            let _ = (&self.selector, target, anchor);
            panic!("rotate is unavailable in an explicit placeholder-codegen build");
        }
        #[cfg(not(sand_placeholder_codegen))]
        crate::cmd::rotate_facing_entity(self.selector.clone(), target)
            .facingAnchor(anchor)
            .to_string()
    }
}

/// Living-only health, damage, effects, and attribute operations.
#[derive(Debug, Clone)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::LivingEntity",
    aliases = ["sand::prelude::LivingEntity"],
    module = "sand::entity",
    summary = "Living-only health, damage, effects, and attribute operations.",
    context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
    minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
    use_when = ["Discovering legal operations on the current or scoped entity"],
    avoid_when = ["Keeping an entity identity across ticks"],
    example = "use sand::prelude::*;",
)]
pub struct LivingEntity<K> {
    selector: Selector,
    marker: PhantomData<fn() -> K>,
}

/// Capability gating rejects living operations for known non-living kinds.
///
/// ```compile_fail
/// use sand_core::entity::{EntityContext, MarkerKind};
/// let marker = EntityContext::<MarkerKind>::default();
/// marker.living().damage(1.0, sand_core::cmd::DamageKind::Generic);
/// ```
impl<K: LivingEntityKind> LivingEntity<K> {
    pub(crate) fn new(selector: Selector) -> Self {
        Self {
            selector,
            marker: PhantomData,
        }
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::LivingEntity::health",
        aliases = ["sand::prelude::LivingEntity::health"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns a capability-gated typed view of the native Health field.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn health(&self) -> EntityData<K, f32> {
        EntityDataRoot::<K>::new(self.selector.clone()).field("Health")
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::LivingEntity::damage",
        aliases = ["sand::prelude::LivingEntity::damage"],
        module = "sand::entity",
        kind = "method",
        summary = "Applies validated typed damage to this entity.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(amount = "The typed amount used by this operation.", kind = "The typed kind used by this operation."),
    )]
    pub fn damage(&self, amount: f64, kind: DamageKind) -> CommandResult<String> {
        sand_commands::__private::try_damage_one(self.selector.clone(), amount, kind)
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::LivingEntity::give_effect",
        aliases = ["sand::prelude::LivingEntity::give_effect"],
        module = "sand::entity",
        kind = "method",
        summary = "Starts the canonical typed status-effect builder for this entity.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(effect = "The typed effect used by this operation."),
    )]
    pub fn give_effect(&self, effect: impl Into<EffectId>) -> EffectGive {
        crate::cmd::effect_give(self.selector.clone(), effect)
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::LivingEntity::clear_effect",
        aliases = ["sand::prelude::LivingEntity::clear_effect"],
        module = "sand::entity",
        kind = "method",
        summary = "Clears one typed status effect from this entity.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(effect = "The typed effect used by this operation."),
    )]
    pub fn clear_effect(&self, effect: impl Into<EffectId>) -> String {
        crate::cmd::effect_clear_effect(self.selector.clone(), effect)
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::LivingEntity::clear_effects",
        aliases = ["sand::prelude::LivingEntity::clear_effects"],
        module = "sand::entity",
        kind = "method",
        summary = "Clears every status effect from this entity.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn clear_effects(&self) -> String {
        crate::cmd::effect_clear(self.selector.clone())
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::LivingEntity::attribute",
        aliases = ["sand::prelude::LivingEntity::attribute"],
        module = "sand::entity",
        kind = "method",
        summary = "Reads a typed native attribute value.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(attribute = "The typed attribute used by this operation."),
    )]
    pub fn attribute(&self, attribute: AttributeType) -> String {
        sand_commands::builtins::attribute_get(self.selector.clone(), attribute.as_str())
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::LivingEntity::set_attribute_base",
        aliases = ["sand::prelude::LivingEntity::set_attribute_base"],
        module = "sand::entity",
        kind = "method",
        summary = "Sets a typed native attribute base value.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(attribute = "The typed attribute used by this operation.", value = "The typed value used by this operation."),
    )]
    pub fn set_attribute_base(
        &self,
        attribute: AttributeType,
        value: f64,
    ) -> CommandResult<String> {
        sand_commands::builtins::try_attribute_base_set(
            self.selector.clone(),
            attribute.as_str(),
            value,
        )
    }
}

/// Typed item locations for an equipment-capable entity.
#[derive(Debug, Clone)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityEquipmentHandle",
    aliases = ["sand::prelude::EntityEquipmentHandle"],
    module = "sand::entity",
    summary = "Typed item locations for an equipment-capable entity.",
    context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
    minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
    use_when = ["Discovering legal operations on the current or scoped entity"],
    avoid_when = ["Keeping an entity identity across ticks"],
    example = "use sand::prelude::*;",
)]
pub struct EntityEquipmentHandle<K> {
    selector: Selector,
    marker: PhantomData<fn() -> K>,
}

/// Equipment access is absent when the known entity kind cannot equip items.
///
/// ```compile_fail
/// use sand_core::entity::{EntityContext, MarkerKind};
/// let marker = EntityContext::<MarkerKind>::default();
/// let _ = marker.equipment();
/// ```
impl<K: EquipmentEntityKind> EntityEquipmentHandle<K> {
    pub(crate) fn new(selector: Selector) -> Self {
        Self {
            selector,
            marker: PhantomData,
        }
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityEquipmentHandle::slot",
        aliases = ["sand::prelude::EntityEquipmentHandle::slot"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns the canonical typed location for one equipment slot.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(slot = "The typed slot used by this operation."),
    )]
    pub fn slot(&self, slot: EquipmentSlot) -> Result<ItemLocation, ItemLocationError> {
        <K as super::kind::sealed::Equipment>::equipment_location(self.selector.clone(), slot)
    }
}

/// Ride and dismount mutations for one entity.
#[derive(Debug, Clone)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityMounts",
    aliases = ["sand::prelude::EntityMounts"],
    module = "sand::entity",
    summary = "Ride and dismount mutations for one entity.",
    context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
    minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
    use_when = ["Discovering legal operations on the current or scoped entity"],
    avoid_when = ["Keeping an entity identity across ticks"],
    example = "use sand::prelude::*;",
)]
pub struct EntityMounts<K> {
    selector: Selector,
    marker: PhantomData<fn() -> K>,
}

impl<K: EntityKind> EntityMounts<K> {
    pub(crate) fn new(selector: Selector) -> Self {
        Self {
            selector,
            marker: PhantomData,
        }
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityMounts::mount_on",
        aliases = ["sand::prelude::EntityMounts::mount_on"],
        module = "sand::entity",
        kind = "method",
        summary = "Mounts this entity on the selected vehicle.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*; let entity = EntityContext::<ZombieKind>::default(); let command = entity.mounts().mount_on(Target::entities().tag(\"vehicle\").nearest());",
        returns = "The requested capability value or canonical Minecraft command.",
        params(vehicle = "The typed vehicle used by this operation."),
    )]
    pub fn mount_on<TargetKind>(&self, vehicle: Target<TargetKind, One>) -> String {
        #[cfg(sand_placeholder_codegen)]
        {
            let _ = (&self.selector, vehicle);
            panic!("ride is unavailable in an explicit placeholder-codegen build");
        }
        #[cfg(not(sand_placeholder_codegen))]
        crate::cmd::ride_mount(self.selector.clone(), vehicle).to_string()
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityMounts::dismount",
        aliases = ["sand::prelude::EntityMounts::dismount"],
        module = "sand::entity",
        kind = "method",
        summary = "Dismounts this entity from its current vehicle.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn dismount(&self) -> String {
        #[cfg(sand_placeholder_codegen)]
        {
            let _ = &self.selector;
            panic!("ride is unavailable in an explicit placeholder-codegen build");
        }
        #[cfg(not(sand_placeholder_codegen))]
        crate::cmd::ride_dismount(self.selector.clone()).to_string()
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityMounts::mount_rider",
        aliases = ["sand::prelude::EntityMounts::mount_rider"],
        module = "sand::entity",
        kind = "method",
        summary = "Mounts the selected rider on this entity.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*; let entity = EntityContext::<ZombieKind>::default(); let command = entity.mounts().mount_rider(Target::nearest_player());",
        returns = "The requested capability value or canonical Minecraft command.",
        params(rider = "The typed rider used by this operation."),
    )]
    pub fn mount_rider<TargetKind>(&self, rider: Target<TargetKind, One>) -> String {
        #[cfg(sand_placeholder_codegen)]
        {
            let _ = (&self.selector, rider);
            panic!("ride is unavailable in an explicit placeholder-codegen build");
        }
        #[cfg(not(sand_placeholder_codegen))]
        crate::cmd::ride_mount(rider, self.selector.clone()).to_string()
    }
}

/// A typed NBT path on one execution-scoped entity.
///
/// Read methods are available for every entity kind. Mutation methods are
/// compiled only for [`SafeEntityDataWriteKind`], so `PlayerKind` cannot issue
/// entity-data writes through this façade.
#[derive(Debug, Clone)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityData",
    aliases = ["sand::prelude::EntityData"],
    module = "sand::entity",
    summary = "A capability-gated typed NBT path on one entity.",
    context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
    minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
    use_when = ["Discovering legal operations on the current or scoped entity"],
    avoid_when = ["Keeping an entity identity across ticks"],
    example = "use sand::prelude::*;",
)]
pub struct EntityData<K, T = UntypedNbt> {
    reference: NbtRef<T>,
    marker: PhantomData<fn() -> K>,
}

/// Player entity NBT is readable but not mutable through this façade.
///
/// ```compile_fail
/// use sand_core::entity::{EntityContext, PlayerKind};
/// let player = EntityContext::<PlayerKind>::default();
/// player.data().field::<i32>("Air").set(300);
/// ```
impl<K: EntityKind, T> EntityData<K, T> {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityData::field",
        aliases = ["sand::prelude::EntityData::field"],
        module = "sand::entity",
        kind = "method",
        summary = "Selects a typed child field without exposing a mutable NbtRef.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(key = "The typed key used by this operation."),
    )]
    pub fn field<U>(&self, key: impl AsRef<str>) -> EntityData<K, U> {
        EntityData {
            reference: self.reference.typed_field(key),
            marker: PhantomData,
        }
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityData::index",
        aliases = ["sand::prelude::EntityData::index"],
        module = "sand::entity",
        kind = "method",
        summary = "Selects a list index while retaining the field marker type.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(index = "The typed index used by this operation."),
    )]
    pub fn index(&self, index: i32) -> EntityData<K, T> {
        EntityData {
            reference: self.reference.index(index),
            marker: PhantomData,
        }
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityData::get",
        aliases = ["sand::prelude::EntityData::get"],
        module = "sand::entity",
        kind = "method",
        summary = "Builds the canonical typed data-get command for this path.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn get(&self) -> DataCommand {
        self.reference.get()
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityData::get_scaled",
        aliases = ["sand::prelude::EntityData::get_scaled"],
        module = "sand::entity",
        kind = "method",
        summary = "Builds the canonical scaled data-get command for this path.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(scale = "The typed scale used by this operation."),
    )]
    pub fn get_scaled(&self, scale: f64) -> DataCommand {
        self.reference.get_scaled(scale)
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityData::copy_to",
        aliases = ["sand::prelude::EntityData::copy_to"],
        module = "sand::entity",
        kind = "method",
        summary = "Copies this readable entity field into a typed NBT destination.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(destination = "The typed destination used by this operation."),
    )]
    pub fn copy_to<U>(&self, destination: &NbtRef<U>) -> DataCommand {
        destination.copy_from(&self.reference)
    }
}

impl<K: SafeEntityDataWriteKind, T> EntityData<K, T> {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityData::set",
        aliases = ["sand::prelude::EntityData::set"],
        module = "sand::entity",
        kind = "method",
        summary = "Sets this entity field from a typed NBT value; unavailable for players.",
        context = "Typed entity NBT writes are available only for SafeEntityDataWriteKind, which excludes PlayerKind.",
        minecraft = "Delegates to NbtRef::set against the current or scoped non-player entity.",
        use_when = ["Mutating a supported stable NBT field on a known non-player entity"],
        avoid_when = ["Mutating player NBT or bypassing a native-property binding"],
        params(value = "The typed NBT value written to the selected path."),
        returns = "The canonical typed data-modify command.",
        example = "use sand::prelude::*;",
    )]
    pub fn set(&self, value: impl Into<NbtValue>) -> DataCommand {
        self.reference.set(value)
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityData::copy_from",
        aliases = ["sand::prelude::EntityData::copy_from"],
        module = "sand::entity",
        kind = "method",
        summary = "Copies a typed NBT source into this entity field; unavailable for players.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(source = "The typed source used by this operation."),
    )]
    pub fn copy_from<U>(&self, source: &NbtRef<U>) -> DataCommand {
        self.reference.copy_from(source)
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityData::remove",
        aliases = ["sand::prelude::EntityData::remove"],
        module = "sand::entity",
        kind = "method",
        summary = "Removes this entity field; unavailable for players.",
        context = "Typed entity NBT writes are available only for SafeEntityDataWriteKind, which excludes PlayerKind.",
        minecraft = "Delegates to NbtRef::remove against the current or scoped non-player entity.",
        use_when = ["Removing a supported stable NBT field on a known non-player entity"],
        avoid_when = ["Mutating player NBT or bypassing a native-property binding"],
        returns = "The canonical typed data-remove command.",
        example = "use sand::prelude::*;",
    )]
    pub fn remove(&self) -> DataCommand {
        self.reference.remove()
    }
}

/// Navigation root for typed data on one execution-scoped entity.
///
/// The root intentionally has no data command methods because vanilla data
/// commands require a non-empty NBT path. Select a typed [`field`](Self::field)
/// or an explicitly unchecked [`raw_path`](Self::raw_path) first.
#[derive(Debug, Clone)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityDataRoot",
    aliases = ["sand::prelude::EntityDataRoot"],
    module = "sand::entity",
    summary = "Navigation root for typed data on one execution-scoped entity.",
    context = "The root carries the current or scoped entity selector but exposes commands only after a non-empty typed or raw path is selected.",
    minecraft = "Vanilla data commands require a non-empty NBT path, so the root itself cannot be read or mutated.",
    use_when = ["Selecting an entity NBT field from EntityContext::data"],
    avoid_when = ["Constructing a root-level data command"],
    example = "use sand::prelude::*;",
)]
pub struct EntityDataRoot<K> {
    selector: Selector,
    marker: PhantomData<fn() -> K>,
}

impl<K: EntityKind> EntityDataRoot<K> {
    pub(crate) fn new(selector: Selector) -> Self {
        Self {
            selector,
            marker: PhantomData,
        }
    }

    /// Selects a typed child field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityDataRoot::field",
        aliases = ["sand::prelude::EntityDataRoot::field"],
        module = "sand::entity",
        kind = "method",
        summary = "Selects a typed entity NBT field.",
        context = "This creates the non-empty typed path required before an entity data command can be built.",
        minecraft = "Delegates path construction to Sand's canonical NbtPath and NbtRef model.",
        use_when = ["Addressing a known vanilla or custom entity field"],
        avoid_when = ["The field is owned by a State/native-property binding"],
        params(key = "The field name selected below the entity NBT root."),
        returns = "A typed non-empty entity-data path.",
        example = "use sand::prelude::*;",
    )]
    pub fn field<T>(&self, key: impl AsRef<str>) -> EntityData<K, T> {
        EntityData {
            reference: DataTarget::entity(self.selector.clone())
                .typed_path(NbtPath::new("").field(key)),
            marker: PhantomData,
        }
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityDataRoot::raw_path",
        aliases = ["sand::prelude::EntityDataRoot::raw_path"],
        module = "sand::entity",
        kind = "method",
        summary = "Selects an explicitly unchecked entity NBT path.",
        context = "Focused entity capability reached from an execution-scoped EntityContext or ScopedEntityRef; it stores no persistent entity identity and creates no state or lifecycle resources.",
        minecraft = "Delegates to Sand's canonical typed command, item, or NBT lowering for the entity selector carried by the handle.",
        use_when = ["Discovering legal operations on the current or scoped entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
        params(path = "The typed path used by this operation."),
    )]
    pub fn raw_path(self, path: impl Into<String>) -> EntityData<K, UntypedNbt> {
        EntityData {
            reference: DataTarget::entity(self.selector).typed_path(NbtPath::raw(path)),
            marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{EntityContext, PlayerKind, ZombieKind};

    #[test]
    fn identity_and_transform_match_canonical_commands() {
        let zombie = EntityContext::<ZombieKind>::default();
        let tag = EntityTag::new("tracked").unwrap();
        let position = Vec3::absolute(1.0, 64.0, -2.0);

        assert_eq!(
            zombie.identity().add_tag(&tag),
            sand_commands::builtins::tag_add(Selector::self_(), tag.as_str())
        );
        assert_eq!(
            zombie.transform().teleport(position.clone()),
            crate::cmd::teleport_4(Selector::self_(), position).to_string()
        );
        assert_eq!(
            zombie.transform().position().get().to_string(),
            DataTarget::entity(Selector::self_())
                .typed_path::<Vec<f64>>(NbtPath::new("Pos"))
                .get()
                .to_string()
        );
    }

    #[test]
    fn living_mount_and_data_operations_match_canonical_commands() {
        let zombie = EntityContext::<ZombieKind>::default();
        let vehicle = Target::entities().tag("vehicle").nearest();

        assert_eq!(
            zombie.living().damage(3.5, DamageKind::Magic).unwrap(),
            sand_commands::builtins::try_damage(
                sand_commands::Target::self_(),
                3.5,
                DamageKind::Magic,
            )
            .unwrap()
        );
        assert_eq!(
            zombie.mounts().mount_on(vehicle.clone()),
            crate::cmd::ride_mount(Selector::self_(), vehicle).to_string()
        );
        assert_eq!(
            zombie.data().field::<i32>("Air").set(42).to_string(),
            DataTarget::entity(Selector::self_())
                .typed_path::<i32>(NbtPath::new("Air"))
                .set(42)
                .to_string()
        );
        assert_eq!(
            zombie.data().field::<i32>("mod.key").get().to_string(),
            "data get entity @s \"mod.key\""
        );
    }

    #[test]
    fn equipment_reuses_item_locations_and_retains_explicit_player_target() {
        let zombie = EntityContext::<ZombieKind>::default();
        let player = EntityContext::<PlayerKind>::default();

        let zombie_hand = zombie.equipment().slot(EquipmentSlot::Mainhand).unwrap();
        let canonical_zombie =
            ItemLocation::entity_equipment(Selector::self_(), EquipmentSlot::Mainhand).unwrap();
        assert_eq!(
            zombie_hand.nbt().get().to_string(),
            canonical_zombie.nbt().get().to_string()
        );

        let player_head = player.equipment().slot(EquipmentSlot::Head).unwrap();
        let canonical_player = ItemLocation::entity(Selector::self_()).helmet();
        assert_eq!(
            player_head.nbt().get().to_string(),
            canonical_player.nbt().get().to_string()
        );
        assert!(player.equipment().slot(EquipmentSlot::Body).is_err());
    }
}
