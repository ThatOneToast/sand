//! Owned portable data resources: custom items, recipes, and registry tags.
use super::{FieldReference, FunctionReference, ScoreOperand, TextPart, Weather};
use sand_components::{EntityTypeId, MobCategory, ResourceLocation, SpawnCost, SpawnEntry};
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
 fields(id = "Identity, also the custom_data marker.", base = "Vanilla base item.", name = "Display name parts.", lore = "Lore lines.", glint = "Force the enchantment glint on or off.", unbreakable = "Never loses durability.", max_stack = "Largest stack size.", rarity = "Name rarity colour.", enchantments = "Enchantment identities and levels.", custom_model_data = "Resource-pack model selector.", book = "Fixed written-book title, author and pages."),
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
    /// Written-book content for a `minecraft:written_book` base item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub book: Option<WrittenBookContent>,
}

/// Fixed pages for a custom written book, using portable rich text.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::WrittenBookContent", module = "sand::advanced::compiler",
 summary = "Fixed pages for a custom written book, using portable rich text.",
 context = "Attach it to a CustomItem whose base is minecraft:written_book. Pages accept styles, translations, tooltips, and native page links. Sand lowers them through the same WrittenBook builder used by Rust.",
 minecraft = "Emits native written_book_content on item commands and recipe results.",
 use_when = ["Declaring a portable written manual or a fixed journal"],
 avoid_when = ["Capturing writable-book input or formatting a runtime State snapshot"],
 fields(title = "Native title, limited to 32 UTF-16 code units.", author = "Book author.", pages = "Ordered rich-text pages, up to 100."),
 example = "use sand::advanced::compiler::WrittenBookContent;",
)]
pub struct WrittenBookContent {
    /// Native title, limited to 32 UTF-16 code units.
    pub title: String,
    /// Book author shown by the native interface.
    pub author: String,
    /// Ordered rich-text pages, up to 100.
    pub pages: Vec<Vec<TextPart>>,
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

/// A crafting, cooking, stonecutting, or smithing recipe.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Recipe", module = "sand::advanced::compiler",
 summary = "A crafting, cooking, stonecutting, or smithing recipe.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Lowers through Sand's typed recipe builders to `data/<ns>/recipe/<path>.json`. Ingredients are item ids or `#tag` ids.",
 use_when = ["Adding a recipe to a portable program"],
 avoid_when = ["Special recipes, which need the Rust API"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(Shaped = "A crafting-table pattern.", Shapeless = "Crafting ingredients in any arrangement.", Cooking = "A furnace-like recipe.", Stonecutting = "A stonecutter recipe.", Smithing = "A smithing-table upgrade.", SmithingTrim = "A smithing-table armor trim."),
 variant_fields(Shaped(id = "Identity.", pattern = "One to three equal-width rows.", key = "Pattern character to ingredient.", result = "Produced item."), Shapeless(id = "Identity.", ingredients = "One to nine ingredients.", result = "Produced item."), Cooking(id = "Identity.", method = "Cooking block.", ingredient = "Cooked ingredient.", result = "Produced item.", experience = "Experience awarded.", time = "Cooking time in ticks."), Stonecutting(id = "Identity.", ingredient = "Cut ingredient.", result = "Produced item."), Smithing(id = "Identity.", template = "Smithing template.", base = "Upgraded item.", addition = "Upgrade material.", result = "Produced item."), SmithingTrim(id = "Identity.", template = "Trim template ingredient.", base = "Armor ingredient.", addition = "Trim material ingredient.", pattern = "Registry identity of the applied trim pattern.")),
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
    /// A smithing-table upgrade.
    Smithing {
        #[schemars(with = "String")]
        id: ResourceLocation,
        template: String,
        base: String,
        addition: String,
        result: RecipeOutput,
    },
    /// Apply an armor trim while preserving the input armor and its components.
    SmithingTrim {
        #[schemars(with = "String")]
        id: ResourceLocation,
        template: String,
        base: String,
        addition: String,
        #[schemars(with = "String")]
        pattern: ResourceLocation,
    },
}

impl Recipe {
    /// The recipe identity.
    pub(crate) fn id(&self) -> &ResourceLocation {
        match self {
            Recipe::Shaped { id, .. }
            | Recipe::Shapeless { id, .. }
            | Recipe::Cooking { id, .. }
            | Recipe::Stonecutting { id, .. }
            | Recipe::Smithing { id, .. }
            | Recipe::SmithingTrim { id, .. } => id,
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

/// One choice in a loot table.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::LootItem", module = "sand::advanced::compiler",
 summary = "One choice in a loot table.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Serialized as a `minecraft:item` loot entry, with `set_count` when more than one.",
 use_when = ["Listing what a loot table can produce"],
 avoid_when = ["Custom items, whose components need the Rust API"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(item = "Vanilla item identity.", weight = "Relative chance.", count = "Stack size produced."),
)]
pub struct LootItem {
    /// Vanilla item identity.
    pub item: String,
    /// Relative chance.
    #[serde(default = "one")]
    pub weight: u32,
    /// Stack size produced.
    #[serde(default = "one")]
    pub count: u32,
}

/// One independently rolled group of weighted loot choices.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::LootPool", module = "sand::advanced::compiler",
 summary = "One independently rolled group of weighted loot choices.",
 context = "The portable program model retains author-defined pool order and optional conditions.",
 minecraft = "Lowers into Sand's typed loot pool with vanilla random chance and weather conditions.",
 use_when = ["Combining fixed and conditional loot in the same resource"],
 avoid_when = ["Using a function that requires a custom loot context"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(rolls = "Choices drawn from this pool.", items = "Weighted item choices.", chance = "Probability that this pool runs.", weather = "Required weather in the loot context.", predicate = "A reusable predicate file checked in the loot context."),
)]
pub struct LootPool {
    /// Choices drawn from this pool.
    #[serde(default = "one")]
    pub rolls: u32,
    /// Weighted item choices.
    pub items: Vec<LootItem>,
    /// Probability that this pool runs, from zero to one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chance: Option<f64>,
    /// Required weather in the loot context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather: Option<Weather>,
    /// Optional reusable predicate that this pool requires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub predicate: Option<ResourceLocation>,
}

/// A loot table containing one or more independently rolled pools.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::LootTable", module = "sand::advanced::compiler",
 summary = "A loot table containing one or more independently rolled pools.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Lowers through Sand's typed loot builder to `data/<ns>/loot_table/<path>.json`.",
 use_when = ["Handing out or dropping randomized rewards"],
 avoid_when = ["Using custom loot functions that require the Rust API"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(id = "Identity.", pools = "Ordered independent, optionally conditional pools."),
)]
pub struct LootTable {
    /// Identity.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Ordered independent, optionally conditional pools.
    pub pools: Vec<LootPool>,
}

/// A reusable native test of location or weather, optionally gated by chance.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::WorldPredicate", module = "sand::advanced::compiler",
 summary = "A reusable native test of location or weather.",
 context = "All selected constraints must pass at the predicate execution origin.",
 minecraft = "Exports a named predicate using Sand's typed PredicateRoot model.",
 use_when = ["Sharing a native world condition between functions and loot pools"],
 avoid_when = ["Testing a mutable scoreboard value without compiling a predicate resource"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(id = "Predicate identity.", biome = "Biome identity.", dimension = "Required dimension.", block = "Required block.", weather = "Required weather.", chance = "Probability the predicate passes."),
)]
pub struct WorldPredicate {
    /// Predicate identity.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Biome at the execution origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biome: Option<String>,
    /// Dimension of the execution origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub dimension: Option<ResourceLocation>,
    /// Block at the execution origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block: Option<String>,
    /// Current weather.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather: Option<Weather>,
    /// Probability between zero and one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chance: Option<f64>,
}

