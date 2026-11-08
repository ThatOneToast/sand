//! Owned portable operation definitions.
use super::{
    Amount, Audience, Condition, EntityQuery, FieldReference, FunctionReference, MessageChannel,
    ScoreOperand, ScoreOperator, TeleportDestination, TextPart,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A gameplay operation with optional host-provided source provenance.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Operation", module = "sand::advanced::compiler",
 summary = "A gameplay operation with optional host-provided source provenance.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(action = "Action for this portable definition.", origin = "Optional host source provenance, excluded from generated identities."),
)]
pub struct Operation {
    /// Typed operation performed in the current executor context.
    pub action: Action,
    /// DSL span or editor node/port metadata; never part of resource identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<serde_json::Value>,
}

/// Supported typed gameplay operations. Raw commands require explicit opt-in.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Action", module = "sand::advanced::compiler",
 summary = "Supported typed gameplay operations. Raw commands require explicit opt-in.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(ScoreSet = "ScoreSet semantic alternative; see Rustdoc for its execution behavior.", ScoreAdd = "ScoreAdd semantic alternative; see Rustdoc for its execution behavior.", Branch = "Branch semantic alternative; see Rustdoc for its execution behavior.", Players = "Players semantic alternative; see Rustdoc for its execution behavior.", Call = "Call semantic alternative; see Rustdoc for its execution behavior.", Raw = "Raw semantic alternative; see Rustdoc for its execution behavior.", ScoreOperation = "Combine a field with another field or a constant.", ScoreRandom = "Store a uniformly random integer.", Message = "Show styled text.", Sound = "Play a sound to the executing player.", Particle = "Spawn particles at the executor.", Effect = "Apply a status effect to the executing player.", ClearEffect = "Remove status effects from the executing player.", GiveItem = "Give items to the executing player.", ClearItem = "Remove items from the executing player.", Schedule = "Run a server function after a delay.", Return = "End the current function, or the current player's turn inside Players.", While = "Repeat a body while a condition holds.", AttributeGet = "Store an attribute base value in a State field.", AttributeSet = "Set the executing player's attribute base value.", Entities = "Run a body as each matching entity, at its position.", Kill = "Kill the executing entity.", EntityTag = "Add or remove a tag on the executing entity.", Summon = "Summon an entity at the executor's position.", Damage = "Damage the executing entity.", CountEntities = "Store how many entities match.", Teleport = "Move the executing entity.", AttachState = "Attach an entity State to the executor.", DetachState = "Detach an entity State from the executor."),
 variant_fields(ScoreSet(score = "Score operand for this operation.", value = "Value operand for this operation."), ScoreAdd(score = "Score operand for this operation.", value = "Value operand for this operation."), Branch(condition = "Condition operand for this operation.", then = "Then operand for this operation.", otherwise = "Otherwise operand for this operation."), Players(body = "Body operand for this operation."), Call(function = "Function operand for this operation."), Raw(command = "Command operand for this operation."), ScoreOperation(score = "Written State field.", operator = "Arithmetic operator.", operand = "Field or constant operand."), ScoreRandom(score = "Written State field.", min = "Inclusive minimum.", max = "Inclusive maximum."), Message(audience = "Recipients.", channel = "Display channel.", text = "Styled text parts."), Sound(sound = "Sound event identity.", volume = "Volume; 1.0 is normal.", pitch = "Pitch from 0.5 to 2.0."), Particle(particle = "Particle type identity.", count = "Particle count.", spread = "Random offset in blocks on each axis.", speed = "Particle speed."), Effect(effect = "Status effect identity.", seconds = "Duration in seconds; omitted means infinite.", amplifier = "Level minus one.", hide_particles = "Hide effect particles."), ClearEffect(effect = "Effect to remove; omitted removes all."), GiveItem(item = "Item identity.", count = "Item count."), ClearItem(item = "Item to remove; omitted removes all.", count = "Maximum count; omitted removes every match."), Schedule(function = "Server-context function to run.", ticks = "Delay in game ticks; at least one."), While(condition = "Checked before every pass.", body = "Repeated body."), AttributeGet(score = "Written State field.", attribute = "Attribute identity.", scale = "Multiplier applied before truncation."), AttributeSet(attribute = "Attribute identity.", value = "Constant or State-derived value."), Entities(query = "Selected entities.", body = "Run as each entity."), EntityTag(tag = "Tag name.", add = "Add when true, remove when false."), Summon(entity_type = "Entity type identity.", tag = "Tag given to the new entity."), Damage(amount = "Damage in half-hearts."), CountEntities(score = "Written State field.", query = "Counted entities."), Teleport(destination = "Where the executor moves."), AttachState(state = "Entity State to attach."), DetachState(state = "Entity State to detach.")),
)]
pub enum Action {
    /// Assign an integer State field using its declared ownership.
    ScoreSet { score: FieldReference, value: i32 },
    /// Add an integer to a State field using its declared ownership.
    ScoreAdd { score: FieldReference, value: i32 },
    /// Evaluate the comparison once and execute exactly one ordered body.
    Branch {
        condition: Condition,
        then: Vec<Operation>,
        #[serde(default)]
        otherwise: Vec<Operation>,
    },
    /// Execute the body as each player without changing positional context.
    Players { body: Vec<Operation> },
    /// Invoke a named function with a checked execution-context requirement.
    Call { function: FunctionReference },
    /// Explicit escape hatch; only command-line safety and syntax are checked.
    Raw { command: String },
    /// Combine a field with another field or a constant using scoreboard
    /// arithmetic, then apply the field's declared bounds.
    ScoreOperation {
        score: FieldReference,
        operator: ScoreOperator,
        operand: ScoreOperand,
    },
    /// Store a uniformly random integer from the inclusive range.
    ScoreRandom {
        score: FieldReference,
        min: i32,
        max: i32,
    },
    /// Show styled text; a `player` audience needs a player executor.
    Message {
        audience: Audience,
        channel: MessageChannel,
        text: Vec<TextPart>,
    },
    /// Play a sound to the executing player at their position.
    Sound {
        sound: String,
        #[serde(default = "one")]
        volume: f32,
        #[serde(default = "one")]
        pitch: f32,
    },
    /// Spawn particles at the executor's position.
    Particle {
        particle: String,
        #[serde(default = "one_count")]
        count: u32,
        #[serde(default)]
        spread: f32,
        #[serde(default)]
        speed: f32,
    },
    /// Apply a status effect to the executing player.
    Effect {
        effect: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        seconds: Option<u32>,
        #[serde(default)]
        amplifier: u8,
        #[serde(default)]
        hide_particles: bool,
    },
    /// Remove one status effect, or all of them, from the executing player.
    ClearEffect {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effect: Option<String>,
    },
    /// Give items to the executing player.
    GiveItem {
        item: String,
        #[serde(default = "one_count")]
        count: u32,
    },
    /// Remove matching items, or every item, from the executing player.
    ClearItem {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        item: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        count: Option<u32>,
    },
    /// Run a server-context function after a delay, replacing any pending
    /// schedule of the same function.
    Schedule {
        function: FunctionReference,
        ticks: u32,
    },
    /// End the current function. Inside a `Players` body it ends only the
    /// current player's turn, like `continue`.
    Return,
    /// Check the condition before each pass and repeat the body while it holds.
    /// Minecraft's command-chain limit bounds runaway loops.
    While {
        condition: Condition,
        body: Vec<Operation>,
    },
    /// Store the executing player's attribute base value, multiplied by
    /// `scale` and truncated, in a State field.
    AttributeGet {
        score: FieldReference,
        attribute: String,
        #[serde(default = "one_f64")]
        scale: f64,
    },
    /// Set the executing player's attribute base value to a constant or a
    /// State-derived amount.
    AttributeSet { attribute: String, value: Amount },
    /// Run the body as each matching entity, positioned at it. A `Return`
    /// inside ends only the current entity's turn.
    Entities {
        query: EntityQuery,
        body: Vec<Operation>,
    },
    /// Kill the executing entity.
    Kill,
    /// Add or remove a tag on the executing entity.
    EntityTag { tag: String, add: bool },
    /// Summon an entity at the executor's position.
    Summon {
        entity_type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
    },
    /// Damage the executing entity, in half-hearts.
    Damage { amount: f32 },
    /// Store how many entities match the query.
    CountEntities {
        score: FieldReference,
        query: EntityQuery,
    },
    /// Move the executing entity.
    Teleport { destination: TeleportDestination },
    /// Attach an entity State to the executor: fields without a value take
    /// their defaults, then the State is marked present.
    AttachState {
        #[schemars(with = "String")]
        state: sand_components::ResourceLocation,
    },
    /// Detach an entity State from the executor, clearing its fields.
    DetachState {
        #[schemars(with = "String")]
        state: sand_components::ResourceLocation,
    },
}

fn one_f64() -> f64 {
    1.0
}

fn one() -> f32 {
    1.0
}

fn one_count() -> u32 {
    1
}
