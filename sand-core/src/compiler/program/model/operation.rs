//! Owned portable operation definitions.
use super::{
    Amount, Audience, BlockDataAddress, BlockPosition, Condition, DataOperand, DataReference,
    EntityData, EntityQuery, FieldReference, FunctionReference, MessageChannel, PlayerGameMode,
    ScoreOperand, ScoreOperator, TeleportDestination, TextPart, Weather,
};
use sand_components::ResourceLocation;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::entity::Relation;

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
 variants(ScoreSet = "ScoreSet semantic alternative; see Rustdoc for its execution behavior.", ScoreAdd = "ScoreAdd semantic alternative; see Rustdoc for its execution behavior.", Branch = "Branch semantic alternative; see Rustdoc for its execution behavior.", Players = "Players semantic alternative; see Rustdoc for its execution behavior.", Call = "Call semantic alternative; see Rustdoc for its execution behavior.", Raw = "Raw semantic alternative; see Rustdoc for its execution behavior.", ScoreOperation = "Combine a field with another field or a constant.", ScoreRandom = "Store a uniformly random integer.", Message = "Show styled text.", Dialog = "Show or clear a native dialog for the executing player.", Sound = "Play a sound to the executing player.", Particle = "Spawn particles at the executor.", Effect = "Apply a status effect to the executing player.", ClearEffect = "Remove status effects from the executing player.", GiveItem = "Give items to the executing player.", SetEquipment = "Set or clear one equipment slot of the entity executor.", ClearItem = "Remove items from the executing player.", CountItems = "Count matching items in the executing player’s inventory.", TransferInventory = "Run a declared inventory transfer as the executing player.", Checkpoint = "Save, restore, or invalidate a player checkpoint.", Archetype = "Spawn, attach or release one entity archetype.", Status = "Apply or remove a player-owned timed status.", Roster = "Operate on a declared online roster.", Encounter = "Start or cancel a player encounter.", Hud = "Start or cancel a player-owned HUD.", Combat = "Apply damage or healing using a named State combat policy.", Ability = "Activate, cancel, or diagnose a player ability.", Progression = "Advance or reset the executing player’s named progression.", Session = "Control the executing player’s named session.", Job = "Start or cancel the executing player’s named job.", Scene = "Show or cancel an owned display scene.", Projectile = "Launch or cancel the executing player’s named projectile.", Menu = "Open or close a named native menu.", Schedule = "Run a server function after a delay.", Suspend = "Resume a continuation later with its own invocation fields and executor.", CancelWaits = "Cancel outstanding waits owned by the executing entity.", Return = "End the current function, or the current player's turn inside Players.", While = "Repeat a body while a condition holds.", AttributeGet = "Store an attribute base value of the executor in a State field.", AttributeSet = "Set the executor's attribute base value.", Entities = "Run a body as each matching entity, at its position.", Kill = "Kill the executing entity.", Ride = "Mount the executor on a retained non-player entity, or dismount it.", EntityTag = "Add or remove a tag on the executing entity.", Summon = "Summon an entity at the executor's position.", CaptureEntity = "Retain the executor or one related entity in a State field.", WithEntity = "Run as a retained entity when present.", Damage = "Damage the executing entity.", CountEntities = "Store how many entities match.", Teleport = "Move the executing entity.", SetVelocity = "Set the executing non-player entity’s velocity.", AttachState = "Attach an entity State to the executor.", DetachState = "Detach an entity State from the executor.", SetBlock = "Place a block.", PlaceTemplate = "Place a native structure template.", FillBlocks = "Fill a region with a block.", CloneBlocks = "Copy or move an inclusive block region.", InDimension = "Change the execution dimension for a body.", ReadBlockData = "Read a block entity value as an integer.", ReadEntityData = "Read an entity NBT path as an integer.", WriteBlockData = "Set or clear a path in a block entity.", AddExperience = "Give the executing player experience.", SetGameMode = "Change the executing player's game mode.", SetTime = "Set the time of day.", SetWeather = "Set the weather.", BossBarAudience = "Show or hide a boss bar.", SetBossBar = "Set a boss bar value or maximum.", GiveLoot = "Give the executing player a loot roll.", DropLoot = "Drop a loot roll at the executor.", PushFrame = "Save State fields before a call that may re-enter this function.", PopFrame = "Restore the State fields saved by the matching PushFrame.", DataGet = "Store a number read from the executor's entity data.", DataModify = "Modify an owned storage value.", DataRemove = "Remove a stored value selection.", DataRead = "Read an integer or collection length into a score.", DataEqual = "Compare stored values for deep equality."),
 variant_fields(Archetype(archetype = "The declared archetype identity.", command = "The lifecycle operation."), Status(status = "Declared status identity.", apply = "Apply or refresh when true, remove this owner’s stacks otherwise."), Roster(roster = "Roster identity.", command = "Membership operation."), Encounter(encounter = "Declared encounter identity.", start = "Start or replace when true; cancel otherwise."), Checkpoint(checkpoint = "Declared checkpoint identity.", command = "Save, restore, or clear."), ScoreSet(score = "Score operand for this operation.", value = "Value operand for this operation."), ScoreAdd(score = "Score operand for this operation.", value = "Value operand for this operation."), Branch(condition = "Condition operand for this operation.", then = "Then operand for this operation.", otherwise = "Otherwise operand for this operation."), Players(body = "Body operand for this operation."), Call(function = "Function operand for this operation."), Raw(command = "Command operand for this operation."), ScoreOperation(score = "Written State field.", operator = "Arithmetic operator.", operand = "Field or constant operand."), ScoreRandom(score = "Written State field.", min = "Inclusive minimum.", max = "Inclusive maximum."), Message(audience = "Recipients.", channel = "Display channel.", text = "Styled text parts."), Dialog(dialog = "Dialog identity to show, or omitted to close the current dialog."), Sound(sound = "Sound event identity.", volume = "Volume; 1.0 is normal.", pitch = "Pitch from 0.5 to 2.0."), Particle(particle = "Particle type identity.", count = "Constant or runtime particle count, from 0 to 100000.", spread = "Constant or runtime offset in blocks on each axis, from 0 to 1000.", speed = "Constant or runtime speed, from 0 to 1000."), Effect(effect = "Status effect identity.", seconds = "Duration in seconds; omitted means infinite.", amplifier = "Level minus one.", hide_particles = "Hide effect particles."), ClearEffect(effect = "Effect to remove; omitted removes all."), GiveItem(item = "Item identity.", count = "Item count."), SetEquipment(slot = "Hand or armor slot.", item = "Item identity; omit it to clear the slot.", count = "Stack count, from one to 99."), ClearItem(item = "Item to remove; omitted removes all.", count = "Maximum constant or runtime count; omitted removes every match."), CountItems(item = "Vanilla or custom item identity.", score = "Score receiving the number of matching items."), TransferInventory(transfer = "The declared transfer definition."), Hud(hud = "The declared HUD definition.", show = "Start when true, cancel the current player’s instances when false."), Menu(menu = "Declared menu identity.", open = "Open a fresh session when true, close this menu’s session when false."), Projectile(projectile = "Declared projectile identity.", launch = "Launch when true, cancel the owner’s flights when false."), Progression(progression = "Declared progression identity.", advance = "Advance when true, reset when false."), Scene(scene = "Declared scene identity.", show = "Start a new scene when true, cancel the owner’s instances when false."), Job(job = "Declared job identity.", start = "Start a new instance when true, cancel the owner’s instances when false."), Session(session = "Declared session identity.", command = "Operation to apply to the session."), Ability(ability = "Declared ability identity.", command = "Operation to apply to the ability."), Combat(combat = "Declared custom health policy.", amount = "Constant or runtime hit points.", heal = "Heal when true, damage when false."), Schedule(function = "Server-context function to run.", ticks = "Delay in game ticks; at least one."), Suspend(ticks = "Positive loaded ticks as a constant or scoreboard field.", fields = "Global invocation slots retained for the continuation.", body = "Operations executed on resume.", group = "Optional pack-wide wait group.", server = "Whether the server owns the continuation."), CancelWaits(group = "Optional group to cancel for this executor.", server = "Whether to cancel server-owned waits."), While(condition = "Checked before every pass.", body = "Repeated body."), AttributeGet(score = "Written State field.", attribute = "Attribute identity.", scale = "Multiplier applied before truncation."), AttributeSet(attribute = "Attribute identity.", value = "Constant or State-derived value."), Entities(query = "Selected entities.", body = "Run as each entity."), Ride(vehicle = "Retained vehicle identity, omitted to dismount."), EntityTag(tag = "Tag name.", add = "Add when true, remove when false."), Summon(entity_type = "Entity type identity.", tag = "Tag given to the new entity."), CaptureEntity(score = "Field receiving the retained identity.", relation = "Optional single-entity relationship to traverse from the executor."), WithEntity(reference = "Retained entity identity.", players = "Resolve only online players.", body = "Body run as the resolved entity."), Damage(amount = "Damage in half-hearts.", by = "Retained attacker identity for kill credit.", damage_type = "Native damage type identity, defaulting to generic."), CountEntities(score = "Written State field.", query = "Counted entities."), Teleport(destination = "Where the executor moves."), SetVelocity(x = "X speed in blocks per tick.", y = "Y speed in blocks per tick.", z = "Z speed in blocks per tick."), AttachState(state = "Entity State to attach."), DetachState(state = "Entity State to detach."), SetBlock(block = "Block identity.", at = "Position."), PlaceTemplate(template = "Structure template identity.", at = "Origin of the placed template."), FillBlocks(block = "Block identity.", from = "One corner.", to = "Opposite corner."), CloneBlocks(from = "One source corner.", to = "Opposite source corner.", destination = "Lower corner of the destination.", masked = "Skip air blocks when true.", filter = "Optional block or block tag filter.", mode = "Vanilla overlap and move behavior."), InDimension(dimension = "Destination dimension.", body = "Body executed in the destination dimension."), ReadBlockData(target = "Block entity and checked path.", score = "Score receiving the vanilla data result."), ReadEntityData(path = "Checked NBT path into the executor.", score = "Written State field."), WriteBlockData(target = "Block entity and checked path.", value = "Typed value source, or omitted to remove the selection."), AddExperience(amount = "Experience, which may be negative.", levels = "Levels instead of points."), SetGameMode(mode = "New game mode."), SetTime(ticks = "Time of day in ticks."), SetWeather(weather = "New weather.", seconds = "Duration; omitted lets Minecraft choose."), BossBarAudience(bar = "Boss bar.", audience = "Who changes.", visible = "Show when true, hide when false."), SetBossBar(bar = "Boss bar.", value = "Constant or State field shown as the bar progress.", max = "Set the maximum instead of the value."), GiveLoot(table = "Loot table."), DropLoot(table = "Loot table."), PushFrame(fields = "Saved State fields, in order."), PopFrame(fields = "Restored State fields, in the PushFrame order."), DataGet(score = "Written State field.", value = "Entity data read.", scale = "Multiplier applied before truncation."), DataModify(target = "Written storage selection.", operator = "Canonical typed NBT modification.", source = "Typed value operand."), DataRemove(target = "Removed storage selection."), DataRead(score = "Written integer field.", source = "Read storage selection."), DataEqual(score = "Boolean result field.", left = "Left stored value.", right = "Right stored value.")),
)]
pub enum Action {
    /// Modify an owned storage value. Source values and dynamic indices are
    /// staged before the write, including when source and target alias.
    DataModify {
        target: DataReference,
        operator: sand_commands::DataModifyOperation,
        source: DataOperand,
    },
    /// Remove a stored selection. A missing selection leaves the value unchanged.
    DataRemove { target: DataReference },
    /// Read a stored integer, string length, or collection length into a score.
    /// Missing values produce zero, following Minecraft's data-get conversion.
    DataRead {
        score: FieldReference,
        source: DataReference,
    },
    /// Compare two present stored values using Minecraft's deep NBT equality.
    /// Missing selections compare false, even when both are absent.
    DataEqual {
        score: FieldReference,
        left: DataReference,
        right: DataReference,
    },

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
    /// Execute the body as each player, positioned at that player.
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
    /// Show a native dialog to the player, or clear their open dialog.
    Dialog {
        #[schemars(with = "Option<String>")]
        #[serde(default, skip_serializing_if = "Option::is_none")]
        dialog: Option<ResourceLocation>,
    },
    /// Play a sound to the executing player at their position.
    Sound {
        sound: String,
        #[serde(default = "one")]
        volume: Amount,
        #[serde(default = "one")]
        pitch: Amount,
    },
    /// Spawn particles one block above the executor, with checked numeric options.
    Particle {
        particle: String,
        #[serde(default = "one_operand")]
        count: ScoreOperand,
        #[serde(default = "zero_amount")]
        spread: Amount,
        #[serde(default = "zero_amount")]
        speed: Amount,
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
    /// Give items to the executing player. Runtime counts outside 1..=6400
    /// perform no action; constants outside that range are rejected.
    GiveItem {
        item: String,
        #[serde(default = "one_operand")]
        count: ScoreOperand,
    },
    /// Replace a hand or armor slot on the executor. An omitted item clears it.
    SetEquipment {
        slot: sand_components::EquipmentSlot,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        item: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        count: Option<u32>,
    },
    /// Remove matching items, or every item, from the executing player.
    ClearItem {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        item: Option<String>,
        /// Constant or runtime maximum. A negative runtime value skips removal.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        count: Option<ScoreOperand>,
    },
    /// Count matching items in the executing player's inventory.
    CountItems {
        /// Vanilla item identity or name of a custom item defined by the pack.
        item: String,
        /// Score receiving Minecraft's quantity without changing the player's inventory.
        score: FieldReference,
    },
    /// Open or close a player-owned native menu session.
    Menu {
        /// Identity of a menu declared by this program.
        #[schemars(with = "String")]
        menu: sand_components::ResourceLocation,
        /// Open a new session when true, or close this menu's current session.
        open: bool,
    },
    /// Apply damage or healing to the actor's authoritative State.
    Combat {
        #[schemars(with = "String")]
        combat: ResourceLocation,
        amount: Amount,
        heal: bool,
    },
    /// Activate, cancel, or diagnose the player's ability.
    Ability {
        #[schemars(with = "String")]
        ability: ResourceLocation,
        command: AbilityCommand,
    },
    /// Advance once or reset the player's durable progression.
    Progression {
        #[schemars(with = "String")]
        progression: ResourceLocation,
        advance: bool,
    },
    /// Start a display scene or cancel the executing player's instances.
    Scene {
        #[schemars(with = "String")]
        scene: ResourceLocation,
        /// Start a fresh scene if true, otherwise cancel the owner's scenes.
        show: bool,
    },
    /// Start a job or cancel the executing player's instances.
    Job {
        #[schemars(with = "String")]
        job: ResourceLocation,
        /// Start a fresh instance if true, otherwise cancel the owner's jobs.
        start: bool,
    },
    /// Control this player's named session.
    Session {
        #[schemars(with = "String")]
        session: ResourceLocation,
        command: SessionCommand,
    },
    /// Launch a named projectile or cancel its flights for this player.
    Projectile {
        #[schemars(with = "String")]
        projectile: ResourceLocation,
        launch: bool,
    },
    /// Start or cancel named presentation for the executing player.
    Hud {
        /// Identity of a HUD declared by this program.
        #[schemars(with = "String")]
        hud: sand_components::ResourceLocation,
        /// Start a new instance when true, or cancel this player's instances.
        show: bool,
    },
    /// Spawn, attach or release an entity archetype.
    Archetype {
        #[schemars(with = "String")]
        archetype: ResourceLocation,
        command: ArchetypeCommand,
    },
    /// Apply or remove a named player status.
    Status {
        #[schemars(with = "String")]
        status: ResourceLocation,
        apply: bool,
    },
    /// Operate on a named online roster.
    Roster {
        #[schemars(with = "String")]
        roster: ResourceLocation,
        command: RosterCommand,
    },
    /// Start or cancel this player's encounter and its child attacks.
    Encounter {
        #[schemars(with = "String")]
        encounter: ResourceLocation,
        start: bool,
    },
    /// Operate on the executing player's checkpoint.
    Checkpoint {
        #[schemars(with = "String")]
        checkpoint: ResourceLocation,
        command: CheckpointCommand,
    },
    /// Run a named inventory transfer for the executing player.
    TransferInventory {
        /// Identity of an inventory transfer declared by this program.
        #[schemars(with = "String")]
        transfer: sand_components::ResourceLocation,
    },
    /// Run a server-context function after a delay, replacing any pending
    /// schedule of the same function.
    Schedule {
        function: FunctionReference,
        ticks: u32,
    },
    /// Run a continuation after a delay with the retained invocation fields.
    Suspend {
        /// Positive number of loaded ticks. An invalid runtime score resumes
        /// immediately in the current execution context.
        ticks: ScoreOperand,
        /// Global slots copied into each admitted invocation.
        fields: Vec<FieldReference>,
        /// Operations to resume with the retained values.
        body: Vec<Operation>,
        /// Optional pack-wide name for selecting waits to cancel for an owner.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        group: Option<String>,
        /// Use a server-owned queue instead of retaining the current entity.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        server: bool,
    },
    /// Cancel this executor's outstanding continuations for this pack.
    CancelWaits {
        /// Group to cancel, or all groups and ungrouped waits when omitted.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        group: Option<String>,
        /// Cancel server-owned queues instead of the executing entity's waits.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        server: bool,
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
    /// Store the executor's attribute base value, multiplied by
    /// `scale` and truncated, in a State field.
    AttributeGet {
        score: FieldReference,
        attribute: String,
        #[serde(default = "one_f64")]
        scale: f64,
    },
    /// Set the executor's attribute base value to a constant or a
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
    /// Mount the executor on a retained non-player entity, or dismount it.
    Ride {
        /// Vehicle identity. Absence requests a dismount.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vehicle: Option<FieldReference>,
    },
    /// Add or remove a tag on the executing entity.
    EntityTag { tag: String, add: bool },
    /// Summon an entity at the executor's position.
    Summon {
        entity_type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
    },
    /// Damage the executing entity, in half-hearts.
    Damage {
        amount: Amount,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(with = "Option<String>")]
        damage_type: Option<sand_components::ResourceLocation>,
        /// Retained attacker identity; absence uses unattributed generic damage.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        by: Option<FieldReference>,
    },
    /// Retain the executor or one related entity in an unbounded score field.
    /// Zero denotes no entity; identity allocation never wraps or reuses IDs.
    CaptureEntity {
        score: FieldReference,
        /// Optional single-entity relationship to traverse from the executor.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        relation: Option<Relation>,
    },
    /// Run a body as the retained entity, positioned at it. Missing, unloaded,
    /// or dead entities skip the body. Players can resolve across dimensions;
    /// non-player resolution uses the current dimension.
    WithEntity {
        reference: FieldReference,
        #[serde(default)]
        players: bool,
        body: Vec<Operation>,
    },
    /// Store how many entities match the query.
    CountEntities {
        score: FieldReference,
        query: EntityQuery,
    },
    /// Move the executing entity.
    Teleport { destination: TeleportDestination },
    /// Replace a non-player executor's velocity in blocks per tick.
    /// Runtime operands outside the native range skip the write.
    SetVelocity { x: Amount, y: Amount, z: Amount },
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
    /// Place a block.
    SetBlock { block: String, at: BlockPosition },
    /// Fill a region, corners inclusive, with a block.
    FillBlocks {
        block: String,
        from: BlockPosition,
        to: BlockPosition,
    },
    /// Copy or move the blocks in an inclusive region to another position.
    CloneBlocks {
        /// One source corner.
        from: BlockPosition,
        /// Opposite source corner.
        to: BlockPosition,
        /// Lower corner of the destination.
        destination: BlockPosition,
        /// Skip air blocks. Mutually exclusive with a block filter.
        #[serde(default)]
        masked: bool,
        /// Optional block identity or tag to clone selectively.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<String>,
        /// Vanilla overlap and move behavior.
        #[serde(default)]
        mode: sand_commands::CloneMode,
    },
    /// Place a native structure template at an absolute or relative block position.
    PlaceTemplate {
        /// A declared template or the namespaced identity of an external template.
        #[schemars(with = "String")]
        template: sand_components::ResourceLocation,
        /// Minimum corner of the template, using its authored orientation.
        at: BlockPosition,
    },
    /// Run a body in another dimension, retaining the current executor.
    InDimension {
        /// Namespaced destination dimension.
        #[schemars(with = "String")]
        dimension: sand_components::ResourceLocation,
        /// Body with the changed dimension and unchanged executor.
        body: Vec<Operation>,
    },
    /// Read a path on the executing entity using vanilla's integer data result.
    ReadEntityData {
        /// NBT path validated by the native data command builder.
        path: String,
        /// Score receiving the result, with zero on a failed read.
        score: FieldReference,
    },
    /// Read a block entity NBT value using vanilla's integer `data get` result.
    ReadBlockData {
        /// Block entity and checked path.
        target: BlockDataAddress,
        /// Score receiving the result, with zero on a failed read.
        score: FieldReference,
    },
    /// Set or remove a typed value in a loaded block entity.
    WriteBlockData {
        /// Block entity and checked path.
        target: BlockDataAddress,
        /// Typed value to set, or omitted to remove the selection.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<DataOperand>,
    },
    /// Give the executing player experience points or levels.
    AddExperience {
        amount: ScoreOperand,
        #[serde(default)]
        levels: bool,
    },
    /// Change the executing player's game mode.
    SetGameMode { mode: PlayerGameMode },
    /// Set the time of day, in ticks.
    SetTime { ticks: u32 },
    /// Set the weather, for a duration or until Minecraft changes it.
    SetWeather {
        weather: Weather,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        seconds: Option<u32>,
    },
    /// Show or hide a boss bar for the executing player or everyone.
    BossBarAudience {
        #[schemars(with = "String")]
        bar: sand_components::ResourceLocation,
        audience: Audience,
        visible: bool,
    },
    /// Set a boss bar's value, or its maximum.
    SetBossBar {
        #[schemars(with = "String")]
        bar: sand_components::ResourceLocation,
        value: ScoreOperand,
        #[serde(default)]
        max: bool,
    },
    /// Give the executing player one roll of a loot table.
    GiveLoot {
        #[schemars(with = "String")]
        table: sand_components::ResourceLocation,
    },
    /// Drop one roll of a loot table at the executor's position.
    DropLoot {
        #[schemars(with = "String")]
        table: sand_components::ResourceLocation,
    },
    /// Save State fields on the call-frame stack before a call that may
    /// re-enter the running function, so recursion keeps each call's values.
    /// Every `PushFrame` is followed by a `PopFrame` naming the same fields.
    PushFrame { fields: Vec<FieldReference> },
    /// Restore the fields saved by the matching `PushFrame` and drop its frame.
    PopFrame { fields: Vec<FieldReference> },
    /// Store a number read from the executing entity's data, multiplied by
    /// `scale` and truncated, in a State field.
    DataGet {
        score: FieldReference,
        value: EntityData,
        #[serde(default = "one_f64")]
        scale: f64,
    },
}

fn one_f64() -> f64 {
    1.0
}

fn one() -> Amount {
    Amount::Value(1.0)
}

fn zero_amount() -> Amount {
    Amount::Value(0.0)
}

fn one_operand() -> ScoreOperand {
    ScoreOperand::Value(1)
}

/// The operation to apply to a portable player checkpoint.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
    registry = sand_api_contract, path = "sand::advanced::compiler::CheckpointCommand", module = "sand::advanced::compiler",
    summary = "Select a checkpoint operation.",
    context = "Applies to the executing player's saved State.",
    minecraft = "Calls the shared Checkpoint transaction.",
    use_when = ["Operating a declared checkpoint"],
    avoid_when = ["Copying arbitrary world data"],
    variants(Save = "Commit selected live scores to saved State.", Restore = "Revalidate and restore the saved projection.", Clear = "Invalidate a saved projection."),
    example = "use sand::advanced::compiler::CheckpointCommand;",
)]
pub enum CheckpointCommand {
    /// Copy the live values to saved State.
    Save,
    /// Copy saved values back to the live fields.
    Restore,
    /// Invalidate the snapshot.
    Clear,
}