/// The colour of a boss bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::BossBarColor", module = "sand::advanced::compiler",
 summary = "The colour of a boss bar.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Serialized as the `bossbar set color` argument.",
 use_when = ["Declaring a boss bar"],
 avoid_when = ["Text colours; use TextColor"],
 example = "let color = sand::advanced::compiler::BossBarColor::Red;",
 variants(Blue = "Blue.", Green = "Green.", Pink = "Pink.", Purple = "Purple.", Red = "Red.", White = "White.", Yellow = "Yellow."),
)]
pub enum BossBarColor {
    /// Blue.
    Blue,
    /// Green.
    Green,
    /// Pink.
    Pink,
    /// Purple.
    Purple,
    /// Red.
    Red,
    /// White.
    White,
    /// Yellow.
    Yellow,
}

/// How a boss bar is divided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::BossBarStyle", module = "sand::advanced::compiler",
 summary = "How a boss bar is divided.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Serialized as the `bossbar set style` argument.",
 use_when = ["Declaring a boss bar"],
 avoid_when = ["Text styling; use TextStyle"],
 example = "let style = sand::advanced::compiler::BossBarStyle::Progress;",
 variants(Progress = "One solid bar.", Notched6 = "Six notches.", Notched10 = "Ten notches.", Notched12 = "Twelve notches.", Notched20 = "Twenty notches."),
)]
pub enum BossBarStyle {
    /// One solid bar.
    Progress,
    /// Six notches.
    Notched6,
    /// Ten notches.
    Notched10,
    /// Twelve notches.
    Notched12,
    /// Twenty notches.
    Notched20,
}

/// A named bar shown at the top of the screen.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::BossBar", module = "sand::advanced::compiler",
 summary = "A named bar shown at the top of the screen.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Created and refreshed at load with `bossbar add` and `bossbar set`.",
 use_when = ["Showing progress or boss health"],
 avoid_when = ["Text that needs no bar; use Message"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(id = "Identity.", title = "Title parts; text only.", color = "Bar colour.", style = "Bar division.", max = "Largest value."),
)]
pub struct BossBar {
    /// Identity.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Title parts; text only.
    pub title: Vec<TextPart>,
    /// Bar colour.
    pub color: BossBarColor,
    /// Bar division.
    pub style: BossBarStyle,
    /// Largest value.
    #[serde(default = "hundred")]
    pub max: u32,
}

fn hundred() -> u32 {
    100
}

/// A named native advancement with typed criteria and optional presentation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Advancement", module = "sand::advanced::compiler",
 summary = "A named native advancement with typed criteria and optional presentation.",
 context = "Criteria names retain player progress across reload. Use the display only for a visible milestone, and set require_any when its criteria are alternatives.",
 minecraft = "Lowers through the canonical Advancement component to a native advancement and optional experience reward.",
 use_when = ["Declaring a player milestone in a portable program"],
 avoid_when = ["Polling scoreboard State for native advancement progress"],
 fields(id = "Advancement identity.", display = "Optional presentation for a visible milestone.", criteria = "Stable authored names and native conditions.", require_any = "Complete on any criterion instead of requiring all.", experience = "Experience awarded at completion."),
 example = "use sand::advanced::compiler::Advancement;",
)]
pub struct Advancement {
    /// Advancement identity.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Optional presentation for a visible milestone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<AdvancementDisplay>,
    /// Nonempty map of stable names to native criteria.
    pub criteria: BTreeMap<String, AdvancementCriterion>,
    /// Whether one criterion is sufficient for completion.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub require_any: bool,
    /// Experience points awarded at completion, or zero for no reward.
    #[serde(default)]
    pub experience: i32,
}

/// The optional native presentation of an advancement.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::AdvancementDisplay", module = "sand::advanced::compiler",
 summary = "The optional native presentation of an advancement.",
 context = "The icon is a vanilla item and the title and description support static rich text. A displayed root needs an explicit background on Minecraft 26.3.",
 minecraft = "Becomes the advancement menu icon, title, description and root background.",
 use_when = ["Showing a portable advancement in the vanilla player menu"],
 avoid_when = ["An internal advancement used as a native signal"],
 fields(icon = "Vanilla item identity.", title = "Static rich title.", description = "Static rich description.", background = "Optional root texture identity."),
 example = "use sand::advanced::compiler::AdvancementDisplay;",
)]
pub struct AdvancementDisplay {
    /// Vanilla item identity used as the icon.
    pub icon: String,
    /// Static rich title.
    pub title: Vec<TextPart>,
    /// Static rich description.
    pub description: Vec<TextPart>,
    /// Root texture identity. Minecraft 26.3 requires one for displayed roots.
    #[schemars(with = "Option<String>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<ResourceLocation>,
}

