//! Generated/forwarded contracts for facade APIs whose definitions live in
//! implementation or proc-macro crates.
//!
//! Stable procedural macros cannot attach an attribute to a `pub use`, so the
//! facade owns these canonical identities and aliases in one auditable table.
//! The build script parses and validates this table against the independently
//! reachable surface, then emits the only runtime inventory registrations.

include!(concat!(env!("OUT_DIR"), "/api_facade_registrations.rs"));

macro_rules! register {
    ($($contract:tt)*) => {};
}

// Rust does not permit attribute proc macros on out-of-line `mod foo;`
// declarations or on `#[macro_export] macro_rules!` definitions. These are the
// only source-owned exceptions to definition-site `#[api]`; each already has
// substantive local Rustdoc, and this table supplies the same structured
// semantics to the catalog.
register! { path: "sand::prelude", aliases: [], module: "sand", kind: Module, signature: "pub mod prelude", summary: "Collects Sand's ordinary datapack-authoring vocabulary in one import.", context: "The prelude aliases canonical APIs from focused topic modules and is the stable starting point for author code.", minecraft: "Importing the prelude has no runtime effect; imported APIs generate commands and resources when used.", use_when: ["Starting a Sand datapack module"], avoid_when: ["A narrow import prevents a Rust name collision"], params: [], returns: None, example: "use sand::prelude::*;" }
register! { path: "sand::advanced", aliases: [], module: "sand", kind: Module, signature: "pub mod advanced", summary: "Provides the narrow custom-export hook outside ordinary datapack authoring.", context: "This module exposes version-aware JSON export without exposing compiler wiring.", minecraft: "The hook validates resources for the selected Minecraft release before emitting JSON.", use_when: ["Embedding Sand export in custom build integration"], avoid_when: ["Using the normal sand build workflow"], params: [], returns: None, example: "let json = sand::advanced::try_export_components_json(\"demo\", \"26.2\")?;" }
register! { path: "sand::command", aliases: ["sand::cmd", "sand::prelude::cmd"], module: "sand", kind: Module, signature: "pub mod command", summary: "Provides Sand's typed Minecraft command builders.", context: "Command builders cover selectors, execute chains, scores, data, particles, sounds, and other vanilla commands.", minecraft: "They validate typed inputs and render command text collected into generated .mcfunction files.", use_when: ["Building a Minecraft command from typed inputs"], avoid_when: ["Defining a reusable JSON datapack resource"], params: [], returns: None, example: "let command = sand::command::raw(\"say Ready\");" }
register! { path: "sand::event", aliases: [], module: "sand", kind: Module, signature: "pub mod event", summary: "Defines advancement-backed event triggers and typed handler context.", context: "This module supplies the event model used by #[on_event] handlers.", minecraft: "Generates advancement trigger resources and dispatch functions.", use_when: ["Declaring or handling a typed event"], avoid_when: ["Composing tick event graphs directly"], params: [], returns: None, example: "use sand::event::Event;" }
register! { path: "sand::event::handle", aliases: [], module: "sand::event", kind: Module, signature: "pub mod handle", summary: "Contains typed handles for controlling registered events.", context: "Event handles expose lifecycle controls for an already registered typed event.", minecraft: "Uses per-event scoreboards and advancement grant or revoke operations.", use_when: ["Controlling a registered event"], avoid_when: ["Defining an event trigger"], params: [], returns: None, example: "use sand::event::handle::*;" }
register! { path: "sand::event::trigger", aliases: [], module: "sand::event", kind: Module, signature: "pub mod trigger", summary: "Contains typed builders for vanilla advancement trigger criteria.", context: "Trigger builders model conditions for advancement-backed event dispatch.", minecraft: "Serializes into advancement criteria instead of immediate commands.", use_when: ["Defining an AdvancementEvent trigger"], avoid_when: ["Emitting an immediate command"], params: [], returns: None, example: "use sand::event::trigger::*;" }
register! { path: "sand::event::vanilla", aliases: [], module: "sand::event", kind: Module, signature: "pub mod vanilla", summary: "Provides short aliases for built-in advancement-backed event markers.", context: "These aliases expose common built-in markers beside the event model.", minecraft: "Selects the same advancement behavior as canonical sand::events markers.", use_when: ["Importing built-in event markers"], avoid_when: ["Defining a custom trigger"], params: [], returns: None, example: "use sand::event::vanilla::*;" }
register! { path: "sand::events", aliases: [], module: "sand", kind: Module, signature: "pub mod events", summary: "Defines custom Sand event dispatch, composition, and built-in event markers.", context: "This module contains the author-facing event graph and built-in event vocabulary.", minecraft: "Lowers typed definitions into advancements, tick functions, and event state.", use_when: ["Defining or handling a Sand event"], avoid_when: ["Inspecting generated event graph state"], params: [], returns: None, example: "use sand::events::*;" }
register! { path: "sand::item", aliases: [], module: "sand", kind: Module, signature: "pub mod item", summary: "Builds custom items and typed item-component behavior.", context: "Groups custom-item builders, predicates, locations, and event-time snapshots.", minecraft: "Generates item JSON and validated item commands or predicates.", use_when: ["Defining, locating, or matching an item"], avoid_when: ["Building an unrelated component"], params: [], returns: None, example: "use sand::item::*;" }
register! { path: "sand::item::location", aliases: [], module: "sand::item", kind: Module, signature: "pub mod location", summary: "Provides validated live item and inventory addressing.", context: "Location types identify where an item currently resides.", minecraft: "Builds inventory targets for item, data, and execute-if-items commands.", use_when: ["Constructing a live item location"], avoid_when: ["Keeping event-time evidence after mutation"], params: [], returns: None, example: "let slot = sand::item::ItemLocation::PlayerMainHand;" }
register! { path: "sand::item::snapshot", aliases: [], module: "sand::item", kind: Module, signature: "pub mod snapshot", summary: "Provides immutable, short-lived event-time item capture handles.", context: "Snapshots preserve source items before handler logic mutates inventory.", minecraft: "Uses guarded copies into deterministic command storage during one dispatch.", use_when: ["A handler needs trigger-time item evidence"], avoid_when: ["Representing a live inventory slot"], params: [], returns: None, example: "use sand::item::snapshot::ItemSnapshot;" }
register! { path: "sand::vfx", aliases: [], module: "sand", kind: Module, signature: "pub mod vfx", summary: "Builds reusable particle, sound, and explicit raw-command effects.", context: "VFX keeps presentation sequences reusable and ordered.", minecraft: "Renders particle and playsound commands at selectors or positions.", use_when: ["Reusing a presentation sequence"], avoid_when: ["Modeling state changes"], params: [], returns: None, example: "let effect = sand::vfx::Vfx::new(\"level_up\");" }
register! { path: "sand::mcfunction", aliases: ["sand::prelude::mcfunction"], module: "sand", kind: Macro, signature: "mcfunction![command; ...]", summary: "Collects command expressions into one Minecraft function body.", context: "Accepts typed builders and literal command strings in source order.", minecraft: "Produces ordered command lines for a generated .mcfunction file.", use_when: ["Passing several commands as one list"], avoid_when: ["A single builder is sufficient"], params: [], returns: None, example: "let commands = sand::mcfunction![sand::command::raw(\"say Ready\");];" }
register! { path: "sand::all", aliases: ["sand::prelude::all"], module: "sand", kind: Macro, signature: "all![condition, ...]", summary: "Combines typed conditions that must all hold.", context: "Concise syntax for Condition::all.", minecraft: "Lowers to clauses requiring every condition.", use_when: ["Multiple guards are all required"], avoid_when: ["Any alternative may pass"], params: [], returns: None, example: "let ready = sand::all![has_key, is_near_door];" }
register! { path: "sand::any", aliases: ["sand::prelude::any"], module: "sand", kind: Macro, signature: "any![condition, ...]", summary: "Combines typed conditions where any alternative may hold.", context: "Concise syntax for Condition::any.", minecraft: "Lowers alternative execute-condition branches preserving OR behavior.", use_when: ["One of several guards may pass"], avoid_when: ["Every condition is required"], params: [], returns: None, example: "let visible = sand::any![is_day, has_night_vision];" }

