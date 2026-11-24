//! Owned portable data resources: custom items, recipes, and registry tags.
use super::{ScoreOperand, TextPart, Weather};
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
 minecraft = "Uses inventory_changed, placed_block, or player_killed_entity.",
 use_when = ["Composing native advancement criteria"],
 avoid_when = ["Executing an arbitrary script as a criterion"],
 variants(InventoryItem = "Acquire an item.", PlaceBlock = "Place the selected block.", KillEntity = "Kill an entity of the selected type."),
 variant_fields(InventoryItem(item = "Vanilla item identity."), PlaceBlock(block = "Vanilla block identity."), KillEntity(entity = "Vanilla entity type identity.")),
 example = "use sand::advanced::compiler::AdvancementCriterion;",
)]
pub enum AdvancementCriterion {
    /// Have an item on a matching inventory change.
    InventoryItem { item: String },
    /// Place a block of the given type.
    PlaceBlock { block: String },
    /// Receive player kill credit for an entity of the given type.
    KillEntity { entity: String },
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

/// Climate and visual settings for a native custom biome.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::CustomBiome", module = "sand::advanced::compiler",
 summary = "Climate and visual settings for a native custom biome.",
 context = "Reference the biome from FlatDimension or a block-position biome condition. Install the pack before starting the world to register the biome.",
 minecraft = "Lowers through Biome and BiomeEffects to a native worldgen biome, with no placed features, carvers or spawn tables.",
 use_when = ["Giving an authored dimension its own climate and colors"],
 avoid_when = ["Declaring placed features or natural mob spawn tables"],
 fields(id = "Biome identity.", precipitation = "Whether rain or snow occurs.", temperature = "Climate temperature.", downfall = "Humidity from zero to one.", fog_color = "Fog RGB.", water_color = "Water surface RGB.", water_fog_color = "Underwater fog RGB.", sky_color = "Sky RGB.", grass_color = "Grass RGB override.", foliage_color = "Foliage RGB override."),
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