/// A vanilla condition that records one piece of player advancement progress.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::AdvancementCriterion", module = "sand::advanced::compiler",
 summary = "A vanilla condition that records one piece of player advancement progress.",
 context = "Give each condition a stable name in its Advancement. The game tracks each player's completion and persists criterion progress.",
 minecraft = "Uses native inventory, consumption, placement, kill, and dimension-change triggers.",
 use_when = ["Composing native advancement criteria"],
 avoid_when = ["Executing an arbitrary script as a criterion"],
 variants(InventoryItem = "Acquire an item.", PlaceBlock = "Place the selected block.", KillEntity = "Kill an entity of the selected type.", ConsumeItem = "Consume the selected item.", ArriveDimension = "Enter the selected dimension."),
 variant_fields(InventoryItem(item = "Vanilla item identity."), PlaceBlock(block = "Vanilla block identity."), KillEntity(entity = "Vanilla entity type identity."), ConsumeItem(item = "Vanilla item identity."), ArriveDimension(dimension = "Destination dimension identity.")),
 example = "use sand::advanced::compiler::AdvancementCriterion;",
)]
pub enum AdvancementCriterion {
    /// Have an item on a matching inventory change.
    InventoryItem { item: String },
    /// Place a block of the given type.
    PlaceBlock { block: String },
    /// Receive player kill credit for an entity of the given type.
    KillEntity { entity: String },
    /// Finish consuming the selected item.
    ConsumeItem { item: String },
    /// Change dimensions and arrive in the selected destination.
    ArriveDimension {
        /// Destination dimension identity.
        #[schemars(with = "String")]
        dimension: sand_components::DimensionId,
    },
}

/// A native notice dialog with static rich text and a dismiss button.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::NoticeDialog", module = "sand::advanced::compiler",
 summary = "A native notice dialog with static rich text and a dismiss button.",
 context = "Use Action::Dialog to display the resource for the executing player. Translated text resolves in the player's client language.",
 minecraft = "Lowers through the canonical Dialog builder to a minecraft:notice resource.",
 use_when = ["Showing a tutorial or information panel"],
 avoid_when = ["Capturing form submissions or executing custom button actions"],
 fields(id = "Dialog identity.", title = "Static rich title.", body = "Ordered static text blocks.", button = "Optional dismiss label, defaulting to the localized Done label."),
 example = "use sand::advanced::compiler::NoticeDialog;",
)]
pub struct NoticeDialog {
    /// Dialog identity.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Static rich title.
    pub title: Vec<TextPart>,
    /// Ordered static text blocks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub body: Vec<Vec<TextPart>>,
    /// Optional dismiss label. An empty list uses the client's Done label.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub button: Vec<TextPart>,
}

/// A native dimension with an ordered stack of flat terrain layers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::FlatDimension", module = "sand::advanced::compiler",
 summary = "A native dimension with an ordered stack of flat terrain layers.",
 context = "Declare the dimension before starting the world. Portable entity continuations can resume in this dimension while its chunks are loaded.",
 minecraft = "Lowers through the canonical Dimension and FlatGenerator to a minecraft:flat dimension resource.",
 use_when = ["Defining a dedicated gameplay or test world"],
 avoid_when = ["Configuring a noise generator or defining a new dimension type"],
 fields(id = "Dimension identity.", dimension_type = "Existing dimension type identity.", flat = "Bottom-to-top layers and shared flat generator settings."),
 example = "use sand::advanced::compiler::FlatDimension;",
)]
pub struct FlatDimension {
    /// Identity used by dimension-scoped operations.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Existing dimension type, which defines height and environmental rules.
    #[schemars(with = "String")]
    pub dimension_type: sand_components::DimensionTypeId,
    /// Canonical bottom-to-top layers and flat-world settings.
    pub flat: sand_components::FlatGenerator,
}

/// A native dimension with flat layers or noise terrain.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::DimensionDefinition", module = "sand::advanced::compiler",
 summary = "A typed native dimension definition.",
 context = "Use flat layers or noise terrain sourced from a named settings resource.",
 minecraft = "Adapts to the canonical Dimension component before validation and export.",
 use_when = ["Declaring worlds in portable gameplay programs"],
 avoid_when = ["Defining the dimension type's environmental rules"],
 variants(Flat = "Generate an ordered stack of layers.", Noise = "Generate noise terrain with a typed biome source."),
 variant_fields(Flat = ["The flat terrain definition."], Noise = ["The noise terrain definition."]),
 example = "use sand::advanced::compiler::DimensionDefinition;",
)]
pub enum DimensionDefinition {
    /// An ordered stack of flat layers.
    Flat(FlatDimension),
    /// A noise world with a typed biome source.
    Noise(NoiseDimension),
}

impl DimensionDefinition {
    pub(crate) fn id(&self) -> &ResourceLocation {
        match self {
            Self::Flat(dimension) => &dimension.id,
            Self::Noise(dimension) => &dimension.id,
        }
    }
}

/// A native noise dimension with a typed biome source.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::NoiseDimension", module = "sand::advanced::compiler",
 summary = "A native noise dimension with a typed biome source.",
 context = "The noise settings and biome reference existing resources. A module can declare the biome in the same program.",
 minecraft = "Lowers through the canonical NoiseGenerator and BiomeSource.",
 use_when = ["Generating noise-shaped terrain with controlled biome placement"],
 avoid_when = ["Defining a stack of flat layers"],
 fields(id = "Native dimension identity.", dimension_type = "Existing dimension type identity.", noise_settings = "Existing noise settings resource.", biome_source = "Biome placement, independent of terrain density."),
 example = "use sand::advanced::compiler::NoiseDimension;",
)]
pub struct NoiseDimension {
    /// Identity of the generated native dimension.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Existing dimension type.
    #[schemars(with = "String")]
    pub dimension_type: sand_components::DimensionTypeId,
    /// The noise settings used to shape terrain.
    #[schemars(with = "String")]
    pub noise_settings: sand_components::NoiseSettingsId,
    /// Biome placement across the generated terrain.
    pub biome_source: sand_components::BiomeSource,
}