// Typed trigger builders keep custom AdvancementEvent definitions in the
// vanilla criterion vocabulary.  `new` starts an unconstrained criterion,
// predicate setters narrow it, and `build` hands the result to the event.
// Built-in advancement-backed event markers. Their aliases in `event::vanilla`
// are short discovery names; `events::*Event` remains the canonical catalog.
// Tick-polled and tracked built-ins. These are bare `SandEvent` markers: the
// handler receives the marker itself, and Sand owns the per-player detector.
register! {
    path: "sand::api",
    aliases: ["sand::prelude::api"],
    module: "sand",
    kind: Macro,
    signature: "#[api(...)]",
    summary: "Attaches Sand's authoritative API contract to a supported declaration.",
    context: "The attribute provides the concise semantic description that drives Rustdoc, installed metadata, CLI discovery, and ordinary-build contract enforcement.",
    minecraft: "The attribute emits no Minecraft data by itself; it records how the annotated API participates in Minecraft authoring.",
    use_when: ["Contributing a new supported Sand API", "Documenting a deliberately public author-facing declaration"],
    avoid_when: ["Annotating private implementation details or using documentation as an item-level exemption"],
    params: [],
    returns: None,
    example: "#[sand::api(path = \"my_crate::feature\", summary = \"...\", context = \"...\", minecraft = \"...\", use_when = [\"...\"], avoid_when = [\"...\"], example = \"...\")]"
}

