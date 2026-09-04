//! Typed registry identifier wrappers.
//!
//! Every entry here wraps a validated [`ResourceLocation`] and provides:
//!
//! - `::minecraft(path)` — construct a `minecraft:` prefixed ID without unwrapping
//! - `::custom(rl)` — wrap any `ResourceLocation` (modded / pack-specific IDs)
//! - `From<ResourceLocation>` — convert a parsed ID directly
//! - `Display` / `Serialize` — emit `namespace:path` strings
//!
//! # Relation to generated enums
//!
//! `sand-core::generated::{Item, Block, EntityType, …}` already provide
//! strongly-typed vanilla constants.  These wrapper types complement them:
//! use the generated enum for vanilla values, and the `*Id` wrappers for
//! modded or pack-specific IDs that are not in the generated list.
//!
//! ```rust
//! use sand_components::registry::ItemId;
//!
//! // Vanilla item (plain resource location)
//! let diamond = ItemId::minecraft("diamond").unwrap();
//! assert_eq!(diamond.to_string(), "minecraft:diamond");
//!
//! // Modded item (custom resource location)
//! use sand_components::ResourceLocation;
//! let custom: ItemId = ResourceLocation::new("mymod", "arcane_sword").unwrap().into();
//! assert_eq!(custom.to_string(), "mymod:arcane_sword");
//! ```

use std::fmt;
use std::marker::PhantomData;

use sand_macros::registry_id;
use serde::{Serialize, Serializer};

use crate::error::Result;
use crate::resource_location::ResourceLocation;

registry_id! {
    /// Typed Minecraft item identifier (e.g. `minecraft:diamond_sword` or `mymod:arcane_blade`).
    ///
    /// For vanilla items, prefer the generated `sand_core::generated::Item` enum.
    /// Use `ItemId` for modded items or when you have a `ResourceLocation` at hand.
    ItemId
}

registry_id! {
    /// Typed Minecraft block identifier (e.g. `minecraft:stone` or `mymod:custom_ore`).
    ///
    /// For vanilla blocks, prefer the generated `sand_core::generated::Block` enum.
    BlockId
}

registry_id! {
    /// Typed Minecraft entity type identifier (e.g. `minecraft:zombie` or `mymod:boss`).
    ///
    /// For vanilla entity types, prefer the generated `sand_core::generated::EntityType` enum.
    EntityTypeId
}

impl sand_commands::IntoTextEntityType for EntityTypeId {
    fn into_text_entity_type(self) -> String {
        self.to_string()
    }
}

impl sand_commands::IntoTextEntityType for &EntityTypeId {
    fn into_text_entity_type(self) -> String {
        self.to_string()
    }
}

impl sand_commands::selector::IntoEntityType for EntityTypeId {
    fn into_entity_type(self) -> String {
        self.to_string()
    }
}

impl sand_commands::selector::IntoEntityType for &EntityTypeId {
    fn into_entity_type(self) -> String {
        self.to_string()
    }
}

registry_id! {
    @contract(
        path = "sand::resource_ref::FunctionId",
        aliases = ["sand::prelude::FunctionId"],
        subject = "Minecraft function resource",
        minecraft = "Serializes as the namespaced identifier of a data/<namespace>/function/<path>.mcfunction resource.",
        use_when = ["Calling or referring to a function by resource identity", "Representing a function supplied by another datapack"],
        avoid_when = ["A registered Rust function pointer is available", "Passing an unvalidated namespace:path string"],
        example_namespace = "demo",
        example_path = "combat/on_hit"
    );
    /// Typed Minecraft function identifier (e.g. `minecraft:load` or `mypack:tick`).
    FunctionId
}

registry_id! {
    @contract(
        path = "sand::resource_ref::AdvancementId",
        aliases = ["sand::prelude::AdvancementId"],
        subject = "Minecraft advancement resource",
        minecraft = "Serializes as the namespaced identifier of a data/<namespace>/advancement/<path>.json resource.",
        use_when = ["Linking an advancement to its parent", "Referring to an advancement from another generated resource"],
        avoid_when = ["Building the advancement payload itself", "Passing an unvalidated namespace:path string"],
        example_namespace = "demo",
        example_path = "story/first_steps"
    );
    /// Typed Minecraft advancement identifier (e.g. `minecraft:story/root` or `mypack:chapter/start`).
    AdvancementId
}

