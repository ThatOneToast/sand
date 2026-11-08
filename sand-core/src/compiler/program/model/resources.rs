//! Owned portable data resources: custom items, recipes, and registry tags.
use super::TextPart;
use sand_components::ResourceLocation;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// How rare an item looks; colours its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Rarity", module = "sand::advanced::compiler",
 summary = "How rare an item looks; colours its name.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Serialized as the `minecraft:rarity` item component.",
 use_when = ["Styling a custom item's name colour"],
 avoid_when = ["Explicit name colours, which a styled name already sets"],
 example = "let rarity = sand::advanced::compiler::Rarity::Epic;",
 variants(Common = "White name.", Uncommon = "Yellow name.", Rare = "Aqua name.", Epic = "Light purple name."),
)]
pub enum Rarity {
    /// White name.
    Common,
    /// Yellow name.
    Uncommon,
    /// Aqua name.
    Rare,
    /// Light purple name.
    Epic,
}

/// A custom item: a vanilla base item with identity and display components.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::CustomItem", module = "sand::advanced::compiler",
 summary = "A custom item: a vanilla base item with identity and display components.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Lowers to Sand's canonical CustomItem, identified by a `custom_data` marker equal to its id; give, clear, and recipe results use that identity.",
 use_when = ["Declaring a named item that operations and recipes refer to by id"],
 avoid_when = ["Plain vanilla items, which operations name directly"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(id = "Identity, also the custom_data marker.", base = "Vanilla base item.", name = "Display name parts.", lore = "Lore lines.", glint = "Force the enchantment glint on or off.", unbreakable = "Never loses durability.", max_stack = "Largest stack size.", rarity = "Name rarity colour.", enchantments = "Enchantment identities and levels.", custom_model_data = "Resource-pack model selector."),
)]
pub struct CustomItem {
    /// Identity, also the `custom_data` marker.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Vanilla base item, such as `minecraft:diamond_sword`.
    pub base: String,
    /// Display name parts; text only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub name: Vec<TextPart>,
    /// Lore lines; text only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lore: Vec<Vec<TextPart>>,
    /// Force the enchantment glint on or off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glint: Option<bool>,
    /// Never loses durability.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unbreakable: bool,
    /// Largest stack size, 1 to 99.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_stack: Option<u32>,
    /// Name rarity colour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rarity: Option<Rarity>,
    /// Enchantment identities and levels.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub enchantments: BTreeMap<String, u32>,
    /// Resource-pack model selector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_model_data: Option<i32>,
}

/// The item a recipe produces.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::RecipeOutput", module = "sand::advanced::compiler",
 summary = "The item a recipe produces.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Serialized as the recipe `result`; a custom item id produces the item with its components.",
 use_when = ["Naming a recipe result"],
 avoid_when = ["Ingredients, which match by item id or tag"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(item = "Vanilla item or custom item id.", count = "Stack size produced."),
)]
pub struct RecipeOutput {
    /// Vanilla item or declared custom item id.
    pub item: String,
    /// Stack size produced.
    #[serde(default = "one")]
    pub count: u32,
}

fn one() -> u32 {
    1
}

/// Which furnace-like block cooks a recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::CookingMethod", module = "sand::advanced::compiler",
 summary = "Which furnace-like block cooks a recipe.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Selects the `minecraft:smelting`, `blasting`, `smoking`, or `campfire_cooking` recipe type.",
 use_when = ["Declaring a cooking recipe"],
 avoid_when = ["Crafting-table recipes"],
 example = "let method = sand::advanced::compiler::CookingMethod::Smelting;",
 variants(Smelting = "Furnace.", Blasting = "Blast furnace.", Smoking = "Smoker.", Campfire = "Campfire."),
)]
pub enum CookingMethod {
    /// Furnace.
    Smelting,
    /// Blast furnace.
    Blasting,
    /// Smoker.
    Smoking,
    /// Campfire.
    Campfire,
}