register! {
    path: "sand::function",
    aliases: ["sand::prelude::function"],
    module: "sand",
    kind: Macro,
    signature: "#[function]",
    summary: "Registers a Rust function as an exported Minecraft function.",
    context: "The attribute turns a command-producing Rust body into a named Sand function that other generated functions and Minecraft load/tag wiring can call.",
    minecraft: "Exports the collected command lines to data/<namespace>/function/<name>.mcfunction.",
    use_when: ["Writing reusable datapack command logic", "Defining a function invoked by another Sand API or a generated tag"],
    avoid_when: ["Returning a JSON component resource; use #[datapack_component]"],
    params: [],
    returns: None,
    example: "#[sand::function]\nfn welcome() { sand::command::raw(\"say Welcome\"); }"
}

register! {
    path: "sand::datapack_component",
    aliases: ["sand::prelude::datapack_component"],
    module: "sand",
    kind: Macro,
    signature: "#[datapack_component(...)]",
    summary: "Registers a component-producing function for datapack JSON export.",
    context: "The attribute connects a typed component value returned by a Rust function to Sand's component registry and optional lifecycle tags.",
    minecraft: "Exports the returned advancement, recipe, loot table, tag, dialog, or other component as its version-valid JSON resource.",
    use_when: ["Defining a reusable datapack JSON component", "Registering load, tick, or tag-oriented component output"],
    avoid_when: ["Writing a command body; use #[function]"],
    params: [],
    returns: None,
    example: "#[sand::datapack_component]\nfn reward() -> sand::component::Advancement { todo!() }"
}

register! {
    path: "sand::on_event",
    aliases: ["sand::prelude::on_event"],
    module: "sand",
    kind: Macro,
    signature: "#[on_event(...)]",
    summary: "Registers a Rust function as a typed Minecraft event handler.",
    context: "The attribute connects an advancement-backed or SandEvent marker to a command-producing handler and supplies its typed event context.",
    minecraft: "Generates the event detection resources and handler dispatch functions required by the selected event type.",
    use_when: ["Responding to a vanilla or custom Sand event", "Writing a handler that needs typed player or participant context"],
    avoid_when: ["Creating a JSON component without runtime handler logic; use #[datapack_component]"],
    params: [],
    returns: None,
    example: "#[sand::on_event]\nfn joined(event: sand::event::Event<OnJoinEvent>) { sand::command::raw(\"say @s event fired\"); }"
}

register! {
    path: "sand::custom_item",
    aliases: ["sand::prelude::custom_item"],
    module: "sand",
    kind: Macro,
    signature: "#[custom_item(...)]",
    summary: "Derives a typed custom-item reference from a CustomItem factory function.",
    context: "The attribute inspects the declared custom-item definition and generates the stable helper type, matching predicate, and equipment checks used by author code.",
    minecraft: "Does not register a payload by itself; it emits the typed reference, item predicate, and equipment-check helpers that later render item commands or predicates.",
    use_when: ["Giving a custom item a reusable typed Rust handle", "Generating equipment checks tied to one item definition"],
    avoid_when: ["Constructing a one-off CustomItem value without generated helpers"],
    params: [],
    returns: None,
    example: "#[sand::custom_item]\nfn compass() -> sand::component::CustomItem { sand::component::CustomItem::new(\"minecraft:compass\").custom_data(\"demo_compass\") }"
}