registry_id! {
    @contract(
        path = "sand::resource_ref::RecipeId",
        aliases = ["sand::prelude::RecipeId"],
        subject = "Minecraft recipe resource",
        minecraft = "Serializes as the namespaced identifier of a data/<namespace>/recipe/<path>.json resource.",
        use_when = ["Granting a recipe from an advancement reward", "Referring to a custom or vanilla recipe resource"],
        avoid_when = ["Building a recipe payload", "Passing an unvalidated namespace:path string"],
        example_namespace = "demo",
        example_path = "crafting/arcane_blade"
    );
    /// Typed Minecraft recipe identifier (e.g. `minecraft:diamond_pickaxe` or `mypack:arcane_blade`).
    RecipeId
}

registry_id! {
    @contract(
        path = "sand::resource_ref::LootTableId",
        aliases = ["sand::prelude::LootTableId"],
        subject = "Minecraft loot table resource",
        minecraft = "Serializes as the namespaced identifier of a data/<namespace>/loot_table/<path>.json resource.",
        use_when = ["Nesting or selecting a loot table by identity", "Referring to a custom or vanilla loot table resource"],
        avoid_when = ["Building a loot table payload", "Passing an unvalidated namespace:path string"],
        example_namespace = "demo",
        example_path = "rewards/boss"
    );
    /// Typed Minecraft loot-table identifier (e.g. `minecraft:chests/simple_dungeon` or `mypack:rewards/boss`).
    LootTableId
}

registry_id! {
    @contract(
        path = "sand::predicate::PredicateId",
        aliases = ["sand::component::PredicateId", "sand::prelude::PredicateId", "sand::resource_ref::PredicateId"],
        subject = "standalone predicate resource",
        minecraft = "Serializes as a namespaced reference to data/<namespace>/predicate/<path>.json wherever Minecraft accepts a predicate resource identifier.",
        use_when = ["Referring to a Predicate from another resource or command", "Representing a custom or non-vanilla predicate identifier"],
        avoid_when = ["Building the predicate condition tree itself", "Passing an unvalidated namespace:path string"],
        example_namespace = "demo",
        example_path = "conditions/is_raining"
    );
    /// Typed Minecraft predicate identifier (e.g. `mypack:conditions/is_raining`).
    ///
    /// Used by [`crate::predicate::PredicateRoot::reference`] and by
    /// `minecraft:reference` loot conditions to point at a standalone
    /// `data/<namespace>/predicate/<id>.json` file.
    PredicateId
}

impl sand_commands::IntoPredicateId for PredicateId {
    fn into_predicate_id(self) -> String {
        self.to_string()
    }
}

impl sand_commands::IntoPredicateId for &PredicateId {
    fn into_predicate_id(self) -> String {
        self.to_string()
    }
}

registry_id! {
    @contract(
        path = "sand::resource_ref::DialogId",
        aliases = ["sand::prelude::DialogId"],
        subject = "Minecraft dialog resource",
        minecraft = "Serializes as the namespaced identifier of a data/<namespace>/dialog/<path>.json resource.",
        use_when = ["Showing or opening a dialog by resource identity", "Referring to a dialog supplied by another datapack"],
        avoid_when = ["Building the dialog payload itself", "Targeting Minecraft versions before dialog resources were introduced"],
        example_namespace = "demo",
        example_path = "menu/welcome",
        availability = ["Minecraft Java 1.21.6+", "Minecraft Java 26.x"],
        local(
            minecraft = "Uses Sand's export-time namespace sentinel so the final datapack namespace is substituted when the dialog is emitted.",
            use_when = ["Referring to a dialog generated by the current Sand project"],
            avoid_when = ["Referring to a dialog owned by Minecraft or another datapack"],
            example_path = "welcome",
            availability = ["Minecraft Java 1.21.6+", "Minecraft Java 26.x"]
        )
    );
    /// Typed Minecraft dialog identifier (e.g. `minecraft:custom_options` or `mypack:menu/welcome`).
    DialogId
}

registry_id! {
    /// Typed Minecraft enchantment identifier (e.g. `minecraft:sharpness` or `mymod:arcane`).
    EnchantmentId
}

registry_id! {
    /// Typed Minecraft biome identifier (e.g. `minecraft:plains` or `mymod:mystic_forest`).
    BiomeId
}

registry_id! {
    /// Typed Minecraft dimension identifier (e.g. `minecraft:overworld` or `mymod:pocket`).
    DimensionId
}