/// Climate, visual settings, and natural spawn rules for a native custom biome.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::CustomBiome", module = "sand::advanced::compiler",
 summary = "Climate, visual settings, and natural spawn rules for a native custom biome.",
 context = "Reference the biome from FlatDimension or a block-position biome condition. Install the pack before starting the world to register the biome.",
 minecraft = "Lowers through Biome, BiomeEffects, and the shared weighted spawn model to a native worldgen biome.",
 use_when = ["Giving an authored dimension its own climate and colors"],
 avoid_when = ["Declaring placed features or terrain carvers"],
 fields(id = "Biome identity.", precipitation = "Whether rain or snow occurs.", temperature = "Climate temperature.", downfall = "Humidity from zero to one.", fog_color = "Fog RGB.", water_color = "Water surface RGB.", water_fog_color = "Underwater fog RGB.", sky_color = "Sky RGB.", grass_color = "Grass RGB override.", foliage_color = "Foliage RGB override.", spawners = "Ordered weighted entries within each mob category.", spawn_costs = "Population density budgets and charges by entity type.", features = "Ordered placed-feature references in each terrain generation step."),
 example = "use sand::advanced::compiler::CustomBiome;",
)]
pub struct CustomBiome {
    /// Identity used in world generators and biome conditions.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Whether the biome has rain or snow. Defaults to true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precipitation: Option<bool>,
    /// Climate temperature. Defaults to 0.5.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Humidity from zero to one. Defaults to 0.5.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downfall: Option<f32>,
    /// Fog color, as an RGB integer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog_color: Option<u32>,
    /// Water surface color, as an RGB integer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water_color: Option<u32>,
    /// Underwater fog color, as an RGB integer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water_fog_color: Option<u32>,
    /// Sky color, as an RGB integer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sky_color: Option<u32>,
    /// Optional grass color override, as an RGB integer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grass_color: Option<u32>,
    /// Optional foliage color override, as an RGB integer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foliage_color: Option<u32>,
    /// Natural spawn entries in authored order within each vanilla category.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub spawners: BTreeMap<MobCategory, Vec<SpawnEntry>>,
    /// Optional population-density budget and charge for each entity.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub spawn_costs: BTreeMap<String, SpawnCost>,
    /// Placed features in declaration order within a native decoration step.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[schemars(with = "BTreeMap<sand_components::GenerationStep, Vec<String>>")]
    pub features: BTreeMap<sand_components::GenerationStep, Vec<sand_components::PlacedFeatureId>>,
}

/// One eligible child attack run by a player encounter.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
    registry = sand_api_contract, path = "sand::advanced::compiler::EncounterAttack", module = "sand::advanced::compiler",
    summary = "A bounded child attack for a portable player encounter.",
    context = "The attack holds the controller until its lifetime, including recovery, ends.",
    minecraft = "Starts a shared Lifetime child as the encounter's owner.",
    use_when = ["Defining one priority choice in an encounter"],
    avoid_when = ["Starting an independent task with no parent"],
    fields(ticks = "Lifetime including recovery in ticks.", when = "Optional State guard for selection.", start = "Player callback at admission.", stop = "Optional player callback at cleanup."),
    example = "use sand::advanced::compiler::EncounterAttack;",
)]
pub struct EncounterAttack {
    /// Lifetime in Minecraft ticks, including the recovery interval.
    pub ticks: u32,
    /// Optional State guard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<super::FieldReference>,
    /// Run as the owner when this child starts.
    pub start: super::FunctionReference,
    /// Run as the owner at cleanup, when the owner is resolvable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<super::FunctionReference>,
}

/// A player-owned bounded priority encounter.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
    registry = sand_api_contract, path = "sand::advanced::compiler::EncounterDefinition", module = "sand::advanced::compiler",
    summary = "A bounded portable encounter with ordered eligible attacks.",
    context = "The controller starts at most one outstanding child and preserves player ownership.",
    minecraft = "Uses the shared Encounter and Lifetime implementations for selection and cleanup.",
    use_when = ["Orchestrating ordered gameplay attacks with recovery"],
    avoid_when = ["Authoring native entity pathfinding"],
    fields(id = "Registered encounter identity.", ticks = "Maximum active duration in ticks.", when = "Optional State condition for keeping the controller active.", attacks = "Child attacks in descending priority order.", stop = "Optional player callback after the controller stops."),
    example = "use sand::advanced::compiler::EncounterDefinition;",
)]
pub struct EncounterDefinition {
    /// Registered encounter identity.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Maximum active duration in loaded ticks.
    pub ticks: u32,
    /// Optional State guard checked on each controller tick.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<super::FieldReference>,
    /// Child attacks in descending priority order.
    pub attacks: Vec<EncounterAttack>,
    /// Optional cleanup callback, invoked for a resolvable owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<super::FunctionReference>,
}

/// Explicit live and saved player State fields in one checkpoint projection.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
    registry = sand_api_contract, path = "sand::advanced::compiler::CheckpointFieldPair", module = "sand::advanced::compiler",
    summary = "A live and saved field pair in a checkpoint.",
    context = "The fields use separate player State components.",
    minecraft = "Both scores are checked before a checkpoint copies either direction.",
    use_when = ["Declaring one integer field copied at a gameplay checkpoint"],
    avoid_when = ["Snapshotting storage or an entity's position"],
    fields(live = "Current gameplay field.", saved = "Snapshot field in saved State."),
    example = "use sand::advanced::compiler::CheckpointFieldPair;",
)]
pub struct CheckpointFieldPair {
    /// The live gameplay field.
    pub live: super::FieldReference,
    /// The saved copy in the checkpoint State.
    pub saved: super::FieldReference,
}

/// An owned portable definition of a canonical player checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
    registry = sand_api_contract, path = "sand::advanced::compiler::CheckpointDefinition", module = "sand::advanced::compiler",
    summary = "A portable player checkpoint with an explicit State projection.",
    context = "Restore revalidates the original player and the snapshot after preparation.",
    minecraft = "Uses the shared Checkpoint runtime for score preflight, transaction locking, and UUID isolation.",
    use_when = ["Saving a selected group of gameplay scores"],
    avoid_when = ["Saving automatically scheduled tasks or native world state"],
    fields(id = "Registered checkpoint identity.", valid = "Initially zero flag in the saved State.", fields = "Live and saved score pairs.", before_restore = "Preparation callback, before revalidation.", after_restore = "Callback after all restored values commit."),
    example = "use sand::advanced::compiler::CheckpointDefinition;",
)]
pub struct CheckpointDefinition {
    /// Registered checkpoint identity.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// The saved State field representing whether this snapshot exists.
    pub valid: super::FieldReference,
    /// Explicit integer projection in authored order.
    pub fields: Vec<CheckpointFieldPair>,
    /// Called as the player before restoring and revalidating the snapshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before_restore: Option<super::FunctionReference>,
    /// Called as the player after the restored fields commit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_restore: Option<super::FunctionReference>,
}

