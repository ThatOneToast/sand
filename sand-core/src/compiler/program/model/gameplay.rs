//! Owned portable gameplay vocabulary: arithmetic, conditions, text, and events.
use super::{FieldReference, FunctionReference, ScoreComparison};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A vanilla scoreboard arithmetic operator applied to a State field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ScoreOperator", module = "sand::advanced::compiler",
 summary = "A vanilla scoreboard arithmetic operator applied to a State field.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Lowers to `scoreboard players operation`, which uses Minecraft's wrapping i32 arithmetic and floor division.",
 use_when = ["Combining a State field with another field or a constant in a portable program"],
 avoid_when = ["Adding a constant; ScoreAdd emits a single cheaper command"],
 example = "let op = sand::advanced::compiler::ScoreOperator::Mul;",
 variants(Set = "Copy the operand into the field.", Add = "Add the operand.", Sub = "Subtract the operand.", Mul = "Multiply by the operand.", Div = "Floor-divide by the operand; division by zero leaves the field unchanged.", Mod = "Floor modulo by the operand; modulo zero leaves the field unchanged.", Min = "Keep the smaller of the field and the operand.", Max = "Keep the larger of the field and the operand."),
)]
pub enum ScoreOperator {
    /// Copy the operand into the field.
    Set,
    /// Add the operand.
    Add,
    /// Subtract the operand.
    Sub,
    /// Multiply by the operand.
    Mul,
    /// Floor-divide by the operand; dividing by zero leaves the field unchanged.
    Div,
    /// Floor modulo by the operand; modulo zero leaves the field unchanged.
    Mod,
    /// Keep the smaller of the field and the operand.
    Min,
    /// Keep the larger of the field and the operand.
    Max,
}

/// The right-hand side of a score operation: another field or a constant.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ScoreOperand", module = "sand::advanced::compiler",
 summary = "The right-hand side of a score operation: another field or a constant.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Fields are read from their declared player or global holder; constants use a compiler-owned constant holder.",
 use_when = ["Supplying the operand of a ScoreOperation"],
 avoid_when = ["Writing a literal with ScoreSet or ScoreAdd"],
 example = "let operand = sand::advanced::compiler::ScoreOperand::Value(2);",
 variants(Field = "Read another State field.", Value = "Use an integer constant."),
 variant_fields(Field = ["The State field read as the operand."], Value = ["The constant operand."]),
)]
pub enum ScoreOperand {
    /// Read another State field.
    Field(FieldReference),
    /// Use an integer constant.
    Value(i32),
}

/// How two State fields relate in a branch condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Relation", module = "sand::advanced::compiler",
 summary = "How two State fields relate in a branch condition.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Lowers to `execute if score <left> <relation> <right>`.",
 use_when = ["Comparing two State fields"],
 avoid_when = ["Comparing a field with constants; use a ScoreComparison range"],
 example = "let relation = sand::advanced::compiler::Relation::Lt;",
 variants(Lt = "Left is less than right.", Le = "Left is at most right.", Eq = "Both fields are equal.", Ge = "Left is at least right.", Gt = "Left is greater than right."),
)]
pub enum Relation {
    /// Left is less than right.
    Lt,
    /// Left is at most right.
    Le,
    /// Both fields are equal.
    Eq,
    /// Left is at least right.
    Ge,
    /// Left is greater than right.
    Gt,
}

/// A comparison between two State fields.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ScoreRelation", module = "sand::advanced::compiler",
 summary = "A comparison between two State fields.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Both fields are read from their declared holders; absent scores make the comparison false.",
 use_when = ["Branching on how two State fields relate"],
 avoid_when = ["Testing one field against constant bounds"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(left = "The left-hand State field.", relation = "How the fields must relate.", right = "The right-hand State field."),
)]
pub struct ScoreRelation {
    /// The left-hand State field.
    pub left: FieldReference,
    /// How the fields must relate.
    pub relation: Relation,
    /// The right-hand State field.
    pub right: FieldReference,
}