register! {
    path: "sand::armor_event",
    aliases: ["sand::prelude::armor_event"],
    module: "sand",
    kind: Macro,
    signature: "#[armor_event(...)]",
    summary: "Registers an equip or unequip handler for a custom armor item.",
    context: "The attribute generates player inventory polling and transition bookkeeping needed to invoke a no-argument handler exactly when the selected armor or offhand item changes.",
    minecraft: "Emits a minecraft:tick function that checks @a players, records prior slot state, and invokes the handler on the requested transition.",
    use_when: ["Reacting when a player equips or removes an armor or offhand item"],
    avoid_when: ["Handling a general inventory change or a non-armor item event"],
    params: [],
    returns: None,
    example: "#[sand::armor_event(Equip, slot = Head, item = \"minecraft:diamond_helmet\")]\nfn helmet_equipped() { sand::command::raw(\"say Protected\"); }"
}

register! {
    path: "sand::schedule",
    aliases: ["sand::prelude::schedule"],
    module: "sand",
    kind: Macro,
    signature: "#[schedule(...)]",
    summary: "Registers a per-player countdown-driven scheduled function.",
    context: "The attribute creates a command-producing body plus generated start and stop entry points; Sand tracks each player's remaining ticks and phase with scoreboard state rather than wrapping Minecraft's /schedule command.",
    minecraft: "Emits tick-driven scoreboard countdown logic and <name>_start / <name>_stop functions that control each player's lifecycle.",
    use_when: ["Defining recurring or delayed datapack command logic"],
    avoid_when: ["Running commands immediately in the current function body"],
    params: [],
    returns: None,
    example: "#[sand::schedule(ticks = 60, every = 5)]\nfn refresh() { sand::command::raw(\"say Refreshing\"); }"
}

register! {
    path: "sand::entity_archetype",
    aliases: ["sand::prelude::entity_archetype"],
    module: "sand",
    kind: Macro,
    signature: "#[entity_archetype]",
    summary: "Registers a typed entity-archetype factory.",
    context: "The attribute links a component-first EntityArchetype definition to Sand's lifecycle registry so composed State lifecycle, native behavior, and derived values are evaluated consistently.",
    minecraft: "Generates the functions and periodic checks required to maintain the declared archetype for loaded entities.",
    use_when: ["Declaring reusable behavior and state for a Minecraft entity kind"],
    avoid_when: ["Issuing a one-time selector command without archetype lifecycle behavior"],
    params: [],
    returns: None,
    example: "#[sand::entity_archetype]\nfn zombie() -> sand::entity::EntityArchetype<ZombieKind> { todo!() }"
}

register! {
    path: "sand::State",
    aliases: ["sand::prelude::State"],
    module: "sand",
    kind: Macro,
    signature: "#[derive(State)]",
    summary: "Derives a typed, versioned Sand state schema from a Rust struct.",
    context: "The derive converts field-level state metadata into concrete typed accessors and lifecycle information so application state does not need hand-written score or storage plumbing.",
    minecraft: "Allocates and maintains the scoreboard or storage-backed data locations declared by the schema.",
    use_when: ["Modeling persistent gameplay state as a Rust struct", "Sharing one validated state schema across functions and events"],
    avoid_when: ["A single transient command-local value needs no schema"],
    params: [],
    returns: None,
    example: "use sand::prelude::*;\n\n#[derive(State)]\n#[state(namespace = \"demo\", scope = player)]\nstruct PlayerState {\n    #[state(default = 0)]\n    score: EntityScore<i32>,\n}"
}

register! {
    path: "sand::EntityStateEnum",
    aliases: ["sand::prelude::EntityStateEnum"],
    module: "sand",
    kind: Macro,
    signature: "#[derive(EntityStateEnum)]",
    summary: "Maps a fieldless Rust enum to a stable integer entity-state encoding.",
    context: "The derive gives a finite state machine an explicit scoreboard representation while preserving enum names at the Rust call site.",
    minecraft: "Stores the selected variant as the derived integer value in Sand's entity-state machinery.",
    use_when: ["Representing a finite entity-state field with named Rust variants"],
    avoid_when: ["The state has data-carrying variants or needs an open-ended numeric range"],
    params: [],
    returns: None,
    example: "#[derive(sand::EntityStateEnum)]\nenum Phase { Idle, Alert }"
}