/// A reusable native player inventory transfer between two numbered slots.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::InventoryTransferDefinition", module = "sand::advanced::compiler",
 summary = "A reusable native player inventory transfer between two numbered slots.",
 context = "Action::TransferInventory attempts the move as the executing player. Use distinct slots. An exact amount splits the source stack, and omitted amount moves the full stack.",
 minecraft = "Lowers through InventoryTransfer with shared identity, stack capacity, locking, and callback semantics.",
 use_when = ["Moving whole stacks or exact quantities without dropping overflow"],
 avoid_when = ["Moving stacks to a block or entity outside the current player's numbered inventory"],
 fields(id = "The transfer identity.", from = "Numbered source slot.", to = "Numbered destination slot.", amount = "Exact quantity, from 1 to 99. Omit for a whole stack.", success = "Optional function invoked as the player after success.", rejected = "Optional function invoked as the player when an admitted transfer is rejected."),
 example = "use sand::advanced::compiler::InventoryTransferDefinition;",
)]
pub struct InventoryTransferDefinition {
    /// Identity used by transfer operations.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Numbered source slot, from 0 to 35.
    #[schemars(with = "u8", range(max = 35))]
    pub from: crate::item::InventoryIndex,
    /// Numbered destination slot, from 0 to 35.
    #[schemars(with = "u8", range(max = 35))]
    pub to: crate::item::InventoryIndex,
    /// Exact quantity to move. The default moves the whole source stack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<u32>,
    /// Invoked as the player after a completed move.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<super::FunctionReference>,
    /// Invoked as the player after an admitted move is rejected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejected: Option<super::FunctionReference>,
}

/// Owner-scoped presentation that lasts for a bounded number of loaded ticks.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::HudDefinition", module = "sand::advanced::compiler",
 summary = "Owner-scoped presentation that lasts for a bounded number of loaded ticks.",
 context = "Use Action::Hud to start or stop presentation as the current player. Supply an actionbar, a private boss bar, or both.",
 minecraft = "Lowers through Hud and Lifetime with owner isolation, exclusive actionbar leases, and cleanup.",
 use_when = ["Presenting player-specific gameplay progress for a loaded duration"],
 avoid_when = ["Showing one shared bar to a group; use BossBar"],
 fields(id = "HUD identity.", ticks = "Positive lifetime in loaded ticks.", every = "Refresh period in loaded ticks; defaults to one.", actionbar = "Optional rich actionbar text.", bossbar = "Optional private progress bar."),
 example = "use sand::advanced::compiler::HudDefinition;",
)]
pub struct HudDefinition {
    /// Identity used to start or cancel the HUD.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Loaded ticks before the native presentation expires.
    pub ticks: u32,
    /// Refresh cadence, defaulting to every loaded tick.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub every: Option<u32>,
    /// Static rich text displayed in the owner's actionbar.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actionbar: Vec<TextPart>,
    /// A native private boss bar with a constant or State-backed value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bossbar: Option<HudBossBar>,
}

/// The title, current value, and maximum of a player's private HUD boss bar.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::HudBossBar", module = "sand::advanced::compiler",
 summary = "The title, current value, and maximum of a player's private HUD boss bar.",
 context = "The value is read in player context each time Hud refreshes and is clamped to 0..=max.",
 minecraft = "Lowers to one native boss bar per admitted player-owned HUD instance.",
 use_when = ["Showing a private progress or resource meter"],
 avoid_when = ["Using one shared boss bar value for all players"],
 fields(title = "Static rich title.", value = "Constant or State score available to the player.", max = "Positive maximum."),
 example = "use sand::advanced::compiler::HudBossBar;",
)]
pub struct HudBossBar {
    /// Text shown over the bar.
    pub title: Vec<TextPart>,
    /// Current value read on each refresh.
    pub value: ScoreOperand,
    /// Positive bar maximum.
    pub max: u32,
}

/// An ordered native dialog menu with one-shot callbacks as the owning player.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::MenuDefinition", module = "sand::advanced::compiler",
 summary = "An ordered native dialog menu with one-shot callbacks as the owning player.",
 context = "Use Action::Menu to open or close the player’s session. Reopening creates fresh input tokens. The shared Menu runtime validates expiry, ownership, and navigation.",
 minecraft = "Lowers through the native Menu system to paginated dialog buttons with generation-bound input.",
 use_when = ["Offering a player a bounded list of gameplay actions"],
 avoid_when = ["Showing a fixed dismissible message; use NoticeDialog"],
 fields(id = "Menu identity.", title = "Static rich title.", ticks = "Positive number of online ticks before expiration.", page_size = "Optional number of choices per page, from 1 to 64.", choices = "Ordered buttons in the main view.", views = "Named secondary views.", start = "Initial view, which defaults to main.", while_open = "Optional score that must equal one throughout a session."),
 example = "use sand::advanced::compiler::MenuDefinition;",
)]
pub struct MenuDefinition {
    /// Identity used to open or close a session.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Title of each native dialog page.
    pub title: Vec<TextPart>,
    /// Online ticks before the open session expires.
    pub ticks: u32,
    /// Optional count of choices per page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Buttons in authored order.
    pub choices: Vec<MenuChoice>,
    /// Additional views in declaration order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub views: Vec<MenuView>,
    /// Initial view name. Omission selects the main view.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// When present, the referenced score must equal one while the session is open.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub while_open: Option<FieldReference>,
}

impl MenuDefinition {
    pub(crate) fn view_index(&self, name: &str) -> Option<i32> {
        if name == "main" {
            Some(0)
        } else {
            self.views
                .iter()
                .position(|view| view.name == name)
                .map(|index| (index + 1) as i32)
        }
    }