/// A branch condition: a constant range test or a field-to-field relation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Condition", module = "sand::advanced::compiler",
 summary = "A branch condition: a constant range test or a field-to-field relation.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Evaluated once by `execute if score` before exactly one branch body runs.",
 use_when = ["Choosing between two ordered bodies at runtime"],
 avoid_when = ["Flow transition guards, which accept only ranges"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(Range = "The field lies within inclusive constant bounds.", Relation = "Two fields satisfy a relation.", Attached = "The executor has an entity State attached."),
 variant_fields(Range = ["The range test."], Relation = ["The field relation."], Attached = ["The attached State."]),
)]
pub enum Condition {
    /// The field lies within inclusive constant bounds.
    Range(ScoreComparison),
    /// Two fields satisfy a relation.
    Relation(ScoreRelation),
    /// The executor has an entity State attached.
    Attached(StateAttached),
}

/// A test that the executor has an entity State attached.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::StateAttached", module = "sand::advanced::compiler",
 summary = "A test that the executor has an entity State attached.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Matches when the State's presence score equals its revision, as Rust's attached-State conditions do.",
 use_when = ["Branching on whether an entity carries a State"],
 avoid_when = ["Player or global State, which is always present"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(attached = "The entity State."),
)]
pub struct StateAttached {
    /// The entity State.
    #[schemars(with = "String")]
    pub attached: sand_components::ResourceLocation,
}

impl From<ScoreComparison> for Condition {
    fn from(range: ScoreComparison) -> Self {
        Self::Range(range)
    }
}

/// Who receives a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Audience", module = "sand::advanced::compiler",
 summary = "Who receives a message.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "`player` targets `@s` and needs a player executor; `everyone` targets `@a`.",
 use_when = ["Choosing the recipients of a Message"],
 avoid_when = ["Targeting arbitrary entities; messages are player-facing"],
 example = "let audience = sand::advanced::compiler::Audience::Everyone;",
 variants(Player = "Only the executing player.", Everyone = "Every online player."),
)]
pub enum Audience {
    /// Only the executing player.
    Player,
    /// Every online player.
    Everyone,
}

/// Where a message appears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::MessageChannel", module = "sand::advanced::compiler",
 summary = "Where a message appears.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Lowers to `tellraw` or the matching `title` sub-command.",
 use_when = ["Choosing how a Message is displayed"],
 avoid_when = ["Persistent HUD elements such as boss bars"],
 example = "let channel = sand::advanced::compiler::MessageChannel::Actionbar;",
 variants(Chat = "A chat line.", Actionbar = "Text above the hotbar.", Title = "Large centred title text.", Subtitle = "Text under the title; shown with the next title."),
)]
pub enum MessageChannel {
    /// A chat line.
    Chat,
    /// Text above the hotbar.
    Actionbar,
    /// Large centred title text.
    Title,
    /// Text under the title; shown with the next title.
    Subtitle,
}

/// One of Minecraft's sixteen named text colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::TextColor", module = "sand::advanced::compiler",
 summary = "One of Minecraft's sixteen named text colours.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Serialized as the text component `color` name.",
 use_when = ["Colouring part of a Message"],
 avoid_when = ["Custom hex colours, which portable programs do not yet support"],
 example = "let color = sand::advanced::compiler::TextColor::Gold;",
 variants(Black = "black", DarkBlue = "dark_blue", DarkGreen = "dark_green", DarkAqua = "dark_aqua", DarkRed = "dark_red", DarkPurple = "dark_purple", Gold = "gold", Gray = "gray", DarkGray = "dark_gray", Blue = "blue", Green = "green", Aqua = "aqua", Red = "red", LightPurple = "light_purple", Yellow = "yellow", White = "white"),
)]
#[allow(missing_docs)]
pub enum TextColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

/// Visual styling shared by every text part.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::TextStyle", module = "sand::advanced::compiler",
 summary = "Visual styling shared by every text part.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Serialized as text component style properties; unset properties inherit from the previous part.",
 use_when = ["Styling part of a Message"],
 avoid_when = ["Interactive click or hover behaviour, which portable programs do not yet support"],
 example = "let style = sand::advanced::compiler::TextStyle::default();",
 fields(color = "Named text colour.", bold = "Bold text.", italic = "Italic text.", underlined = "Underlined text."),
)]
pub struct TextStyle {
    /// Named text colour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<TextColor>,
    /// Bold text.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bold: bool,
    /// Italic text.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub italic: bool,
    /// Underlined text.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub underlined: bool,
}

