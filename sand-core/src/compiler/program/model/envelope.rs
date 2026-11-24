//! Owned portable envelope definitions.
use super::{
    Advancement, CustomBiome, EventHook, FlatDimension, Function, HudDefinition,
    InventoryTransferDefinition, MenuDefinition, NoticeDialog, Operation, State, TagMembership,
    WorldPredicate,
};
use sand_components::{DamageType, ResourceLocation, StructureTemplate};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A complete compilation request with explicit protocol and target metadata.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Program", module = "sand::advanced::compiler",
 summary = "A complete compilation request with explicit protocol and target metadata.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(format = "Protocol format discriminator, sand.program.", format_version = "Protocol revision, currently 1.", target = "ProgramTarget for this portable definition.", pack = "Pack for this portable definition.", requires = "Requires for this portable definition.", modules = "Modules for this portable definition.", structure_assets = "Gzip NBT bytes keyed by the source names in module structure templates. JSON represents each value as standard Base64."),
)]
pub struct Program<M = Module> {
    /// Must be `sand.program`.
    pub format: String,
    /// Portable protocol revision; currently 1.
    pub format_version: u32,
    /// Exact verified Minecraft release.
    pub target: ProgramTarget,
    /// Pack identity and description.
    pub pack: Pack,
    /// Capabilities required by this input, checked independently of actual use.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    /// Flat module declarations. Resolved programs contain no paths.
    pub modules: Vec<M>,
    /// Binary structure template assets, supplied explicitly by the host.
    ///
    /// Keys match the package-relative paths in structure template declarations.
    /// JSON encodes each byte sequence as standard Base64.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        with = "asset_encoding"
    )]
    #[schemars(with = "BTreeMap<String, String>")]
    pub structure_assets: BTreeMap<String, Vec<u8>>,
}

/// A target supplied explicitly by the host, never inferred from environment.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ProgramTarget", module = "sand::advanced::compiler",
 summary = "A target supplied explicitly by the host, never inferred from environment.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(minecraft = "Minecraft for this portable definition."),
)]
pub struct ProgramTarget {
    /// Exact release such as `26.2`; `latest` is not portable.
    pub minecraft: String,
}

/// Metadata shared by all resources in one pack.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Pack", module = "sand::advanced::compiler",
 summary = "Metadata shared by all resources in one pack.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(namespace = "Namespace for this portable definition.", description = "Description for this portable definition."),
)]
pub struct Pack {
    /// Namespace for compiler lifecycle resources.
    pub namespace: String,
    /// Human-readable description emitted into pack.mcmeta.
    pub description: String,
}

/// An embedded module or a manifest-relative local module document path.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ModuleSource", module = "sand::advanced::compiler",
 summary = "An embedded module or a manifest-relative local module document path.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(Embedded = "Embedded semantic alternative; see Rustdoc for its execution behavior.", Path = "Path semantic alternative; see Rustdoc for its execution behavior."),
 variant_fields(Embedded = ["An owned embedded module definition."], Path = ["A manifest-relative local module document path."]),
)]
pub enum ModuleSource {
    /// A module embedded in the envelope.
    Embedded(Box<Module>),
    /// A path resolved by an explicit host resolver.
    Path(String),
}

/// One named owner of declarations and ordered lifecycle contributions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Module", module = "sand::advanced::compiler",
 summary = "One named owner of declarations and ordered lifecycle contributions.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(flows = "State machines bound to canonical player or global State fields.", id = "Id for this portable definition.", states = "States for this portable definition.", functions = "Functions for this portable definition.", load = "Load for this portable definition.", tick = "Tick for this portable definition.", tags = "Tags for this portable definition.", events = "Player event handlers, dispatched in declaration order.", items = "Custom items referred to by id.", recipes = "Crafting, cooking, and stonecutting recipes.", resource_tags = "Item, block, and entity-type tags.", loot_tables = "Loot tables.", boss_bars = "Boss bars, created at load.", damage_types = "Native damage type definitions.", predicates = "Reusable native world predicates.", advancements = "Native advancement resources.", dialogs = "Native notice dialog resources.", dimensions = "Native flat dimension resources.", biomes = "Native custom biome resources.", structure_templates = "Native structure template resources backed by Program::structure_assets.", inventory_transfers = "Reusable inventory transfers between numbered player slots.", huds = "Reusable player-owned HUD definitions.", menus = "Player-owned native dialog menus.", projectiles = "Player-owned sampled projectile definitions."),
)]
pub struct Module {
    /// Explicit semantic identity, independent of source filename.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Gameplay state machines owned by this module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flows: Vec<super::Flow>,
    /// Player or global State definitions owned by this module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<State>,
    /// Named functions, including declarations unreachable from tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<Function>,
    /// Server-context bodies run at load in authored order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub load: Vec<Operation>,
    /// Server-context bodies run each tick in authored order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tick: Vec<Operation>,
    /// Typed function-tag membership.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<TagMembership>,
    /// Player event handlers, dispatched in declaration order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<EventHook>,
    /// Custom items that operations and recipes refer to by id.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<super::CustomItem>,
    /// Crafting, cooking, and stonecutting recipes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recipes: Vec<super::Recipe>,
    /// Item, block, and entity-type tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resource_tags: Vec<super::ResourceTag>,
    /// Loot tables.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loot_tables: Vec<super::LootTable>,
    /// Boss bars, created at load.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boss_bars: Vec<super::BossBar>,
    /// Native damage type definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub damage_types: Vec<DamageType>,
    /// Reusable native world predicates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub predicates: Vec<WorldPredicate>,
    /// Native advancements declared by this module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub advancements: Vec<Advancement>,
    /// Native notice dialogs declared by this module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dialogs: Vec<NoticeDialog>,
    /// Native flat dimensions declared by this module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dimensions: Vec<FlatDimension>,
    /// Native custom biomes declared by this module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub biomes: Vec<CustomBiome>,
    /// Native templates whose bytes are supplied in the envelope.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structure_templates: Vec<StructureTemplate>,
    /// Named inventory transfer definitions owned by this module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inventory_transfers: Vec<InventoryTransferDefinition>,
    /// Named owner-scoped HUD definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub huds: Vec<HudDefinition>,
    /// Native menus with bounded sessions and ordered choices.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub menus: Vec<MenuDefinition>,
    /// Projectiles launched and cancelled by the owning player.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projectiles: Vec<super::ProjectileDefinition>,
}

mod asset_encoding {
    use super::BTreeMap;
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde::{Deserialize, Deserializer, Serializer, ser::SerializeMap};

    pub(super) fn serialize<S: Serializer>(
        assets: &BTreeMap<String, Vec<u8>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(assets.len()))?;
        for (name, bytes) in assets {
            map.serialize_entry(name, &STANDARD.encode(bytes))?;
        }
        map.end()
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<String, Vec<u8>>, D::Error> {
        BTreeMap::<String, String>::deserialize(deserializer)?
            .into_iter()
            .map(|(name, encoded)| {
                let bytes = STANDARD.decode(encoded).map_err(|error| {
                    serde::de::Error::custom(format!(
                        "invalid Base64 for structure asset `{name}`: {error}"
                    ))
                })?;
                Ok((name, bytes))
            })
            .collect()
    }
}