registry_id! {
    /// Typed Minecraft configured-feature identifier (e.g. `minecraft:oak` or
    /// `mymod:ashen_ore`).
    ///
    /// Referenced by [`crate::worldgen::PlacedFeature`]; obtain one for a
    /// component you authored with
    /// [`crate::worldgen::ConfiguredFeature::id`].
    ConfiguredFeatureId
}

registry_id! {
    /// Typed Minecraft configured-carver identifier (e.g. `minecraft:cave` or
    /// `mymod:arcane_cave`).
    ///
    /// Referenced by [`crate::worldgen::Biome::carver_step`]; obtain one for
    /// a component you authored with
    /// [`crate::worldgen::ConfiguredCarver::id`].
    ConfiguredCarverId
}

registry_id! {
    /// Typed Minecraft dimension-type identifier (e.g. `minecraft:overworld` or `mymod:skylands`).
    DimensionTypeId
}

registry_id! {
    /// Typed Minecraft noise-parameter identifier (e.g. `minecraft:temperature`
    /// or `mymod:ridges`), referencing `worldgen/noise/<id>.json`.
    NoiseId
}

registry_id! {
    /// Typed Minecraft density-function identifier (e.g.
    /// `minecraft:overworld/base_3d_noise` or `mymod:ridge_density`),
    /// referencing `worldgen/density_function/<id>.json`.
    DensityFunctionId
}

registry_id! {
    /// Typed Minecraft damage type identifier (e.g. `minecraft:generic` or `mymod:arcane`).
    DamageTypeId
}

registry_id! {
    /// Typed Minecraft structure identifier (e.g. `minecraft:village` or `mymod:dungeon`).
    StructureId
}

registry_id! {
    /// Typed `minecraft:worldgen/structure_set` identifier (e.g. `minecraft:villages`).
    StructureSetId
}

registry_id! {
    /// Typed `minecraft:worldgen/template_pool` identifier
    /// (e.g. `minecraft:village/plains/town_centers`).
    TemplatePoolId
}

impl TemplatePoolId {
    /// `minecraft:empty` — the vanilla terminal fallback pool.
    pub fn empty() -> Self {
        Self::minecraft("empty").expect("\"empty\" is a valid resource path")
    }
}

registry_id! {
    /// Typed `minecraft:worldgen/processor_list` identifier
    /// (e.g. `minecraft:empty` or `mypack:mossify`).
    ProcessorListId
}

impl ProcessorListId {
    /// `minecraft:empty` — the vanilla no-op processor list.
    pub fn empty() -> Self {
        Self::minecraft("empty").expect("\"empty\" is a valid resource path")
    }
}

registry_id! {
    /// Typed identifier for a `.nbt` structure template asset stored under
    /// `data/<namespace>/structure/` (e.g. `minecraft:village/plains/houses/plains_small_house_1`).
    ///
    /// This references the template *file*, not a `worldgen/structure`
    /// registry entry — see [`StructureId`] for the latter.
    StructureTemplateId
}

registry_id! {
    /// Typed structure *type* identifier — the `type` field of a
    /// `worldgen/structure` entry (e.g. `minecraft:jigsaw`).
    StructureTypeId
}

impl StructureTypeId {
    /// `minecraft:jigsaw` — the type used by villages, pillager outposts, and
    /// most custom template-pool driven structures.
    pub fn jigsaw() -> Self {
        Self::minecraft("jigsaw").expect("\"jigsaw\" is a valid resource path")
    }
}

registry_id! {
    /// Typed sound-event identifier (e.g. `minecraft:entity.player.burp` or `mymod:arcane_chime`).
    SoundEventId
}

registry_id! {
    /// Typed equipment-model identifier used by the `equippable` item component.
    EquipmentModelId
}

registry_id! {
    /// Resource-location-backed Minecraft status-effect identifier.
    ///
    /// This is the shared registry form used for dynamic, generated, and modded
    /// IDs. [`crate::EffectId`] remains available as the enum-style vanilla
    /// convenience and converts to and from this type.
    StatusEffectId
}

registry_id! {
    /// Resource-location-backed Minecraft potion identifier.
    ///
    /// The `PotionRegistryId` name deliberately avoids colliding with the
    /// existing enum-style [`crate::PotionId`] compatibility API.
    PotionRegistryId
}