/// One styled piece of a message.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::TextPart", module = "sand::advanced::compiler",
 summary = "One styled piece of a message.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Each part becomes one text component; scores and names resolve when the message is shown.",
 use_when = ["Building Message text from literals, live scores, and player names"],
 avoid_when = ["Static resource text such as item names"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(Text = "Literal text.", Score = "The live value of a State field.", PlayerName = "The executing player's name."),
 variant_fields(Text(text = "Literal text.", style = "Styling."), Score(score = "Displayed State field.", style = "Styling."), PlayerName(style = "Styling.")),
)]
pub enum TextPart {
    /// Literal text.
    Text {
        text: String,
        #[serde(default)]
        style: TextStyle,
    },
    /// The live value of a State field.
    Score {
        score: FieldReference,
        #[serde(default)]
        style: TextStyle,
    },
    /// The executing player's name; needs a player executor.
    PlayerName {
        #[serde(default)]
        style: TextStyle,
    },
}

/// A player occurrence that runs handler functions as that player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::PlayerEvent", module = "sand::advanced::compiler",
 summary = "A player occurrence that runs handler functions as that player.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Observed each tick through namespace-owned scoreboard criteria or tags, so other packs cannot consume the observation.",
 use_when = ["Reacting to joins, deaths, kills, or item use in a portable program"],
 avoid_when = ["Occurrences not listed here, which use the Rust event API"],
 example = "let event = sand::advanced::compiler::PlayerEvent::Death;",
 variants(Join = "Every join, including the first.", FirstJoin = "Only the first join ever.", Death = "The player died.", KillEntity = "The player killed any entity.", KillPlayer = "The player killed another player.", UseItem = "The player used an item.", Jump = "The player jumped.", DropItem = "The player dropped an item.", Sleep = "The player slept in a bed.", TakeDamage = "The player took damage.", DealDamage = "The player dealt damage.", CraftItem = "The player crafted an item.", PickUpItem = "The player picked up an item.", MineBlock = "The player mined a block.", KillEntityType = "The player killed an entity of one type.", KilledBy = "An entity of one type killed the player."),
 variant_fields(UseItem(item = "Used item identity, such as `minecraft:carrot_on_a_stick`."), CraftItem(item = "Crafted item identity."), PickUpItem(item = "Picked-up item identity."), MineBlock(block = "Mined block identity."), KillEntityType(entity = "Killed entity type identity."), KilledBy(entity = "Killer entity type identity.")),
)]
pub enum PlayerEvent {
    /// Every join, including the first; observed on the next tick.
    Join,
    /// Only the first time a player ever joins.
    FirstJoin,
    /// The player died (any cause).
    Death,
    /// The player killed any entity.
    KillEntity,
    /// The player killed another player.
    KillPlayer,
    /// The player used (right-clicked or consumed) an item.
    UseItem {
        /// Used item identity, such as `minecraft:carrot_on_a_stick`.
        item: String,
    },
    /// The player jumped.
    Jump,
    /// The player dropped an item.
    DropItem,
    /// The player slept in a bed.
    Sleep,
    /// The player took damage.
    TakeDamage,
    /// The player dealt damage.
    DealDamage,
    /// The player crafted an item.
    CraftItem {
        /// Crafted item identity.
        item: String,
    },
    /// The player picked up an item.
    PickUpItem {
        /// Picked-up item identity.
        item: String,
    },
    /// The player mined a block.
    MineBlock {
        /// Mined block identity.
        block: String,
    },
    /// The player killed an entity of one type.
    KillEntityType {
        /// Killed entity type identity.
        entity: String,
    },
    /// An entity of one type killed the player.
    KilledBy {
        /// Killer entity type identity.
        entity: String,
    },
}

