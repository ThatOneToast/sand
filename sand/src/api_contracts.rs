//! Generated/forwarded contracts for facade APIs whose definitions live in
//! implementation or proc-macro crates.
//!
//! Stable procedural macros cannot attach an attribute to a `pub use`, so the
//! facade owns these canonical identities and aliases in one auditable table.
//! Signatures are written once here and checked by focused facade tests.

use sand_api_contract::{ApiKind, ApiRegistration};

macro_rules! register {
    (
        path: $path:literal,
        aliases: [$($alias:literal),* $(,)?],
        module: $module:literal,
        kind: $kind:ident,
        signature: $signature:literal,
        summary: $summary:literal,
        context: $context:literal,
        minecraft: $minecraft:literal,
        use_when: [$($use_when:literal),+ $(,)?],
        avoid_when: [$($avoid_when:literal),+ $(,)?],
        params: [$($param:literal => $param_doc:literal),* $(,)?],
        returns: $returns:expr,
        example: $example:literal $(,
        availability: [$($availability:literal),* $(,)?])?
    ) => {
        sand_api_contract::inventory::submit! {
            ApiRegistration {
                canonical_path: $path,
                aliases: &[$($alias),*],
                canonical_module: $module,
                kind: ApiKind::$kind,
                signature: $signature,
                summary: $summary,
                context: $context,
                minecraft: $minecraft,
                use_when: &[$($use_when),+],
                avoid_when: &[$($avoid_when),+],
                parameters: &[$(StaticApiParameter { name: $param, description: $param_doc }),*],
                returns: $returns,
                example: $example,
                availability: &[$($($availability),*)?],
            }
        }
    };
}

register! {
    path: "sand::prelude",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod prelude",
    summary: "Collects Sand's ordinary datapack-authoring vocabulary in one import.",
    context: "The prelude aliases canonical APIs from focused topic modules and is the stable starting point for author code.",
    minecraft: "Importing the prelude has no runtime effect; the imported builders and macros generate commands and datapack resources when used.",
    use_when: ["Starting a Sand datapack module", "Following Sand examples and book code"],
    avoid_when: ["A narrow import is needed to prevent a Rust name collision"],
    params: [],
    returns: None,
    example: "use sand::prelude::*;"
}

register! {
    path: "sand::vfx",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod vfx",
    summary: "Builds reusable particle, sound, and explicit raw-command effects.",
    context: "VFX keeps a small sequence of presentation commands reusable across functions instead of repeating command strings at every call site.",
    minecraft: "Renders particle and playsound commands, optionally wrapped at a selector or position.",
    use_when: ["Reusing a named presentation sequence", "Keeping particle and sound ordering deterministic"],
    avoid_when: ["Modeling game rules, state changes, or a datapack resource"],
    params: [],
    returns: None,
    example: "let effect = sand::vfx::Vfx::new(\"level_up\");"
}

register! {
    path: "sand::advanced",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod advanced",
    summary: "Provides the narrow custom-export hook outside ordinary datapack authoring.",
    context: "The advanced module deliberately exposes one version-aware JSON export operation without turning compiler wiring into a supported API.",
    minecraft: "The hook validates resources against the configured Minecraft release before emitting their datapack JSON.",
    use_when: ["Embedding Sand export in a custom build integration"],
    avoid_when: ["Using the normal sand build workflow or typed datapack builders"],
    params: [],
    returns: None,
    example: "let json = sand::advanced::try_export_components_json(\"example\", \"26.2\")?;"
}

register! {
    path: "sand::ResourceLocation",
    aliases: ["sand::prelude::ResourceLocation"],
    module: "sand",
    kind: Struct,
    signature: "pub struct ResourceLocation",
    summary: "Stores a validated Minecraft namespace:path identifier.",
    context: "Resource locations are the common identity representation behind Sand's typed resource IDs and command targets, so validation happens before an identifier reaches generated output.",
    minecraft: "Serializes as Minecraft's namespace:path resource-location syntax.",
    use_when: ["Naming a datapack resource", "Passing a validated identifier to an API that accepts multiple resource kinds"],
    avoid_when: ["A resource-kind-specific ID such as FunctionId or ItemId is available", "Keeping an unchecked user-provided identifier"],
    params: [],
    returns: None,
    example: "let id = sand::ResourceLocation::new(\"demo\", \"functions/start\")?;"
}