registry_id! {
    /// Typed identifier for an enchantment effect component key (e.g.
    /// `minecraft:damage`, `minecraft:knockback`).
    ///
    /// Custom/modded namespaced components are supported via
    /// [`EnchantmentEffectComponentId::custom`]. See
    /// [`crate::enchantment`] for the small typed slice of well-known
    /// vanilla value-effect components ([`EnchantmentEffectComponentId::damage`],
    /// [`EnchantmentEffectComponentId::knockback`],
    /// [`EnchantmentEffectComponentId::armor_effectiveness`]).
    EnchantmentEffectComponentId
}

impl EnchantmentEffectComponentId {
    /// `minecraft:damage` — used by Sharpness, Smite, Bane of Arthropods,
    /// Impaling, and Power.
    pub fn damage() -> Self {
        Self::minecraft("damage").expect("\"damage\" is a valid resource path")
    }

    /// `minecraft:knockback` — used by Knockback and Punch.
    pub fn knockback() -> Self {
        Self::minecraft("knockback").expect("\"knockback\" is a valid resource path")
    }

    /// `minecraft:armor_effectiveness` — used by Breach.
    pub fn armor_effectiveness() -> Self {
        Self::minecraft("armor_effectiveness")
            .expect("\"armor_effectiveness\" is a valid resource path")
    }
}

registry_id! {
    /// Typed identifier for the `chicken_variant` registry (e.g.
    /// `minecraft:cold` or `mymod:arcane_chicken`). Introduced in 1.21.5.
    ChickenVariantId
}

registry_id! {
    /// Typed identifier for the `cow_variant` registry (e.g.
    /// `minecraft:warm` or `mymod:arcane_cow`). Introduced in 1.21.5.
    CowVariantId
}

registry_id! {
    /// Typed identifier for the `pig_variant` registry (e.g.
    /// `minecraft:cold` or `mymod:arcane_pig`). Introduced in 1.21.5.
    PigVariantId
}

registry_id! {
    /// Typed identifier for the `villager_trade` registry (e.g.
    /// `minecraft:trades/enchanted_pickaxe` or `mypack:blacksmith/novice/coal_purchase`).
    /// Introduced in Minecraft 26.1.
    VillagerTradeId
}

registry_id! {
    /// Typed identifier for the `trade_set` registry (e.g.
    /// `minecraft:armorer/level_1` or `mypack:blacksmith/novice`).
    /// Introduced in Minecraft 26.1.
    TradeSetId
}

registry_id! {
    /// Typed identifier for a named random sequence (`minecraft:random_sequence`
    /// scoping), e.g. `minecraft:trade_set/armorer/level_1` or
    /// `mypack:trade_set/blacksmith/novice`.
    RandomSequenceId
}

// ── TagId<T> ─────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::TagId",
    aliases = ["sand::prelude::TagId"],
    module = "sand::component",
    summary = "A typed tag identifier scoped to a specific registry kind `T`.",
    context = "A typed tag identifier scoped to a specific registry kind `T`. The phantom `T` marker allows you to distinguish `TagId<ItemId>` from `TagId<BlockId>` in API signatures, preventing accidental cross-registry mixing. Minecraft serializes tags as `#namespace:path` in some contexts (item predicates) and `namespace:path` in others (data files).  Use [`TagId::to_tag_string`] for the `#`-prefixed form and [`fmt::Display`] for the plain form.",
    minecraft = "Minecraft serializes tags as `#namespace:path` in some contexts (item predicates) and `namespace:path` in others (data files).  Use [`TagId::to_tag_string`] for the `#`-prefixed form and [`fmt::Display`] for the plain form.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::TagId;",
)]
/// A typed tag identifier scoped to a specific registry kind `T`.
///
/// The phantom `T` marker allows you to distinguish `TagId<ItemId>` from
/// `TagId<BlockId>` in API signatures, preventing accidental cross-registry
/// mixing.
///
/// Minecraft serializes tags as `#namespace:path` in some contexts (item
/// predicates) and `namespace:path` in others (data files).  Use
/// [`TagId::to_tag_string`] for the `#`-prefixed form and [`fmt::Display`] for
/// the plain form.
///
/// # Example
/// ```rust
/// use sand_components::registry::{TagId, ItemId};
///
/// let tag: TagId<ItemId> = TagId::minecraft("logs").unwrap();
/// assert_eq!(tag.to_string(), "minecraft:logs");
/// assert_eq!(tag.to_tag_string(), "#minecraft:logs");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagId<T> {
    rl: ResourceLocation,
    _marker: PhantomData<T>,
}