impl PlayerEvent {
    /// The identity this event filters on, with its kind, if any.
    pub(crate) fn filter(&self) -> Option<(&str, &'static str)> {
        match self {
            PlayerEvent::UseItem { item }
            | PlayerEvent::CraftItem { item }
            | PlayerEvent::PickUpItem { item } => Some((item, "item")),
            PlayerEvent::MineBlock { block } => Some((block, "block")),
            PlayerEvent::KillEntityType { entity } | PlayerEvent::KilledBy { entity } => {
                Some((entity, "entity type"))
            }
            _ => None,
        }
    }

    /// A stable, path-safe key that groups identical observations.
    pub(crate) fn key(&self) -> String {
        let base = match self {
            PlayerEvent::Join => "join",
            PlayerEvent::FirstJoin => "first_join",
            PlayerEvent::Death => "death",
            PlayerEvent::KillEntity => "kill_entity",
            PlayerEvent::KillPlayer => "kill_player",
            PlayerEvent::UseItem { .. } => "use_item",
            PlayerEvent::Jump => "jump",
            PlayerEvent::DropItem => "drop_item",
            PlayerEvent::Sleep => "sleep",
            PlayerEvent::TakeDamage => "take_damage",
            PlayerEvent::DealDamage => "deal_damage",
            PlayerEvent::CraftItem { .. } => "craft_item",
            PlayerEvent::PickUpItem { .. } => "pick_up_item",
            PlayerEvent::MineBlock { .. } => "mine_block",
            PlayerEvent::KillEntityType { .. } => "kill_entity_type",
            PlayerEvent::KilledBy { .. } => "killed_by",
        };
        match self.filter() {
            Some((id, _)) => format!("{base}/{}", id.replace(':', "/")),
            None => base.to_owned(),
        }
    }

    /// The scoreboard criterion that observes this event, when one does.
    pub(crate) fn criterion(&self) -> Option<String> {
        let stat = |kind: &str, id: &str| format!("minecraft.{kind}:{}", id.replace(':', "."));
        Some(match self {
            PlayerEvent::Join => "minecraft.custom:minecraft.leave_game".into(),
            PlayerEvent::FirstJoin => return None,
            PlayerEvent::Death => "deathCount".into(),
            PlayerEvent::KillEntity => "totalKillCount".into(),
            PlayerEvent::KillPlayer => "playerKillCount".into(),
            PlayerEvent::UseItem { item } => stat("used", item),
            PlayerEvent::Jump => "minecraft.custom:minecraft.jump".into(),
            PlayerEvent::DropItem => "minecraft.custom:minecraft.drop".into(),
            PlayerEvent::Sleep => "minecraft.custom:minecraft.sleep_in_bed".into(),
            PlayerEvent::TakeDamage => "minecraft.custom:minecraft.damage_taken".into(),
            PlayerEvent::DealDamage => "minecraft.custom:minecraft.damage_dealt".into(),
            PlayerEvent::CraftItem { item } => stat("crafted", item),
            PlayerEvent::PickUpItem { item } => stat("picked_up", item),
            PlayerEvent::MineBlock { block } => stat("mined", block),
            PlayerEvent::KillEntityType { entity } => stat("killed", entity),
            PlayerEvent::KilledBy { entity } => stat("killed_by", entity),
        })
    }
}

/// A handler for a player event.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::EventHook", module = "sand::advanced::compiler",
 summary = "A handler for a player event.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Each tick, after State maintenance and flows, every observed player runs its handlers in declaration order.",
 use_when = ["Running a function when a player event occurs"],
 avoid_when = ["Periodic work; use module tick bodies"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(event = "Observed occurrence.", function = "Handler, run as the player."),
)]
pub struct EventHook {
    /// Observed occurrence.
    pub event: PlayerEvent,
    /// Handler, run as the player.
    pub function: FunctionReference,
}