/// A control action for a player's named session.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::SessionCommand", module = "sand::advanced::compiler",
 summary = "A control action for a player session.",
 context = "All controls affect the executing player's session. Session locks reject reentry during callbacks.",
 minecraft = "Lowers to a shared Session runtime action.",
 use_when = ["Operating a declared session"],
 avoid_when = ["Mutating progression without managing its session"],
 variants(Start = "Start or replace while preserving progress.", Restart = "Reset progress, then start.", Reset = "Cancel and reset progress.", Cancel = "Cancel while preserving progress."),
 example = "use sand::advanced::compiler::SessionCommand;",
)]
pub enum SessionCommand {
    /// Start or replace this player's instance while retaining progress.
    Start,
    /// Reset progress and start a fresh instance.
    Restart,
    /// Cancel the current instance and reset progress.
    Reset,
    /// Cancel the current instance and retain progress.
    Cancel,
}

/// A control action for one player-owned ability.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::AbilityCommand", module = "sand::advanced::compiler",
 summary = "A control action for a player-owned ability.",
 context = "All controls apply to the executing player. Diagnostics never spend State.",
 minecraft = "Calls the shared Ability runtime.",
 use_when = ["Operating a declared ability"],
 avoid_when = ["Controlling a native projectile independently"],
 variants(Activate = "Attempt checked admission.", Cancel = "Cancel this owner's instances.", Diagnose = "Report failed admission checks without spending State."),
 example = "use sand::advanced::compiler::AbilityCommand;",
)]
pub enum AbilityCommand {
    /// Attempt checked admission and commit associated State.
    Activate,
    /// Stop this owner's instances without refunding costs.
    Cancel,
    /// Show failed admission checks without mutating State.
    Diagnose,
}