    pub(crate) fn choice_lists(&self) -> impl Iterator<Item = (&[MenuChoice], String)> {
        std::iter::once((&self.choices[..], "choices".into())).chain(
            self.views
                .iter()
                .enumerate()
                .map(|(index, view)| (&view.choices[..], format!("views/{index}/choices"))),
        )
    }
}

/// A named view within a native conversation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::MenuView", module = "sand::advanced::compiler",
 summary = "A named view in a native menu.",
 context = "The menu may start here or enter it through a choice. An entered view receives newly issued player tokens.",
 minecraft = "Uses the shared native Menu pagination and navigation.",
 use_when = ["Building a conversation or confirmation screen"],
 avoid_when = ["Persisting progress; store durable decisions in State"],
 fields(name = "Name referenced by choices or the opening view.", title = "Rich heading on this view.", choices = "Ordered buttons for this view."),
 example = "use sand::advanced::compiler::MenuView;",
)]
pub struct MenuView {
    /// Identifier used for transitions into the view.
    pub name: String,
    /// Heading displayed on each page of this view.
    pub title: Vec<TextPart>,
    /// Buttons in authored order.
    pub choices: Vec<MenuChoice>,
}

/// A native menu button and the gameplay function it invokes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::MenuChoice", module = "sand::advanced::compiler",
 summary = "A native menu button and the gameplay function it invokes.",
 context = "The menu consumes the player’s session before invoking the parameterless action, which may open new presentation.",
 minecraft = "Uses a typed Menu choice and the shared one-shot native dialog routing.",
 use_when = ["Adding one action to a player's menu"],
 avoid_when = ["Adding a fixed notice's dismiss label"],
 fields(label = "Static rich button label.", action = "Optional function invoked as the player after the session is consumed.", next = "Optional destination view entered after the callback.", when = "Optional score that must equal one at dispatch."),
 example = "use sand::advanced::compiler::MenuChoice;",
)]
pub struct MenuChoice {
    /// Label shown on the button.
    pub label: Vec<TextPart>,
    /// Function invoked with the choosing player as executor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<FunctionReference>,
    /// View to open after the optional callback completes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    /// When present, the referenced score must equal one to authorize the choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<FieldReference>,
}

/// A player-owned sampled projectile, built through Sand's shared flight runtime.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ProjectileDefinition", module = "sand::advanced::compiler",
 summary = "A player-owned sampled projectile.",
 context = "Flight ends on collision, expiry, cancellation, or entry into an unloaded chunk. The hit callback runs as the original player at the impact position.",
 minecraft = "Reuses the Rust Projectile system and its per-owner lifecycle, capped sampling, and native damage attribution.",
 use_when = ["Launching a player-owned projectile from a portable gameplay function"],
 avoid_when = ["Moving a native arrow or simulating swept hitboxes"],
 fields(id = "Projectile identity.", ticks = "Maximum flight in loaded ticks.", speed = "Blocks traveled each loaded tick; defaults to one.", radius = "Entity collision radius; defaults to half a block.", target = "Optional vanilla entity type filter.", damage = "Optional native damage in hit points attributed to the owner.", hit = "Optional player function invoked at entity or block impact."),
 example = "use sand::advanced::compiler::ProjectileDefinition;",
)]
pub struct ProjectileDefinition {
    /// Identity used to launch or cancel flight.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Maximum flight in loaded ticks.
    pub ticks: u32,
    /// Speed in blocks per tick. Defaults to one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    /// Entity collision radius. Defaults to half a block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
    /// Filter potential hits to this entity type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub target: Option<EntityTypeId>,
    /// Native damage in hit points, attributed to the launching player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage: Option<f64>,
    /// Player function run at the first block or entity impact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit: Option<FunctionReference>,
}

/// An ordered player track backed by a canonical State number.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ProgressionDefinition", module = "sand::advanced::compiler",
 summary = "An ordered player track backed by State.",
 context = "The next objective index lives in the supplied player score. Advance consumes at most one objective. Use an explicit reset to replay the track.",
 minecraft = "Lowers through the shared Progression system, including committed rewards and its shared lock.",
 use_when = ["Declaring a player quest or phase track"],
 avoid_when = ["Creating an arbitrary looping State machine"],
 fields(id = "Progression identity.", progress = "Player State number storing the next objective index.", objectives = "Ordered objectives from zero to the completion index.", branches = "Optional forward branches resolved in declaration order."),
 example = "use sand::advanced::compiler::ProgressionDefinition;",
)]
pub struct ProgressionDefinition {
    /// Identity used to advance, reset or test this track.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// The player's durable, non-ticking score initialized to zero.
    pub progress: FieldReference,
    /// Objectives in their persisted order.
    pub objectives: Vec<ProgressionObjective>,
    /// Optional forward successors in declaration order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<ProgressionBranch>,
}

/// One optional eligibility flag and committed reward.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ProgressionObjective", module = "sand::advanced::compiler",
 summary = "One objective with optional eligibility and a reward.",
 context = "The shared runtime commits progress before calling the reward as the owner.",
 minecraft = "Uses the existing State guard and callback command model.",
 use_when = ["Adding one stage to a player progression"],
 avoid_when = ["Running work on every tick without advancing progression"],
 fields(when = "Optional score that must equal one.", reward = "Optional parameterless player function after commitment."),
 example = "use sand::advanced::compiler::ProgressionObjective;",
)]
pub struct ProgressionObjective {
    /// Optional score that must equal one to advance at this objective.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<FieldReference>,
    /// Optional committed reward, executed as the advancing player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reward: Option<FunctionReference>,
}

/// A conditional forward successor in a player progression.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ProgressionBranch", module = "sand::advanced::compiler",
 summary = "A forward successor chosen by an eligibility score.",
 context = "The first eligible branch at the current objective wins. Skipped objectives issue no reward.",
 minecraft = "Uses the shared Progression runtime's pre-commit condition checks.",
 use_when = ["Skipping optional objectives or routing a player based on State"],
 avoid_when = ["Replaying an already committed objective"],
 fields(from = "Zero-based source objective.", to = "Later objective index or total objective count.", when = "Score that must equal one."),
 example = "use sand::advanced::compiler::ProgressionBranch;",
)]
pub struct ProgressionBranch {
    /// Zero-based source objective.
    pub from: u32,
    /// A later index, including the total objective count for completion.
    pub to: u32,
    /// Score that must equal one to choose this successor.
    pub when: FieldReference,
}

