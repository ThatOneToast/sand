//! Shared capability for typed Minecraft registry references.
//!
//! Command builders use one capability, parameterized by the registry kind,
//! instead of defining a conversion trait per command family. Implementations
//! are provided by Sand's generated vanilla IDs and validated custom ID types.

/// Marker for the entity-type registry.
#[sand_macros::api(registry = sand_api_contract, path = "sand::resource_ref::EntityTypeRegistry", module = "sand::resource_ref", summary = "Registry marker for typed entity-type references.", context = "Used with RegistryReference to keep entity IDs distinct from other resources.", minecraft = "Represents the entity_type registry consumed by selectors and summon commands.", use_when = ["Constraining a generic registry reference to entity types"], avoid_when = ["Naming an entity type value; use EntityType or EntityTypeId"], example = "fn accepts_entity(id: impl RegistryReference<EntityTypeRegistry>) {}")]
pub enum EntityType {}

/// Marker for the particle-type registry.
#[sand_macros::api(registry = sand_api_contract, path = "sand::resource_ref::ParticleRegistry", module = "sand::resource_ref", summary = "Registry marker for typed particle references.", context = "Used with RegistryReference to keep particle IDs distinct from other resources.", minecraft = "Represents the particle_type registry consumed by particle commands.", use_when = ["Constraining a generic registry reference to particles"], avoid_when = ["Naming a particle value; use a generated particle or ParticleId"], example = "fn accepts_particle(id: impl RegistryReference<ParticleRegistry>) {}")]
pub enum ParticleType {}

/// Marker for the sound-event registry.
#[sand_macros::api(registry = sand_api_contract, path = "sand::resource_ref::SoundEventRegistry", module = "sand::resource_ref", summary = "Registry marker for typed sound-event references.", context = "Used with RegistryReference to keep sound IDs distinct from other resources.", minecraft = "Represents the sound_event registry consumed by sound commands.", use_when = ["Constraining a generic registry reference to sound events"], avoid_when = ["Naming a sound value; use SoundEvent or SoundEventId"], example = "fn accepts_sound(id: impl RegistryReference<SoundEventRegistry>) {}")]
pub enum SoundEvent {}

/// Marker for standalone predicate resources.
#[sand_macros::api(registry = sand_api_contract, path = "sand::resource_ref::PredicateRegistry", module = "sand::resource_ref", summary = "Resource marker for typed predicate references.", context = "Used with RegistryReference to keep predicate IDs distinct from registry entries and other datapack resources.", minecraft = "Represents named predicate resources consumed by target filters.", use_when = ["Constraining a generic registry reference to predicates"], avoid_when = ["Naming a predicate value; use PredicateId"], example = "fn accepts_predicate(id: impl RegistryReference<PredicateRegistry>) {}")]
pub enum Predicate {}

/// Marker for the status-effect registry.
#[sand_macros::api(registry = sand_api_contract, path = "sand::resource_ref::StatusEffectRegistry", module = "sand::resource_ref", summary = "Registry marker for typed status-effect references.", context = "Used with RegistryReference so effect commands accept generated and validated status-effect IDs without accepting unrelated resources.", minecraft = "Represents the mob_effect registry consumed by effect commands.", use_when = ["Constraining a generic registry reference to status effects"], avoid_when = ["Naming an effect value; use EffectId or StatusEffectId"], example = "fn accepts_effect(id: impl RegistryReference<StatusEffectRegistry>) {}")]
pub enum StatusEffect {}

/// Marker for namespaced command-storage resources.
#[sand_macros::api(registry = sand_api_contract, path = "sand::resource_ref::CommandStorageRegistry", module = "sand::resource_ref", summary = "Resource marker for typed command-storage references.", context = "Used with RegistryReference so Nbt::storage accepts validated ResourceLocation values without accepting plain strings.", minecraft = "Represents the namespaced storage identifier consumed by data commands.", use_when = ["Constraining a generic reference to command storage"], avoid_when = ["Naming a storage value; use ResourceLocation"], example = "fn accepts_storage(id: impl RegistryReference<CommandStorageRegistry>) {}")]
pub enum CommandStorage {}

#[doc(hidden)]
pub mod sealed {
    pub trait Sealed<K> {}

    impl<K, T: Sealed<K> + ?Sized> Sealed<K> for &T {}
}

/// A typed reference to one Minecraft registry or datapack-resource kind.
///
/// The kind parameter prevents identifiers from different registries from
/// being mixed. Plain strings intentionally do not implement this trait;
/// command APIs expose separately named `_raw` entry points for unsupported
/// future or modded syntax.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::resource_ref::RegistryReference",
    aliases = ["sand::command::RegistryReference", "sand::cmd::RegistryReference", "sand::prelude::cmd::RegistryReference"],
    module = "sand::resource_ref",
    summary = "Capability implemented by canonical typed IDs for one Minecraft registry kind.",
    context = "The marker type preserves registry identity while allowing generated vanilla IDs and validated custom IDs to share command consumers.",
    minecraft = "Lowers a typed registry ID to its validated namespace:path token.",
    use_when = ["Writing a generic helper over one registry kind"],
    avoid_when = ["Passing arbitrary text; use the consumer's explicit `_raw` API"],
    example = "fn entity_id(id: impl sand::resource_ref::RegistryReference<sand::resource_ref::EntityTypeRegistry>) {}",
)]
pub trait RegistryReference<K>: sealed::Sealed<K> {
    /// Returns the validated `namespace:path` identifier used for lowering.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::resource_ref::RegistryReference::registry_id",
        aliases = ["sand::command::RegistryReference::registry_id", "sand::cmd::RegistryReference::registry_id", "sand::prelude::cmd::RegistryReference::registry_id"],
        module = "sand::resource_ref",
        summary = "Returns the validated namespace:path token for this typed registry ID.",
        context = "Command lowering uses this method after the registry marker has enforced the resource kind.",
        minecraft = "Produces the registry token consumed by Minecraft commands and data resources.",
        use_when = ["Lowering a typed registry reference"],
        avoid_when = ["Constructing an unchecked raw resource token"],
        returns = "The validated namespaced registry identifier.",
        example = "let token = id.registry_id();",
    )]
    fn registry_id(&self) -> String;
}

impl<K, T: RegistryReference<K> + ?Sized> RegistryReference<K> for &T {
    fn registry_id(&self) -> String {
        (*self).registry_id()
    }
}