register! {
    path: "sand::command",
    aliases: ["sand::cmd", "sand::prelude::cmd"],
    module: "sand",
    kind: Module,
    signature: "pub mod command",
    summary: "Provides Sand's typed Minecraft command builders.",
    context: "The command module exposes builders for selectors, execute chains, scores, data, particles, sounds, and other vanilla commands while keeping rendering and validation consistent.",
    minecraft: "Renders the vanilla command text collected into generated .mcfunction files.",
    use_when: ["Building a Minecraft command from typed inputs", "Using a command family not imported directly by the prelude"],
    avoid_when: ["Defining a reusable datapack resource builder such as a predicate or recipe"],
    params: [],
    returns: None,
    example: "let command = sand::command::say(\"Ready\");"
}

register! {
    path: "sand::event",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod event",
    summary: "Defines advancement-backed event triggers and typed handler context.",
    context: "This module supplies the event model used by #[on_event] handlers, including custom advancement triggers and the player context passed to an event function.",
    minecraft: "Generates advancement trigger resources and the functions that dispatch their handlers.",
    use_when: ["Declaring or handling a typed Minecraft event", "Building a custom advancement-backed trigger"],
    avoid_when: ["Composing tick and dependency dispatch graphs directly; use sand::events"],
    params: [],
    returns: None,
    example: "use sand::event::Event;"
}

register! {
    path: "sand::item",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod item",
    summary: "Builds custom items and typed item-component behavior.",
    context: "The item module groups the custom-item builder, item predicates, and location helpers used by #[custom_item] functions and live inventory checks.",
    minecraft: "Generates item component JSON and renders vanilla item commands or item predicates where used.",
    use_when: ["Defining or matching a custom item", "Working with an item stack outside a resource component"],
    avoid_when: ["Addressing only a live slot; use sand::inventory for location construction"],
    params: [],
    returns: None,
    example: "use sand::item::CustomItem;"
}

register! {
    path: "sand::state",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod state",
    summary: "Provides low-level typed scoreboard, timer, and storage state primitives.",
    context: "State is the implementation vocabulary beneath derived schemas and gives authors explicit control over score-backed and storage-backed values when a reusable schema is not appropriate.",
    minecraft: "Renders scoreboard and data-storage commands and describes their persistent datapack state locations.",
    use_when: ["Building typed score, timer, flag, or storage operations", "Implementing a reusable state abstraction"],
    avoid_when: ["A #[derive(State)] schema expresses the application state directly"],
    params: [],
    returns: None,
    example: "use sand::state::ScoreVar;"
}

register! {
    path: "sand::component",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod component",
    summary: "Builds datapack JSON components such as recipes, loot tables, tags, dialogs, and advancements.",
    context: "Component builders model vanilla data resources as typed Rust values that #[datapack_component] can register for export.",
    minecraft: "Serializes the selected component into its corresponding datapack JSON resource.",
    use_when: ["Constructing a datapack JSON resource", "Returning a component from a #[datapack_component] function"],
    avoid_when: ["Emitting one command in a function body"],
    params: [],
    returns: None,
    example: "use sand::component::Advancement;"
}

register! {
    path: "sand::systems",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod systems",
    summary: "Contains optional higher-level gameplay systems built on Sand primitives.",
    context: "Systems bundle reusable patterns such as damage tracking, cooldowns, player data, movement, inventory, and entity behavior behind explicit Cargo features.",
    minecraft: "Each enabled system emits the commands, scores, storage, and lifecycle functions required by its gameplay pattern.",
    use_when: ["Opting into a supported higher-level gameplay subsystem", "Reusing its typed configuration and runtime helpers"],
    avoid_when: ["A small feature can be expressed more clearly with the underlying state and command APIs"],
    params: [],
    returns: None,
    example: "use sand::systems::damage;"
}

register! {
    path: "sand::resourcepack",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod resourcepack",
    summary: "Provides optional typed resource-pack authoring APIs.",
    context: "The feature-gated module accompanies HUD and texture macros with types that register generated assets alongside a Sand datapack project.",
    minecraft: "Writes resource-pack assets such as GUI textures and HUD definitions rather than datapack data resources.",
    use_when: ["Authoring a resource pack together with a Sand datapack", "Using the resourcepack feature's HUD or texture facilities"],
    avoid_when: ["Building a datapack-only project without client asset output"],
    params: [],
    returns: None,
    example: "use sand::resourcepack;",
    availability: ["Cargo feature: resourcepack"]
}