/// Owned native block, item and text displays with keyframed local poses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::SceneDefinition", module = "sand::advanced::compiler",
 summary = "Owned native block, item and text displays with keyframed local poses.",
 context = "Each player start creates a fresh instance. Displays follow the root at its origin, or follow the owner when enabled. Each part has independent non-overlapping keyframes.",
 minecraft = "Uses shared Scene and Lifetime registration and native display interpolation.",
 use_when = ["Authoring bounded world-space visual presentations"],
 avoid_when = ["Visual geometry must collide with players"],
 fields(id = "Scene identity.", ticks = "Positive deadline in loaded ticks.", follow = "Follow the owner when enabled.", parts = "Named visual parts in authored order.", keyframes = "Optional interpolation keyframes for those parts."),
 example = "use sand::advanced::compiler::SceneDefinition;",
)]
pub struct SceneDefinition {
    /// Identity used to start or cancel this scene.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Maximum number of loaded ticks before cleanup.
    pub ticks: u32,
    /// Follow the player when true, otherwise remain where started.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub follow: bool,
    /// Displays in authored order.
    pub parts: Vec<ScenePart>,
    /// Optional animations for declared parts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keyframes: Vec<SceneKeyframe>,
}

/// A named visual display and its starting local pose.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ScenePart", module = "sand::advanced::compiler",
 summary = "A named visual display and its starting local pose.",
 context = "Each name is unique within its scene and can be referenced by its keyframes.",
 minecraft = "Lowers to a vanilla display entity created by the shared scene runtime.",
 use_when = ["Defining content and pose for an owned display"],
 avoid_when = ["The entity needs physical collision"],
 fields(name = "Unique name used by keyframes.", content = "Display payload.", pose = "Initial local translation, scale and yaw."),
 example = "use sand::advanced::compiler::ScenePart;",
)]
pub struct ScenePart {
    /// Name referenced by keyframes within this scene.
    pub name: String,
    /// Native display payload.
    pub content: SceneContent,
    /// Local pose, defaulting to identity.
    #[serde(default)]
    pub pose: crate::gameplay::Transform,
}

/// The three display payloads supported by the shared scene runtime.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::SceneContent", module = "sand::advanced::compiler",
 summary = "The three display payloads supported by the shared scene runtime.",
 context = "A display's content stays fixed while keyframes animate its pose. Custom items retain their authored components.",
 minecraft = "Uses native block, item and text display entities.",
 use_when = ["Choosing the payload of an owned display"],
 avoid_when = ["The entity needs physical collision"],
 variants(Block = "Display the default state of a block.", Item = "Display a vanilla or custom item.", Text = "Display rich text."),
 variant_fields(Block(block = "Typed default-state block."), Item(item = "Vanilla or custom item identity."), Text(text = "Rich text parts.")),
 example = "use sand::advanced::compiler::SceneContent;",
)]
pub enum SceneContent {
    /// Default state of a vanilla block.
    Block {
        /// Block to display.
        #[schemars(with = "String")]
        block: sand_components::BlockId,
    },
    /// One vanilla or custom item with its components.
    Item {
        /// Item resource identity.
        item: String,
    },
    /// Rich text, resolved when the display is created.
    Text {
        /// Text parts in display order.
        text: Vec<TextPart>,
    },
}

/// A positive interpolation interval and its target pose.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::SceneKeyframe", module = "sand::advanced::compiler",
 summary = "A positive interpolation interval and its target pose.",
 context = "A part's intervals may touch but must not overlap. A keyframe replaces the complete local pose.",
 minecraft = "The shared Scene runtime applies vanilla display interpolation at the requested loaded tick.",
 use_when = ["Animating a named native display"],
 avoid_when = ["The geometry must affect collision"],
 fields(part = "Existing part name.", at = "Positive start tick.", duration = "Positive interpolation length in ticks.", pose = "Target local pose."),
 example = "use sand::advanced::compiler::SceneKeyframe;",
)]
pub struct SceneKeyframe {
    /// Existing part name within this scene.
    pub part: String,
    /// Positive start tick, relative to instance creation.
    pub at: u32,
    /// Positive interpolation length.
    pub duration: u32,
    /// Complete target pose.
    pub pose: crate::gameplay::Transform,
}

/// Player-owned indexed work with a bounded number of callbacks per loaded tick.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::JobDefinition", module = "sand::advanced::compiler",
 summary = "Player-owned indexed work with a bounded number of callbacks per loaded tick.",
 context = "Each admitted job visits indices from zero through count minus one. A callback runs as the current owner with the index written to its declared global score slot. A completed job invokes complete once; expiry or cancellation discards unfinished work.",
 minecraft = "Uses the shared Job and Lifetime runtimes.",
 use_when = ["Running a finite batch of authored gameplay work across ticks"],
 avoid_when = ["Work must advance in unloaded chunks"],
 fields(id = "Job identity.", ticks = "Positive deadline in loaded ticks.", count = "Positive number of indexed steps.", per_tick = "Maximum step callbacks per instance per tick, default one.", capacity = "Maximum simultaneous instances across all owners.", index = "Global integer score slot receiving the current index immediately before step.", step = "Function invoked as the player for each index.", complete = "Optional function invoked as the player after the last step."),
 example = "use sand::advanced::compiler::JobDefinition;",
)]
pub struct JobDefinition {
    /// Identity used to start or cancel this job.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Loaded-tick deadline for each instance.
    pub ticks: u32,
    /// Number of steps, starting at index zero.
    pub count: u32,
    /// Maximum callbacks per instance per tick. Defaults to one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_tick: Option<u32>,
    /// Maximum simultaneous instances of this definition. Defaults to 128.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u32>,
    /// Writable global integer score slot receiving the zero-based index.
    pub index: FieldReference,
    /// Step callback invoked as the player.
    pub step: FunctionReference,
    /// Callback invoked as the player when the work completes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complete: Option<FunctionReference>,
}