register! {
    path: "sand::StateEnum",
    aliases: ["sand::prelude::StateEnum"],
    module: "sand",
    kind: Macro,
    signature: "#[derive(StateEnum)]",
    summary: "Maps a fieldless Rust enum to the canonical stable State encoding.",
    context: "The derive preserves explicit discriminants and exposes named variants through State enum fields.",
    minecraft: "Stores each variant as its stable signed scoreboard encoding.",
    use_when: ["Declaring a finite State field with named Rust variants"],
    avoid_when: ["Variants carry payload data"],
    params: [],
    returns: None,
    example: "#[derive(sand::StateEnum)]\nenum Phase { Idle, Alert }"
}

register! {
    path: "sand::StateBundle",
    aliases: ["sand::prelude::StateBundle"],
    module: "sand",
    kind: Macro,
    signature: "#[derive(StateBundle)]",
    summary: "Derives a concrete named view over reusable State components and nested bundles.",
    context: "Bundles compose component APIs without merging storage, versions, presence, or lifecycle ownership.",
    minecraft: "Reuses each referenced component's existing objectives and lifecycle commands.",
    use_when: ["Several systems or archetypes share the same component composition"],
    avoid_when: ["Declaring new physical state fields"],
    params: [],
    returns: None,
    example: "#[derive(sand::StateBundle)]\nstruct Combat { attack: Attack, defense: Defense }"
}

register! {
    path: "sand::StateQuery",
    aliases: ["sand::prelude::StateQuery"],
    module: "sand",
    kind: Macro,
    signature: "#[derive(StateQuery)]",
    summary: "Derives a concrete query item and Minecraft iteration for State component presence.",
    context: "Named fields declare required, optional, or forbidden components and bundles without public query tuples or borrowing wrappers.",
    minecraft: "Required presence lowers into a typed scores selector; forbidden and optional presence lower into runtime execute guards inside the generated iteration.",
    use_when: ["Processing loaded entities selected by attached State components"],
    avoid_when: ["A persistent Rust reference to a Minecraft entity is required"],
    params: [],
    returns: None,
    example: "#[derive(sand::StateQuery)]\nstruct Combatants { attack: Attack, #[without] dead: Dead }"
}

register! {
    path: "sand::state_lifecycle",
    aliases: ["sand::prelude::state_lifecycle"],
    module: "sand",
    kind: Macro,
    signature: "#[state_lifecycle]",
    summary: "Registers one optional StateLifecycle implementation for generated lifecycle planning.",
    context: "The attribute connects ordinary trait hooks to the same immutable component descriptors used by State.",
    minecraft: "Adds hook commands to generated provisioning, initialization, tick, reconciliation, migration, and cleanup phases.",
    use_when: ["A component needs behavior beyond inferred field lifecycle work"],
    avoid_when: ["Defaults and automatic timer/cooldown ticking are sufficient"],
    params: [],
    returns: None,
    example: "#[sand::state_lifecycle]\nimpl sand::state::StateLifecycle for PlayerState {}"
}

register! {
    path: "sand::system",
    aliases: ["sand::prelude::system"],
    module: "sand",
    kind: Macro,
    signature: "#[system(tick, every = N)]",
    summary: "Registers a deterministic tick system over a concrete StateQuery.",
    context: "Function and grouped-impl forms lower query callbacks through Sand's existing function, tick-tag, and dynamic-function machinery.",
    minecraft: "Emits one globally scheduled system function at the requested cadence; the StateQuery performs entity iteration with the correct executor and position.",
    use_when: ["Running component-filtered behavior every server tick or at a fixed cadence"],
    avoid_when: ["Dispatching an existing typed event directly; use on_event"],
    params: [],
    returns: None,
    example: "#[sand::system(tick, every = 20)]\nfn regenerate(query: Combatants) { query.each(|item| item.defense.armor.add(1)); }"
}