impl<T> TagId<T> {
    /// Construct a `minecraft:<path>` tag.  Returns an error if `path` is invalid.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TagId::minecraft",
        aliases = ["sand::prelude::TagId::minecraft"],
        module = "sand::component",
        kind = "method",
        summary = "Construct a `minecraft:<path>` tag.  Returns an error if `path` is invalid.",
        context = "Construct a `minecraft:<path>` tag.  Returns an error if `path` is invalid. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(path = "Construct a `minecraft:<path>` tag.  Returns an error if `path` is invalid."),
        returns = "Construct a `minecraft:<path>` tag.  Returns an error if `path` is invalid.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(path: impl AsRef < str >)  {\n    let minecraft = sand::component::TagId ::< T >::minecraft(path);\n}",
    )]
    pub fn minecraft(path: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            rl: ResourceLocation::minecraft(path)?,
            _marker: PhantomData,
        })
    }

    /// Wrap any [`ResourceLocation`] as a tag ID.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TagId::custom",
        aliases = ["sand::prelude::TagId::custom"],
        module = "sand::component",
        kind = "method",
        summary = "Wrap any [`ResourceLocation`] as a tag ID.",
        context = "Wrap any [`ResourceLocation`] as a tag ID. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(rl = "`rl` provides the typed Minecraft resource identifier used to wrap any [`ResourceLocation`] as a tag ID."),
        returns = "A `TagId` wrapping any [`ResourceLocation`] as a tag ID.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(rl: sand::ResourceLocation)  {\n    let tag_id = sand::component::TagId ::< T >::custom(rl);\n}",
    )]
    pub fn custom(rl: ResourceLocation) -> Self {
        Self {
            rl,
            _marker: PhantomData,
        }
    }

    /// Returns the `#namespace:path` form used in item predicates and ingredients.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TagId::to_tag_string",
        aliases = ["sand::prelude::TagId::to_tag_string"],
        module = "sand::component",
        kind = "method",
        summary = "Returns the `#namespace:path` form used in item predicates and ingredients.",
        context = "Returns the `#namespace:path` form used in item predicates and ingredients. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "Returns the `#namespace:path` form used in item predicates and ingredients.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(tag_id_value: &sand::component::TagId < T >)  {\n    let to_tag_string = tag_id_value.to_tag_string();\n}",
    )]
    pub fn to_tag_string(&self) -> String {
        format!("#{}", self.rl)
    }

    /// Access the inner [`ResourceLocation`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TagId::as_resource_location",
        aliases = ["sand::prelude::TagId::as_resource_location"],
        module = "sand::component",
        kind = "method",
        summary = "Access the inner [`ResourceLocation`].",
        context = "Access the inner [`ResourceLocation`]. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The `& ResourceLocation` value produced to acces the inner [`ResourceLocation`].",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(tag_id_value: &sand::component::TagId < T >)  {\n    let as_resource_location = tag_id_value.as_resource_location();\n}",
    )]
    pub fn as_resource_location(&self) -> &ResourceLocation {
        &self.rl
    }
}

impl<T> fmt::Display for TagId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.rl.fmt(f)
    }
}

impl<T> Serialize for TagId<T> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        self.rl.serialize(s)
    }
}

impl<T> std::str::FromStr for TagId<T> {
    type Err = crate::error::SandError;
    fn from_str(s: &str) -> Result<Self> {
        let stripped = s.strip_prefix('#').unwrap_or(s);
        Ok(Self {
            rl: stripped.parse()?,
            _marker: PhantomData,
        })
    }
}