/// A crafting, cooking, or stonecutting recipe.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Recipe", module = "sand::advanced::compiler",
 summary = "A crafting, cooking, or stonecutting recipe.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Lowers through Sand's typed recipe builders to `data/<ns>/recipe/<path>.json`. Ingredients are item ids or `#tag` ids.",
 use_when = ["Adding a recipe to a portable program"],
 avoid_when = ["Smithing or special recipes, which need the Rust API"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(Shaped = "A crafting-table pattern.", Shapeless = "Crafting ingredients in any arrangement.", Cooking = "A furnace-like recipe.", Stonecutting = "A stonecutter recipe."),
 variant_fields(Shaped(id = "Identity.", pattern = "One to three equal-width rows.", key = "Pattern character to ingredient.", result = "Produced item."), Shapeless(id = "Identity.", ingredients = "One to nine ingredients.", result = "Produced item."), Cooking(id = "Identity.", method = "Cooking block.", ingredient = "Cooked ingredient.", result = "Produced item.", experience = "Experience awarded.", time = "Cooking time in ticks."), Stonecutting(id = "Identity.", ingredient = "Cut ingredient.", result = "Produced item.")),
)]
pub enum Recipe {
    /// A crafting-table pattern.
    Shaped {
        #[schemars(with = "String")]
        id: ResourceLocation,
        pattern: Vec<String>,
        key: BTreeMap<String, String>,
        result: RecipeOutput,
    },
    /// Crafting ingredients in any arrangement.
    Shapeless {
        #[schemars(with = "String")]
        id: ResourceLocation,
        ingredients: Vec<String>,
        result: RecipeOutput,
    },
    /// A furnace-like recipe.
    Cooking {
        #[schemars(with = "String")]
        id: ResourceLocation,
        method: CookingMethod,
        ingredient: String,
        result: RecipeOutput,
        #[serde(default)]
        experience: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        time: Option<u32>,
    },
    /// A stonecutter recipe.
    Stonecutting {
        #[schemars(with = "String")]
        id: ResourceLocation,
        ingredient: String,
        result: RecipeOutput,
    },
}

impl Recipe {
    /// The recipe identity.
    pub(crate) fn id(&self) -> &ResourceLocation {
        match self {
            Recipe::Shaped { id, .. }
            | Recipe::Shapeless { id, .. }
            | Recipe::Cooking { id, .. }
            | Recipe::Stonecutting { id, .. } => id,
        }
    }
}

/// Which registry a tag groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::TagKind", module = "sand::advanced::compiler",
 summary = "Which registry a tag groups.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Selects `data/<ns>/tags/item`, `tags/block`, or `tags/entity_type`.",
 use_when = ["Declaring a registry tag"],
 avoid_when = ["Function tags, which modules declare as tag memberships"],
 example = "let kind = sand::advanced::compiler::TagKind::Item;",
 variants(Item = "Items.", Block = "Blocks.", EntityType = "Entity types."),
)]
pub enum TagKind {
    /// Items.
    Item,
    /// Blocks.
    Block,
    /// Entity types.
    EntityType,
}

/// A registry tag: a named group of ids and other tags.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ResourceTag", module = "sand::advanced::compiler",
 summary = "A registry tag: a named group of ids and other tags.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Lowers through Sand's typed tags; `#ns:name` values reference other tags.",
 use_when = ["Grouping items, blocks, or entity types for recipes and queries"],
 avoid_when = ["Function tags"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(kind = "Grouped registry.", id = "Tag identity.", values = "Ids or `#tag` references.", replace = "Replace lower-priority definitions instead of merging."),
)]
pub struct ResourceTag {
    /// Grouped registry.
    pub kind: TagKind,
    /// Tag identity.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Ids or `#tag` references.
    pub values: Vec<String>,
    /// Replace lower-priority definitions instead of merging.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub replace: bool,
}