register! {
    path: "sand::SandStorage",
    aliases: ["sand::prelude::SandStorage"],
    module: "sand",
    kind: Macro,
    signature: "#[derive(SandStorage)]",
    summary: "Derives typed NBT-storage accessors from a Rust struct.",
    context: "The derive connects named Rust fields to one declared command-storage root so callers use validated paths instead of repeated string literals.",
    minecraft: "Addresses the configured data storage and its NBT fields through typed data-command locations.",
    use_when: ["Defining a fixed storage schema shared by several functions"],
    avoid_when: ["Reading an arbitrary dynamic NBT path"],
    params: [],
    returns: None,
    example: "#[derive(sand::SandStorage)]\n#[sand(storage = \"demo:state\", root = \"player\")]\nstruct PlayerStorage { score: i32 }"
}

register! {
    path: "sand::run_fn",
    aliases: ["sand::prelude::run_fn"],
    module: "sand",
    kind: Macro,
    signature: "run_fn!(...)",
    summary: "Creates a generated Sand function for an inline command sequence.",
    context: "The macro gives a local sequence of command expressions a stable generated function identity so it can be scheduled, called, or registered without manually declaring a separate item.",
    minecraft: "Exports the inline sequence as a generated .mcfunction and references it from the surrounding Sand output.",
    use_when: ["A command sequence needs a function reference but does not justify a named Rust function"],
    avoid_when: ["The sequence is part of the current function body or should be reusable by name"],
    params: [],
    returns: None,
    example: "let delayed = sand::run_fn! { sand::command::raw(\"say Later\") };"
}

register! {
    path: "sand::hud_bar",
    aliases: [],
    module: "sand",
    kind: Macro,
    signature: "hud_bar!(...)",
    summary: "Declares a custom resource-pack HUD bar.",
    context: "The macro packages HUD frame, fill, and layout metadata into Sand's resource-pack registry so gameplay code can address the resulting handle.",
    minecraft: "Writes the GUI textures and resource-pack metadata used to render the configured HUD bar on clients.",
    use_when: ["Adding a reusable custom status bar to a Sand resource pack"],
    avoid_when: ["Displaying a one-off chat or actionbar message"],
    params: [],
    returns: None,
    example: "let health = sand::hud_bar!(/* resource-pack HUD fields */);",
    availability: ["Cargo feature: resourcepack"]
}

register! {
    path: "sand::hud_element",
    aliases: [],
    module: "sand",
    kind: Macro,
    signature: "hud_element!(...)",
    summary: "Declares a custom resource-pack HUD element.",
    context: "The macro registers a positioned UI element and its asset metadata as a typed handle that Sand's resource-pack runtime can update.",
    minecraft: "Writes client resource-pack assets and metadata for the configured HUD element.",
    use_when: ["Showing a reusable texture-backed HUD indicator"],
    avoid_when: ["Rendering structured text through tellraw or an ordinary title"],
    params: [],
    returns: None,
    example: "let indicator = sand::hud_element!(/* resource-pack HUD fields */);",
    availability: ["Cargo feature: resourcepack"]
}

register! {
    path: "sand::texture",
    aliases: [],
    module: "sand",
    kind: Macro,
    signature: "texture!(...)",
    summary: "Registers a texture asset for Sand resource-pack output.",
    context: "The macro associates a source texture and its destination identity with the resource-pack registry so HUD and GUI declarations can refer to it consistently.",
    minecraft: "Copies or emits the declared texture into the matching assets/<namespace>/textures resource-pack path.",
    use_when: ["Bundling a named texture used by Sand's resource-pack APIs"],
    avoid_when: ["Referencing a vanilla texture that does not need to be packaged"],
    params: [],
    returns: None,
    example: "let icon = sand::texture!(/* texture source and destination */);",
    availability: ["Cargo feature: resourcepack"]
}

// BEGIN ENTITY API CONTRACTS
// END ENTITY API CONTRACTS

// BEGIN STATE API CONTRACTS
// END STATE API CONTRACTS
// END PARTICIPANT API CONTRACTS
// END TEXT API CONTRACTS
// END DATA API CONTRACTS
// END SYSTEMS API CONTRACTS
// END COMMAND API CONTRACTS
// END COMPONENT API CONTRACTS
// END RESOURCEPACK API CONTRACTS
// END RESOLVED PRELUDE OWNERSHIP CONTRACTS