/// An owned lifetime that drives a named player progression.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::SessionDefinition", module = "sand::advanced::compiler",
 summary = "An owned lifetime that drives a player progression.",
 context = "Each admitted session checks at most one objective on its cadence. Completion ends the session. Cancelling preserves progress.",
 minecraft = "Lowers through the existing Session, Progression and Lifetime systems.",
 use_when = ["Starting a bounded player quest or round controller"],
 avoid_when = ["Selecting teams or participants; author their State separately"],
 fields(id = "Session identity.", progression = "Track advanced by the session.", ticks = "Positive maximum lifetime in loaded ticks.", every = "Optional positive cadence in loaded ticks."),
 example = "use sand::advanced::compiler::SessionDefinition;",
)]
pub struct SessionDefinition {
    /// Identity used to address the player's session.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Named track advanced by this session.
    #[schemars(with = "String")]
    pub progression: ResourceLocation,
    /// Maximum lifetime in loaded ticks.
    pub ticks: u32,
    /// Positive loaded-tick cadence. Defaults to one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub every: Option<u32>,
}

/// A player-owned activation with optional State cost, cooldown, and projectile.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::AbilityDefinition", module = "sand::advanced::compiler",
 summary = "A player-owned activation with optional cost, cooldown and projectile.",
 context = "Admission checks canonical State and capacity before committing cost or cooldown. A start callback runs after both writes.",
 minecraft = "Uses the Sand Ability and Lifetime runtime, with optional shared projectile motion.",
 use_when = ["Building player spells or temporary abilities"],
 avoid_when = ["Charging a one-off inventory purchase"],
 fields(id = "Ability identity.", ticks = "Positive maximum loaded duration.", cost = "Optional State balance and amount.", cooldown = "Optional timer field and duration.", when = "Optional State score that must equal one.", start = "Optional parameterless player function called after admission.", projectile = "Optional projectile definition supplying flight and hit behavior."),
 example = "use sand::advanced::compiler::AbilityDefinition;",
)]
pub struct AbilityDefinition {
    /// Ability identity used by typed control operations.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Maximum loaded duration of an admitted instance.
    pub ticks: u32,
    /// Optional cost charged only when admission succeeds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<AbilityCost>,
    /// Optional countdown started only when admission succeeds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooldown: Option<StateCooldown>,
    /// Optional score that must equal one before admission.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<FieldReference>,
    /// Player callback after the canonical activation commits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<FunctionReference>,
    /// Optional flight and hit behavior, using this ability's lifetime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub projectile: Option<ResourceLocation>,
}

/// A charge against a player's non-ticking State score.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::AbilityCost", module = "sand::advanced::compiler",
 summary = "A charge against a non-ticking player State score.",
 context = "The charge is conditional on successful admission and preserves the field's minimum.",
 minecraft = "Uses the shared Ability admission and State commit.",
 use_when = ["Charging resources for an activated ability"],
 avoid_when = ["Writing an unconditional State mutation"],
 fields(balance = "Player State number holding the available resource.", amount = "Nonnegative resource amount charged."),
 example = "use sand::advanced::compiler::AbilityCost;",
)]
pub struct AbilityCost {
    /// Resource balance charged by this activation.
    pub balance: FieldReference,
    /// Amount removed on admission.
    pub amount: u32,
}

/// A duration written to a player's canonical State timer.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::StateCooldown", module = "sand::advanced::compiler",
 summary = "A duration written to a player State timer.",
 context = "Admission or a committed combat hit starts the timer. The State lifecycle owns its countdown.",
 minecraft = "Uses the canonical player timer lifecycle.",
 use_when = ["Setting ability cooldowns and combat invulnerability durations"],
 avoid_when = ["Timing an unrelated global task"],
 fields(timer = "Canonical player timer.", ticks = "Positive cooldown duration."),
 example = "use sand::advanced::compiler::StateCooldown;",
)]
pub struct StateCooldown {
    /// Player timer that counts down through the State lifecycle.
    pub timer: FieldReference,
    /// Positive ticks written on successful admission.
    pub ticks: u32,
}

/// Custom health rules backed by the executing player's State.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::CombatDefinition", module = "sand::advanced::compiler",
 summary = "A custom health policy over canonical player State.",
 context = "Combat damage applies configured mitigation and shields before committing health. Healing stops at the configured maximum and never revives zero health.",
 minecraft = "Uses the shared Combat transaction and its State guards, numeric writes and isolated callbacks.",
 use_when = ["Authoring custom player health, shields and recovery"],
 avoid_when = ["Damaging vanilla health directly"],
 fields(id = "Combat policy identity.", health = "Player State number storing authoritative health.", max = "Positive maximum health.", shield = "Optional player State number consumed before health.", resistance = "Optional damage fraction removed, from zero to one.", immunity = "Optional player timer and invulnerability duration after a hit.", critical = "Optional guard and multiplier for critical hits.", hit = "Optional callback after a committed hit.", defeat = "Optional callback after health reaches zero.", healed = "Optional callback after committed healing."),
 example = "use sand::advanced::compiler::CombatDefinition;",
)]
pub struct CombatDefinition {
    /// Identity used to apply this policy.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Canonical number holding authoritative health.
    pub health: FieldReference,
    /// Positive maximum health.
    pub max: f64,
    /// Optional shield used before health.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shield: Option<FieldReference>,
    /// Optional mitigation fraction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resistance: Option<f64>,
    /// Optional timer started by committed damage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub immunity: Option<StateCooldown>,
    /// Optional conditional critical-hit multiplier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub critical: Option<CombatCritical>,
    /// Optional callback after a committed hit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit: Option<FunctionReference>,
    /// Optional callback when a hit reaches zero health.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defeat: Option<FunctionReference>,
    /// Optional callback after accepted healing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub healed: Option<FunctionReference>,
}

/// A conditional critical-hit multiplier.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::CombatCritical", module = "sand::advanced::compiler",
 summary = "A conditional multiplier for a custom State hit.",
 context = "The multiplier applies when the target-context score equals one.",
 minecraft = "Uses the shared Combat calculation before shield absorption.",
 use_when = ["Authoring a conditional custom critical-hit bonus"],
 avoid_when = ["Applying native Minecraft weapon criticals"],
 fields(when = "Score that must equal one.", multiplier = "Finite multiplier of at least one."),
 example = "use sand::advanced::compiler::CombatCritical;",
)]
pub struct CombatCritical {
    /// Optional bonuses apply only while this score equals one.
    pub when: FieldReference,
    /// Finite damage multiplier of at least one.
    pub multiplier: f64,
}