/// One operation on an online roster.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::RosterCommand", module = "sand::advanced::compiler",
 summary = "One operation on an online roster.",
 context = "Admission and iteration run under the canonical roster's generation, capacity, and mutation rules.",
 minecraft = "Runs shared native scoreboard and player-selection functions.",
 use_when = ["Opening a round or checking in players"],
 avoid_when = ["Directly writing roster membership fields"],
 variants(Open = "Open a new generation.", Close = "Close the current generation.", Join = "Admit the current player.", Leave = "Remove the current player.", Each = "Invoke a function for each current player."),
 variant_fields(Join(role = "Declared score encoding of the role."), Each(role = "Optional role encoding filter.", callback = "Synchronous player callback.")),
 example = "use sand::advanced::compiler::RosterCommand;",
)]
pub enum RosterCommand {
    /// Open a fresh generation.
    Open,
    /// Close the current generation.
    Close,
    /// Admit the executing player.
    Join {
        /// Declared role encoding to commit.
        role: i32,
    },
    /// Remove the executing player.
    Leave,
    /// Visit matching current members under the roster's mutation lock.
    Each {
        /// Optional role filter.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        role: Option<i32>,
        /// Synchronous callback executed as and at each matching player.
        callback: FunctionReference,
    },
}

/// One entity archetype lifecycle operation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ArchetypeCommand", module = "sand::advanced::compiler",
 summary = "An entity archetype lifecycle operation.",
 context = "Attachment and release require an executing entity. Spawning binds initialization to the newly spawned entity.",
 minecraft = "Runs the canonical state ownership and entity initialization functions.",
 use_when = ["Explicitly managing an archetype's entities"],
 avoid_when = ["Only spawning a plain native entity"],
 variants(Spawn = "Summon and initialize a new entity.", Attach = "Initialize the executing entity.", Release = "Release the executing entity and clean up relinquished State."),
 variant_fields(Spawn(at = "Where to spawn the new entity.")),
 example = "use sand::advanced::compiler::ArchetypeCommand;",
)]
pub enum ArchetypeCommand {
    /// Summon and initialize a new entity.
    Spawn {
        /// Absolute or relative block coordinates.
        at: BlockPosition,
    },
    /// Initialize the executing entity if its kind matches.
    Attach,
    /// Release the executing entity and run canonical cleanup.
    Release,
}