register! {
    path: "sand::all",
    aliases: ["sand::prelude::all"],
    module: "sand",
    kind: Macro,
    signature: "all![condition, ...]",
    summary: "Combines typed conditions that must all hold.",
    context: "The macro is concise syntax for Condition::all and preserves Sand's typed condition model when several guards must be true together.",
    minecraft: "Lowers to the execute-condition clauses needed for every supplied condition.",
    use_when: ["Guarding a command or branch with multiple required conditions"],
    avoid_when: ["Any alternative may pass; use any! instead"],
    params: [],
    returns: None,
    example: "let ready = sand::all![has_key, is_near_door];"
}

register! {
    path: "sand::any",
    aliases: ["sand::prelude::any"],
    module: "sand",
    kind: Macro,
    signature: "any![condition, ...]",
    summary: "Combines typed conditions where any alternative may hold.",
    context: "The macro is concise syntax for Condition::any and makes alternative guard branches explicit in the typed condition model.",
    minecraft: "Lowers each alternative into the execute-condition plan required to preserve OR behavior.",
    use_when: ["A command or branch should run when one of several typed conditions is true"],
    avoid_when: ["Every condition is required; use all! instead"],
    params: [],
    returns: None,
    example: "let visible = sand::any![is_day, has_night_vision];"
}

register! {
    path: "sand::mcfunction",
    aliases: ["sand::prelude::mcfunction"],
    module: "sand",
    kind: Macro,
    signature: "mcfunction![command; ...]",
    summary: "Collects command expressions into one Minecraft function body.",
    context: "The macro accepts displayable command builders and literal command strings in source order, making a short function body explicit without manual Vec assembly.",
    minecraft: "Produces the ordered command lines exported into a .mcfunction file by a surrounding Sand function or registration.",
    use_when: ["Passing several commands to a Sand builder that expects a command list", "Writing a compact function body from typed command expressions"],
    avoid_when: ["A single typed command builder is sufficient", "Using raw strings where a typed command builder exists"],
    params: [],
    returns: None,
    example: "let commands = sand::mcfunction![sand::command::say(\"Ready\");];"
}

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
    example: "#[sand::api(path = \"sand::feature\", summary = \"...\", context = \"...\", minecraft = \"...\", use_when = [\"...\"], avoid_when = [\"...\"], example = \"...\")]"
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
    example: "#[sand::function]\nfn welcome() { sand::command::say(\"Welcome\"); }"
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
    example: "#[sand::on_event]\nfn joined(event: sand::event::Event<OnJoinEvent>) { sand::command::say(event.player()); }"
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
    example: "#[sand::custom_item]\nfn compass() -> sand::item::CustomItem { sand::item::CustomItem::new(\"minecraft:compass\").custom_data(\"demo_compass\") }"
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
    example: "#[sand::armor_event(Equip, slot = Head, item = \"minecraft:diamond_helmet\")]\nfn helmet_equipped() { sand::command::say(\"Protected\"); }"
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
    example: "#[sand::schedule(ticks = 60, every = 5)]\nfn refresh() { sand::command::say(\"Refreshing\"); }"
}

register! {
    path: "sand::entity_archetype",
    aliases: ["sand::prelude::entity_archetype"],
    module: "sand",
    kind: Macro,
    signature: "#[entity_archetype]",
    summary: "Registers a typed entity-archetype factory.",
    context: "The attribute links an EntityArchetype definition to Sand's lifecycle registry so adoption, state initialization, and derived values are evaluated consistently.",
    minecraft: "Generates the functions and periodic checks required to maintain the declared archetype for loaded entities.",
    use_when: ["Declaring reusable behavior and state for a Minecraft entity kind"],
    avoid_when: ["Issuing a one-time selector command without archetype lifecycle behavior"],
    params: [],
    returns: None,
    example: "#[sand::entity_archetype]\nfn zombie() -> sand::entity::EntityArchetype<ZombieKind, Mob> { todo!() }"
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
    example: "let delayed = sand::run_fn! { sand::command::say(\"Later\") };"
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