impl<T> From<ResourceLocation> for TagId<T> {
    fn from(rl: ResourceLocation) -> Self {
        Self {
            rl,
            _marker: PhantomData,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_id_minecraft() {
        let id = ItemId::minecraft("diamond_sword").unwrap();
        assert_eq!(id.to_string(), "minecraft:diamond_sword");
    }

    #[test]
    fn item_id_custom() {
        let rl = ResourceLocation::new("mymod", "arcane_blade").unwrap();
        let id = ItemId::custom(rl);
        assert_eq!(id.to_string(), "mymod:arcane_blade");
    }

    #[test]
    fn item_id_from_resource_location() {
        let rl = ResourceLocation::new("mymod", "sword").unwrap();
        let id: ItemId = rl.into();
        assert_eq!(id.to_string(), "mymod:sword");
    }

    #[test]
    fn status_effect_and_potion_registry_ids_validate_and_serialize() {
        let effect = StatusEffectId::minecraft("speed").unwrap();
        let potion: PotionRegistryId = "mymod:arcane_brew".parse().unwrap();
        assert_eq!(effect.to_string(), "minecraft:speed");
        assert_eq!(potion.to_string(), "mymod:arcane_brew");
        assert_eq!(
            serde_json::to_value(effect).unwrap(),
            serde_json::json!("minecraft:speed")
        );
        assert!("not namespaced".parse::<StatusEffectId>().is_err());
        assert!("minecraft:bad path".parse::<PotionRegistryId>().is_err());
    }

    #[test]
    fn sound_event_and_equipment_model_ids_validate_and_serialize() {
        let sound = SoundEventId::minecraft("entity.player.burp").unwrap();
        let model: EquipmentModelId = "mymod:royal_armor".parse().unwrap();
        assert_eq!(sound.to_string(), "minecraft:entity.player.burp");
        assert_eq!(model.to_string(), "mymod:royal_armor");
        assert_eq!(
            serde_json::to_value(sound).unwrap(),
            serde_json::json!("minecraft:entity.player.burp")
        );
        assert!("not namespaced".parse::<SoundEventId>().is_err());
        assert!("minecraft:bad path".parse::<EquipmentModelId>().is_err());
    }

    #[test]
    fn item_id_parse() {
        let id: ItemId = "minecraft:golden_apple".parse().unwrap();
        assert_eq!(id.to_string(), "minecraft:golden_apple");
    }

    #[test]
    fn item_id_invalid_namespace_rejected() {
        assert!(ItemId::minecraft("Invalid Path").is_err());
    }

    #[test]
    fn tag_id_minecraft() {
        let tag: TagId<ItemId> = TagId::minecraft("logs").unwrap();
        assert_eq!(tag.to_string(), "minecraft:logs");
        assert_eq!(tag.to_tag_string(), "#minecraft:logs");
    }

    #[test]
    fn tag_id_parse_with_hash() {
        let tag: TagId<ItemId> = "#minecraft:logs".parse().unwrap();
        assert_eq!(tag.to_string(), "minecraft:logs");
    }

    #[test]
    fn tag_id_parse_without_hash() {
        let tag: TagId<BlockId> = "minecraft:planks".parse().unwrap();
        assert_eq!(tag.to_tag_string(), "#minecraft:planks");
    }

    #[test]
    fn tag_id_custom() {
        let rl = ResourceLocation::new("mymod", "special_blocks").unwrap();
        let tag: TagId<BlockId> = TagId::custom(rl);
        assert_eq!(tag.to_string(), "mymod:special_blocks");
    }

    #[test]
    fn dimension_id_minecraft() {
        let id = DimensionId::minecraft("overworld").unwrap();
        assert_eq!(id.to_string(), "minecraft:overworld");
    }

    #[test]
    fn animal_variant_ids_validate_and_serialize() {
        let chicken = ChickenVariantId::minecraft("cold").unwrap();
        let cow = CowVariantId::minecraft("warm").unwrap();
        let pig: PigVariantId = "mymod:arcane_pig".parse().unwrap();
        assert_eq!(chicken.to_string(), "minecraft:cold");
        assert_eq!(cow.to_string(), "minecraft:warm");
        assert_eq!(pig.to_string(), "mymod:arcane_pig");
        assert_eq!(
            serde_json::to_value(chicken).unwrap(),
            serde_json::json!("minecraft:cold")
        );
        assert!("not namespaced".parse::<ChickenVariantId>().is_err());
        assert!("minecraft:bad path".parse::<CowVariantId>().is_err());
    }

    #[test]
    fn entity_type_id_builds_typed_text_hover() {
        let text = sand_commands::Text::new("Inspect").hover_entity(
            EntityTypeId::minecraft("zombie").unwrap(),
            sand_commands::Text::new("Undead"),
        );
        let value: serde_json::Value = serde_json::from_str(&text.to_string()).unwrap();
        assert_eq!(value["hoverEvent"]["type"], "minecraft:zombie");
        assert_eq!(value["hoverEvent"]["name"]["text"], "Undead");
    }
}