/// A number supplied to a command: a constant or a scaled State field.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Amount", module = "sand::advanced::compiler",
 summary = "A number supplied to a command: a constant or a scaled State field.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Field amounts are copied into command storage and applied through a generated macro function.",
 use_when = ["Passing a runtime value to a command argument"],
 avoid_when = ["Integer arithmetic between fields; use ScoreOperation"],
 example = "let amount = sand::advanced::compiler::Amount::Value(20.0);",
 variants(Value = "A constant number.", Field = "A State field divided by its scale."),
 variant_fields(Value = ["The constant number."], Field = ["The scaled field."]),
)]
pub enum Amount {
    /// A constant number.
    Value(f64),
    /// A State field divided by its scale.
    Field(ScaledField),
}

/// A State field read as `value / scale`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ScaledField", module = "sand::advanced::compiler",
 summary = "A State field read as `value / scale`.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Copied into command storage with `execute store result storage ... double <1/scale>`.",
 use_when = ["Supplying a decimal command argument from an integer score"],
 avoid_when = ["Constant amounts"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(score = "The State field.", scale = "Divisor; a score of 25 with scale 10 means 2.5."),
)]
pub struct ScaledField {
    /// The State field.
    pub score: FieldReference,
    /// Divisor; a score of 25 with scale 10 means 2.5.
    #[serde(default = "one")]
    pub scale: f64,
}

fn one() -> f64 {
    1.0
}

/// How matching entities are ordered before a limit applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::EntitySort", module = "sand::advanced::compiler",
 summary = "How matching entities are ordered before a limit applies.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Serialized as the target selector `sort` argument.",
 use_when = ["Choosing which entities a limited query selects"],
 avoid_when = ["Unlimited queries, where order rarely matters"],
 example = "let sort = sand::advanced::compiler::EntitySort::Nearest;",
 variants(Nearest = "Nearest to the executor first.", Furthest = "Furthest from the executor first.", Random = "Random order.", Arbitrary = "Minecraft's internal order; fastest."),
)]
pub enum EntitySort {
    /// Nearest to the executor first.
    Nearest,
    /// Furthest from the executor first.
    Furthest,
    /// Random order.
    Random,
    /// Minecraft's internal order; fastest.
    Arbitrary,
}

/// Which entities an operation selects.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::EntityQuery", module = "sand::advanced::compiler",
 summary = "Which entities an operation selects.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Lowers to an `@e[...]` target selector; distances are measured from the executor's position.",
 use_when = ["Running work as, or counting, entities that match filters"],
 avoid_when = ["Iterating players; use Players"],
 example = "let query = sand::advanced::compiler::EntityQuery::default();",
 fields(entity_type = "Entity type identity, such as `minecraft:zombie`.", tag = "Required entity tag.", with_state = "Only entities with this entity State attached.", min_distance = "Inclusive minimum distance in blocks.", max_distance = "Inclusive maximum distance in blocks.", limit = "Most entities selected.", sort = "Order before the limit applies."),
)]
pub struct EntityQuery {
    /// Entity type identity, such as `minecraft:zombie`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    /// Required entity tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Only entities with this entity State attached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub with_state: Option<sand_components::ResourceLocation>,
    /// Inclusive minimum distance in blocks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_distance: Option<f64>,
    /// Inclusive maximum distance in blocks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_distance: Option<f64>,
    /// Most entities selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Order before the limit applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<EntitySort>,
}

/// Where a teleport moves the executor.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::TeleportDestination", module = "sand::advanced::compiler",
 summary = "Where a teleport moves the executor.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Lowers to `tp @s` with absolute coordinates, relative coordinates, or one selected entity.",
 use_when = ["Moving the executing player or entity"],
 avoid_when = ["Spreading many entities at once"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(Position = "Absolute coordinates.", Offset = "Coordinates relative to the executor.", Entity = "The first entity matching a query."),
 variant_fields(Position(x = "X coordinate.", y = "Y coordinate.", z = "Z coordinate."), Offset(x = "X offset.", y = "Y offset.", z = "Z offset."), Entity(query = "Destination entity; limited to one, nearest by default.")),
)]
pub enum TeleportDestination {
    /// Absolute coordinates.
    Position { x: f64, y: f64, z: f64 },
    /// Coordinates relative to the executor.
    Offset { x: f64, y: f64, z: f64 },
    /// The first entity matching a query.
    Entity { query: EntityQuery },
}
