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

// Built-in event markers are a semantic family: each one is a stateless type
// consumed by `#[on_event]`, while its trigger/lifecycle meaning is declared
// here once for both the installed catalog and build-time enforcement. Keeping
// this provider beside the facade avoids duplicating an `#[api]` block beside
// every marker implementation in sand-core.
macro_rules! register_event_marker {
    (
        path: $path:literal,
        aliases: [$($alias:literal),* $(,)?],
        summary: $summary:literal,
        minecraft: $minecraft:literal,
        example: $example:literal
    ) => {
        register! {
            path: $path,
            aliases: [$($alias),*],
            module: "sand::events",
            kind: Struct,
            signature: "pub struct event marker",
            summary: $summary,
            context: "This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
            minecraft: $minecraft,
            use_when: ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
            avoid_when: ["Representing mutable event data; read typed handler context or declared participants instead"],
            params: [],
            returns: None,
            example: $example
        }
    };
}

// Event wrappers and trigger builders are implemented in sand-core, while the
// facade owns their supported identities.  Each invocation supplies the
// behavioral facts that differ between APIs; the shared framing keeps the
// catalog concise without inventing a second documentation source.
macro_rules! register_event_api {
    (
        path: $path:literal,
        aliases: [$($alias:literal),* $(,)?],
        module: $module:literal,
        kind: $kind:ident,
        signature: $signature:literal,
        summary: $summary:literal,
        minecraft: $minecraft:literal
    ) => {
        register! {
            path: $path,
            aliases: [$($alias),*],
            module: $module,
            kind: $kind,
            signature: $signature,
            summary: $summary,
            context: "This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
            minecraft: $minecraft,
            use_when: ["Defining, composing, or handling a typed Sand event"],
            avoid_when: ["Inspecting generated advancement or event-graph implementation state"],
            params: [],
            returns: None,
            example: "use sand::prelude::*;"
        }
    };
}

// Entity definitions are implemented in sand-core, while the facade owns the
// stable author paths. Each declaration below remains explicit so adding a new
// public member still fails closed; this shared frame keeps the catalog focused
// on the semantic entity model instead of repeating compiler-boundary prose.
macro_rules! register_entity_api {
    (
        path: $path:literal,
        aliases: [$($alias:literal),* $(,)?],
        kind: $kind:ident,
        summary: $summary:literal
    ) => {
        register! {
            path: $path,
            aliases: [$($alias),*],
            module: "sand::entity",
            kind: $kind,
            signature: "author-facing entity API",
            summary: $summary,
            context: "This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
            minecraft: "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
            use_when: ["Defining or using typed entity behavior in a Sand datapack"],
            avoid_when: ["Inspecting generated objectives, functions, or compiler lowering plans"],
            params: [],
            returns: None,
            example: "use sand::entity::*;"
        }
    };
}

macro_rules! register_state_api {
    (
        path: $path:literal,
        aliases: [$($alias:literal),* $(,)?],
        kind: $kind:ident,
        summary: $summary:literal
    ) => {
        register! {
            path: $path,
            aliases: [$($alias),*],
            module: "sand::state",
            kind: $kind,
            signature: "author-facing typed state API",
            summary: $summary,
            context: "This declaration provides the typed scoreboard or lifecycle primitives used directly and by #[derive(State)]; generated schema registration remains private.",
            minecraft: "Operations render validated scoreboard commands or conditions against the selected score holder, with lifecycle setup emitted at load when required.",
            use_when: ["Working with typed gameplay state or composing state transitions"],
            avoid_when: ["Manually reproducing metadata generated by #[derive(State)]"],
            params: [],
            returns: None,
            example: "use sand::state::*;"
        }
    };
}

// Participant contracts describe the lifecycle guarantees authors can rely
// on. Capture storage, generated cleanup functions, and exporter transport
// validation are deliberately outside this public semantic layer.
macro_rules! register_participant_api {
    (
        path: $path:literal,
        aliases: [$($alias:literal),* $(,)?],
        kind: $kind:ident,
        summary: $summary:literal
    ) => {
        register! {
            path: $path,
            aliases: [$($alias),*],
            module: "sand::participant",
            kind: $kind,
            signature: "author-facing typed event participant API",
            summary: $summary,
            context: "Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
            minecraft: "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
            use_when: ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
            avoid_when: ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
            params: [],
            returns: None,
            example: "use sand::participant::*;"
        }
    };
}

macro_rules! register_text_api {
    (
        path: $path:literal,
        aliases: [$($alias:literal),* $(,)?],
        kind: $kind:ident,
        summary: $summary:literal
    ) => {
        register! {
            path: $path,
            aliases: [$($alias),*],
            module: "sand::text",
            kind: $kind,
            signature: "typed Minecraft text component API",
            summary: $summary,
            context: "Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
            minecraft: "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
            use_when: ["Building player-visible text with typed styling or interactions"],
            avoid_when: ["Passing an unvalidated JSON string when a typed text component can express the same value"],
            params: [],
            returns: None,
            example: "let text = sand::text::Text::new(\"Ready\").gold();"
        }
    };
}

macro_rules! register_data_api {
    (
        path: $path:literal,
        aliases: [$($alias:literal),* $(,)?],
        kind: $kind:ident,
        summary: $summary:literal
    ) => {
        register! {
            path: $path,
            aliases: [$($alias),*],
            module: "sand::data",
            kind: $kind,
            signature: "typed Minecraft NBT and command-storage API",
            summary: $summary,
            context: "This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
            minecraft: "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
            use_when: ["Reading or mutating structured Minecraft NBT through typed paths and values"],
            avoid_when: ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
            params: [],
            returns: None,
            example: "use sand::data::{NbtPath, StorageLocation};"
        }
    };
}

macro_rules! register_systems_api {
    (
        path: $path:literal,
        aliases: [$($alias:literal),* $(,)?],
        kind: $kind:ident,
        summary: $summary:literal
    ) => {
        register! {
            path: $path,
            aliases: [$($alias),*],
            module: "sand::systems",
            kind: $kind,
            signature: "feature-gated author-facing gameplay system API",
            summary: $summary,
            context: "This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
            minecraft: "The configured system emits validated scoreboard, execute, item, effect, or entity commands according to the selected feature and target version.",
            use_when: ["Opting into this higher-level gameplay behavior instead of assembling its commands manually"],
            avoid_when: ["Inspecting compiler registries or generated lifecycle bookkeeping"],
            params: [],
            returns: None,
            example: "use sand::systems;"
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

register! { path: "sand::event::IntoEventId", aliases: [], module: "sand::event", kind: Trait, signature: "pub trait IntoEventId", summary: "Converts an event ID input into a validated Minecraft resource location.", context: "Sand implements this for ResourceLocation and ordinary text inputs so EventId has one validation boundary.", minecraft: "Validation enforces namespace:path syntax before an advancement resource is generated.", use_when: ["Accepting the same ergonomic inputs as EventId::explicit"], avoid_when: ["Defining an unrelated identifier conversion"], params: [], returns: None, example: "let id = sand::event::EventId::explicit(\"demo:events/join\");" }
register! { path: "sand::event::IntoEventId::into_event_resource_location", aliases: [], module: "sand::event", kind: Method, signature: "fn into_event_resource_location(self) -> ResourceLocation", summary: "Validates and returns this value as an event resource location.", context: "This is the common conversion seam behind explicit event IDs.", minecraft: "The result is valid namespace:path text for an advancement resource.", use_when: ["Implementing a supported EventId input"], avoid_when: ["Formatting unchecked resource names"], params: [], returns: Some("A validated Minecraft resource location."), example: "let id = sand::event::EventId::explicit(\"demo:events/join\");" }
register! { path: "sand::event::AdvancementEvent", aliases: ["sand::prelude::AdvancementEvent"], module: "sand::event", kind: Trait, signature: "pub trait AdvancementEvent", summary: "Defines a custom event backed by one Minecraft advancement trigger.", context: "Implement this stateless trait for the common custom-event case, then receive Event<Self> in an #[on_event] handler.", minecraft: "Sand exports one advancement criterion and reward function, applying reset, guard, and participant behavior.", use_when: ["A custom event maps directly to one vanilla advancement trigger"], avoid_when: ["The event needs tick polling, composition, or lifecycle setup; implement SandEvent instead"], params: [], returns: None, example: "struct AteApple;\nimpl sand::event::AdvancementEvent for AteApple { type Trigger = sand::component::AdvancementTrigger; fn trigger() -> Self::Trigger { sand::component::AdvancementTrigger::Tick } }" }
register! { path: "sand::event::AdvancementEvent::Trigger", aliases: ["sand::prelude::AdvancementEvent::Trigger"], module: "sand::event", kind: AssociatedType, signature: "type Trigger: Into<AdvancementTrigger>", summary: "Names the typed vanilla trigger emitted for an event.", context: "The associated trigger keeps a custom event definition tied to Sand's validated trigger model.", minecraft: "It serializes into the advancement criterion Minecraft watches.", use_when: ["Implementing AdvancementEvent"], avoid_when: ["Handling an existing event"], params: [], returns: None, example: "type Trigger = sand::component::AdvancementTrigger;" }
register! { path: "sand::event::AdvancementEvent::trigger", aliases: ["sand::prelude::AdvancementEvent::trigger"], module: "sand::event", kind: Method, signature: "fn trigger() -> Self::Trigger", summary: "Returns the criterion Minecraft watches for this event.", context: "Sand calls it during export to construct advancement JSON.", minecraft: "The result becomes the event advancement's criterion conditions.", use_when: ["Implementing a custom advancement-backed event"], avoid_when: ["Inspecting a built-in event's export internals"], params: [], returns: Some("The event's typed advancement trigger."), example: "fn trigger() -> Self::Trigger { sand::component::AdvancementTrigger::Tick }" }
register! { path: "sand::event::AdvancementEvent::id", aliases: ["sand::prelude::AdvancementEvent::id"], module: "sand::event", kind: Method, signature: "fn id() -> EventId", summary: "Selects the generated advancement resource ID.", context: "The default follows the event handler path, avoiding manually synchronized names.", minecraft: "Controls the exported advancement namespace:path and revoke target.", use_when: ["A custom event needs a stable non-default ID"], avoid_when: ["The generated handler path is sufficient"], params: [], returns: Some("The event ID policy, Auto by default."), example: "fn id() -> sand::event::EventId { sand::event::EventId::Auto }" }
register! { path: "sand::event::AdvancementEvent::reset", aliases: ["sand::prelude::AdvancementEvent::reset"], module: "sand::event", kind: Method, signature: "fn reset() -> EventReset", summary: "Selects how advancement grant state re-arms after this event fires.", context: "The default supports repeating triggers while milestones can retain their grant.", minecraft: "AfterFire emits an advancement revoke for the triggering player.", use_when: ["Making an event one-shot or manually resettable"], avoid_when: ["Defining a tick-polled SandEvent"], params: [], returns: Some("The event reset policy, AfterFire by default."), example: "fn reset() -> sand::event::EventReset { sand::event::EventReset::OncePerPlayer }" }
register! { path: "sand::event::AdvancementEvent::visibility", aliases: ["sand::prelude::AdvancementEvent::visibility"], module: "sand::event", kind: Method, signature: "fn visibility() -> EventVisibility", summary: "Selects the intended announcement visibility for this event.", context: "The default keeps mechanical event advancements silent.", minecraft: "Carries the display policy into advancement export.", use_when: ["A custom event should request advancement-style visibility"], avoid_when: ["Sending a handler-authored message"], params: [], returns: Some("The event visibility policy, Hidden by default."), example: "fn visibility() -> sand::event::EventVisibility { sand::event::EventVisibility::Hidden }" }
register! { path: "sand::event::AdvancementEvent::guard", aliases: ["sand::prelude::AdvancementEvent::guard"], module: "sand::event", kind: Method, signature: "fn guard() -> Option<Condition>", summary: "Adds an optional typed condition checked before the handler runs.", context: "This complements, rather than replaces, the advancement trigger's own criterion conditions.", minecraft: "Sand emits an execute-unless guard that returns before user commands when it fails.", use_when: ["A score, entity, or predicate condition must gate a broad trigger"], avoid_when: ["The trigger itself can express the required criterion"], params: [], returns: Some("A guard condition, or None for no extra guard."), example: "fn guard() -> Option<sand::condition::Condition> { None }" }
register! { path: "sand::event::AdvancementEvent::state_defines", aliases: ["sand::prelude::AdvancementEvent::state_defines"], module: "sand::event", kind: Method, signature: "fn state_defines() -> Vec<String>", summary: "Lists initialization commands required by event-owned state.", context: "This is the low-level seam for a custom event that owns scoreboard-backed variables.", minecraft: "The commands normally create objectives before the event can fire.", use_when: ["A custom event explicitly owns score or timer setup"], avoid_when: ["Derived State or ordinary load setup owns the state"], params: [], returns: Some("Commands that initialize the event's declared state."), example: "fn state_defines() -> Vec<String> { Vec::new() }" }
register! { path: "sand::event::AdvancementEvent::participants", aliases: ["sand::prelude::AdvancementEvent::participants"], module: "sand::event", kind: Method, signature: "fn participants() -> EventParticipantPlan", summary: "Declares the event-time observations this event can expose.", context: "The plan is applied around generated handler execution, giving context access an explicit evidence and cleanup model.", minecraft: "Sand emits the storage observations and cleanup around the advancement reward function.", use_when: ["A custom event needs typed attacker, weapon, or other participant context"], avoid_when: ["The handler only needs its triggering player"], params: [], returns: Some("The participant observation plan, empty by default."), example: "fn participants() -> sand::participant::EventParticipantPlan { sand::participant::EventParticipantPlan::none() }" }

register_event_api! { path: "sand::event::handle", aliases: [], module: "sand::event", kind: Module, signature: "pub mod handle", summary: "Contains typed handles for controlling registered advancement events.", minecraft: "Its operations render advancement grant and revoke commands against a selected player." }
register_event_api! { path: "sand::event::trigger", aliases: [], module: "sand::event", kind: Module, signature: "pub mod trigger", summary: "Contains typed builders for vanilla advancement trigger criteria.", minecraft: "Each builder serializes into an advancement criterion rather than executing a command immediately." }
register_event_api! { path: "sand::event::vanilla", aliases: [], module: "sand::event", kind: Module, signature: "pub mod vanilla", summary: "Provides short aliases for Sand's built-in advancement-backed event markers.", minecraft: "The aliases select the same generated event and advancement behavior as their canonical sand::events markers." }

register_event_api! { path: "sand::event::Event", aliases: ["sand::prelude::Event"], module: "sand::event", kind: Struct, signature: "pub struct Event<E>", summary: "Provides typed context for one advancement-backed event dispatch.", minecraft: "It runs in the triggering player's advancement reward function and resolves only participants declared by that event." }
register_event_api! { path: "sand::event::Event::context", aliases: ["sand::prelude::Event::context"], module: "sand::event", kind: Method, signature: "fn context() -> Event<E>", summary: "Constructs the zero-sized typed context passed to an advancement event handler.", minecraft: "The context reads the execution player and participant observations stored around the generated reward function." }
register_event_api! { path: "sand::event::Event::player", aliases: ["sand::prelude::Event::player"], module: "sand::event", kind: Method, signature: "fn player(&self) -> Selector", summary: "Selects the player that triggered this advancement event.", minecraft: "Resolves to the reward function's @s player, not an arbitrary entity." }
register_event_api! { path: "sand::event::Event::subject", aliases: ["sand::prelude::Event::subject"], module: "sand::event", kind: Method, signature: "fn subject(&self) -> Selector", summary: "Returns the event's execution subject selector.", minecraft: "For advancement events this is the triggering player bound to @s." }
register_event_api! { path: "sand::event::Event::state_init", aliases: ["sand::prelude::Event::state_init"], module: "sand::event", kind: Method, signature: "fn state_init() -> Vec<String>", summary: "Returns setup commands declared by an advancement event definition.", minecraft: "The commands normally create scoreboard objectives before generated event resources execute." }
register_event_api! { path: "sand::event::Event::entity", aliases: ["sand::prelude::Event::entity"], module: "sand::event", kind: Method, signature: "fn entity(&self, role: EntityParticipantRole) -> EntityParticipant", summary: "Reads an entity participant observed for this dispatch role.", minecraft: "The value is backed by Sand's event-cycle observation storage and is only available when the event declared that role." }
register_event_api! { path: "sand::event::Event::item", aliases: ["sand::prelude::Event::item"], module: "sand::event", kind: Method, signature: "fn item(&self, role: ItemParticipantRole) -> ItemSnapshot", summary: "Reads an item snapshot observed for this dispatch role.", minecraft: "Sand captures matching item NBT at trigger time so later commands do not depend on a live inventory slot." }
register_event_api! { path: "sand::event::Event::attacker", aliases: ["sand::prelude::Event::attacker"], module: "sand::event", kind: Method, signature: "fn attacker(&self) -> EntityParticipant", summary: "Returns the evidence-backed attacker participant when declared.", minecraft: "The participant is available only for triggers and plans that can observe an attacker." }
register_event_api! { path: "sand::event::Event::killer", aliases: ["sand::prelude::Event::killer"], module: "sand::event", kind: Method, signature: "fn killer(&self) -> EntityParticipant", summary: "Returns the evidence-backed killer participant when declared.", minecraft: "It is read from the current event dispatch record, not looked up again after the kill." }
register_event_api! { path: "sand::event::Event::victim", aliases: ["sand::prelude::Event::victim"], module: "sand::event", kind: Method, signature: "fn victim(&self) -> EntityParticipant", summary: "Returns the victim participant observed for this event.", minecraft: "The value follows the event's participant plan and dispatch-cycle lifetime." }
register_event_api! { path: "sand::event::Event::interacted_entity", aliases: ["sand::prelude::Event::interacted_entity"], module: "sand::event", kind: Method, signature: "fn interacted_entity(&self) -> EntityParticipant", summary: "Returns the entity captured by an interaction event.", minecraft: "It reflects the entity matched by the vanilla interaction trigger when that observation is declared." }
register_event_api! { path: "sand::event::Event::weapon", aliases: ["sand::prelude::Event::weapon"], module: "sand::event", kind: Method, signature: "fn weapon(&self) -> ItemSnapshot", summary: "Returns the weapon snapshot captured for a combat event.", minecraft: "Sand stores the trigger-time item NBT so it remains stable through the handler." }
register_event_api! { path: "sand::event::Event::bounded_item", aliases: ["sand::prelude::Event::bounded_item"], module: "sand::event", kind: Method, signature: "fn bounded_item(&self, role: ItemParticipantRole) -> BoundedItemSnapshot", summary: "Reads an item observation retained for a bounded correlation window.", minecraft: "The snapshot is available only while the event graph's configured tick window has not expired." }
register_event_api! { path: "sand::event::Event::damage", aliases: ["sand::prelude::Event::damage"], module: "sand::event", kind: Method, signature: "fn damage(&self) -> Damage", summary: "Builds the typed damage view for a damage-backed event.", minecraft: "It reflects the damage data exposed by Minecraft's advancement trigger context." }
register_event_api! { path: "sand::event::DamageAdvancementEvent", aliases: ["sand::prelude::DamageAdvancementEvent"], module: "sand::event", kind: Trait, signature: "pub trait DamageAdvancementEvent", summary: "Marks an advancement event whose handler receives typed damage context.", minecraft: "It is used with vanilla damage triggers and does not synthesize damage data for unrelated events." }
register_event_api! { path: "sand::event::DamageEvent", aliases: ["sand::prelude::DamageEvent"], module: "sand::event", kind: Struct, signature: "pub struct DamageEvent<E>", summary: "Provides the specialized player and damage context for a damage advancement event.", minecraft: "Runs in the triggering player's advancement reward function and reflects the captured vanilla damage predicate context." }
register_event_api! { path: "sand::event::DamageEvent::context", aliases: ["sand::prelude::DamageEvent::context"], module: "sand::event", kind: Method, signature: "fn context() -> DamageEvent<E>", summary: "Constructs typed context for a damage event handler.", minecraft: "The value resolves its subject and damage view in the generated reward function." }
register_event_api! { path: "sand::event::DamageEvent::player", aliases: ["sand::prelude::DamageEvent::player"], module: "sand::event", kind: Method, signature: "fn player(&self) -> SinglePlayer", summary: "Selects the player whose advancement damage trigger fired.", minecraft: "Always resolves the advancement reward function's @s player." }
register_event_api! { path: "sand::event::DamageEvent::subject", aliases: ["sand::prelude::DamageEvent::subject"], module: "sand::event", kind: Method, signature: "fn subject(&self) -> SingleEntity", summary: "Returns the exact execution subject for a damage event.", minecraft: "It is the triggering player under Minecraft's advancement reward execution." }
register_event_api! { path: "sand::event::DamageEvent::reflect_damage", aliases: ["sand::prelude::DamageEvent::reflect_damage"], module: "sand::event", kind: Method, signature: "fn reflect_damage(&self) -> Damage", summary: "Builds a typed damage command using this event's trigger context.", minecraft: "Uses the same damage source information Sand can reflect from the advancement-backed event." }

register_event_api! { path: "sand::event::handle::EventHandle", aliases: ["sand::prelude::EventHandle"], module: "sand::event", kind: Struct, signature: "pub struct EventHandle<E>", summary: "References a registered advancement event without exposing its exporter state.", minecraft: "Renders grant, revoke, and criterion checks against the generated advancement resource for E." }
register_event_api! { path: "sand::event::handle::EventHandle::new", aliases: ["sand::prelude::EventHandle::new"], module: "sand::event", kind: Method, signature: "const fn new() -> EventHandle<E>", summary: "Creates a typed handle for an advancement event marker.", minecraft: "Creation has no runtime effect; commands are emitted only when a handle operation is used." }
register_event_api! { path: "sand::event::handle::EventHandle::define", aliases: ["sand::prelude::EventHandle::define"], module: "sand::event", kind: Method, signature: "fn define(&self) -> String", summary: "Returns the generated advancement resource ID for this event.", minecraft: "The string names the advancement Sand exports for E." }
register_event_api! { path: "sand::event::handle::EventHandle::condition", aliases: ["sand::prelude::EventHandle::condition"], module: "sand::event", kind: Method, signature: "fn condition(&self) -> Condition", summary: "Builds a condition that tests whether this event advancement is granted.", minecraft: "Renders an advancement criterion condition for the executing player." }
register_event_api! { path: "sand::event::handle::EventHandle::enable", aliases: ["sand::prelude::EventHandle::enable"], module: "sand::event", kind: Method, signature: "fn enable(&self, selector: impl Display) -> String", summary: "Builds a command that grants this event advancement to selected players.", minecraft: "Uses advancement grant and therefore suppresses a trigger until a reset/revoke occurs." }
register_event_api! { path: "sand::event::handle::EventHandle::disable", aliases: ["sand::prelude::EventHandle::disable"], module: "sand::event", kind: Method, signature: "fn disable(&self, selector: impl Display) -> String", summary: "Builds a command that revokes this event advancement from selected players.", minecraft: "Revoke re-arms a repeating advancement trigger for those players." }
register_event_api! { path: "sand::event::handle::EventHandle::revoke", aliases: ["sand::prelude::EventHandle::revoke"], module: "sand::event", kind: Method, signature: "fn revoke(&self, selector: impl Display) -> String", summary: "Builds the explicit re-arm command for this event.", minecraft: "Renders advancement revoke for each selected player." }
register_event_api! { path: "sand::event::handle::EventHandle::reset", aliases: ["sand::prelude::EventHandle::reset"], module: "sand::event", kind: Method, signature: "fn reset(&self, selector: impl Display) -> String", summary: "Builds the legacy-named explicit re-arm command for this event.", minecraft: "It emits the same advancement revoke operation as revoke." }
register_event_api! { path: "sand::event::handle::EventHandle::grant", aliases: ["sand::prelude::EventHandle::grant"], module: "sand::event", kind: Method, signature: "fn grant(&self, selector: impl Display) -> String", summary: "Builds a command that marks this event advancement granted.", minecraft: "Grant affects Minecraft's advancement state and is useful for deliberate one-shot lifecycle control." }

// Typed trigger builders keep custom AdvancementEvent definitions in the
// vanilla criterion vocabulary.  `new` starts an unconstrained criterion,
// predicate setters narrow it, and `build` hands the result to the event.
register_event_api! { path: "sand::event::trigger::TickTrigger", aliases: ["sand::prelude::TickTrigger"], module: "sand::event", kind: Struct, signature: "pub struct TickTrigger", summary: "Builds Minecraft's always-true tick advancement trigger.", minecraft: "The criterion is checked by Minecraft every tick and normally needs an explicit guard to avoid unconditional dispatch." }
register_event_api! { path: "sand::event::trigger::TickTrigger::new", aliases: ["sand::prelude::TickTrigger::new"], module: "sand::event", kind: Method, signature: "fn new() -> TickTrigger", summary: "Starts an unconstrained tick trigger builder.", minecraft: "The resulting criterion matches each player tick." }
register_event_api! { path: "sand::event::trigger::TickTrigger::build", aliases: ["sand::prelude::TickTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts this tick builder into an advancement criterion.", minecraft: "Serializes as Minecraft's tick trigger." }
register_event_api! { path: "sand::event::trigger::ImpossibleTrigger", aliases: ["sand::prelude::ImpossibleTrigger"], module: "sand::event", kind: Struct, signature: "pub struct ImpossibleTrigger", summary: "Builds Minecraft's deliberately never-matching advancement trigger.", minecraft: "The criterion never fires without explicit advancement grant control." }
register_event_api! { path: "sand::event::trigger::ImpossibleTrigger::new", aliases: ["sand::prelude::ImpossibleTrigger::new"], module: "sand::event", kind: Method, signature: "fn new() -> ImpossibleTrigger", summary: "Starts a never-matching trigger builder.", minecraft: "It has no matching vanilla action." }
register_event_api! { path: "sand::event::trigger::ImpossibleTrigger::build", aliases: ["sand::prelude::ImpossibleTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts this never-matching builder into an advancement criterion.", minecraft: "Serializes as Minecraft's impossible trigger." }
register_event_api! { path: "sand::event::trigger::ConsumeItemTrigger", aliases: ["sand::prelude::ConsumeItemTrigger"], module: "sand::event", kind: Struct, signature: "pub struct ConsumeItemTrigger", summary: "Builds a criterion for a player consuming an item.", minecraft: "Uses minecraft:consume_item and can constrain the consumed item with an item predicate." }
register_event_api! { path: "sand::event::trigger::ConsumeItemTrigger::new", aliases: ["sand::prelude::ConsumeItemTrigger::new"], module: "sand::event", kind: Method, signature: "fn new() -> ConsumeItemTrigger", summary: "Starts an unconstrained consume-item criterion.", minecraft: "Matches any item consumption until narrowed with item." }
register_event_api! { path: "sand::event::trigger::ConsumeItemTrigger::item", aliases: ["sand::prelude::ConsumeItemTrigger::item"], module: "sand::event", kind: Method, signature: "fn item(self, predicate: ItemPredicate) -> Self", summary: "Narrows a consume-item event to a typed item predicate.", minecraft: "Minecraft evaluates the predicate against the consumed stack." }
register_event_api! { path: "sand::event::trigger::ConsumeItemTrigger::build", aliases: ["sand::prelude::ConsumeItemTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts the consume-item builder into an advancement criterion.", minecraft: "Serializes the selected consume-item conditions into advancement JSON." }
register_event_api! { path: "sand::event::trigger::PlayerKilledEntityTrigger", aliases: ["sand::prelude::PlayerKilledEntityTrigger"], module: "sand::event", kind: Struct, signature: "pub struct PlayerKilledEntityTrigger", summary: "Builds a criterion for a player killing an entity.", minecraft: "Uses minecraft:player_killed_entity with optional victim and killing-blow predicates." }
register_event_api! { path: "sand::event::trigger::PlayerKilledEntityTrigger::new", aliases: ["sand::prelude::PlayerKilledEntityTrigger::new"], module: "sand::event", kind: Method, signature: "fn new() -> PlayerKilledEntityTrigger", summary: "Starts an unconstrained player-killed-entity criterion.", minecraft: "Matches any entity killed by the triggering player." }
register_event_api! { path: "sand::event::trigger::PlayerKilledEntityTrigger::entity", aliases: ["sand::prelude::PlayerKilledEntityTrigger::entity"], module: "sand::event", kind: Method, signature: "fn entity(self, predicate: EntityPredicate) -> Self", summary: "Narrows a player kill criterion to a victim predicate.", minecraft: "Minecraft evaluates the predicate against the killed entity." }
register_event_api! { path: "sand::event::trigger::PlayerKilledEntityTrigger::killing_blow", aliases: ["sand::prelude::PlayerKilledEntityTrigger::killing_blow"], module: "sand::event", kind: Method, signature: "fn killing_blow(self, predicate: DamagePredicate) -> Self", summary: "Narrows a player kill criterion by its damage source.", minecraft: "Minecraft evaluates the killing blow predicate when the victim dies." }
register_event_api! { path: "sand::event::trigger::PlayerKilledEntityTrigger::build", aliases: ["sand::prelude::PlayerKilledEntityTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts the player-kill builder into an advancement criterion.", minecraft: "Serializes player_killed_entity conditions into advancement JSON." }
register_event_api! { path: "sand::event::trigger::EntityKilledPlayerTrigger", aliases: ["sand::prelude::EntityKilledPlayerTrigger"], module: "sand::event", kind: Struct, signature: "pub struct EntityKilledPlayerTrigger", summary: "Builds a criterion for an entity killing the triggering player.", minecraft: "Uses minecraft:entity_killed_player with optional killer and damage predicates." }
register_event_api! { path: "sand::event::trigger::EntityKilledPlayerTrigger::new", aliases: ["sand::prelude::EntityKilledPlayerTrigger::new"], module: "sand::event", kind: Method, signature: "fn new() -> EntityKilledPlayerTrigger", summary: "Starts an unconstrained entity-killed-player criterion.", minecraft: "Matches any death of the triggering player caused by an entity." }
register_event_api! { path: "sand::event::trigger::EntityKilledPlayerTrigger::entity", aliases: ["sand::prelude::EntityKilledPlayerTrigger::entity"], module: "sand::event", kind: Method, signature: "fn entity(self, predicate: EntityPredicate) -> Self", summary: "Narrows an entity-killed-player criterion to a killer predicate.", minecraft: "Minecraft evaluates the predicate against the killing entity." }
register_event_api! { path: "sand::event::trigger::EntityKilledPlayerTrigger::killing_blow", aliases: ["sand::prelude::EntityKilledPlayerTrigger::killing_blow"], module: "sand::event", kind: Method, signature: "fn killing_blow(self, predicate: DamagePredicate) -> Self", summary: "Narrows a player-death criterion by its killing blow.", minecraft: "Minecraft evaluates the supplied damage predicate at death." }
register_event_api! { path: "sand::event::trigger::EntityKilledPlayerTrigger::build", aliases: ["sand::prelude::EntityKilledPlayerTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts the entity-killed-player builder into an advancement criterion.", minecraft: "Serializes entity_killed_player conditions into advancement JSON." }
register_event_api! { path: "sand::event::trigger::RecipeUnlockedTrigger", aliases: ["sand::prelude::RecipeUnlockedTrigger"], module: "sand::event", kind: Struct, signature: "pub struct RecipeUnlockedTrigger", summary: "Builds a criterion for a player unlocking a recipe.", minecraft: "Uses minecraft:recipe_unlocked for the specified recipe resource location." }
register_event_api! { path: "sand::event::trigger::RecipeUnlockedTrigger::new", aliases: ["sand::prelude::RecipeUnlockedTrigger::new"], module: "sand::event", kind: Method, signature: "fn new(recipe: impl Into<String>) -> RecipeUnlockedTrigger", summary: "Starts a recipe-unlocked criterion from recipe text.", minecraft: "The recipe identifier is written into the vanilla advancement condition." }
register_event_api! { path: "sand::event::trigger::RecipeUnlockedTrigger::from_id", aliases: ["sand::prelude::RecipeUnlockedTrigger::from_id"], module: "sand::event", kind: Method, signature: "fn from_id(recipe: ResourceLocation) -> RecipeUnlockedTrigger", summary: "Starts a recipe-unlocked criterion from a validated resource location.", minecraft: "Uses the exact namespace:path recipe ID in advancement JSON." }
register_event_api! { path: "sand::event::trigger::RecipeUnlockedTrigger::build", aliases: ["sand::prelude::RecipeUnlockedTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts the recipe-unlocked builder into an advancement criterion.", minecraft: "Serializes minecraft:recipe_unlocked with its recipe ID." }
register_event_api! { path: "sand::event::trigger::InventoryChangedTrigger", aliases: ["sand::prelude::InventoryChangedTrigger"], module: "sand::event", kind: Struct, signature: "pub struct InventoryChangedTrigger", summary: "Builds a criterion for a player's inventory changing.", minecraft: "Uses minecraft:inventory_changed and can constrain occupied slots or a matching stack." }
register_event_api! { path: "sand::event::trigger::InventoryChangedTrigger::new", aliases: ["sand::prelude::InventoryChangedTrigger::new"], module: "sand::event", kind: Method, signature: "fn new() -> InventoryChangedTrigger", summary: "Starts an unconstrained inventory-change criterion.", minecraft: "Matches inventory changes until predicates narrow it." }
register_event_api! { path: "sand::event::trigger::InventoryChangedTrigger::slots", aliases: ["sand::prelude::InventoryChangedTrigger::slots"], module: "sand::event", kind: Method, signature: "fn slots(self, slots: InventorySlotsPredicate) -> Self", summary: "Narrows an inventory-change event by occupied-slot counts.", minecraft: "Minecraft evaluates the supplied slots predicate when inventory state changes." }
register_event_api! { path: "sand::event::trigger::InventoryChangedTrigger::item", aliases: ["sand::prelude::InventoryChangedTrigger::item"], module: "sand::event", kind: Method, signature: "fn item(self, predicate: ItemPredicate) -> Self", summary: "Narrows an inventory-change event to a matching item stack.", minecraft: "Minecraft evaluates the predicate against changed inventory contents." }
register_event_api! { path: "sand::event::trigger::InventoryChangedTrigger::build", aliases: ["sand::prelude::InventoryChangedTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts the inventory-change builder into an advancement criterion.", minecraft: "Serializes inventory_changed conditions into advancement JSON." }
register_event_api! { path: "sand::event::trigger::ItemObtainedTrigger", aliases: ["sand::prelude::ItemObtainedTrigger"], module: "sand::event", kind: Struct, signature: "pub struct ItemObtainedTrigger", summary: "Builds a criterion for a player obtaining an item.", minecraft: "Uses minecraft:inventory_changed with a matching item condition." }
register_event_api! { path: "sand::event::trigger::ItemObtainedTrigger::new", aliases: ["sand::prelude::ItemObtainedTrigger::new"], module: "sand::event", kind: Method, signature: "fn new() -> ItemObtainedTrigger", summary: "Starts an unconstrained item-obtained criterion.", minecraft: "Matches inventory observations until narrowed with item." }
register_event_api! { path: "sand::event::trigger::ItemObtainedTrigger::item", aliases: ["sand::prelude::ItemObtainedTrigger::item"], module: "sand::event", kind: Method, signature: "fn item(self, predicate: ItemPredicate) -> Self", summary: "Narrows an item-obtained event to a typed item predicate.", minecraft: "The predicate is emitted as an inventory_changed item condition." }
register_event_api! { path: "sand::event::trigger::ItemObtainedTrigger::build", aliases: ["sand::prelude::ItemObtainedTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts the item-obtained builder into an advancement criterion.", minecraft: "Serializes the inventory_changed criterion used for item acquisition." }
register_event_api! { path: "sand::event::trigger::ItemEnchantTrigger", aliases: ["sand::prelude::ItemEnchantTrigger"], module: "sand::event", kind: Struct, signature: "pub struct ItemEnchantTrigger", summary: "Builds a criterion for enchanting an item.", minecraft: "Uses minecraft:enchanted_item with optional item and experience-level constraints." }
register_event_api! { path: "sand::event::trigger::ItemEnchantTrigger::new", aliases: ["sand::prelude::ItemEnchantTrigger::new"], module: "sand::event", kind: Method, signature: "fn new() -> ItemEnchantTrigger", summary: "Starts an unconstrained item-enchantment criterion.", minecraft: "Matches any successful enchantment until predicates narrow it." }
register_event_api! { path: "sand::event::trigger::ItemEnchantTrigger::item", aliases: ["sand::prelude::ItemEnchantTrigger::item"], module: "sand::event", kind: Method, signature: "fn item(self, predicate: ItemPredicate) -> Self", summary: "Narrows an enchantment event to a typed item predicate.", minecraft: "Minecraft evaluates the stack being enchanted." }
register_event_api! { path: "sand::event::trigger::ItemEnchantTrigger::levels", aliases: ["sand::prelude::ItemEnchantTrigger::levels"], module: "sand::event", kind: Method, signature: "fn levels(self, levels: IntRange) -> Self", summary: "Narrows an enchantment event by spent experience levels.", minecraft: "Minecraft evaluates the advancement levels range for the enchantment." }
register_event_api! { path: "sand::event::trigger::ItemEnchantTrigger::build", aliases: ["sand::prelude::ItemEnchantTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts the enchantment builder into an advancement criterion.", minecraft: "Serializes enchanted_item conditions into advancement JSON." }
register_event_api! { path: "sand::event::trigger::UsingItemTrigger", aliases: ["sand::prelude::UsingItemTrigger"], module: "sand::event", kind: Struct, signature: "pub struct UsingItemTrigger", summary: "Builds a criterion for a player starting to use an item.", minecraft: "Uses minecraft:using_item with an optional item predicate." }
register_event_api! { path: "sand::event::trigger::UsingItemTrigger::new", aliases: ["sand::prelude::UsingItemTrigger::new"], module: "sand::event", kind: Method, signature: "fn new() -> UsingItemTrigger", summary: "Starts an unconstrained using-item criterion.", minecraft: "Matches item-use observations until narrowed with item." }
register_event_api! { path: "sand::event::trigger::UsingItemTrigger::item", aliases: ["sand::prelude::UsingItemTrigger::item"], module: "sand::event", kind: Method, signature: "fn item(self, predicate: ItemPredicate) -> Self", summary: "Narrows item use to a typed item predicate.", minecraft: "Minecraft evaluates the item currently being used." }
register_event_api! { path: "sand::event::trigger::UsingItemTrigger::build", aliases: ["sand::prelude::UsingItemTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts the using-item builder into an advancement criterion.", minecraft: "Serializes using_item conditions into advancement JSON." }
register_event_api! { path: "sand::event::trigger::MultiKillTrigger", aliases: ["sand::prelude::MultiKillTrigger"], module: "sand::event", kind: Struct, signature: "pub struct MultiKillTrigger", summary: "Builds a criterion for killing multiple entity types in one combat sequence.", minecraft: "Uses minecraft:player_killed_entity with unique-entity-type and victim constraints." }
register_event_api! { path: "sand::event::trigger::MultiKillTrigger::new", aliases: ["sand::prelude::MultiKillTrigger::new"], module: "sand::event", kind: Method, signature: "fn new() -> MultiKillTrigger", summary: "Starts an unconstrained multi-kill criterion.", minecraft: "Matches vanilla kill progress until its range or victim predicate is set." }
register_event_api! { path: "sand::event::trigger::MultiKillTrigger::unique_entity_types", aliases: ["sand::prelude::MultiKillTrigger::unique_entity_types"], module: "sand::event", kind: Method, signature: "fn unique_entity_types(self, count: IntRange) -> Self", summary: "Constrains a multi-kill criterion by distinct entity-type count.", minecraft: "The range is serialized into the trigger's unique entity type condition." }
register_event_api! { path: "sand::event::trigger::MultiKillTrigger::victim", aliases: ["sand::prelude::MultiKillTrigger::victim"], module: "sand::event", kind: Method, signature: "fn victim(self, predicate: EntityPredicate) -> Self", summary: "Constrains a multi-kill criterion by victim entity predicate.", minecraft: "Minecraft evaluates the predicate for each qualifying kill." }
register_event_api! { path: "sand::event::trigger::MultiKillTrigger::build", aliases: ["sand::prelude::MultiKillTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts the multi-kill builder into an advancement criterion.", minecraft: "Serializes the kill-progress conditions into advancement JSON." }
register_event_api! { path: "sand::event::trigger::PlayerInteractedWithEntityTrigger", aliases: ["sand::prelude::PlayerInteractedWithEntityTrigger"], module: "sand::event", kind: Struct, signature: "pub struct PlayerInteractedWithEntityTrigger", summary: "Builds a criterion for a player interacting with an entity.", minecraft: "Uses minecraft:player_interacted_with_entity with optional held-item and target predicates." }
register_event_api! { path: "sand::event::trigger::PlayerInteractedWithEntityTrigger::new", aliases: ["sand::prelude::PlayerInteractedWithEntityTrigger::new"], module: "sand::event", kind: Method, signature: "fn new() -> PlayerInteractedWithEntityTrigger", summary: "Starts an unconstrained player-interaction criterion.", minecraft: "Matches any interaction with an entity until narrowed." }
register_event_api! { path: "sand::event::trigger::PlayerInteractedWithEntityTrigger::item", aliases: ["sand::prelude::PlayerInteractedWithEntityTrigger::item"], module: "sand::event", kind: Method, signature: "fn item(self, predicate: ItemPredicate) -> Self", summary: "Constrains an interaction by the item used.", minecraft: "Minecraft evaluates the held interaction stack against the predicate." }
register_event_api! { path: "sand::event::trigger::PlayerInteractedWithEntityTrigger::entity", aliases: ["sand::prelude::PlayerInteractedWithEntityTrigger::entity"], module: "sand::event", kind: Method, signature: "fn entity(self, predicate: EntityPredicate) -> Self", summary: "Constrains an interaction by its target entity.", minecraft: "Minecraft evaluates the target entity against the predicate." }
register_event_api! { path: "sand::event::trigger::PlayerInteractedWithEntityTrigger::build", aliases: ["sand::prelude::PlayerInteractedWithEntityTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts the interaction builder into an advancement criterion.", minecraft: "Serializes player_interacted_with_entity conditions into advancement JSON." }
register_event_api! { path: "sand::event::trigger::SummonedEntityTrigger", aliases: ["sand::prelude::SummonedEntityTrigger"], module: "sand::event", kind: Struct, signature: "pub struct SummonedEntityTrigger", summary: "Builds a criterion for a player summoning an entity.", minecraft: "Uses minecraft:summoned_entity with an optional entity predicate." }
register_event_api! { path: "sand::event::trigger::SummonedEntityTrigger::new", aliases: ["sand::prelude::SummonedEntityTrigger::new"], module: "sand::event", kind: Method, signature: "fn new() -> SummonedEntityTrigger", summary: "Starts an unconstrained summoned-entity criterion.", minecraft: "Matches a player's entity summons until narrowed." }
register_event_api! { path: "sand::event::trigger::SummonedEntityTrigger::entity", aliases: ["sand::prelude::SummonedEntityTrigger::entity"], module: "sand::event", kind: Method, signature: "fn entity(self, predicate: EntityPredicate) -> Self", summary: "Constrains summons to a typed entity predicate.", minecraft: "Minecraft evaluates the summoned entity against the predicate." }
register_event_api! { path: "sand::event::trigger::SummonedEntityTrigger::build", aliases: ["sand::prelude::SummonedEntityTrigger::build"], module: "sand::event", kind: Method, signature: "fn build(self) -> AdvancementTrigger", summary: "Converts the summoned-entity builder into an advancement criterion.", minecraft: "Serializes summoned_entity conditions into advancement JSON." }

register_event_api! { path: "sand::events", aliases: [], module: "sand", kind: Module, signature: "pub mod events", summary: "Defines custom Sand event dispatch, composition, and built-in event markers.", minecraft: "Sand lowers these typed definitions into advancements, tick functions, and bounded event-state storage." }
register_event_api! { path: "sand::events::TickScope", aliases: [], module: "sand::events", kind: Enum, signature: "pub enum TickScope", summary: "States the execution subject available to a custom tick or advancement dispatch.", minecraft: "Players polls each online player; AdvancementPlayer is the exact triggering player in an advancement reward function." }
register_event_api! { path: "sand::events::TickScope::Players", aliases: [], module: "sand::events", kind: Variant, signature: "TickScope::Players", summary: "Selects per-online-player tick evaluation.", minecraft: "Sand emits execute-as @a at @s polling." }
register_event_api! { path: "sand::events::TickScope::AdvancementPlayer", aliases: [], module: "sand::events", kind: Variant, signature: "TickScope::AdvancementPlayer", summary: "Describes an exact player supplied by an advancement reward.", minecraft: "The event runs as Minecraft's triggering player rather than a tick loop." }
register_event_api! { path: "sand::events::TickScope::has_player_subject", aliases: [], module: "sand::events", kind: Method, signature: "fn has_player_subject(self) -> bool", summary: "Reports whether this scope provides one player execution subject.", minecraft: "Sand uses this capability when validating participant inheritance in composed events." }
register_event_api! { path: "sand::events::PersistentEventCondition", aliases: [], module: "sand::events", kind: Struct, signature: "pub struct PersistentEventCondition", summary: "Represents a current-state condition usable by a persistent event parent.", minecraft: "It is evaluated under the inherited player at a child dispatch boundary instead of independently firing a detector." }
register_event_api! { path: "sand::events::PersistentEventCondition::players", aliases: [], module: "sand::events", kind: Method, signature: "fn players(condition: impl Into<Condition>) -> PersistentEventCondition", summary: "Creates a persistent condition evaluated as the inherited player.", minecraft: "Its typed condition becomes execute clauses under @s when a composed child is considered." }
register_event_api! { path: "sand::events::PersistentEventCondition::scope", aliases: [], module: "sand::events", kind: Method, signature: "fn scope(&self) -> TickScope", summary: "Returns the execution capability required by this persistent condition.", minecraft: "The scope determines whether a parent can safely provide the condition's @s player." }
register_event_api! { path: "sand::events::PersistentSandEvent", aliases: [], module: "sand::events", kind: Trait, signature: "pub trait PersistentSandEvent", summary: "Opts a custom Sand event into use as a directly queryable current-state parent.", minecraft: "Unlike an occurrence event, its condition is evaluated when the child dispatches and does not rerun the parent's detector." }
register_event_api! { path: "sand::events::PersistentSandEvent::persistent_condition", aliases: [], module: "sand::events", kind: Method, signature: "fn persistent_condition() -> PersistentEventCondition", summary: "Returns the current-state condition represented by this persistent event.", minecraft: "Sand emits it at the composed child's dispatch boundary under the inherited player." }
register_event_api! { path: "sand::events::TickWindow", aliases: [], module: "sand::events", kind: Struct, signature: "pub struct TickWindow", summary: "Validates the bounded tick interval used to correlate event occurrences.", minecraft: "Sand stores occurrence age and accepts a parent only before the configured tick window expires." }
register_event_api! { path: "sand::events::TickWindow::MIN_TICKS", aliases: [], module: "sand::events", kind: AssociatedConst, signature: "const MIN_TICKS: u32", summary: "The smallest valid bounded-correlation window.", minecraft: "One tick includes the current dispatch cycle only." }
register_event_api! { path: "sand::events::TickWindow::MAX_TICKS", aliases: [], module: "sand::events", kind: AssociatedConst, signature: "const MAX_TICKS: u32", summary: "The largest supported bounded-correlation window.", minecraft: "Sand rejects larger windows to keep generated scoreboard lifecycle bounded." }
register_event_api! { path: "sand::events::TickWindow::new", aliases: [], module: "sand::events", kind: Method, signature: "fn new(ticks: u32) -> Result<TickWindow, TickWindowError>", summary: "Validates a correlation window length.", minecraft: "Rejects zero and values above Sand's generated event-state limit." }
register_event_api! { path: "sand::events::TickWindow::ticks", aliases: [], module: "sand::events", kind: Method, signature: "fn ticks(self) -> u32", summary: "Returns this validated correlation-window length.", minecraft: "The value controls how long Sand retains an occurrence mark." }
register_event_api! { path: "sand::events::TickWindowError", aliases: [], module: "sand::events", kind: Enum, signature: "pub enum TickWindowError", summary: "Explains why a bounded event correlation window is invalid.", minecraft: "Invalid windows cannot be represented safely by Sand's generated tick lifecycle." }
register_event_api! { path: "sand::events::TickWindowError::Zero", aliases: [], module: "sand::events", kind: Variant, signature: "TickWindowError::Zero", summary: "Reports that a zero-tick window was requested.", minecraft: "Zero would mean no valid event cycle and is rejected." }
register_event_api! { path: "sand::events::TickWindowError::TooLarge", aliases: [], module: "sand::events", kind: Variant, signature: "TickWindowError::TooLarge { requested, max }", summary: "Reports that a requested correlation window exceeds Sand's limit.", minecraft: "The limit bounds generated scoreboard age tracking." }
register_event_api! { path: "sand::events::TickWindowError::TooLarge::requested", aliases: [], module: "sand::events", kind: Field, signature: "requested: u32", summary: "The rejected requested window length.", minecraft: "It is reported for diagnostics and has no runtime effect." }
register_event_api! { path: "sand::events::TickWindowError::TooLarge::max", aliases: [], module: "sand::events", kind: Field, signature: "max: u32", summary: "The largest window Sand accepts.", minecraft: "It bounds the generated occurrence-age lifecycle." }
register_event_api! { path: "sand::events::EventSetup", aliases: [], module: "sand::events", kind: Struct, signature: "pub struct EventSetup", summary: "Declares load and observation lifecycle commands owned by a custom Sand event.", minecraft: "Objectives run at load; pre-observation commands run before detection and post-observation commands run after each detector pass." }
register_event_api! { path: "sand::events::EventSetup::objectives", aliases: [], module: "sand::events", kind: Field, signature: "objectives: Vec<String>", summary: "Stores load-time initialization commands for an event.", minecraft: "These normally add scoreboard objectives at minecraft:load." }
register_event_api! { path: "sand::events::EventSetup::pre_observation", aliases: [], module: "sand::events", kind: Field, signature: "pre_observation: Vec<String>", summary: "Stores commands that prepare state before an event detector runs.", minecraft: "Typical use snapshots a score or NBT value before comparison." }
register_event_api! { path: "sand::events::EventSetup::post_observation", aliases: [], module: "sand::events", kind: Field, signature: "post_observation: Vec<String>", summary: "Stores commands that advance event state after detection.", minecraft: "They run after each tick's detection pass even when no handler dispatches." }
register_event_api! { path: "sand::events::EventSetup::none", aliases: [], module: "sand::events", kind: Method, signature: "fn none() -> EventSetup", summary: "Creates an event with no lifecycle-owned commands.", minecraft: "It emits no load or detector setup resources." }
register_event_api! { path: "sand::events::EventSetup::is_empty", aliases: [], module: "sand::events", kind: Method, signature: "fn is_empty(&self) -> bool", summary: "Reports whether an event owns no lifecycle setup.", minecraft: "Sand uses this distinction when validating dispatch composition." }
register_event_api! { path: "sand::events::EventSetup::first_non_empty_category", aliases: [], module: "sand::events", kind: Method, signature: "fn first_non_empty_category(&self) -> Option<&str>", summary: "Names the first lifecycle category populated by an event setup.", minecraft: "It supports deterministic diagnostics for generated load and tick wiring." }
register_event_api! { path: "sand::events::EventSetup::with_participants", aliases: [], module: "sand::events", kind: Method, signature: "fn with_participants<E>(self, plan: EventParticipantPlan, profile: &VersionProfile) -> Result<EventSetup, _>", summary: "Merges declared participant observation lifecycle into an event setup.", minecraft: "Sand emits participant capture before dispatch and cleanup after the event cycle for the selected version profile." }
register_event_api! { path: "sand::events::TickEventDispatch", aliases: [], module: "sand::events", kind: Struct, signature: "pub struct TickEventDispatch", summary: "Builds a typed event detector evaluated each tick.", minecraft: "Sand emits per-player execute checks and dispatches a handler only while all typed conditions hold." }
register_event_api! { path: "sand::events::TickEventDispatch::as_players", aliases: [], module: "sand::events", kind: Method, signature: "fn as_players(self) -> Self", summary: "Makes a tick detector evaluate as each online player.", minecraft: "Sand renders execute as @a at @s for the detector." }
register_event_api! { path: "sand::events::TickEventDispatch::when", aliases: [], module: "sand::events", kind: Method, signature: "fn when(self, condition: impl Into<Condition>) -> Self", summary: "Adds a positive typed condition to a tick event.", minecraft: "All when clauses become conjunctions in the generated execute test." }
register_event_api! { path: "sand::events::TickEventDispatch::if_", aliases: [], module: "sand::events", kind: Method, signature: "fn if_(self, condition: impl Into<Condition>) -> Self", summary: "Adds a positive condition using execute-style naming.", minecraft: "It has the same conjunction semantics as when." }
register_event_api! { path: "sand::events::TickEventDispatch::unless", aliases: [], module: "sand::events", kind: Method, signature: "fn unless(self, condition: impl Into<Condition>) -> Self", summary: "Adds a negative typed condition to a tick event.", minecraft: "Each clause becomes an execute-unless requirement." }
register_event_api! { path: "sand::events::TickEventDispatch::every_tick", aliases: [], module: "sand::events", kind: Method, signature: "fn every_tick(self) -> Self", summary: "Marks the only currently supported tick-event cadence explicitly.", minecraft: "The detector runs once per Minecraft tick." }
register_event_api! { path: "sand::events::TickEventDispatch::combined_condition", aliases: [], module: "sand::events", kind: Method, signature: "fn combined_condition(&self) -> Option<Condition>", summary: "Returns the conjunction of this detector's positive and negative conditions.", minecraft: "The resulting condition corresponds to generated execute clauses, or None for an unconditional detector." }
register_event_api! { path: "sand::events::SameCycleEventDependency", aliases: [], module: "sand::events", kind: Struct, signature: "pub struct SameCycleEventDependency", summary: "Represents one typed parent dependency supplied by Sand's sealed event-group tuples.", minecraft: "It is consumed during export to connect same-cycle event marks; authors select it through after_any or after_all rather than constructing it." }
register_event_api! { path: "sand::events::SameCycleEventGroup", aliases: [], module: "sand::events", kind: Trait, signature: "pub trait SameCycleEventGroup", summary: "Marks supported tuples of two through eight same-cycle parent events.", minecraft: "Sand uses the tuple's concrete event types to generate deterministic any-parent or all-parent correlation." }
register_event_api! { path: "sand::events::SameCycleEventGroup::dependencies", aliases: [], module: "sand::events", kind: Method, signature: "fn dependencies() -> Vec<SameCycleEventDependency>", summary: "Returns the typed parent list for a sealed event-group tuple.", minecraft: "This is the trait's export hook; normal event code chooses a tuple through after_any or after_all." }
register_event_api! { path: "sand::events::ChainEventDispatch", aliases: [], module: "sand::events", kind: Struct, signature: "pub struct ChainEventDispatch", summary: "Builds a child event that is evaluated from parent event occurrences.", minecraft: "Sand propagates the parent subject in the same dispatch cycle and can retain bounded parent marks across ticks." }
register_event_api! { path: "sand::events::ChainEventDispatch::occurrence", aliases: [], module: "sand::events", kind: Field, signature: "occurrence: Vec<SameCycleEventRequirement>", summary: "Stores explicit same-cycle parent requirements for a composed event.", minecraft: "Sand lowers these requirements into parent occurrence marks and subject propagation." }
register_event_api! { path: "sand::events::ChainEventDispatch::persistent", aliases: [], module: "sand::events", kind: Field, signature: "persistent: Vec<PersistentEventDependency>", summary: "Stores current-state parent requirements for a composed event.", minecraft: "Each condition is evaluated at the child dispatch boundary rather than firing a parent detector." }
register_event_api! { path: "sand::events::ChainEventDispatch::bounded", aliases: [], module: "sand::events", kind: Field, signature: "bounded: Vec<BoundedEventDependency>", summary: "Stores bounded cross-tick parent requirements for a composed event.", minecraft: "Sand retains and ages matching parent occurrence marks." }
register_event_api! { path: "sand::events::ChainEventDispatch::conditions", aliases: [], module: "sand::events", kind: Field, signature: "conditions: Vec<Condition>", summary: "Stores additional positive gates for a composed child event.", minecraft: "Every condition becomes part of the generated child execute check." }
register_event_api! { path: "sand::events::ChainEventDispatch::excluded_conditions", aliases: [], module: "sand::events", kind: Field, signature: "excluded_conditions: Vec<Condition>", summary: "Stores additional negative gates for a composed child event.", minecraft: "Each condition becomes an execute-unless requirement." }
register_event_api! { path: "sand::events::BoundedEventDependency", aliases: [], module: "sand::events", kind: Struct, signature: "pub struct BoundedEventDependency", summary: "Describes one raw bounded parent dependency for advanced composition construction.", minecraft: "Sand uses its factories and window to retain a parent occurrence mark across ticks." }
register_event_api! { path: "sand::events::BoundedEventDependency::event_type_id", aliases: [], module: "sand::events", kind: Field, signature: "event_type_id: fn() -> TypeId", summary: "Returns the dependency event's stable Rust type identity.", minecraft: "Sand uses it to deduplicate generated event wiring." }
register_event_api! { path: "sand::events::BoundedEventDependency::event_type_name", aliases: [], module: "sand::events", kind: Field, signature: "event_type_name: fn() -> &'static str", summary: "Returns the dependency event's diagnostic type name.", minecraft: "Sand includes it in deterministic export diagnostics." }
register_event_api! { path: "sand::events::BoundedEventDependency::event_dispatch", aliases: [], module: "sand::events", kind: Field, signature: "event_dispatch: fn() -> SandEventDispatch", summary: "Builds the parent event's dispatch definition.", minecraft: "Sand lowers it before creating bounded occurrence tracking." }
register_event_api! { path: "sand::events::BoundedEventDependency::event_setup", aliases: [], module: "sand::events", kind: Field, signature: "event_setup: fn() -> EventSetup", summary: "Builds lifecycle setup for the bounded parent.", minecraft: "Its objectives and observation commands are deduplicated during export." }
register_event_api! { path: "sand::events::BoundedEventDependency::window", aliases: [], module: "sand::events", kind: Field, signature: "window: TickWindow", summary: "Sets the dependency's validated retention duration.", minecraft: "The generated occurrence mark expires after this window." }
register_event_api! { path: "sand::events::PersistentEventDependency", aliases: [], module: "sand::events", kind: Struct, signature: "pub struct PersistentEventDependency", summary: "Describes one raw current-state parent dependency for advanced composition construction.", minecraft: "Sand evaluates its persistent condition at the child dispatch boundary." }
register_event_api! { path: "sand::events::PersistentEventDependency::event_type_id", aliases: [], module: "sand::events", kind: Field, signature: "event_type_id: fn() -> TypeId", summary: "Returns the persistent parent's stable type identity.", minecraft: "Sand uses it to deduplicate exported dependencies." }
register_event_api! { path: "sand::events::PersistentEventDependency::event_type_name", aliases: [], module: "sand::events", kind: Field, signature: "event_type_name: fn() -> &'static str", summary: "Returns the persistent parent's diagnostic type name.", minecraft: "It is used in deterministic composition diagnostics." }
register_event_api! { path: "sand::events::PersistentEventDependency::event_dispatch", aliases: [], module: "sand::events", kind: Field, signature: "event_dispatch: fn() -> SandEventDispatch", summary: "Builds the persistent parent's dispatch definition.", minecraft: "Sand validates that the parent supports persistent-state use." }
register_event_api! { path: "sand::events::PersistentEventDependency::event_setup", aliases: [], module: "sand::events", kind: Field, signature: "event_setup: fn() -> EventSetup", summary: "Builds lifecycle setup for the persistent parent.", minecraft: "Sand merges it into generated event resources." }
register_event_api! { path: "sand::events::PersistentEventDependency::make_condition", aliases: [], module: "sand::events", kind: Field, signature: "make_condition: fn() -> PersistentEventCondition", summary: "Builds the parent's current-state condition.", minecraft: "It is evaluated under the child event's inherited subject." }
register_event_api! { path: "sand::events::SameCycleEventDependency::event_type_id", aliases: [], module: "sand::events", kind: Field, signature: "event_type_id: fn() -> TypeId", summary: "Returns a same-cycle parent's stable type identity.", minecraft: "Sand uses it to join occurrence marks deterministically." }
register_event_api! { path: "sand::events::SameCycleEventDependency::event_type_name", aliases: [], module: "sand::events", kind: Field, signature: "event_type_name: fn() -> &'static str", summary: "Returns a same-cycle parent's diagnostic type name.", minecraft: "It appears in composition validation diagnostics." }
register_event_api! { path: "sand::events::SameCycleEventDependency::event_dispatch", aliases: [], module: "sand::events", kind: Field, signature: "event_dispatch: fn() -> SandEventDispatch", summary: "Builds a same-cycle parent's dispatch definition.", minecraft: "Sand lowers it to connect parent success to the child cycle." }
register_event_api! { path: "sand::events::SameCycleEventDependency::event_setup", aliases: [], module: "sand::events", kind: Field, signature: "event_setup: fn() -> EventSetup", summary: "Builds merged lifecycle setup for a same-cycle parent.", minecraft: "Sand keeps participant setup consistent with direct event registration." }
register_event_api! { path: "sand::events::SameCycleEventDependency::event_raw_setup", aliases: [], module: "sand::events", kind: Field, signature: "event_raw_setup: fn() -> EventSetup", summary: "Builds a same-cycle parent's unmerged lifecycle setup.", minecraft: "Sand uses it to validate advancement-bridge eligibility." }
register_event_api! { path: "sand::events::SameCycleEventDependency::event_participants", aliases: [], module: "sand::events", kind: Field, signature: "event_participants: fn() -> EventParticipantPlan", summary: "Builds the parent's declared participant plan.", minecraft: "Sand applies it around an advancement bridge when required." }
register_event_api! { path: "sand::events::SameCycleEventDependency::event_revoke", aliases: [], module: "sand::events", kind: Field, signature: "event_revoke: fn() -> bool", summary: "Reports whether an advancement parent re-arms after dispatch.", minecraft: "Sand applies it only to advancement-backed parent bridges." }
register_event_api! { path: "sand::events::SameCycleEventRequirement", aliases: [], module: "sand::events", kind: Enum, signature: "pub enum SameCycleEventRequirement", summary: "Represents one raw same-cycle parent requirement for advanced composition construction.", minecraft: "Sand lowers after, any-parent, and all-parent requirements into event occurrence marks." }
register_event_api! { path: "sand::events::SameCycleEventRequirement::After", aliases: [], module: "sand::events", kind: Variant, signature: "After(SameCycleEventDependency)", summary: "Requires one parent event occurrence in the current cycle.", minecraft: "The child inherits that parent's generated execution subject." }
register_event_api! { path: "sand::events::SameCycleEventRequirement::After::0", aliases: [], module: "sand::events", kind: Field, signature: "SameCycleEventDependency", summary: "The required same-cycle parent dependency.", minecraft: "It supplies the parent dispatch and lifecycle factories." }
register_event_api! { path: "sand::events::SameCycleEventRequirement::AfterAny", aliases: [], module: "sand::events", kind: Variant, signature: "AfterAny(Vec<SameCycleEventDependency>)", summary: "Requires at least one parent occurrence in the current cycle.", minecraft: "Sand coalesces the supplied parent marks into one any-parent gate." }
register_event_api! { path: "sand::events::SameCycleEventRequirement::AfterAny::0", aliases: [], module: "sand::events", kind: Field, signature: "Vec<SameCycleEventDependency>", summary: "The alternative same-cycle parent dependencies.", minecraft: "Any qualifying parent can open the generated child gate." }
register_event_api! { path: "sand::events::SameCycleEventRequirement::AfterAll", aliases: [], module: "sand::events", kind: Variant, signature: "AfterAll(Vec<SameCycleEventDependency>)", summary: "Requires every parent occurrence in the current cycle.", minecraft: "Sand waits for all supplied marks before dispatching the child." }
register_event_api! { path: "sand::events::SameCycleEventRequirement::AfterAll::0", aliases: [], module: "sand::events", kind: Field, signature: "Vec<SameCycleEventDependency>", summary: "The required same-cycle parent dependencies.", minecraft: "Each dependency must produce its occurrence mark for the same subject." }
register_event_api! { path: "sand::events::ChainEventDispatch::after", aliases: [], module: "sand::events", kind: Method, signature: "fn after<E: SandEvent>(self) -> Self", summary: "Requires one parent event to fire for the same subject in this cycle.", minecraft: "The child is wired after the parent's successful generated dispatch, without an independent detector." }
register_event_api! { path: "sand::events::ChainEventDispatch::after_any", aliases: [], module: "sand::events", kind: Method, signature: "fn after_any<G: SameCycleEventGroup>(self) -> Self", summary: "Requires any parent in a typed tuple to fire in this cycle.", minecraft: "Sand coalesces the tuple's occurrence marks into one deterministic any-parent gate." }
register_event_api! { path: "sand::events::ChainEventDispatch::after_all", aliases: [], module: "sand::events", kind: Method, signature: "fn after_all<G: SameCycleEventGroup>(self) -> Self", summary: "Requires every parent in a typed tuple to fire in this cycle.", minecraft: "Sand waits until all tuple occurrence marks exist for the same dispatch subject." }
register_event_api! { path: "sand::events::ChainEventDispatch::while_", aliases: [], module: "sand::events", kind: Method, signature: "fn while_<E: PersistentSandEvent>(self) -> Self", summary: "Requires a persistent parent state to be true when the child is considered.", minecraft: "Sand evaluates the parent's current condition under the inherited player without rerunning its detector." }
register_event_api! { path: "sand::events::ChainEventDispatch::within", aliases: [], module: "sand::events", kind: Method, signature: "fn within<E: SandEvent>(self, window: TickWindow) -> Self", summary: "Requires a parent occurrence within a validated bounded tick window.", minecraft: "Sand stores and ages the parent's occurrence mark until the window expires." }
register_event_api! { path: "sand::events::ChainEventDispatch::when", aliases: [], module: "sand::events", kind: Method, signature: "fn when(self, condition: impl Into<Condition>) -> Self", summary: "Adds a positive gate to a composed child event.", minecraft: "The condition is evaluated after its parent relationship has been satisfied." }
register_event_api! { path: "sand::events::ChainEventDispatch::if_", aliases: [], module: "sand::events", kind: Method, signature: "fn if_(self, condition: impl Into<Condition>) -> Self", summary: "Adds a positive composed-event gate using execute-style naming.", minecraft: "It has the same dispatch semantics as when." }
register_event_api! { path: "sand::events::ChainEventDispatch::unless", aliases: [], module: "sand::events", kind: Method, signature: "fn unless(self, condition: impl Into<Condition>) -> Self", summary: "Adds a negative gate to a composed child event.", minecraft: "The child dispatches only when the condition does not hold." }
register_event_api! { path: "sand::events::ChainEventDispatch::combined_condition", aliases: [], module: "sand::events", kind: Method, signature: "fn combined_condition(&self) -> Option<Condition>", summary: "Returns the composed child's explicit condition gate.", minecraft: "It excludes parent occurrence requirements, which Sand lowers separately." }
register_event_api! { path: "sand::events::SandEventDispatch", aliases: ["sand::prelude::SandEventDispatch"], module: "sand::events", kind: Enum, signature: "pub enum SandEventDispatch", summary: "Selects how a custom Sand event is detected and dispatched.", minecraft: "Sand lowers its variants into advancement resources, tick polling, composed event functions, or tracked transition state." }
register_event_api! { path: "sand::events::SandEventDispatch::AdvancementTrigger", aliases: ["sand::prelude::SandEventDispatch::AdvancementTrigger"], module: "sand::events", kind: Variant, signature: "AdvancementTrigger(AdvancementTrigger)", summary: "Uses one vanilla advancement criterion as a custom Sand event dispatch.", minecraft: "Sand exports its criterion and reward function, rearming it according to SandEvent::revoke." }
register_event_api! { path: "sand::events::SandEventDispatch::AdvancementTrigger::0", aliases: ["sand::prelude::SandEventDispatch::AdvancementTrigger::0"], module: "sand::events", kind: Field, signature: "AdvancementTrigger", summary: "The typed vanilla criterion used by an advancement dispatch.", minecraft: "It serializes into the event's advancement JSON." }
register_event_api! { path: "sand::events::SandEventDispatch::TickCondition", aliases: ["sand::prelude::SandEventDispatch::TickCondition"], module: "sand::events", kind: Variant, signature: "TickCondition(String)", summary: "Uses an explicit raw execute-if fragment as a tick dispatch.", minecraft: "Sand polls it for each player; prefer the typed tick builder when possible." }
register_event_api! { path: "sand::events::SandEventDispatch::TickCondition::0", aliases: ["sand::prelude::SandEventDispatch::TickCondition::0"], module: "sand::events", kind: Field, signature: "String", summary: "The raw Minecraft execute-if condition fragment.", minecraft: "Sand writes it into generated tick execution and cannot validate its target-version semantics." }
register_event_api! { path: "sand::events::SandEventDispatch::Tick", aliases: ["sand::prelude::SandEventDispatch::Tick"], module: "sand::events", kind: Variant, signature: "Tick(TickEventDispatch)", summary: "Uses Sand's typed tick-poll dispatch builder.", minecraft: "Sand emits the builder's typed conditions as per-player execute checks." }
register_event_api! { path: "sand::events::SandEventDispatch::Tick::0", aliases: ["sand::prelude::SandEventDispatch::Tick::0"], module: "sand::events", kind: Field, signature: "TickEventDispatch", summary: "The structured tick detector for this event.", minecraft: "It supplies the generated tick condition and scope." }
register_event_api! { path: "sand::events::SandEventDispatch::Chain", aliases: ["sand::prelude::SandEventDispatch::Chain"], module: "sand::events", kind: Variant, signature: "Chain(ChainEventDispatch)", summary: "Uses a same-cycle or bounded parent composition dispatch.", minecraft: "Sand lowers parent occurrences and inherited subjects into generated composition functions." }
register_event_api! { path: "sand::events::SandEventDispatch::Chain::0", aliases: ["sand::prelude::SandEventDispatch::Chain::0"], module: "sand::events", kind: Field, signature: "ChainEventDispatch", summary: "The structured parent-composition definition.", minecraft: "It controls generated same-cycle and bounded-correlation wiring." }
register_event_api! { path: "sand::events::SandEventDispatch::Tracked", aliases: ["sand::prelude::SandEventDispatch::Tracked"], module: "sand::events", kind: Variant, signature: "Tracked(TrackedTransition)", summary: "Uses a reusable previous/current transition tracker as event dispatch.", minecraft: "Sand shares the tracker state across handlers with the same tracker ID." }
register_event_api! { path: "sand::events::SandEventDispatch::Tracked::0", aliases: ["sand::prelude::SandEventDispatch::Tracked::0"], module: "sand::events", kind: Field, signature: "TrackedTransition", summary: "The typed transition tracker backing this dispatch.", minecraft: "It stores prior state and fires when the selected transition occurs." }
register_event_api! { path: "sand::events::SandEventDispatch::tick", aliases: ["sand::prelude::SandEventDispatch::tick"], module: "sand::events", kind: Method, signature: "fn tick() -> TickEventDispatch", summary: "Starts a typed per-tick event detector.", minecraft: "The resulting builder emits execute checks for online players." }
register_event_api! { path: "sand::events::SandEventDispatch::chain", aliases: ["sand::prelude::SandEventDispatch::chain"], module: "sand::events", kind: Method, signature: "fn chain<P: SandEvent>() -> ChainEventDispatch", summary: "Starts a child dispatch after one typed parent event.", minecraft: "The child can run in the parent's successful event cycle with its subject inherited." }
register_event_api! { path: "sand::events::SandEventDispatch::compose", aliases: ["sand::prelude::SandEventDispatch::compose"], module: "sand::events", kind: Method, signature: "fn compose() -> ChainEventDispatch", summary: "Starts a parent-composition dispatch before choosing requirements.", minecraft: "An empty composition is rejected during export because it has no occurrence source." }
register_event_api! { path: "sand::events::SandEventDispatch::after_any", aliases: ["sand::prelude::SandEventDispatch::after_any"], module: "sand::events", kind: Method, signature: "fn after_any<G: SameCycleEventGroup>() -> ChainEventDispatch", summary: "Starts a composition requiring any event in a typed parent tuple.", minecraft: "Sand coalesces tuple occurrence marks in the current cycle." }
register_event_api! { path: "sand::events::SandEventDispatch::after_all", aliases: ["sand::prelude::SandEventDispatch::after_all"], module: "sand::events", kind: Method, signature: "fn after_all<G: SameCycleEventGroup>() -> ChainEventDispatch", summary: "Starts a composition requiring all events in a typed parent tuple.", minecraft: "Sand waits for every tuple occurrence mark in the same cycle." }
register_event_api! { path: "sand::events::SandEvent", aliases: ["sand::prelude::SandEvent"], module: "sand::events", kind: Trait, signature: "pub trait SandEvent", summary: "Defines a custom Sand-native event marker and its detection lifecycle.", minecraft: "Sand inspects the dispatch and setup at build time to generate tick, advancement, or transition resources." }
register_event_api! { path: "sand::events::SandEvent::dispatch", aliases: ["sand::prelude::SandEvent::dispatch"], module: "sand::events", kind: Method, signature: "fn dispatch() -> impl Into<SandEventDispatch>", summary: "Returns the event's typed detection strategy.", minecraft: "Sand lowers it to the generated runtime mechanism for each subscribed handler." }
register_event_api! { path: "sand::events::SandEvent::setup", aliases: ["sand::prelude::SandEvent::setup"], module: "sand::events", kind: Method, signature: "fn setup() -> EventSetup", summary: "Returns lifecycle resources owned by a custom event.", minecraft: "Sand deduplicates those objectives and observation commands across handlers for the same event type." }
register_event_api! { path: "sand::events::SandEvent::participants", aliases: ["sand::prelude::SandEvent::participants"], module: "sand::events", kind: Method, signature: "fn participants() -> EventParticipantPlan", summary: "Declares participant observations available to a custom event handler.", minecraft: "Sand captures and cleans up the selected participant evidence around dispatch." }
register_event_api! { path: "sand::events::SandEvent::revoke", aliases: ["sand::prelude::SandEvent::revoke"], module: "sand::events", kind: Method, signature: "fn revoke() -> bool", summary: "Selects whether an advancement-form custom event is rearmed after firing.", minecraft: "True emits an advancement revoke for the triggering player; tick events have no advancement grant to revoke." }
register_event_api! { path: "sand::events::SandEventParticipants", aliases: ["sand::prelude::SandEventParticipants"], module: "sand::events", kind: Trait, signature: "pub trait SandEventParticipants", summary: "Provides typed participant accessors on bare SandEvent handler markers.", minecraft: "Each accessor reads evidence Sand captured for the current dispatch cycle." }
register_event_api! { path: "sand::events::SandEventParticipants::entity", aliases: ["sand::prelude::SandEventParticipants::entity"], module: "sand::events", kind: Method, signature: "fn entity(&self, role: EntityParticipantRole) -> EntityParticipant", summary: "Reads a declared entity participant by role.", minecraft: "The value is available only when the event plan captured that role." }
register_event_api! { path: "sand::events::SandEventParticipants::item", aliases: ["sand::prelude::SandEventParticipants::item"], module: "sand::events", kind: Method, signature: "fn item(&self, role: ItemParticipantRole) -> ItemSnapshot", summary: "Reads a declared current-cycle item snapshot by role.", minecraft: "Sand captures its NBT at dispatch time rather than depending on a live slot." }
register_event_api! { path: "sand::events::SandEventParticipants::attacker", aliases: ["sand::prelude::SandEventParticipants::attacker"], module: "sand::events", kind: Method, signature: "fn attacker(&self) -> EntityParticipant", summary: "Reads the declared attacker participant.", minecraft: "It is evidence-backed and unavailable unless the event plan captured an attacker." }
register_event_api! { path: "sand::events::SandEventParticipants::killer", aliases: ["sand::prelude::SandEventParticipants::killer"], module: "sand::events", kind: Method, signature: "fn killer(&self) -> EntityParticipant", summary: "Reads the declared killer participant.", minecraft: "The value belongs to the current event dispatch record." }
register_event_api! { path: "sand::events::SandEventParticipants::victim", aliases: ["sand::prelude::SandEventParticipants::victim"], module: "sand::events", kind: Method, signature: "fn victim(&self) -> EntityParticipant", summary: "Reads the declared victim participant.", minecraft: "The value belongs to the current event dispatch record." }
register_event_api! { path: "sand::events::SandEventParticipants::interacted_entity", aliases: ["sand::prelude::SandEventParticipants::interacted_entity"], module: "sand::events", kind: Method, signature: "fn interacted_entity(&self) -> EntityParticipant", summary: "Reads the declared interaction target participant.", minecraft: "It reflects evidence captured by the trigger or participant plan." }
register_event_api! { path: "sand::events::SandEventParticipants::weapon", aliases: ["sand::prelude::SandEventParticipants::weapon"], module: "sand::events", kind: Method, signature: "fn weapon(&self) -> ItemSnapshot", summary: "Reads the declared weapon item snapshot.", minecraft: "Sand stores the trigger-time weapon NBT for the dispatch cycle." }
register_event_api! { path: "sand::events::SandEventParticipants::bounded_item", aliases: ["sand::prelude::SandEventParticipants::bounded_item"], module: "sand::events", kind: Method, signature: "fn bounded_item(&self, role: ItemParticipantRole) -> BoundedItemSnapshot", summary: "Reads an item participant retained across a bounded correlation window.", minecraft: "The snapshot expires according to the composed event's TickWindow." }
register_event_api! { path: "sand::events::PlayerLevelUpEvent::previous_level", aliases: ["sand::event::vanilla::PlayerLevelsUp::previous_level"], module: "sand::events", kind: Method, signature: "fn previous_level(&self) -> ScoreRef<i32>", summary: "Reads the player level captured before the level-up transition.", minecraft: "Sand snapshots the player's experience level before its generated transition check." }
register_event_api! { path: "sand::events::PlayerLevelUpEvent::current_level", aliases: ["sand::event::vanilla::PlayerLevelsUp::current_level"], module: "sand::events", kind: Method, signature: "fn current_level(&self) -> ScoreRef<i32>", summary: "Reads the player level captured after the level-up transition.", minecraft: "Sand synchronizes the current experience level during its generated tick observation." }
register_event_api! { path: "sand::events::PlayerLevelUpEvent::level_delta", aliases: ["sand::event::vanilla::PlayerLevelsUp::level_delta"], module: "sand::events", kind: Method, signature: "fn level_delta(&self) -> ScoreRef<i32>", summary: "Reads the change in player experience level for this transition.", minecraft: "Sand derives it from the synchronized previous and current level scores." }
register_event_api! { path: "sand::events::StatusEffectMarker", aliases: [], module: "sand::events", kind: Trait, signature: "pub trait StatusEffectMarker", summary: "Describes one vanilla status effect for typed effect transition events.", minecraft: "Sand uses its effect ID and predicate condition to build a shared transition tracker." }
register_event_api! { path: "sand::events::StatusEffectMarker::EFFECT_ID", aliases: [], module: "sand::events", kind: AssociatedConst, signature: "const EFFECT_ID: &str", summary: "The vanilla status-effect resource identifier.", minecraft: "It names the effect used by generated entity-properties predicates." }
register_event_api! { path: "sand::events::StatusEffectMarker::TRACKER_ID", aliases: [], module: "sand::events", kind: AssociatedConst, signature: "const TRACKER_ID: &str", summary: "The stable Sand tracker identity for this status effect.", minecraft: "Handlers for the same effect share its generated previous/current state." }
register_event_api! { path: "sand::events::StatusEffectMarker::CONDITION", aliases: [], module: "sand::events", kind: AssociatedConst, signature: "const CONDITION: &str", summary: "The typed condition fragment that observes this status effect.", minecraft: "Sand uses it to test the executing player's active effects." }

// Built-in advancement-backed event markers. Their aliases in `event::vanilla`
// are short discovery names; `events::*Event` remains the canonical catalog.
register_event_marker! { path: "sand::events::OnJoinEvent", aliases: ["sand::event::vanilla::OnJoin"], summary: "Marks Sand's load-or-new-player join observation.", minecraft: "Sand uses persistent per-player state, so it fires after load for present players and for newly observed players; reconnecting within the same persisted session is not a fresh login.", example: "#[sand::on_event]\nfn joined(event: sand::event::Event<sand::events::OnJoinEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::FirstJoinEvent", aliases: ["sand::event::vanilla::FirstJoin"], summary: "Marks the first observed join for a player.", minecraft: "Uses Sand's persisted first-join state and does not re-arm for that player.", example: "#[sand::on_event]\nfn first_join(event: sand::event::Event<sand::events::FirstJoinEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::OnDeathEvent", aliases: ["sand::event::vanilla::OnDeath"], summary: "Marks a player's death.", minecraft: "Backed by the vanilla death advancement trigger and dispatched as that player's reward function.", example: "#[sand::on_event]\nfn died(event: sand::event::Event<sand::events::OnDeathEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::OnRespawnEvent", aliases: ["sand::event::vanilla::OnRespawn"], summary: "Marks the first Sand observation after a player respawns.", minecraft: "Sand detects post-death player activity rather than receiving a standalone vanilla respawn trigger.", example: "#[sand::on_event]\nfn respawned(event: sand::event::Event<sand::events::OnRespawnEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::EntityKillEvent", aliases: ["sand::event::vanilla::EntityKill"], summary: "Marks a player killing an entity.", minecraft: "Uses minecraft:player_killed_entity; Sand can capture the player's main-hand weapon at trigger time.", example: "#[sand::on_event]\nfn kill(event: sand::event::Event<sand::events::EntityKillEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::PlayerKillEvent", aliases: ["sand::event::vanilla::PlayerKill"], summary: "Marks an entity killing a player.", minecraft: "Uses minecraft:entity_killed_player and exposes only evidence-backed correlated participants.", example: "#[sand::on_event]\nfn killed(event: sand::event::Event<sand::events::PlayerKillEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::PlayerDamageEntityEvent", aliases: ["sand::event::vanilla::PlayerDamagesEntity"], summary: "Marks a player damaging another entity.", minecraft: "Uses minecraft:player_hurt_entity and can capture the triggering player's weapon.", example: "#[sand::on_event]\nfn hit(event: sand::event::DamageEvent<sand::events::PlayerDamageEntityEvent>) { event.reflect_damage(); }" }
register_event_marker! { path: "sand::events::EntityDamagePlayerEvent", aliases: ["sand::event::vanilla::EntityDamagesPlayer"], summary: "Marks an entity damaging a player.", minecraft: "Uses minecraft:entity_hurt_player with a bounded correlated-attacker observation when available.", example: "#[sand::on_event]\nfn hurt(event: sand::event::DamageEvent<sand::events::EntityDamagePlayerEvent>) { event.reflect_damage(); }" }
register_event_marker! { path: "sand::events::ShotCrossbowEvent", aliases: ["sand::event::vanilla::CrossbowShot"], summary: "Marks a player shooting a crossbow.", minecraft: "Uses minecraft:shot_crossbow and re-arms its generated advancement after dispatch.", example: "#[sand::on_event]\nfn shot(event: sand::event::Event<sand::events::ShotCrossbowEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ChanneledLightningEvent", aliases: [], summary: "Marks a player channeling lightning through a trident action.", minecraft: "Uses minecraft:channeled_lightning advancement criteria.", example: "#[sand::on_event]\nfn lightning(event: sand::event::Event<sand::events::ChanneledLightningEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ItemConsumeEvent", aliases: ["sand::event::vanilla::AnyItemConsumed"], summary: "Marks a player consuming an item.", minecraft: "Uses minecraft:consume_item and fires for the player that consumed the stack.", example: "#[sand::on_event]\nfn consumed(event: sand::event::Event<sand::events::ItemConsumeEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ItemCraftEvent", aliases: ["sand::event::vanilla::AnyItemCrafted"], summary: "Marks a player crafting an item.", minecraft: "Uses minecraft:recipe_crafted with Sand's broad built-in criterion.", example: "#[sand::on_event]\nfn crafted(event: sand::event::Event<sand::events::ItemCraftEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ItemEnchantEvent", aliases: ["sand::event::vanilla::AnyItemEnchanted"], summary: "Marks a player enchanting an item.", minecraft: "Uses minecraft:enchanted_item with no extra built-in item filter.", example: "#[sand::on_event]\nfn enchanted(event: sand::event::Event<sand::events::ItemEnchantEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::BucketFillEvent", aliases: [], summary: "Marks a player filling a bucket.", minecraft: "Uses minecraft:filled_bucket advancement criteria.", example: "#[sand::on_event]\nfn filled(event: sand::event::Event<sand::events::BucketFillEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::BucketEmptyEvent", aliases: [], summary: "Marks a player emptying a bucket.", minecraft: "Uses minecraft:filled_bucket's complementary empty-bucket criterion as emitted by Sand's trigger model.", example: "#[sand::on_event]\nfn emptied(event: sand::event::Event<sand::events::BucketEmptyEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::FishingEvent", aliases: [], summary: "Marks a player hooking something with a fishing rod.", minecraft: "Uses minecraft:fishing_rod_hooked, including all vanilla hooked-item/entity outcomes.", example: "#[sand::on_event]\nfn fish(event: sand::event::Event<sand::events::FishingEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ItemPickedUpEvent", aliases: [], summary: "Marks a player picking up an item.", minecraft: "Uses minecraft:inventory_changed detection for the built-in item-pickup criterion.", example: "#[sand::on_event]\nfn pickup(event: sand::event::Event<sand::events::ItemPickedUpEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ItemDurabilityChangeEvent", aliases: [], summary: "Marks a durability change on a player's item.", minecraft: "Uses minecraft:item_durability_changed advancement criteria.", example: "#[sand::on_event]\nfn durability(event: sand::event::Event<sand::events::ItemDurabilityChangeEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::BrewPotionEvent", aliases: ["sand::event::vanilla::PotionBrewed"], summary: "Marks a player brewing a potion.", minecraft: "Uses minecraft:brewed_potion without narrowing to one potion recipe.", example: "#[sand::on_event]\nfn brewed(event: sand::event::Event<sand::events::BrewPotionEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::TotemActivateEvent", aliases: [], summary: "Marks a player activating a totem of undying.", minecraft: "Uses minecraft:used_totem; advancement reset permits later totem activations.", example: "#[sand::on_event]\nfn saved(event: sand::event::Event<sand::events::TotemActivateEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::RecipeUnlockEvent", aliases: [], summary: "Marks a player unlocking a recipe.", minecraft: "Uses Sand's recipe-unlock advancement trigger registration.", example: "#[sand::on_event]\nfn recipe(event: sand::event::Event<sand::events::RecipeUnlockEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::BlockPlaceEvent", aliases: ["sand::event::vanilla::AnyBlockPlaced"], summary: "Marks a player placing a block.", minecraft: "Uses minecraft:placed_block with no built-in block or item filter.", example: "#[sand::on_event]\nfn placed(event: sand::event::Event<sand::events::BlockPlaceEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::EnterBlockEvent", aliases: [], summary: "Marks a player entering a matching block volume.", minecraft: "Uses minecraft:enter_block without a built-in block-state filter.", example: "#[sand::on_event]\nfn entered(event: sand::event::Event<sand::events::EnterBlockEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::SlideDownBlockEvent", aliases: [], summary: "Marks a player sliding down a block.", minecraft: "Uses minecraft:slide_down_block advancement criteria.", example: "#[sand::on_event]\nfn slid(event: sand::event::Event<sand::events::SlideDownBlockEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::TargetHitEvent", aliases: [], summary: "Marks a player hitting a target block.", minecraft: "Uses minecraft:target_hit advancement criteria.", example: "#[sand::on_event]\nfn target(event: sand::event::Event<sand::events::TargetHitEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::BeeNestDestroyedEvent", aliases: [], summary: "Marks a player destroying a bee nest or hive.", minecraft: "Uses minecraft:bee_nest_destroyed criteria.", example: "#[sand::on_event]\nfn bees(event: sand::event::Event<sand::events::BeeNestDestroyedEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ChangeDimensionEvent", aliases: ["sand::event::vanilla::DimensionChanged"], summary: "Marks a player changing dimension.", minecraft: "Uses minecraft:changed_dimension with Sand's broad built-in criterion.", example: "#[sand::on_event]\nfn dimension(event: sand::event::Event<sand::events::ChangeDimensionEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::PlayerSleepEvent", aliases: [], summary: "Marks a player sleeping in a bed.", minecraft: "Uses minecraft:slept_in_bed advancement criteria.", example: "#[sand::on_event]\nfn slept(event: sand::event::Event<sand::events::PlayerSleepEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::FallFromHeightEvent", aliases: [], summary: "Marks a player satisfying the vanilla fall-from-height trigger.", minecraft: "Uses minecraft:fall_from_height with Sand's unfiltered built-in threshold.", example: "#[sand::on_event]\nfn fell(event: sand::event::Event<sand::events::FallFromHeightEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::PlayerLevelUpEvent", aliases: ["sand::event::vanilla::PlayerLevelsUp"], summary: "Marks an increase in a player's experience level.", minecraft: "Minecraft has no level-up advancement; Sand compares generated scoreboard snapshots in its tick lifecycle.", example: "#[sand::on_event]\nfn leveled(event: sand::event::Event<sand::events::PlayerLevelUpEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::EffectsChangedEvent", aliases: [], summary: "Marks a change to a player's active effects.", minecraft: "Uses minecraft:effects_changed with Sand's broad built-in predicate.", example: "#[sand::on_event]\nfn effects(event: sand::event::Event<sand::events::EffectsChangedEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::StartRidingEvent", aliases: [], summary: "Marks a player beginning to ride an entity.", minecraft: "Uses minecraft:started_riding advancement criteria.", example: "#[sand::on_event]\nfn ride(event: sand::event::Event<sand::events::StartRidingEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::UseEnderEyeEvent", aliases: [], summary: "Marks a player using an eye of ender.", minecraft: "Uses minecraft:used_ender_eye advancement criteria.", example: "#[sand::on_event]\nfn eye(event: sand::event::Event<sand::events::UseEnderEyeEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::HeroOfTheVillageEvent", aliases: [], summary: "Marks a player gaining Hero of the Village.", minecraft: "Uses minecraft:hero_of_the_village advancement criteria.", example: "#[sand::on_event]\nfn hero(event: sand::event::Event<sand::events::HeroOfTheVillageEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::LightningStrikeEvent", aliases: [], summary: "Marks a player observing a lightning strike.", minecraft: "Uses minecraft:lightning_strike advancement criteria.", example: "#[sand::on_event]\nfn strike(event: sand::event::Event<sand::events::LightningStrikeEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::TameAnimalEvent", aliases: ["sand::event::vanilla::AnimalTamed"], summary: "Marks a player taming an animal.", minecraft: "Uses minecraft:tame_animal advancement criteria.", example: "#[sand::on_event]\nfn tame(event: sand::event::Event<sand::events::TameAnimalEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::BreedAnimalsEvent", aliases: ["sand::event::vanilla::AnimalsBreed"], summary: "Marks a player breeding animals.", minecraft: "Uses minecraft:bred_animals advancement criteria.", example: "#[sand::on_event]\nfn breed(event: sand::event::Event<sand::events::BreedAnimalsEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::SummonEntityEvent", aliases: ["sand::event::vanilla::EntitySummoned"], summary: "Marks a player summoning an entity.", minecraft: "Uses minecraft:summoned_entity advancement criteria.", example: "#[sand::on_event]\nfn summon(event: sand::event::Event<sand::events::SummonEntityEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::InteractWithEntityEvent", aliases: [], summary: "Marks a player interacting with an entity.", minecraft: "Uses minecraft:player_interacted_with_entity criteria.", example: "#[sand::on_event]\nfn interact(event: sand::event::Event<sand::events::InteractWithEntityEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::VillagerTradeEvent", aliases: [], summary: "Marks a player completing a villager trade.", minecraft: "Uses minecraft:villager_trade advancement criteria.", example: "#[sand::on_event]\nfn trade(event: sand::event::Event<sand::events::VillagerTradeEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ConstructBeaconEvent", aliases: [], summary: "Marks a player constructing a beacon.", minecraft: "Uses minecraft:construct_beacon advancement criteria.", example: "#[sand::on_event]\nfn beacon(event: sand::event::Event<sand::events::ConstructBeaconEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::CureZombieVillagerEvent", aliases: [], summary: "Marks a player curing a zombie villager.", minecraft: "Uses minecraft:cured_zombie_villager advancement criteria.", example: "#[sand::on_event]\nfn cure(event: sand::event::Event<sand::events::CureZombieVillagerEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::LootContainerOpenEvent", aliases: [], summary: "Marks a player opening a loot container.", minecraft: "Uses minecraft:player_generates_container_loot criteria.", example: "#[sand::on_event]\nfn loot(event: sand::event::Event<sand::events::LootContainerOpenEvent>) { sand::command::say(event.player()); }" }

// Tick-polled and tracked built-ins. These are bare `SandEvent` markers: the
// handler receives the marker itself, and Sand owns the per-player detector.
register_event_marker! { path: "sand::events::PlayerStartSneakingEvent", aliases: ["sand::event::vanilla::PlayerStartsSneaking"], summary: "Marks the transition into sneaking.", minecraft: "Sand compares a tracked player-sneaking predicate across ticks and fires only on false-to-true transitions.", example: "#[sand::on_event]\nfn start_sneak(_: sand::events::PlayerStartSneakingEvent) { sand::command::say(\"Sneak\"); }" }
register_event_marker! { path: "sand::events::PlayerStopSneakingEvent", aliases: ["sand::event::vanilla::PlayerStopsSneaking"], summary: "Marks the transition out of sneaking.", minecraft: "Sand's tracked sneaking predicate fires only on true-to-false transitions.", example: "#[sand::on_event]\nfn stop_sneak(_: sand::events::PlayerStopSneakingEvent) { sand::command::say(\"Stop\"); }" }
register_event_marker! { path: "sand::events::PlayerSneakEvent", aliases: ["sand::event::vanilla::PlayerSneaking"], summary: "Marks each tick a player is sneaking.", minecraft: "Sand evaluates its player-sneaking predicate for each online player every tick.", example: "#[sand::on_event]\nfn sneaking(_: sand::events::PlayerSneakEvent) { sand::command::say(\"Sneaking\"); }" }
register_event_marker! { path: "sand::events::PlayerSprintEvent", aliases: ["sand::event::vanilla::PlayerSprinting"], summary: "Marks each tick a player is sprinting.", minecraft: "Sand evaluates its tracked sprinting predicate for each online player every tick.", example: "#[sand::on_event]\nfn sprinting(_: sand::events::PlayerSprintEvent) { sand::command::say(\"Sprint\"); }" }
register_event_marker! { path: "sand::events::PlayerStartSprintingEvent", aliases: [], summary: "Marks the transition into sprinting.", minecraft: "Sand emits a false-to-true transition from its per-player sprinting tracker.", example: "#[sand::on_event]\nfn start_sprint(_: sand::events::PlayerStartSprintingEvent) { sand::command::say(\"Sprint\"); }" }
register_event_marker! { path: "sand::events::PlayerStopSprintingEvent", aliases: [], summary: "Marks the transition out of sprinting.", minecraft: "Sand emits a true-to-false transition from its per-player sprinting tracker.", example: "#[sand::on_event]\nfn stop_sprint(_: sand::events::PlayerStopSprintingEvent) { sand::command::say(\"Stop\"); }" }
register_event_marker! { path: "sand::events::PlayerSwimmingEvent", aliases: ["sand::event::vanilla::PlayerSwimming"], summary: "Marks each tick a player is swimming.", minecraft: "Sand evaluates the swimming entity predicate as each online player.", example: "#[sand::on_event]\nfn swimming(_: sand::events::PlayerSwimmingEvent) { sand::command::say(\"Swim\"); }" }
register_event_marker! { path: "sand::events::PlayerStartSwimmingEvent", aliases: [], summary: "Marks the transition into swimming.", minecraft: "Sand's tracked swimming predicate fires only on false-to-true transitions.", example: "#[sand::on_event]\nfn start_swim(_: sand::events::PlayerStartSwimmingEvent) { sand::command::say(\"Swim\"); }" }
register_event_marker! { path: "sand::events::PlayerStopSwimmingEvent", aliases: [], summary: "Marks the transition out of swimming.", minecraft: "Sand's tracked swimming predicate fires only on true-to-false transitions.", example: "#[sand::on_event]\nfn stop_swim(_: sand::events::PlayerStopSwimmingEvent) { sand::command::say(\"Stop\"); }" }
register_event_marker! { path: "sand::events::PlayerFlyingEvent", aliases: [], summary: "Marks each tick a player has flying enabled.", minecraft: "Sand evaluates the player's abilities.flying NBT predicate each tick; it is a player-only detector.", example: "#[sand::on_event]\nfn flying(_: sand::events::PlayerFlyingEvent) { sand::command::say(\"Fly\"); }" }
register_event_marker! { path: "sand::events::PlayerStartFlyingEvent", aliases: [], summary: "Marks the transition into flying.", minecraft: "Sand tracks the player flying predicate and fires on false-to-true transitions.", example: "#[sand::on_event]\nfn start_flying(_: sand::events::PlayerStartFlyingEvent) { sand::command::say(\"Fly\"); }" }
register_event_marker! { path: "sand::events::PlayerStopFlyingEvent", aliases: [], summary: "Marks the transition out of flying.", minecraft: "Sand tracks the player flying predicate and fires on true-to-false transitions.", example: "#[sand::on_event]\nfn stop_flying(_: sand::events::PlayerStopFlyingEvent) { sand::command::say(\"Stop\"); }" }
register_event_marker! { path: "sand::events::PlayerOnFireEvent", aliases: ["sand::event::vanilla::PlayerOnFire"], summary: "Marks each tick a player is on fire.", minecraft: "Sand evaluates its player-on-fire predicate for each online player.", example: "#[sand::on_event]\nfn burning(_: sand::events::PlayerOnFireEvent) { sand::command::say(\"Fire\"); }" }
register_event_marker! { path: "sand::events::PlayerCaughtFireEvent", aliases: [], summary: "Marks the transition into being on fire.", minecraft: "Sand's tracked fire predicate fires only on false-to-true transitions.", example: "#[sand::on_event]\nfn caught_fire(_: sand::events::PlayerCaughtFireEvent) { sand::command::say(\"Fire\"); }" }
register_event_marker! { path: "sand::events::PlayerExtinguishedEvent", aliases: [], summary: "Marks the transition out of being on fire.", minecraft: "Sand's tracked fire predicate fires only on true-to-false transitions.", example: "#[sand::on_event]\nfn extinguished(_: sand::events::PlayerExtinguishedEvent) { sand::command::say(\"Safe\"); }" }
register_event_marker! { path: "sand::events::PlayerInCreativeEvent", aliases: [], summary: "Marks each tick a player is in creative mode.", minecraft: "Sand evaluates an entity gamemode=creative condition per online player.", example: "#[sand::on_event]\nfn creative(_: sand::events::PlayerInCreativeEvent) { sand::command::say(\"Creative\"); }" }
register_event_marker! { path: "sand::events::PlayerInAdventureEvent", aliases: [], summary: "Marks each tick a player is in adventure mode.", minecraft: "Sand evaluates an entity gamemode=adventure condition per online player.", example: "#[sand::on_event]\nfn adventure(_: sand::events::PlayerInAdventureEvent) { sand::command::say(\"Adventure\"); }" }
register_event_marker! { path: "sand::events::PlayerInSpectatorEvent", aliases: [], summary: "Marks each tick a player is in spectator mode.", minecraft: "Sand evaluates an entity gamemode=spectator condition per online player.", example: "#[sand::on_event]\nfn spectator(_: sand::events::PlayerInSpectatorEvent) { sand::command::say(\"Spectator\"); }" }
register_event_marker! { path: "sand::events::PlayerHealthChangedEvent", aliases: [], summary: "Marks a change in a player's health score.", minecraft: "Sand tracks health through a generated scoreboard baseline and fires on any change.", example: "#[sand::on_event]\nfn health(_: sand::events::PlayerHealthChangedEvent) { sand::command::say(\"Health\"); }" }
register_event_marker! { path: "sand::events::PlayerHealthLostEvent", aliases: [], summary: "Marks a decrease in a player's health score.", minecraft: "Sand compares generated current and previous health score baselines.", example: "#[sand::on_event]\nfn lost(_: sand::events::PlayerHealthLostEvent) { sand::command::say(\"Hurt\"); }" }
register_event_marker! { path: "sand::events::PlayerHealthGainedEvent", aliases: [], summary: "Marks an increase in a player's health score.", minecraft: "Sand compares generated current and previous health score baselines.", example: "#[sand::on_event]\nfn gained(_: sand::events::PlayerHealthGainedEvent) { sand::command::say(\"Healed\"); }" }
register_event_marker! { path: "sand::events::ArmorEquipEvent", aliases: [], summary: "Marks a player equipping an item in a selected equipment slot.", minecraft: "Sand tracks equipment-slot state each tick; #[on_event] requires a slot filter and may further filter item/custom data.", example: "#[sand::on_event(slot = Feet)]\nfn equip(event: sand::event::Event<sand::events::ArmorEquipEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ArmorUnequipEvent", aliases: [], summary: "Marks a player removing an item from a selected equipment slot.", minecraft: "Sand tracks equipment-slot state each tick; #[on_event] requires a slot filter and may further filter item/custom data.", example: "#[sand::on_event(slot = Feet)]\nfn unequip(event: sand::event::Event<sand::events::ArmorUnequipEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::HoldingItemEvent", aliases: [], summary: "Marks each tick a player holds a selected item.", minecraft: "Sand renders execute-if-items checks against the requested mainhand or offhand slot.", example: "#[sand::on_event(item = \"minecraft:shield\", slot = Offhand)]\nfn shield(event: sand::event::Event<sand::events::HoldingItemEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::CurrentlyWearingEvent", aliases: [], summary: "Marks each tick a player wears a selected armor item.", minecraft: "Sand renders execute-if-items checks against the requested armor slot and item filter.", example: "#[sand::on_event(slot = Head, item = \"minecraft:diamond_helmet\")]\nfn helmet(event: sand::event::Event<sand::events::CurrentlyWearingEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::PlayerLowHealthEvent", aliases: [], summary: "Marks crossing down to a configured player health threshold.", minecraft: "Sand tracks vanilla health through a shared scoreboard baseline; one threshold value is allowed per exported pack.", example: "#[sand::on_event]\nfn low(_: sand::events::PlayerLowHealthEvent<6>) { sand::command::say(\"Low health\"); }" }
register_event_marker! { path: "sand::events::PlayerRecoveredHealthEvent", aliases: [], summary: "Marks crossing back above a configured low-health threshold.", minecraft: "Shares PlayerLowHealthEvent's generated health tracker, so both types must use the same half-heart threshold.", example: "#[sand::on_event]\nfn recovered(_: sand::events::PlayerRecoveredHealthEvent<6>) { sand::command::say(\"Recovered\"); }" }
register_event_marker! { path: "sand::events::EffectStarted", aliases: [], summary: "Marks a player gaining a selected generated status-effect marker.", minecraft: "Sand emits an entity-properties effect predicate and a false-to-true transition tracker for the selected effect.", example: "#[sand::on_event]\nfn speed(_: sand::events::EffectStarted<sand::events::Speed>) { sand::command::say(\"Speed\"); }" }
register_event_marker! { path: "sand::events::EffectStopped", aliases: [], summary: "Marks a player losing a selected generated status-effect marker.", minecraft: "Sand emits a true-to-false transition from the same generated effect tracker used by EffectStarted.", example: "#[sand::on_event]\nfn speed_end(_: sand::events::EffectStopped<sand::events::Speed>) { sand::command::say(\"Speed ended\"); }" }

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
    path: "sand::item::location",
    aliases: [],
    module: "sand::item",
    kind: Module,
    signature: "pub mod location",
    summary: "Provides validated live item and inventory addressing.",
    context: "Location types describe where an item currently resides without confusing that mutable location with captured event-time data.",
    minecraft: "Builds the entity and block inventory targets used by item, data, and execute-if-items commands.",
    use_when: ["Constructing a live equipment, inventory, container, or item-entity location"],
    avoid_when: ["Keeping event-time item evidence after a handler changes the source"],
    params: [],
    returns: None,
    example: "let slot = sand::item::ItemLocation::PlayerMainHand;"
}

register! {
    path: "sand::item::snapshot",
    aliases: [],
    module: "sand::item",
    kind: Module,
    signature: "pub mod snapshot",
    summary: "Provides immutable, short-lived event-time item capture handles.",
    context: "Snapshots preserve the source item before handler logic mutates inventory and communicate their capture reliability explicitly.",
    minecraft: "Uses guarded data copies into deterministic command-storage paths during one synchronous event dispatch.",
    use_when: ["A handler needs evidence of the item present at trigger time"],
    avoid_when: ["Representing a live inventory slot or durable player inventory state"],
    params: [],
    returns: None,
    example: "use sand::item::snapshot::ItemSnapshot;"
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

// BEGIN ENTITY API CONTRACTS
register_entity_api! { path: "sand::entity::Adoption", aliases: ["sand::prelude::Adoption"], kind: Struct, summary: "Represents adoption in a typed entity archetype definition." }
register_entity_api! { path: "sand::entity::Adoption::every", aliases: ["sand::prelude::Adoption::every"], kind: Method, summary: "Configures or performs every for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::Adoption::excluding_tag", aliases: ["sand::prelude::Adoption::excluding_tag"], kind: Method, summary: "Configures or performs excluding tag for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::Adoption::external", aliases: ["sand::prelude::Adoption::external"], kind: Method, summary: "Configures or performs external for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::Adoption::natural", aliases: ["sand::prelude::Adoption::natural"], kind: Method, summary: "Configures or performs natural for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::Adoption::natural_and_external", aliases: ["sand::prelude::Adoption::natural_and_external"], kind: Method, summary: "Configures or performs natural and external for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::Adoption::requiring_tag", aliases: ["sand::prelude::Adoption::requiring_tag"], kind: Method, summary: "Configures or performs requiring tag for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::Adoption::special_entities", aliases: ["sand::prelude::Adoption::special_entities"], kind: Method, summary: "Configures or performs special entities for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::Adoption::where_state", aliases: ["sand::prelude::Adoption::where_state"], kind: Method, summary: "Configures or performs where state for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::Adoption::within_blocks", aliases: ["sand::prelude::Adoption::within_blocks"], kind: Method, summary: "Configures or performs within blocks for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::AdoptionSource", aliases: ["sand::prelude::AdoptionSource"], kind: Enum, summary: "Represents adoption source in a typed entity archetype definition." }
register_entity_api! { path: "sand::entity::AdoptionSource::External", aliases: ["sand::prelude::AdoptionSource::External"], kind: Variant, summary: "Selects the external semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::AdoptionSource::Natural", aliases: ["sand::prelude::AdoptionSource::Natural"], kind: Variant, summary: "Selects the natural semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::AdoptionSource::NaturalAndExternal", aliases: ["sand::prelude::AdoptionSource::NaturalAndExternal"], kind: Variant, summary: "Selects the natural and external semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::DerivedScoreEncoding", aliases: ["sand::prelude::DerivedScoreEncoding"], kind: Enum, summary: "Represents derived score encoding in a typed entity archetype definition." }
register_entity_api! { path: "sand::entity::DerivedScoreEncoding::FixedPoint", aliases: ["sand::prelude::DerivedScoreEncoding::FixedPoint"], kind: Variant, summary: "Selects the fixed point semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::DerivedScoreEncoding::Whole", aliases: ["sand::prelude::DerivedScoreEncoding::Whole"], kind: Variant, summary: "Selects the whole semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityAction", aliases: ["sand::prelude::EntityAction"], kind: Enum, summary: "Represents entity action in a typed entity archetype definition." }
register_entity_api! { path: "sand::entity::EntityAction::AddTag", aliases: ["sand::prelude::EntityAction::AddTag"], kind: Variant, summary: "Selects the add tag semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityAction::AddTag::0", aliases: ["sand::prelude::EntityAction::AddTag::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityAction::ApplyEffect", aliases: ["sand::prelude::EntityAction::ApplyEffect"], kind: Variant, summary: "Selects the apply effect semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityAction::ApplyEffect::0", aliases: ["sand::prelude::EntityAction::ApplyEffect::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityAction::Despawn", aliases: ["sand::prelude::EntityAction::Despawn"], kind: Variant, summary: "Selects the despawn semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityAction::Dispatch", aliases: ["sand::prelude::EntityAction::Dispatch"], kind: Variant, summary: "Selects the dispatch semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityAction::Dispatch::0", aliases: ["sand::prelude::EntityAction::Dispatch::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityAction::RemoveEffect", aliases: ["sand::prelude::EntityAction::RemoveEffect"], kind: Variant, summary: "Selects the remove effect semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityAction::RemoveEffect::0", aliases: ["sand::prelude::EntityAction::RemoveEffect::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityAction::RemoveTag", aliases: ["sand::prelude::EntityAction::RemoveTag"], kind: Variant, summary: "Selects the remove tag semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityAction::RemoveTag::0", aliases: ["sand::prelude::EntityAction::RemoveTag::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityAction::Run", aliases: ["sand::prelude::EntityAction::Run"], kind: Variant, summary: "Selects the run semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityAction::Run::0", aliases: ["sand::prelude::EntityAction::Run::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityArchetype", aliases: ["sand::prelude::EntityArchetype"], kind: Struct, summary: "Represents entity archetype in a typed entity archetype definition." }
register_entity_api! { path: "sand::entity::EntityArchetype::adopt", aliases: ["sand::prelude::EntityArchetype::adopt"], kind: Method, summary: "Configures or performs adopt for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::attach", aliases: ["sand::prelude::EntityArchetype::attach"], kind: Method, summary: "Configures or performs attach for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::attribute", aliases: ["sand::prelude::EntityArchetype::attribute"], kind: Method, summary: "Configures or performs attribute for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::attribute_modifier", aliases: ["sand::prelude::EntityArchetype::attribute_modifier"], kind: Method, summary: "Configures or performs attribute modifier for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::cleanup_with", aliases: ["sand::prelude::EntityArchetype::cleanup_with"], kind: Method, summary: "Configures or performs cleanup with for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::derive", aliases: ["sand::prelude::EntityArchetype::derive"], kind: Method, summary: "Configures or performs derive for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::effect", aliases: ["sand::prelude::EntityArchetype::effect"], kind: Method, summary: "Configures or performs effect for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::effect_when", aliases: ["sand::prelude::EntityArchetype::effect_when"], kind: Method, summary: "Configures or performs effect when for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::equipment", aliases: ["sand::prelude::EntityArchetype::equipment"], kind: Method, summary: "Configures or performs equipment for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::equipment_when", aliases: ["sand::prelude::EntityArchetype::equipment_when"], kind: Method, summary: "Configures or performs equipment when for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::external_adoption_tag", aliases: ["sand::prelude::EntityArchetype::external_adoption_tag"], kind: Method, summary: "Configures or performs external adoption tag for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::health", aliases: ["sand::prelude::EntityArchetype::health"], kind: Method, summary: "Configures or performs health for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::id", aliases: ["sand::prelude::EntityArchetype::id"], kind: Method, summary: "Returns id for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::initialize_with", aliases: ["sand::prelude::EntityArchetype::initialize_with"], kind: Method, summary: "Configures or performs initialize with for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::initialized_tag", aliases: ["sand::prelude::EntityArchetype::initialized_tag"], kind: Method, summary: "Configures or performs initialized tag for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::migration", aliases: ["sand::prelude::EntityArchetype::migration"], kind: Method, summary: "Configures or performs migration for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::name", aliases: ["sand::prelude::EntityArchetype::name"], kind: Method, summary: "Returns name for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::native_data", aliases: ["sand::prelude::EntityArchetype::native_data"], kind: Method, summary: "Configures or performs native data for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::new", aliases: ["sand::prelude::EntityArchetype::new"], kind: Method, summary: "Constructs new for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::on", aliases: ["sand::prelude::EntityArchetype::on"], kind: Method, summary: "Configures or performs on for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::reconcile", aliases: ["sand::prelude::EntityArchetype::reconcile"], kind: Method, summary: "Configures or performs reconcile for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::summon", aliases: ["sand::prelude::EntityArchetype::summon"], kind: Method, summary: "Configures or performs summon for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::tag", aliases: ["sand::prelude::EntityArchetype::tag"], kind: Method, summary: "Configures or performs tag for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::tag_when", aliases: ["sand::prelude::EntityArchetype::tag_when"], kind: Method, summary: "Configures or performs tag when for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::team", aliases: ["sand::prelude::EntityArchetype::team"], kind: Method, summary: "Configures or performs team for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::team_when", aliases: ["sand::prelude::EntityArchetype::team_when"], kind: Method, summary: "Configures or performs team when for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityArchetype::version", aliases: ["sand::prelude::EntityArchetype::version"], kind: Method, summary: "Configures or performs version for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityDerivation", aliases: ["sand::prelude::EntityDerivation"], kind: Struct, summary: "Represents entity derivation in a typed entity archetype definition." }
register_entity_api! { path: "sand::entity::EntityDerivation::curve", aliases: ["sand::prelude::EntityDerivation::curve"], kind: Method, summary: "Configures or performs curve for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityDerivation::fixed", aliases: ["sand::prelude::EntityDerivation::fixed"], kind: Method, summary: "Configures or performs fixed for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityDerivation::fixed_point", aliases: ["sand::prelude::EntityDerivation::fixed_point"], kind: Method, summary: "Configures or performs fixed point for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityDerivation::name", aliases: ["sand::prelude::EntityDerivation::name"], kind: Method, summary: "Returns name for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityDerivation::new", aliases: ["sand::prelude::EntityDerivation::new"], kind: Method, summary: "Constructs new for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityDerivation::output_encoding", aliases: ["sand::prelude::EntityDerivation::output_encoding"], kind: Method, summary: "Configures or performs output encoding for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityDerivation::store_fixed_point", aliases: ["sand::prelude::EntityDerivation::store_fixed_point"], kind: Method, summary: "Configures or performs store fixed point for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityDerivation::target", aliases: ["sand::prelude::EntityDerivation::target"], kind: Method, summary: "Returns target for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityTransition", aliases: ["sand::prelude::EntityTransition"], kind: Enum, summary: "Represents entity transition in a typed entity archetype definition." }
register_entity_api! { path: "sand::entity::EntityTransition::Changed", aliases: ["sand::prelude::EntityTransition::Changed"], kind: Variant, summary: "Selects the changed semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityTransition::Changed::0", aliases: ["sand::prelude::EntityTransition::Changed::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::CooldownReady", aliases: ["sand::prelude::EntityTransition::CooldownReady"], kind: Variant, summary: "Selects the cooldown ready semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityTransition::CooldownReady::0", aliases: ["sand::prelude::EntityTransition::CooldownReady::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::EnumChangedTo", aliases: ["sand::prelude::EntityTransition::EnumChangedTo"], kind: Variant, summary: "Selects the enum changed to semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityTransition::EnumChangedTo::encoding", aliases: ["sand::prelude::EntityTransition::EnumChangedTo::encoding"], kind: Field, summary: "Carries the encoding value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::EnumChangedTo::field", aliases: ["sand::prelude::EntityTransition::EnumChangedTo::field"], kind: Field, summary: "Carries the field value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::FlagDisabled", aliases: ["sand::prelude::EntityTransition::FlagDisabled"], kind: Variant, summary: "Selects the flag disabled semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityTransition::FlagDisabled::0", aliases: ["sand::prelude::EntityTransition::FlagDisabled::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::FlagEnabled", aliases: ["sand::prelude::EntityTransition::FlagEnabled"], kind: Variant, summary: "Selects the flag enabled semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityTransition::FlagEnabled::0", aliases: ["sand::prelude::EntityTransition::FlagEnabled::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::HealthPercentage", aliases: ["sand::prelude::EntityTransition::HealthPercentage"], kind: Variant, summary: "Selects the health percentage semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityTransition::HealthPercentage::basis_points", aliases: ["sand::prelude::EntityTransition::HealthPercentage::basis_points"], kind: Field, summary: "Carries the basis points value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::HealthPercentage::current", aliases: ["sand::prelude::EntityTransition::HealthPercentage::current"], kind: Field, summary: "Carries the current value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::HealthPercentage::direction", aliases: ["sand::prelude::EntityTransition::HealthPercentage::direction"], kind: Field, summary: "Carries the direction value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::HealthPercentage::maximum", aliases: ["sand::prelude::EntityTransition::HealthPercentage::maximum"], kind: Field, summary: "Carries the maximum value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::Threshold", aliases: ["sand::prelude::EntityTransition::Threshold"], kind: Variant, summary: "Selects the threshold semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityTransition::Threshold::direction", aliases: ["sand::prelude::EntityTransition::Threshold::direction"], kind: Field, summary: "Carries the direction value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::Threshold::field", aliases: ["sand::prelude::EntityTransition::Threshold::field"], kind: Field, summary: "Carries the field value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::Threshold::value", aliases: ["sand::prelude::EntityTransition::Threshold::value"], kind: Field, summary: "Carries the value value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::TimerElapsed", aliases: ["sand::prelude::EntityTransition::TimerElapsed"], kind: Variant, summary: "Selects the timer elapsed semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityTransition::TimerElapsed::0", aliases: ["sand::prelude::EntityTransition::TimerElapsed::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTransition::changed", aliases: ["sand::prelude::EntityTransition::changed"], kind: Method, summary: "Configures or performs changed for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityTransition::cooldown_ready", aliases: ["sand::prelude::EntityTransition::cooldown_ready"], kind: Method, summary: "Configures or performs cooldown ready for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityTransition::enum_changed_to", aliases: ["sand::prelude::EntityTransition::enum_changed_to"], kind: Method, summary: "Configures or performs enum changed to for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityTransition::flag_disabled", aliases: ["sand::prelude::EntityTransition::flag_disabled"], kind: Method, summary: "Configures or performs flag disabled for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityTransition::flag_enabled", aliases: ["sand::prelude::EntityTransition::flag_enabled"], kind: Method, summary: "Configures or performs flag enabled for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityTransition::health_percentage", aliases: ["sand::prelude::EntityTransition::health_percentage"], kind: Method, summary: "Configures or performs health percentage for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityTransition::threshold", aliases: ["sand::prelude::EntityTransition::threshold"], kind: Method, summary: "Configures or performs threshold for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityTransition::timer_elapsed", aliases: ["sand::prelude::EntityTransition::timer_elapsed"], kind: Method, summary: "Configures or performs timer elapsed for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::EntityTransitionField", aliases: [], kind: Struct, summary: "Represents entity transition field in a typed entity archetype definition." }
register_entity_api! { path: "sand::entity::Migration", aliases: ["sand::prelude::Migration"], kind: Struct, summary: "Represents migration in a typed entity archetype definition." }
register_entity_api! { path: "sand::entity::Migration::action", aliases: ["sand::prelude::Migration::action"], kind: Field, summary: "Carries the action value required by this typed entity case." }
register_entity_api! { path: "sand::entity::Migration::from", aliases: ["sand::prelude::Migration::from"], kind: Field, summary: "Carries the from value required by this typed entity case." }
register_entity_api! { path: "sand::entity::Migration::new", aliases: ["sand::prelude::Migration::new"], kind: Method, summary: "Constructs new for the typed archetype entity API." }
register_entity_api! { path: "sand::entity::Migration::to", aliases: ["sand::prelude::Migration::to"], kind: Field, summary: "Carries the to value required by this typed entity case." }
register_entity_api! { path: "sand::entity::ReconcilePolicy", aliases: ["sand::prelude::ReconcilePolicy"], kind: Enum, summary: "Represents reconcile policy in a typed entity archetype definition." }
register_entity_api! { path: "sand::entity::ReconcilePolicy::Every", aliases: ["sand::prelude::ReconcilePolicy::Every"], kind: Variant, summary: "Selects the every semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::ReconcilePolicy::Every::0", aliases: ["sand::prelude::ReconcilePolicy::Every::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::ReconcilePolicy::InitializeOnly", aliases: ["sand::prelude::ReconcilePolicy::InitializeOnly"], kind: Variant, summary: "Selects the initialize only semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::ReconcilePolicy::Manual", aliases: ["sand::prelude::ReconcilePolicy::Manual"], kind: Variant, summary: "Selects the manual semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::ReconcilePolicy::WhenDirty", aliases: ["sand::prelude::ReconcilePolicy::WhenDirty"], kind: Variant, summary: "Selects the when dirty semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::ReconcilePolicy::WhenSchemaChanges", aliases: ["sand::prelude::ReconcilePolicy::WhenSchemaChanges"], kind: Variant, summary: "Selects the when schema changes semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::SpecialEntityPolicy", aliases: ["sand::prelude::SpecialEntityPolicy"], kind: Enum, summary: "Represents special entity policy in a typed entity archetype definition." }
register_entity_api! { path: "sand::entity::SpecialEntityPolicy::Exclude", aliases: ["sand::prelude::SpecialEntityPolicy::Exclude"], kind: Variant, summary: "Selects the exclude semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::SpecialEntityPolicy::Include", aliases: ["sand::prelude::SpecialEntityPolicy::Include"], kind: Variant, summary: "Selects the include semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::SpecialEntityPolicy::Preserve", aliases: ["sand::prelude::SpecialEntityPolicy::Preserve"], kind: Variant, summary: "Selects the preserve semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::ThresholdDirection", aliases: ["sand::prelude::ThresholdDirection"], kind: Enum, summary: "Represents threshold direction in a typed entity archetype definition." }
register_entity_api! { path: "sand::entity::ThresholdDirection::Falling", aliases: ["sand::prelude::ThresholdDirection::Falling"], kind: Variant, summary: "Selects the falling semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::ThresholdDirection::Rising", aliases: ["sand::prelude::ThresholdDirection::Rising"], kind: Variant, summary: "Selects the rising semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityContext", aliases: ["sand::prelude::EntityContext"], kind: Struct, summary: "Represents entity context while commands execute with a Minecraft entity bound to @s." }
register_entity_api! { path: "sand::entity::EntityContext::add_tag", aliases: ["sand::entity::PlayerContext::add_tag", "sand::prelude::EntityContext::add_tag", "sand::prelude::PlayerContext::add_tag"], kind: Method, summary: "Configures or performs add tag for the typed context entity API." }
register_entity_api! { path: "sand::entity::EntityContext::attacker", aliases: ["sand::entity::PlayerContext::attacker", "sand::prelude::EntityContext::attacker", "sand::prelude::PlayerContext::attacker"], kind: Method, summary: "Configures or performs attacker for the typed context entity API." }
register_entity_api! { path: "sand::entity::EntityContext::controller", aliases: ["sand::entity::PlayerContext::controller", "sand::prelude::EntityContext::controller", "sand::prelude::PlayerContext::controller"], kind: Method, summary: "Configures or performs controller for the typed context entity API." }
register_entity_api! { path: "sand::entity::EntityContext::leasher", aliases: ["sand::entity::PlayerContext::leasher", "sand::prelude::EntityContext::leasher", "sand::prelude::PlayerContext::leasher"], kind: Method, summary: "Configures or performs leasher for the typed context entity API." }
register_entity_api! { path: "sand::entity::EntityContext::origin", aliases: ["sand::entity::PlayerContext::origin", "sand::prelude::EntityContext::origin", "sand::prelude::PlayerContext::origin"], kind: Method, summary: "Configures or performs origin for the typed context entity API." }
register_entity_api! { path: "sand::entity::EntityContext::owner", aliases: ["sand::entity::PlayerContext::owner", "sand::prelude::EntityContext::owner", "sand::prelude::PlayerContext::owner"], kind: Method, summary: "Configures or performs owner for the typed context entity API." }
register_entity_api! { path: "sand::entity::EntityContext::passengers", aliases: ["sand::entity::PlayerContext::passengers", "sand::prelude::EntityContext::passengers", "sand::prelude::PlayerContext::passengers"], kind: Method, summary: "Configures or performs passengers for the typed context entity API." }
register_entity_api! { path: "sand::entity::EntityContext::remove_tag", aliases: ["sand::entity::PlayerContext::remove_tag", "sand::prelude::EntityContext::remove_tag", "sand::prelude::PlayerContext::remove_tag"], kind: Method, summary: "Configures or performs remove tag for the typed context entity API." }
register_entity_api! { path: "sand::entity::EntityContext::state", aliases: ["sand::entity::PlayerContext::state", "sand::prelude::EntityContext::state", "sand::prelude::PlayerContext::state"], kind: Method, summary: "Configures or performs state for the typed context entity API." }
register_entity_api! { path: "sand::entity::EntityContext::target", aliases: ["sand::entity::PlayerContext::target", "sand::prelude::EntityContext::target", "sand::prelude::PlayerContext::target"], kind: Method, summary: "Returns target for the typed context entity API." }
register_entity_api! { path: "sand::entity::EntityContext::vehicle", aliases: ["sand::entity::PlayerContext::vehicle", "sand::prelude::EntityContext::vehicle", "sand::prelude::PlayerContext::vehicle"], kind: Method, summary: "Configures or performs vehicle for the typed context entity API." }
register_entity_api! { path: "sand::entity::EntityScope", aliases: ["sand::prelude::EntityScope"], kind: Struct, summary: "Represents entity scope while commands execute with a Minecraft entity bound to @s." }
register_entity_api! { path: "sand::entity::EntityScope::bind", aliases: ["sand::prelude::EntityScope::bind"], kind: Method, summary: "Configures or performs bind for the typed context entity API." }
register_entity_api! { path: "sand::entity::PlayerContext", aliases: ["sand::prelude::PlayerContext"], kind: TypeAlias, summary: "Represents player context while commands execute with a Minecraft entity bound to @s." }
register_entity_api! { path: "sand::entity::ScopedEntityRef", aliases: ["sand::prelude::ScopedEntityRef"], kind: Struct, summary: "Represents scoped entity ref while commands execute with a Minecraft entity bound to @s." }
register_entity_api! { path: "sand::entity::ScopedEntityRef::add_tag", aliases: ["sand::prelude::ScopedEntityRef::add_tag"], kind: Method, summary: "Configures or performs add tag for the typed context entity API." }
register_entity_api! { path: "sand::entity::ScopedEntityRef::attacker", aliases: ["sand::prelude::ScopedEntityRef::attacker"], kind: Method, summary: "Configures or performs attacker for the typed context entity API." }
register_entity_api! { path: "sand::entity::ScopedEntityRef::controller", aliases: ["sand::prelude::ScopedEntityRef::controller"], kind: Method, summary: "Configures or performs controller for the typed context entity API." }
register_entity_api! { path: "sand::entity::ScopedEntityRef::leasher", aliases: ["sand::prelude::ScopedEntityRef::leasher"], kind: Method, summary: "Configures or performs leasher for the typed context entity API." }
register_entity_api! { path: "sand::entity::ScopedEntityRef::origin", aliases: ["sand::prelude::ScopedEntityRef::origin"], kind: Method, summary: "Configures or performs origin for the typed context entity API." }
register_entity_api! { path: "sand::entity::ScopedEntityRef::owner", aliases: ["sand::prelude::ScopedEntityRef::owner"], kind: Method, summary: "Configures or performs owner for the typed context entity API." }
register_entity_api! { path: "sand::entity::ScopedEntityRef::passengers", aliases: ["sand::prelude::ScopedEntityRef::passengers"], kind: Method, summary: "Configures or performs passengers for the typed context entity API." }
register_entity_api! { path: "sand::entity::ScopedEntityRef::remove_tag", aliases: ["sand::prelude::ScopedEntityRef::remove_tag"], kind: Method, summary: "Configures or performs remove tag for the typed context entity API." }
register_entity_api! { path: "sand::entity::ScopedEntityRef::target", aliases: ["sand::prelude::ScopedEntityRef::target"], kind: Method, summary: "Returns target for the typed context entity API." }
register_entity_api! { path: "sand::entity::ScopedEntityRef::vehicle", aliases: ["sand::prelude::ScopedEntityRef::vehicle"], kind: Method, summary: "Configures or performs vehicle for the typed context entity API." }
register_entity_api! { path: "sand::entity::CurveEvaluationError", aliases: ["sand::prelude::CurveEvaluationError"], kind: Enum, summary: "Represents curve evaluation error in a deterministic semantic entity-stat curve." }
register_entity_api! { path: "sand::entity::CurveEvaluationError::Custom", aliases: ["sand::prelude::CurveEvaluationError::Custom"], kind: Variant, summary: "Selects the custom semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::CurveEvaluationError::Custom::callback", aliases: ["sand::prelude::CurveEvaluationError::Custom::callback"], kind: Field, summary: "Carries the callback value required by this typed entity case." }
register_entity_api! { path: "sand::entity::CurveEvaluationError::Custom::message", aliases: ["sand::prelude::CurveEvaluationError::Custom::message"], kind: Field, summary: "Carries the message value required by this typed entity case." }
register_entity_api! { path: "sand::entity::CurveEvaluationError::Diagnostic", aliases: ["sand::prelude::CurveEvaluationError::Diagnostic"], kind: Variant, summary: "Selects the diagnostic semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::CurveEvaluationError::Diagnostic::0", aliases: ["sand::prelude::CurveEvaluationError::Diagnostic::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::CurveEvaluationError::DivisionByZero", aliases: ["sand::prelude::CurveEvaluationError::DivisionByZero"], kind: Variant, summary: "Selects the division by zero semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::CurveEvaluationError::DivisionByZero::archetype", aliases: ["sand::prelude::CurveEvaluationError::DivisionByZero::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::CurveEvaluationError::DivisionByZero::derivation", aliases: ["sand::prelude::CurveEvaluationError::DivisionByZero::derivation"], kind: Field, summary: "Carries the derivation value required by this typed entity case." }
register_entity_api! { path: "sand::entity::CurveEvaluationError::MissingInput", aliases: ["sand::prelude::CurveEvaluationError::MissingInput"], kind: Variant, summary: "Selects the missing input semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::CurveEvaluationError::MissingInput::archetype", aliases: ["sand::prelude::CurveEvaluationError::MissingInput::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::CurveEvaluationError::MissingInput::derivation", aliases: ["sand::prelude::CurveEvaluationError::MissingInput::derivation"], kind: Field, summary: "Carries the derivation value required by this typed entity case." }
register_entity_api! { path: "sand::entity::CurveEvaluationError::MissingInput::input", aliases: ["sand::prelude::CurveEvaluationError::MissingInput::input"], kind: Field, summary: "Carries the input value required by this typed entity case." }
register_entity_api! { path: "sand::entity::CurveInputs", aliases: ["sand::prelude::CurveInputs"], kind: Struct, summary: "Represents curve inputs in a deterministic semantic entity-stat curve." }
register_entity_api! { path: "sand::entity::CurveInputs::get", aliases: ["sand::prelude::CurveInputs::get"], kind: Method, summary: "Configures or performs get for the typed curve entity API." }
register_entity_api! { path: "sand::entity::CurveInputs::insert", aliases: ["sand::prelude::CurveInputs::insert"], kind: Method, summary: "Configures or performs insert for the typed curve entity API." }
register_entity_api! { path: "sand::entity::CurveInputs::insert_score", aliases: ["sand::prelude::CurveInputs::insert_score"], kind: Method, summary: "Configures or performs insert score for the typed curve entity API." }
register_entity_api! { path: "sand::entity::CurveInputs::iter", aliases: ["sand::prelude::CurveInputs::iter"], kind: Method, summary: "Configures or performs iter for the typed curve entity API." }
register_entity_api! { path: "sand::entity::CurveInputs::new", aliases: ["sand::prelude::CurveInputs::new"], kind: Method, summary: "Constructs new for the typed curve entity API." }
register_entity_api! { path: "sand::entity::DEFAULT_FIXED_POINT_SCALE", aliases: ["sand::prelude::DEFAULT_FIXED_POINT_SCALE"], kind: Constant, summary: "Configures or performs default fixed point scale for the typed curve entity API." }
register_entity_api! { path: "sand::entity::FixedPoint", aliases: ["sand::prelude::FixedPoint"], kind: Struct, summary: "Represents fixed point in a deterministic semantic entity-stat curve." }
register_entity_api! { path: "sand::entity::FixedPoint::decode_score", aliases: ["sand::prelude::FixedPoint::decode_score"], kind: Method, summary: "Configures or performs decode score for the typed curve entity API." }
register_entity_api! { path: "sand::entity::FixedPoint::encode", aliases: ["sand::prelude::FixedPoint::encode"], kind: Method, summary: "Configures or performs encode for the typed curve entity API." }
register_entity_api! { path: "sand::entity::FixedPoint::encode_score", aliases: ["sand::prelude::FixedPoint::encode_score"], kind: Method, summary: "Configures or performs encode score for the typed curve entity API." }
register_entity_api! { path: "sand::entity::FixedPoint::new", aliases: ["sand::prelude::FixedPoint::new"], kind: Method, summary: "Constructs new for the typed curve entity API." }
register_entity_api! { path: "sand::entity::FixedPoint::overflow", aliases: ["sand::prelude::FixedPoint::overflow"], kind: Method, summary: "Configures or performs overflow for the typed curve entity API." }
register_entity_api! { path: "sand::entity::FixedPoint::rounding", aliases: ["sand::prelude::FixedPoint::rounding"], kind: Method, summary: "Configures or performs rounding for the typed curve entity API." }
register_entity_api! { path: "sand::entity::FixedPoint::scale", aliases: ["sand::prelude::FixedPoint::scale"], kind: Method, summary: "Configures or performs scale for the typed curve entity API." }
register_entity_api! { path: "sand::entity::FixedValue", aliases: ["sand::prelude::FixedValue"], kind: Struct, summary: "Represents fixed value in a deterministic semantic entity-stat curve." }
register_entity_api! { path: "sand::entity::FixedValue::as_f64", aliases: ["sand::prelude::FixedValue::as_f64"], kind: Method, summary: "Returns as f64 for the typed curve entity API." }
register_entity_api! { path: "sand::entity::FixedValue::from_units", aliases: ["sand::prelude::FixedValue::from_units"], kind: Method, summary: "Constructs from units for the typed curve entity API." }
register_entity_api! { path: "sand::entity::FixedValue::units", aliases: ["sand::prelude::FixedValue::units"], kind: Method, summary: "Configures or performs units for the typed curve entity API." }
register_entity_api! { path: "sand::entity::OverflowPolicy", aliases: ["sand::prelude::OverflowPolicy"], kind: Enum, summary: "Represents overflow policy in a deterministic semantic entity-stat curve." }
register_entity_api! { path: "sand::entity::OverflowPolicy::Error", aliases: ["sand::prelude::OverflowPolicy::Error"], kind: Variant, summary: "Selects the error semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::OverflowPolicy::Saturate", aliases: ["sand::prelude::OverflowPolicy::Saturate"], kind: Variant, summary: "Selects the saturate semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RoundingPolicy", aliases: ["sand::prelude::RoundingPolicy"], kind: Enum, summary: "Represents rounding policy in a deterministic semantic entity-stat curve." }
register_entity_api! { path: "sand::entity::RoundingPolicy::Ceiling", aliases: ["sand::prelude::RoundingPolicy::Ceiling"], kind: Variant, summary: "Selects the ceiling semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RoundingPolicy::Floor", aliases: ["sand::prelude::RoundingPolicy::Floor"], kind: Variant, summary: "Selects the floor semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RoundingPolicy::NearestTiesAwayFromZero", aliases: ["sand::prelude::RoundingPolicy::NearestTiesAwayFromZero"], kind: Variant, summary: "Selects the nearest ties away from zero semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RoundingPolicy::NearestTiesToEven", aliases: ["sand::prelude::RoundingPolicy::NearestTiesToEven"], kind: Variant, summary: "Selects the nearest ties to even semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RoundingPolicy::TowardZero", aliases: ["sand::prelude::RoundingPolicy::TowardZero"], kind: Variant, summary: "Selects the toward zero semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::StatCurve", aliases: ["sand::prelude::StatCurve"], kind: Struct, summary: "Represents stat curve in a deterministic semantic entity-stat curve." }
register_entity_api! { path: "sand::entity::StatCurve::add", aliases: ["sand::prelude::StatCurve::add"], kind: Method, summary: "Configures or performs add for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::clamped_linear", aliases: ["sand::prelude::StatCurve::clamped_linear"], kind: Method, summary: "Configures or performs clamped linear for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::constant", aliases: ["sand::prelude::StatCurve::constant"], kind: Method, summary: "Configures or performs constant for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::custom", aliases: ["sand::prelude::StatCurve::custom"], kind: Method, summary: "Configures or performs custom for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::custom_with_raw_inputs", aliases: ["sand::prelude::StatCurve::custom_with_raw_inputs"], kind: Method, summary: "Configures or performs custom with raw inputs for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::enum_mapping", aliases: ["sand::prelude::StatCurve::enum_mapping"], kind: Method, summary: "Configures or performs enum mapping for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::enum_mapping_raw", aliases: ["sand::prelude::StatCurve::enum_mapping_raw"], kind: Method, summary: "Configures or performs enum mapping raw for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::evaluate", aliases: ["sand::prelude::StatCurve::evaluate"], kind: Method, summary: "Configures or performs evaluate for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::flag_mapping", aliases: ["sand::prelude::StatCurve::flag_mapping"], kind: Method, summary: "Configures or performs flag mapping for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::flag_mapping_raw", aliases: ["sand::prelude::StatCurve::flag_mapping_raw"], kind: Method, summary: "Configures or performs flag mapping raw for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::input_raw", aliases: ["sand::prelude::StatCurve::input_raw"], kind: Method, summary: "Configures or performs input raw for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::inputs", aliases: ["sand::prelude::StatCurve::inputs"], kind: Method, summary: "Returns inputs for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::linear", aliases: ["sand::prelude::StatCurve::linear"], kind: Method, summary: "Configures or performs linear for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::lookup", aliases: ["sand::prelude::StatCurve::lookup"], kind: Method, summary: "Configures or performs lookup for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::lookup_raw", aliases: ["sand::prelude::StatCurve::lookup_raw"], kind: Method, summary: "Configures or performs lookup raw for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::multiply", aliases: ["sand::prelude::StatCurve::multiply"], kind: Method, summary: "Configures or performs multiply for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::piecewise", aliases: ["sand::prelude::StatCurve::piecewise"], kind: Method, summary: "Configures or performs piecewise for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::ratio", aliases: ["sand::prelude::StatCurve::ratio"], kind: Method, summary: "Configures or performs ratio for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::state", aliases: ["sand::prelude::StatCurve::state"], kind: Method, summary: "Configures or performs state for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::stepped", aliases: ["sand::prelude::StatCurve::stepped"], kind: Method, summary: "Configures or performs stepped for the typed curve entity API." }
register_entity_api! { path: "sand::entity::StatCurve::validate", aliases: ["sand::prelude::StatCurve::validate"], kind: Method, summary: "Configures or performs validate for the typed curve entity API." }
register_entity_api! { path: "sand::entity::EntityDiagnostic", aliases: ["sand::prelude::EntityDiagnostic"], kind: Enum, summary: "Reports entity diagnostic that an archetype author can act on before export." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::ConflictingOwnership", aliases: ["sand::prelude::EntityDiagnostic::ConflictingOwnership"], kind: Variant, summary: "Selects the conflicting ownership semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::ConflictingOwnership::archetype", aliases: ["sand::prelude::EntityDiagnostic::ConflictingOwnership::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::ConflictingOwnership::first", aliases: ["sand::prelude::EntityDiagnostic::ConflictingOwnership::first"], kind: Field, summary: "Carries the first value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::ConflictingOwnership::property", aliases: ["sand::prelude::EntityDiagnostic::ConflictingOwnership::property"], kind: Field, summary: "Carries the property value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::ConflictingOwnership::second", aliases: ["sand::prelude::EntityDiagnostic::ConflictingOwnership::second"], kind: Field, summary: "Carries the second value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::DerivationCycle", aliases: ["sand::prelude::EntityDiagnostic::DerivationCycle"], kind: Variant, summary: "Selects the derivation cycle semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::DerivationCycle::archetype", aliases: ["sand::prelude::EntityDiagnostic::DerivationCycle::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::DerivationCycle::cycle", aliases: ["sand::prelude::EntityDiagnostic::DerivationCycle::cycle"], kind: Field, summary: "Carries the cycle value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::DuplicateStateField", aliases: ["sand::prelude::EntityDiagnostic::DuplicateStateField"], kind: Variant, summary: "Selects the duplicate state field semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::DuplicateStateField::detail", aliases: ["sand::prelude::EntityDiagnostic::DuplicateStateField::detail"], kind: Field, summary: "Carries the detail value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::DuplicateStateField::field", aliases: ["sand::prelude::EntityDiagnostic::DuplicateStateField::field"], kind: Field, summary: "Carries the field value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::DuplicateStateField::schema", aliases: ["sand::prelude::EntityDiagnostic::DuplicateStateField::schema"], kind: Field, summary: "Carries the schema value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::FixedPointOverflow", aliases: ["sand::prelude::EntityDiagnostic::FixedPointOverflow"], kind: Variant, summary: "Selects the fixed point overflow semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::FixedPointOverflow::archetype", aliases: ["sand::prelude::EntityDiagnostic::FixedPointOverflow::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::FixedPointOverflow::derivation", aliases: ["sand::prelude::EntityDiagnostic::FixedPointOverflow::derivation"], kind: Field, summary: "Carries the derivation value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::FixedPointOverflow::detail", aliases: ["sand::prelude::EntityDiagnostic::FixedPointOverflow::detail"], kind: Field, summary: "Carries the detail value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidEnumEncoding", aliases: ["sand::prelude::EntityDiagnostic::InvalidEnumEncoding"], kind: Variant, summary: "Selects the invalid enum encoding semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidEnumEncoding::detail", aliases: ["sand::prelude::EntityDiagnostic::InvalidEnumEncoding::detail"], kind: Field, summary: "Carries the detail value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidEnumEncoding::field", aliases: ["sand::prelude::EntityDiagnostic::InvalidEnumEncoding::field"], kind: Field, summary: "Carries the field value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidEnumEncoding::schema", aliases: ["sand::prelude::EntityDiagnostic::InvalidEnumEncoding::schema"], kind: Field, summary: "Carries the schema value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidHealthResizePolicy", aliases: ["sand::prelude::EntityDiagnostic::InvalidHealthResizePolicy"], kind: Variant, summary: "Selects the invalid health resize policy semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidHealthResizePolicy::archetype", aliases: ["sand::prelude::EntityDiagnostic::InvalidHealthResizePolicy::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidHealthResizePolicy::detail", aliases: ["sand::prelude::EntityDiagnostic::InvalidHealthResizePolicy::detail"], kind: Field, summary: "Carries the detail value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidHealthResizePolicy::policy", aliases: ["sand::prelude::EntityDiagnostic::InvalidHealthResizePolicy::policy"], kind: Field, summary: "Carries the policy value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidRange", aliases: ["sand::prelude::EntityDiagnostic::InvalidRange"], kind: Variant, summary: "Selects the invalid range semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidRange::field", aliases: ["sand::prelude::EntityDiagnostic::InvalidRange::field"], kind: Field, summary: "Carries the field value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidRange::range", aliases: ["sand::prelude::EntityDiagnostic::InvalidRange::range"], kind: Field, summary: "Carries the range value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidRange::schema", aliases: ["sand::prelude::EntityDiagnostic::InvalidRange::schema"], kind: Field, summary: "Carries the schema value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidRawExtension", aliases: ["sand::prelude::EntityDiagnostic::InvalidRawExtension"], kind: Variant, summary: "Selects the invalid raw extension semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidRawExtension::archetype", aliases: ["sand::prelude::EntityDiagnostic::InvalidRawExtension::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidRawExtension::detail", aliases: ["sand::prelude::EntityDiagnostic::InvalidRawExtension::detail"], kind: Field, summary: "Carries the detail value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidRawExtension::extension", aliases: ["sand::prelude::EntityDiagnostic::InvalidRawExtension::extension"], kind: Field, summary: "Carries the extension value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidRefreshInterval", aliases: ["sand::prelude::EntityDiagnostic::InvalidRefreshInterval"], kind: Variant, summary: "Selects the invalid refresh interval semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidRefreshInterval::archetype", aliases: ["sand::prelude::EntityDiagnostic::InvalidRefreshInterval::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::InvalidRefreshInterval::property", aliases: ["sand::prelude::EntityDiagnostic::InvalidRefreshInterval::property"], kind: Field, summary: "Carries the property value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::MissingMigrationPath", aliases: ["sand::prelude::EntityDiagnostic::MissingMigrationPath"], kind: Variant, summary: "Selects the missing migration path semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::MissingMigrationPath::archetype", aliases: ["sand::prelude::EntityDiagnostic::MissingMigrationPath::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::MissingMigrationPath::from", aliases: ["sand::prelude::EntityDiagnostic::MissingMigrationPath::from"], kind: Field, summary: "Carries the from value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::MissingMigrationPath::to", aliases: ["sand::prelude::EntityDiagnostic::MissingMigrationPath::to"], kind: Field, summary: "Carries the to value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::NonFiniteCurve", aliases: ["sand::prelude::EntityDiagnostic::NonFiniteCurve"], kind: Variant, summary: "Selects the non finite curve semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::NonFiniteCurve::archetype", aliases: ["sand::prelude::EntityDiagnostic::NonFiniteCurve::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::NonFiniteCurve::derivation", aliases: ["sand::prelude::EntityDiagnostic::NonFiniteCurve::derivation"], kind: Field, summary: "Carries the derivation value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::NonFiniteCurve::value", aliases: ["sand::prelude::EntityDiagnostic::NonFiniteCurve::value"], kind: Field, summary: "Carries the value value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::PersistentReferenceMisuse", aliases: ["sand::prelude::EntityDiagnostic::PersistentReferenceMisuse"], kind: Variant, summary: "Selects the persistent reference misuse semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::PersistentReferenceMisuse::context", aliases: ["sand::prelude::EntityDiagnostic::PersistentReferenceMisuse::context"], kind: Field, summary: "Carries the context value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::ResourceCollision", aliases: ["sand::prelude::EntityDiagnostic::ResourceCollision"], kind: Variant, summary: "Selects the resource collision semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::ResourceCollision::first", aliases: ["sand::prelude::EntityDiagnostic::ResourceCollision::first"], kind: Field, summary: "Carries the first value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::ResourceCollision::resource", aliases: ["sand::prelude::EntityDiagnostic::ResourceCollision::resource"], kind: Field, summary: "Carries the resource value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::ResourceCollision::second", aliases: ["sand::prelude::EntityDiagnostic::ResourceCollision::second"], kind: Field, summary: "Carries the second value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnconstrainedAdoption", aliases: ["sand::prelude::EntityDiagnostic::UnconstrainedAdoption"], kind: Variant, summary: "Selects the unconstrained adoption semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnconstrainedAdoption::archetype", aliases: ["sand::prelude::EntityDiagnostic::UnconstrainedAdoption::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnconstrainedAdoption::detail", aliases: ["sand::prelude::EntityDiagnostic::UnconstrainedAdoption::detail"], kind: Field, summary: "Carries the detail value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsafePlayerMutation", aliases: ["sand::prelude::EntityDiagnostic::UnsafePlayerMutation"], kind: Variant, summary: "Selects the unsafe player mutation semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsafePlayerMutation::archetype", aliases: ["sand::prelude::EntityDiagnostic::UnsafePlayerMutation::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsafePlayerMutation::property", aliases: ["sand::prelude::EntityDiagnostic::UnsafePlayerMutation::property"], kind: Field, summary: "Carries the property value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedCapability", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedCapability"], kind: Variant, summary: "Selects the unsupported capability semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedCapability::archetype", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedCapability::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedCapability::entity_kind", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedCapability::entity_kind"], kind: Field, summary: "Carries the entity kind value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedCapability::property", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedCapability::property"], kind: Field, summary: "Carries the property value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedFunctionMacro", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedFunctionMacro"], kind: Variant, summary: "Selects the unsupported function macro semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedFunctionMacro::archetype", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedFunctionMacro::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedFunctionMacro::profile", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedFunctionMacro::profile"], kind: Field, summary: "Carries the profile value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedFunctionMacro::resource", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedFunctionMacro::resource"], kind: Field, summary: "Carries the resource value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedProfile", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedProfile"], kind: Variant, summary: "Selects the unsupported profile semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedProfile::archetype", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedProfile::archetype"], kind: Field, summary: "Carries the archetype value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedProfile::profile", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedProfile::profile"], kind: Field, summary: "Carries the profile value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedProfile::property", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedProfile::property"], kind: Field, summary: "Carries the property value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::UnsupportedProfile::reason", aliases: ["sand::prelude::EntityDiagnostic::UnsupportedProfile::reason"], kind: Field, summary: "Carries the reason value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityDiagnostic::code", aliases: ["sand::prelude::EntityDiagnostic::code"], kind: Method, summary: "Configures or performs code for the typed diagnostic entity API." }
register_entity_api! { path: "sand::entity::AnyEntity", aliases: ["sand::prelude::AnyEntity"], kind: Struct, summary: "Represents any entity as a compile-time Minecraft entity capability." }
register_entity_api! { path: "sand::entity::EntityKind", aliases: ["sand::prelude::EntityKind"], kind: Trait, summary: "Represents entity kind as a compile-time Minecraft entity capability." }
register_entity_api! { path: "sand::entity::EntityKind::LABEL", aliases: ["sand::prelude::EntityKind::LABEL"], kind: AssociatedConst, summary: "Provides the canonical label metadata for this typed entity abstraction." }
register_entity_api! { path: "sand::entity::KnownEntityKind", aliases: ["sand::prelude::KnownEntityKind"], kind: Trait, summary: "Represents known entity kind as a compile-time Minecraft entity capability." }
register_entity_api! { path: "sand::entity::KnownEntityKind::entity_type", aliases: ["sand::prelude::KnownEntityKind::entity_type"], kind: TraitMethod, summary: "Defines how an entity type supplies entity type to Sand's typed model." }
register_entity_api! { path: "sand::entity::LivingEntityKind", aliases: ["sand::prelude::LivingEntityKind"], kind: Trait, summary: "Represents living entity kind as a compile-time Minecraft entity capability." }
register_entity_api! { path: "sand::entity::MarkerKind", aliases: ["sand::prelude::MarkerKind"], kind: Struct, summary: "Represents marker kind as a compile-time Minecraft entity capability." }
register_entity_api! { path: "sand::entity::MutableLivingEntityKind", aliases: ["sand::prelude::MutableLivingEntityKind"], kind: Trait, summary: "Represents mutable living entity kind as a compile-time Minecraft entity capability." }
register_entity_api! { path: "sand::entity::PlayerKind", aliases: ["sand::prelude::PlayerKind"], kind: Struct, summary: "Represents player kind as a compile-time Minecraft entity capability." }
register_entity_api! { path: "sand::entity::SafeEntityDataWriteKind", aliases: ["sand::prelude::SafeEntityDataWriteKind"], kind: Trait, summary: "Represents safe entity data write kind as a compile-time Minecraft entity capability." }
register_entity_api! { path: "sand::entity::ZombieKind", aliases: ["sand::prelude::ZombieKind"], kind: Struct, summary: "Represents zombie kind as a compile-time Minecraft entity capability." }
register_entity_api! { path: "sand::entity::AttributeBinding", aliases: ["sand::prelude::AttributeBinding"], kind: Struct, summary: "Represents attribute binding in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::AttributeBinding::attribute", aliases: ["sand::prelude::AttributeBinding::attribute"], kind: Method, summary: "Configures or performs attribute for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeBinding::new", aliases: ["sand::prelude::AttributeBinding::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeBinding::ownership", aliases: ["sand::prelude::AttributeBinding::ownership"], kind: Method, summary: "Configures or performs ownership for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeBinding::ownership_policy", aliases: ["sand::prelude::AttributeBinding::ownership_policy"], kind: Method, summary: "Configures or performs ownership policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeBinding::refresh", aliases: ["sand::prelude::AttributeBinding::refresh"], kind: Method, summary: "Configures or performs refresh for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeBinding::refresh_policy", aliases: ["sand::prelude::AttributeBinding::refresh_policy"], kind: Method, summary: "Configures or performs refresh policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeBinding::source", aliases: ["sand::prelude::AttributeBinding::source"], kind: Method, summary: "Configures or performs source for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeBinding::validate", aliases: ["sand::prelude::AttributeBinding::validate"], kind: Method, summary: "Configures or performs validate for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeModifierBinding", aliases: ["sand::prelude::AttributeModifierBinding"], kind: Struct, summary: "Represents attribute modifier binding in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::AttributeModifierBinding::attribute", aliases: ["sand::prelude::AttributeModifierBinding::attribute"], kind: Method, summary: "Configures or performs attribute for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeModifierBinding::id", aliases: ["sand::prelude::AttributeModifierBinding::id"], kind: Method, summary: "Returns id for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeModifierBinding::new", aliases: ["sand::prelude::AttributeModifierBinding::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeModifierBinding::operation", aliases: ["sand::prelude::AttributeModifierBinding::operation"], kind: Method, summary: "Configures or performs operation for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeModifierBinding::ownership", aliases: ["sand::prelude::AttributeModifierBinding::ownership"], kind: Method, summary: "Configures or performs ownership for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeModifierBinding::ownership_policy", aliases: ["sand::prelude::AttributeModifierBinding::ownership_policy"], kind: Method, summary: "Configures or performs ownership policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeModifierBinding::refresh", aliases: ["sand::prelude::AttributeModifierBinding::refresh"], kind: Method, summary: "Configures or performs refresh for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeModifierBinding::refresh_policy", aliases: ["sand::prelude::AttributeModifierBinding::refresh_policy"], kind: Method, summary: "Configures or performs refresh policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeModifierBinding::source", aliases: ["sand::prelude::AttributeModifierBinding::source"], kind: Method, summary: "Configures or performs source for the typed property entity API." }
register_entity_api! { path: "sand::entity::AttributeModifierBinding::validate", aliases: ["sand::prelude::AttributeModifierBinding::validate"], kind: Method, summary: "Configures or performs validate for the typed property entity API." }
register_entity_api! { path: "sand::entity::CurrentHealthSync", aliases: ["sand::prelude::CurrentHealthSync"], kind: Enum, summary: "Represents current health sync in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::CurrentHealthSync::ApplyState", aliases: ["sand::prelude::CurrentHealthSync::ApplyState"], kind: Variant, summary: "Selects the apply state semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::CurrentHealthSync::Bidirectional", aliases: ["sand::prelude::CurrentHealthSync::Bidirectional"], kind: Variant, summary: "Selects the bidirectional semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::CurrentHealthSync::None", aliases: ["sand::prelude::CurrentHealthSync::None"], kind: Variant, summary: "Selects the none semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::CurrentHealthSync::ObserveNative", aliases: ["sand::prelude::CurrentHealthSync::ObserveNative"], kind: Variant, summary: "Selects the observe native semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EffectBinding", aliases: ["sand::prelude::EffectBinding"], kind: Struct, summary: "Represents effect binding in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::EffectBinding::amplifier", aliases: ["sand::prelude::EffectBinding::amplifier"], kind: Method, summary: "Configures or performs amplifier for the typed property entity API." }
register_entity_api! { path: "sand::entity::EffectBinding::amplifier_value", aliases: ["sand::prelude::EffectBinding::amplifier_value"], kind: Method, summary: "Configures or performs amplifier value for the typed property entity API." }
register_entity_api! { path: "sand::entity::EffectBinding::duration", aliases: ["sand::prelude::EffectBinding::duration"], kind: Method, summary: "Configures or performs duration for the typed property entity API." }
register_entity_api! { path: "sand::entity::EffectBinding::effect", aliases: ["sand::prelude::EffectBinding::effect"], kind: Method, summary: "Configures or performs effect for the typed property entity API." }
register_entity_api! { path: "sand::entity::EffectBinding::new", aliases: ["sand::prelude::EffectBinding::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::EffectBinding::ownership", aliases: ["sand::prelude::EffectBinding::ownership"], kind: Method, summary: "Configures or performs ownership for the typed property entity API." }
register_entity_api! { path: "sand::entity::EffectBinding::ownership_policy", aliases: ["sand::prelude::EffectBinding::ownership_policy"], kind: Method, summary: "Configures or performs ownership policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::EffectBinding::refresh", aliases: ["sand::prelude::EffectBinding::refresh"], kind: Method, summary: "Configures or performs refresh for the typed property entity API." }
register_entity_api! { path: "sand::entity::EffectBinding::refresh_policy", aliases: ["sand::prelude::EffectBinding::refresh_policy"], kind: Method, summary: "Configures or performs refresh policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::EffectBinding::validate", aliases: ["sand::prelude::EffectBinding::validate"], kind: Method, summary: "Configures or performs validate for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityEventId", aliases: ["sand::prelude::EntityEventId"], kind: Struct, summary: "Represents entity event id in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::EntityEventId::location", aliases: ["sand::prelude::EntityEventId::location"], kind: Method, summary: "Configures or performs location for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityEventId::new", aliases: ["sand::prelude::EntityEventId::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityNbtBinding", aliases: ["sand::prelude::EntityNbtBinding"], kind: Struct, summary: "Represents entity nbt binding in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::EntityNbtBinding::new", aliases: ["sand::prelude::EntityNbtBinding::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityNbtBinding::ownership", aliases: ["sand::prelude::EntityNbtBinding::ownership"], kind: Method, summary: "Configures or performs ownership for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityNbtBinding::ownership_policy", aliases: ["sand::prelude::EntityNbtBinding::ownership_policy"], kind: Method, summary: "Configures or performs ownership policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityNbtBinding::property", aliases: ["sand::prelude::EntityNbtBinding::property"], kind: Method, summary: "Configures or performs property for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityNbtBinding::refresh", aliases: ["sand::prelude::EntityNbtBinding::refresh"], kind: Method, summary: "Configures or performs refresh for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityNbtBinding::refresh_policy", aliases: ["sand::prelude::EntityNbtBinding::refresh_policy"], kind: Method, summary: "Configures or performs refresh policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityNbtBinding::validate_for", aliases: ["sand::prelude::EntityNbtBinding::validate_for"], kind: Method, summary: "Checks validate for for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityNbtBinding::value", aliases: ["sand::prelude::EntityNbtBinding::value"], kind: Method, summary: "Configures or performs value for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityNbtProperty", aliases: ["sand::prelude::EntityNbtProperty"], kind: Enum, summary: "Represents entity nbt property in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::EntityNbtProperty::Absorption", aliases: ["sand::prelude::EntityNbtProperty::Absorption"], kind: Variant, summary: "Selects the absorption semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtProperty::AirTicks", aliases: ["sand::prelude::EntityNbtProperty::AirTicks"], kind: Variant, summary: "Selects the air ticks semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtProperty::FallDistance", aliases: ["sand::prelude::EntityNbtProperty::FallDistance"], kind: Variant, summary: "Selects the fall distance semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtProperty::FireTicks", aliases: ["sand::prelude::EntityNbtProperty::FireTicks"], kind: Variant, summary: "Selects the fire ticks semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtProperty::FrozenTicks", aliases: ["sand::prelude::EntityNbtProperty::FrozenTicks"], kind: Variant, summary: "Selects the frozen ticks semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtProperty::Glowing", aliases: ["sand::prelude::EntityNbtProperty::Glowing"], kind: Variant, summary: "Selects the glowing semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtProperty::Invulnerable", aliases: ["sand::prelude::EntityNbtProperty::Invulnerable"], kind: Variant, summary: "Selects the invulnerable semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtProperty::NoGravity", aliases: ["sand::prelude::EntityNbtProperty::NoGravity"], kind: Variant, summary: "Selects the no gravity semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtProperty::Persistent", aliases: ["sand::prelude::EntityNbtProperty::Persistent"], kind: Variant, summary: "Selects the persistent semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtProperty::Silent", aliases: ["sand::prelude::EntityNbtProperty::Silent"], kind: Variant, summary: "Selects the silent semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtProperty::path", aliases: ["sand::prelude::EntityNbtProperty::path"], kind: Method, summary: "Configures or performs path for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityNbtProperty::wire_type", aliases: ["sand::prelude::EntityNbtProperty::wire_type"], kind: Method, summary: "Configures or performs wire type for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityNbtType", aliases: ["sand::prelude::EntityNbtType"], kind: Enum, summary: "Represents entity nbt type in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::EntityNbtType::Boolean", aliases: ["sand::prelude::EntityNbtType::Boolean"], kind: Variant, summary: "Selects the boolean semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtType::Compound", aliases: ["sand::prelude::EntityNbtType::Compound"], kind: Variant, summary: "Selects the compound semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtType::Float", aliases: ["sand::prelude::EntityNbtType::Float"], kind: Variant, summary: "Selects the float semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtType::Integer", aliases: ["sand::prelude::EntityNbtType::Integer"], kind: Variant, summary: "Selects the integer semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtType::String", aliases: ["sand::prelude::EntityNbtType::String"], kind: Variant, summary: "Selects the string semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtValue", aliases: ["sand::prelude::EntityNbtValue"], kind: Enum, summary: "Represents entity nbt value in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::EntityNbtValue::Boolean", aliases: ["sand::prelude::EntityNbtValue::Boolean"], kind: Variant, summary: "Selects the boolean semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtValue::Boolean::0", aliases: ["sand::prelude::EntityNbtValue::Boolean::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityNbtValue::Fixed", aliases: ["sand::prelude::EntityNbtValue::Fixed"], kind: Variant, summary: "Selects the fixed semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtValue::Fixed::scale", aliases: ["sand::prelude::EntityNbtValue::Fixed::scale"], kind: Field, summary: "Carries the scale value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityNbtValue::Fixed::units", aliases: ["sand::prelude::EntityNbtValue::Fixed::units"], kind: Field, summary: "Carries the units value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityNbtValue::Integer", aliases: ["sand::prelude::EntityNbtValue::Integer"], kind: Variant, summary: "Selects the integer semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityNbtValue::Integer::0", aliases: ["sand::prelude::EntityNbtValue::Integer::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityNbtValue::fixed", aliases: ["sand::prelude::EntityNbtValue::fixed"], kind: Method, summary: "Configures or performs fixed for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityTag", aliases: ["sand::prelude::EntityTag"], kind: Struct, summary: "Represents entity tag in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::EntityTag::as_str", aliases: ["sand::prelude::EntityTag::as_str"], kind: Method, summary: "Returns as str for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityTag::new", aliases: ["sand::prelude::EntityTag::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityTeam", aliases: ["sand::prelude::EntityTeam"], kind: Struct, summary: "Represents entity team in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::EntityTeam::as_str", aliases: ["sand::prelude::EntityTeam::as_str"], kind: Method, summary: "Returns as str for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityTeam::new", aliases: ["sand::prelude::EntityTeam::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityText", aliases: ["sand::prelude::EntityText"], kind: Struct, summary: "Represents entity text in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::EntityText::color_last", aliases: ["sand::prelude::EntityText::color_last"], kind: Method, summary: "Configures or performs color last for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityText::enum_value", aliases: ["sand::prelude::EntityText::enum_value"], kind: Method, summary: "Configures or performs enum value for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityText::flag", aliases: ["sand::prelude::EntityText::flag"], kind: Method, summary: "Configures or performs flag for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityText::literal", aliases: ["sand::prelude::EntityText::literal"], kind: Method, summary: "Configures or performs literal for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityText::new", aliases: ["sand::prelude::EntityText::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityText::score", aliases: ["sand::prelude::EntityText::score"], kind: Method, summary: "Configures or performs score for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityText::segments", aliases: ["sand::prelude::EntityText::segments"], kind: Method, summary: "Configures or performs segments for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityTextSegment", aliases: ["sand::prelude::EntityTextSegment"], kind: Enum, summary: "Represents entity text segment in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Enum", aliases: ["sand::prelude::EntityTextSegment::Enum"], kind: Variant, summary: "Selects the enum semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Enum::color", aliases: ["sand::prelude::EntityTextSegment::Enum::color"], kind: Field, summary: "Carries the color value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Enum::dirty_objective", aliases: ["sand::prelude::EntityTextSegment::Enum::dirty_objective"], kind: Field, summary: "Carries the dirty objective value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Enum::objective", aliases: ["sand::prelude::EntityTextSegment::Enum::objective"], kind: Field, summary: "Carries the objective value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Enum::variants", aliases: ["sand::prelude::EntityTextSegment::Enum::variants"], kind: Field, summary: "Carries the variants value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Flag", aliases: ["sand::prelude::EntityTextSegment::Flag"], kind: Variant, summary: "Selects the flag semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Flag::color", aliases: ["sand::prelude::EntityTextSegment::Flag::color"], kind: Field, summary: "Carries the color value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Flag::dirty_objective", aliases: ["sand::prelude::EntityTextSegment::Flag::dirty_objective"], kind: Field, summary: "Carries the dirty objective value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Flag::disabled", aliases: ["sand::prelude::EntityTextSegment::Flag::disabled"], kind: Field, summary: "Carries the disabled value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Flag::enabled", aliases: ["sand::prelude::EntityTextSegment::Flag::enabled"], kind: Field, summary: "Carries the enabled value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Flag::objective", aliases: ["sand::prelude::EntityTextSegment::Flag::objective"], kind: Field, summary: "Carries the objective value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Literal", aliases: ["sand::prelude::EntityTextSegment::Literal"], kind: Variant, summary: "Selects the literal semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Literal::color", aliases: ["sand::prelude::EntityTextSegment::Literal::color"], kind: Field, summary: "Carries the color value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Literal::text", aliases: ["sand::prelude::EntityTextSegment::Literal::text"], kind: Field, summary: "Carries the text value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Numeric", aliases: ["sand::prelude::EntityTextSegment::Numeric"], kind: Variant, summary: "Selects the numeric semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Numeric::color", aliases: ["sand::prelude::EntityTextSegment::Numeric::color"], kind: Field, summary: "Carries the color value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Numeric::dirty_objective", aliases: ["sand::prelude::EntityTextSegment::Numeric::dirty_objective"], kind: Field, summary: "Carries the dirty objective value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::Numeric::objective", aliases: ["sand::prelude::EntityTextSegment::Numeric::objective"], kind: Field, summary: "Carries the objective value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EntityTextSegment::color", aliases: ["sand::prelude::EntityTextSegment::color"], kind: Method, summary: "Configures or performs color for the typed property entity API." }
register_entity_api! { path: "sand::entity::EquipmentBinding", aliases: ["sand::prelude::EquipmentBinding"], kind: Struct, summary: "Represents equipment binding in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::EquipmentBinding::new", aliases: ["sand::prelude::EquipmentBinding::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::EquipmentBinding::ownership", aliases: ["sand::prelude::EquipmentBinding::ownership"], kind: Method, summary: "Configures or performs ownership for the typed property entity API." }
register_entity_api! { path: "sand::entity::EquipmentBinding::ownership_policy", aliases: ["sand::prelude::EquipmentBinding::ownership_policy"], kind: Method, summary: "Configures or performs ownership policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::EquipmentBinding::refresh", aliases: ["sand::prelude::EquipmentBinding::refresh"], kind: Method, summary: "Configures or performs refresh for the typed property entity API." }
register_entity_api! { path: "sand::entity::EquipmentBinding::refresh_policy", aliases: ["sand::prelude::EquipmentBinding::refresh_policy"], kind: Method, summary: "Configures or performs refresh policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::EquipmentBinding::slot", aliases: ["sand::prelude::EquipmentBinding::slot"], kind: Method, summary: "Configures or performs slot for the typed property entity API." }
register_entity_api! { path: "sand::entity::EquipmentBinding::stack", aliases: ["sand::prelude::EquipmentBinding::stack"], kind: Method, summary: "Configures or performs stack for the typed property entity API." }
register_entity_api! { path: "sand::entity::EquipmentBinding::validate", aliases: ["sand::prelude::EquipmentBinding::validate"], kind: Method, summary: "Configures or performs validate for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding", aliases: ["sand::prelude::HealthBinding"], kind: Struct, summary: "Represents health binding in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::HealthBinding::current_health", aliases: ["sand::prelude::HealthBinding::current_health"], kind: Method, summary: "Configures or performs current health for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::current_health_field", aliases: ["sand::prelude::HealthBinding::current_health_field"], kind: Method, summary: "Configures or performs current health field for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::current_health_sync", aliases: ["sand::prelude::HealthBinding::current_health_sync"], kind: Method, summary: "Configures or performs current health sync for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::max_health_field", aliases: ["sand::prelude::HealthBinding::max_health_field"], kind: Method, summary: "Configures or performs max health field for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::new", aliases: ["sand::prelude::HealthBinding::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::observation_interval", aliases: ["sand::prelude::HealthBinding::observation_interval"], kind: Method, summary: "Configures or performs observation interval for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::observe_native_every", aliases: ["sand::prelude::HealthBinding::observe_native_every"], kind: Method, summary: "Configures or performs observe native every for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::ownership", aliases: ["sand::prelude::HealthBinding::ownership"], kind: Method, summary: "Configures or performs ownership for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::ownership_policy", aliases: ["sand::prelude::HealthBinding::ownership_policy"], kind: Method, summary: "Configures or performs ownership policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::refresh", aliases: ["sand::prelude::HealthBinding::refresh"], kind: Method, summary: "Configures or performs refresh for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::refresh_policy", aliases: ["sand::prelude::HealthBinding::refresh_policy"], kind: Method, summary: "Configures or performs refresh policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::resize", aliases: ["sand::prelude::HealthBinding::resize"], kind: Method, summary: "Configures or performs resize for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::resize_policy", aliases: ["sand::prelude::HealthBinding::resize_policy"], kind: Method, summary: "Configures or performs resize policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthBinding::validate", aliases: ["sand::prelude::HealthBinding::validate"], kind: Method, summary: "Configures or performs validate for the typed property entity API." }
register_entity_api! { path: "sand::entity::HealthResizePolicy", aliases: ["sand::prelude::HealthResizePolicy"], kind: Enum, summary: "Represents health resize policy in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::HealthResizePolicy::PreserveAbsolute", aliases: ["sand::prelude::HealthResizePolicy::PreserveAbsolute"], kind: Variant, summary: "Selects the preserve absolute semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::HealthResizePolicy::PreserveRatio", aliases: ["sand::prelude::HealthResizePolicy::PreserveRatio"], kind: Variant, summary: "Selects the preserve ratio semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::HealthResizePolicy::Refill", aliases: ["sand::prelude::HealthResizePolicy::Refill"], kind: Variant, summary: "Selects the refill semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::NameBinding", aliases: ["sand::prelude::NameBinding"], kind: Struct, summary: "Represents name binding in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::NameBinding::is_visible", aliases: ["sand::prelude::NameBinding::is_visible"], kind: Method, summary: "Checks is visible for the typed property entity API." }
register_entity_api! { path: "sand::entity::NameBinding::new", aliases: ["sand::prelude::NameBinding::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::NameBinding::ownership", aliases: ["sand::prelude::NameBinding::ownership"], kind: Method, summary: "Configures or performs ownership for the typed property entity API." }
register_entity_api! { path: "sand::entity::NameBinding::ownership_policy", aliases: ["sand::prelude::NameBinding::ownership_policy"], kind: Method, summary: "Configures or performs ownership policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::NameBinding::refresh", aliases: ["sand::prelude::NameBinding::refresh"], kind: Method, summary: "Configures or performs refresh for the typed property entity API." }
register_entity_api! { path: "sand::entity::NameBinding::refresh_policy", aliases: ["sand::prelude::NameBinding::refresh_policy"], kind: Method, summary: "Configures or performs refresh policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::NameBinding::text", aliases: ["sand::prelude::NameBinding::text"], kind: Method, summary: "Configures or performs text for the typed property entity API." }
register_entity_api! { path: "sand::entity::NameBinding::validate", aliases: ["sand::prelude::NameBinding::validate"], kind: Method, summary: "Configures or performs validate for the typed property entity API." }
register_entity_api! { path: "sand::entity::NameBinding::visible", aliases: ["sand::prelude::NameBinding::visible"], kind: Method, summary: "Configures or performs visible for the typed property entity API." }
register_entity_api! { path: "sand::entity::NumericPropertySource", aliases: ["sand::prelude::NumericPropertySource"], kind: Enum, summary: "Represents numeric property source in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::NumericPropertySource::Fixed", aliases: ["sand::prelude::NumericPropertySource::Fixed"], kind: Variant, summary: "Selects the fixed semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::NumericPropertySource::Fixed::scale", aliases: ["sand::prelude::NumericPropertySource::Fixed::scale"], kind: Field, summary: "Carries the scale value required by this typed entity case." }
register_entity_api! { path: "sand::entity::NumericPropertySource::Fixed::units", aliases: ["sand::prelude::NumericPropertySource::Fixed::units"], kind: Field, summary: "Carries the units value required by this typed entity case." }
register_entity_api! { path: "sand::entity::NumericPropertySource::StateScore", aliases: ["sand::prelude::NumericPropertySource::StateScore"], kind: Variant, summary: "Selects the state score semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::NumericPropertySource::StateScore::dirty_objective", aliases: ["sand::prelude::NumericPropertySource::StateScore::dirty_objective"], kind: Field, summary: "Carries the dirty objective value required by this typed entity case." }
register_entity_api! { path: "sand::entity::NumericPropertySource::StateScore::objective", aliases: ["sand::prelude::NumericPropertySource::StateScore::objective"], kind: Field, summary: "Carries the objective value required by this typed entity case." }
register_entity_api! { path: "sand::entity::NumericPropertySource::fixed", aliases: ["sand::prelude::NumericPropertySource::fixed"], kind: Method, summary: "Configures or performs fixed for the typed property entity API." }
register_entity_api! { path: "sand::entity::NumericPropertySource::state", aliases: ["sand::prelude::NumericPropertySource::state"], kind: Method, summary: "Configures or performs state for the typed property entity API." }
register_entity_api! { path: "sand::entity::OwnershipPolicy", aliases: ["sand::prelude::OwnershipPolicy"], kind: Enum, summary: "Represents ownership policy in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::OwnershipPolicy::Exact", aliases: ["sand::prelude::OwnershipPolicy::Exact"], kind: Variant, summary: "Selects the exact semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::OwnershipPolicy::InitializeMissing", aliases: ["sand::prelude::OwnershipPolicy::InitializeMissing"], kind: Variant, summary: "Selects the initialize missing semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::OwnershipPolicy::Observe", aliases: ["sand::prelude::OwnershipPolicy::Observe"], kind: Variant, summary: "Selects the observe semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::OwnershipPolicy::Preserve", aliases: ["sand::prelude::OwnershipPolicy::Preserve"], kind: Variant, summary: "Selects the preserve semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::OwnershipPolicy::ReconcileWhenDirty", aliases: ["sand::prelude::OwnershipPolicy::ReconcileWhenDirty"], kind: Variant, summary: "Selects the reconcile when dirty semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::OwnershipPolicy::claims_write_ownership", aliases: ["sand::prelude::OwnershipPolicy::claims_write_ownership"], kind: Method, summary: "Configures or performs claims write ownership for the typed property entity API." }
register_entity_api! { path: "sand::entity::OwnershipPolicy::observes_native_state", aliases: ["sand::prelude::OwnershipPolicy::observes_native_state"], kind: Method, summary: "Configures or performs observes native state for the typed property entity API." }
register_entity_api! { path: "sand::entity::PropertyNameError", aliases: [], kind: Struct, summary: "Represents property name error in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::PropertyNameError::kind", aliases: [], kind: Method, summary: "Configures or performs kind for the typed property entity API." }
register_entity_api! { path: "sand::entity::PropertyNameError::reason", aliases: [], kind: Method, summary: "Configures or performs reason for the typed property entity API." }
register_entity_api! { path: "sand::entity::PropertyNameError::value", aliases: [], kind: Method, summary: "Configures or performs value for the typed property entity API." }
register_entity_api! { path: "sand::entity::RawEntityProperty", aliases: ["sand::prelude::RawEntityProperty"], kind: Struct, summary: "Represents raw entity property in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::RawEntityProperty::access", aliases: ["sand::prelude::RawEntityProperty::access"], kind: Method, summary: "Configures or performs access for the typed property entity API." }
register_entity_api! { path: "sand::entity::RawEntityProperty::new", aliases: ["sand::prelude::RawEntityProperty::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::RawEntityProperty::path", aliases: ["sand::prelude::RawEntityProperty::path"], kind: Method, summary: "Configures or performs path for the typed property entity API." }
register_entity_api! { path: "sand::entity::RawEntityProperty::validate_for", aliases: ["sand::prelude::RawEntityProperty::validate_for"], kind: Method, summary: "Checks validate for for the typed property entity API." }
register_entity_api! { path: "sand::entity::RawEntityProperty::wire_type", aliases: ["sand::prelude::RawEntityProperty::wire_type"], kind: Method, summary: "Configures or performs wire type for the typed property entity API." }
register_entity_api! { path: "sand::entity::RawEntityStateField", aliases: ["sand::prelude::RawEntityStateField"], kind: Struct, summary: "Represents raw entity state field in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::RawEntityStateField::backend", aliases: ["sand::prelude::RawEntityStateField::backend"], kind: Method, summary: "Configures or performs backend for the typed property entity API." }
register_entity_api! { path: "sand::entity::RawEntityStateField::name", aliases: ["sand::prelude::RawEntityStateField::name"], kind: Method, summary: "Returns name for the typed property entity API." }
register_entity_api! { path: "sand::entity::RawEntityStateField::new", aliases: ["sand::prelude::RawEntityStateField::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::RawPropertyAccess", aliases: [], kind: Enum, summary: "Represents raw property access in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::RawPropertyAccess::Mutable", aliases: [], kind: Variant, summary: "Selects the mutable semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RawPropertyAccess::ReadOnly", aliases: [], kind: Variant, summary: "Selects the read only semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RawStateBackend", aliases: [], kind: Enum, summary: "Represents raw state backend in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::RawStateBackend::SandStorage", aliases: [], kind: Variant, summary: "Selects the sand storage semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RawStateBackend::Scoreboard", aliases: [], kind: Variant, summary: "Selects the scoreboard semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RawStateBackend::Tag", aliases: [], kind: Variant, summary: "Selects the tag semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RefreshPolicy", aliases: ["sand::prelude::RefreshPolicy"], kind: Enum, summary: "Represents refresh policy in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::RefreshPolicy::Every", aliases: ["sand::prelude::RefreshPolicy::Every"], kind: Variant, summary: "Selects the every semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RefreshPolicy::Every::0", aliases: ["sand::prelude::RefreshPolicy::Every::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::RefreshPolicy::Initialize", aliases: ["sand::prelude::RefreshPolicy::Initialize"], kind: Variant, summary: "Selects the initialize semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RefreshPolicy::Manual", aliases: ["sand::prelude::RefreshPolicy::Manual"], kind: Variant, summary: "Selects the manual semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RefreshPolicy::OnEvent", aliases: ["sand::prelude::RefreshPolicy::OnEvent"], kind: Variant, summary: "Selects the on event semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RefreshPolicy::OnEvent::0", aliases: ["sand::prelude::RefreshPolicy::OnEvent::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::RefreshPolicy::OnFunction", aliases: ["sand::prelude::RefreshPolicy::OnFunction"], kind: Variant, summary: "Selects the on function semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RefreshPolicy::OnFunction::0", aliases: ["sand::prelude::RefreshPolicy::OnFunction::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::RefreshPolicy::WhenSourceChanges", aliases: ["sand::prelude::RefreshPolicy::WhenSourceChanges"], kind: Variant, summary: "Selects the when source changes semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::RefreshPolicy::validate", aliases: ["sand::prelude::RefreshPolicy::validate"], kind: Method, summary: "Configures or performs validate for the typed property entity API." }
register_entity_api! { path: "sand::entity::TagBinding", aliases: ["sand::prelude::TagBinding"], kind: Struct, summary: "Represents tag binding in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::TagBinding::new", aliases: ["sand::prelude::TagBinding::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::TagBinding::ownership", aliases: ["sand::prelude::TagBinding::ownership"], kind: Method, summary: "Configures or performs ownership for the typed property entity API." }
register_entity_api! { path: "sand::entity::TagBinding::ownership_policy", aliases: ["sand::prelude::TagBinding::ownership_policy"], kind: Method, summary: "Configures or performs ownership policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::TagBinding::refresh", aliases: ["sand::prelude::TagBinding::refresh"], kind: Method, summary: "Configures or performs refresh for the typed property entity API." }
register_entity_api! { path: "sand::entity::TagBinding::refresh_policy", aliases: ["sand::prelude::TagBinding::refresh_policy"], kind: Method, summary: "Configures or performs refresh policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::TagBinding::tag", aliases: ["sand::prelude::TagBinding::tag"], kind: Method, summary: "Configures or performs tag for the typed property entity API." }
register_entity_api! { path: "sand::entity::TagBinding::validate", aliases: ["sand::prelude::TagBinding::validate"], kind: Method, summary: "Configures or performs validate for the typed property entity API." }
register_entity_api! { path: "sand::entity::TeamBinding", aliases: ["sand::prelude::TeamBinding"], kind: Struct, summary: "Represents team binding in an entity archetype's semantic property bindings." }
register_entity_api! { path: "sand::entity::TeamBinding::new", aliases: ["sand::prelude::TeamBinding::new"], kind: Method, summary: "Constructs new for the typed property entity API." }
register_entity_api! { path: "sand::entity::TeamBinding::ownership", aliases: ["sand::prelude::TeamBinding::ownership"], kind: Method, summary: "Configures or performs ownership for the typed property entity API." }
register_entity_api! { path: "sand::entity::TeamBinding::ownership_policy", aliases: ["sand::prelude::TeamBinding::ownership_policy"], kind: Method, summary: "Configures or performs ownership policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::TeamBinding::refresh", aliases: ["sand::prelude::TeamBinding::refresh"], kind: Method, summary: "Configures or performs refresh for the typed property entity API." }
register_entity_api! { path: "sand::entity::TeamBinding::refresh_policy", aliases: ["sand::prelude::TeamBinding::refresh_policy"], kind: Method, summary: "Configures or performs refresh policy for the typed property entity API." }
register_entity_api! { path: "sand::entity::TeamBinding::team", aliases: ["sand::prelude::TeamBinding::team"], kind: Method, summary: "Configures or performs team for the typed property entity API." }
register_entity_api! { path: "sand::entity::TeamBinding::validate", aliases: ["sand::prelude::TeamBinding::validate"], kind: Method, summary: "Configures or performs validate for the typed property entity API." }
register_entity_api! { path: "sand::entity::EntityQueries", aliases: ["sand::prelude::EntityQueries"], kind: TypeAlias, summary: "Represents entity queries in Sand's cardinality-aware typed entity query model." }
register_entity_api! { path: "sand::entity::EntityQuery", aliases: ["sand::prelude::EntityQuery"], kind: Struct, summary: "Represents entity query in Sand's cardinality-aware typed entity query model." }
register_entity_api! { path: "sand::entity::EntityQuery::distance_range", aliases: ["sand::entity::EntityQueries::distance_range", "sand::entity::SingleEntityQuery::distance_range", "sand::prelude::EntityQueries::distance_range", "sand::prelude::EntityQuery::distance_range", "sand::prelude::SingleEntityQuery::distance_range"], kind: Method, summary: "Configures or performs distance range for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::each", aliases: ["sand::entity::EntityQueries::each", "sand::entity::SingleEntityQuery::each", "sand::prelude::EntityQueries::each", "sand::prelude::EntityQuery::each", "sand::prelude::SingleEntityQuery::each"], kind: Method, summary: "Configures or performs each for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::entities", aliases: ["sand::entity::EntityQueries::entities", "sand::entity::SingleEntityQuery::entities", "sand::prelude::EntityQueries::entities", "sand::prelude::EntityQuery::entities", "sand::prelude::SingleEntityQuery::entities"], kind: Method, summary: "Configures or performs entities for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::entity_type", aliases: ["sand::entity::EntityQueries::entity_type", "sand::entity::SingleEntityQuery::entity_type", "sand::prelude::EntityQueries::entity_type", "sand::prelude::EntityQuery::entity_type", "sand::prelude::SingleEntityQuery::entity_type"], kind: Method, summary: "Configures or performs entity type for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::excluding_players", aliases: ["sand::entity::EntityQueries::excluding_players", "sand::entity::SingleEntityQuery::excluding_players", "sand::prelude::EntityQueries::excluding_players", "sand::prelude::EntityQuery::excluding_players", "sand::prelude::SingleEntityQuery::excluding_players"], kind: Method, summary: "Configures or performs excluding players for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::excluding_self", aliases: ["sand::entity::EntityQueries::excluding_self", "sand::entity::SingleEntityQuery::excluding_self", "sand::prelude::EntityQueries::excluding_self", "sand::prelude::EntityQuery::excluding_self", "sand::prelude::SingleEntityQuery::excluding_self"], kind: Method, summary: "Configures or performs excluding self for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::get", aliases: ["sand::entity::EntityQueries::get", "sand::entity::SingleEntityQuery::get", "sand::prelude::EntityQueries::get", "sand::prelude::EntityQuery::get", "sand::prelude::SingleEntityQuery::get"], kind: Method, summary: "Configures or performs get for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::limit", aliases: ["sand::entity::EntityQueries::limit", "sand::entity::SingleEntityQuery::limit", "sand::prelude::EntityQueries::limit", "sand::prelude::EntityQuery::limit", "sand::prelude::SingleEntityQuery::limit"], kind: Method, summary: "Configures or performs limit for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::nearby", aliases: ["sand::entity::EntityQueries::nearby", "sand::entity::SingleEntityQuery::nearby", "sand::prelude::EntityQueries::nearby", "sand::prelude::EntityQuery::nearby", "sand::prelude::SingleEntityQuery::nearby"], kind: Method, summary: "Configures or performs nearby for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::nearest", aliases: ["sand::entity::EntityQueries::nearest", "sand::entity::SingleEntityQuery::nearest", "sand::prelude::EntityQueries::nearest", "sand::prelude::EntityQuery::nearest", "sand::prelude::SingleEntityQuery::nearest"], kind: Method, summary: "Configures or performs nearest for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::not_entity_type", aliases: ["sand::entity::EntityQueries::not_entity_type", "sand::entity::SingleEntityQuery::not_entity_type", "sand::prelude::EntityQueries::not_entity_type", "sand::prelude::EntityQuery::not_entity_type", "sand::prelude::SingleEntityQuery::not_entity_type"], kind: Method, summary: "Configures or performs not entity type for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::selector", aliases: ["sand::entity::EntityQueries::selector", "sand::entity::SingleEntityQuery::selector", "sand::prelude::EntityQueries::selector", "sand::prelude::EntityQuery::selector", "sand::prelude::SingleEntityQuery::selector"], kind: Method, summary: "Returns selector for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::sort", aliases: ["sand::entity::EntityQueries::sort", "sand::entity::SingleEntityQuery::sort", "sand::prelude::EntityQueries::sort", "sand::prelude::EntityQuery::sort", "sand::prelude::SingleEntityQuery::sort"], kind: Method, summary: "Configures or performs sort for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::state", aliases: ["sand::entity::EntityQueries::state", "sand::entity::SingleEntityQuery::state", "sand::prelude::EntityQueries::state", "sand::prelude::EntityQuery::state", "sand::prelude::SingleEntityQuery::state"], kind: Method, summary: "Configures or performs state for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::tag", aliases: ["sand::entity::EntityQueries::tag", "sand::entity::SingleEntityQuery::tag", "sand::prelude::EntityQueries::tag", "sand::prelude::EntityQuery::tag", "sand::prelude::SingleEntityQuery::tag"], kind: Method, summary: "Configures or performs tag for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::within_blocks", aliases: ["sand::entity::EntityQueries::within_blocks", "sand::entity::SingleEntityQuery::within_blocks", "sand::prelude::EntityQueries::within_blocks", "sand::prelude::EntityQuery::within_blocks", "sand::prelude::SingleEntityQuery::within_blocks"], kind: Method, summary: "Configures or performs within blocks for the typed query entity API." }
register_entity_api! { path: "sand::entity::EntityQuery::without_tag", aliases: ["sand::entity::EntityQueries::without_tag", "sand::entity::SingleEntityQuery::without_tag", "sand::prelude::EntityQueries::without_tag", "sand::prelude::EntityQuery::without_tag", "sand::prelude::SingleEntityQuery::without_tag"], kind: Method, summary: "Configures or performs without tag for the typed query entity API." }
register_entity_api! { path: "sand::entity::PlayerQueries", aliases: ["sand::prelude::PlayerQueries"], kind: TypeAlias, summary: "Represents player queries in Sand's cardinality-aware typed entity query model." }
register_entity_api! { path: "sand::entity::PlayerQuery", aliases: ["sand::prelude::PlayerQuery"], kind: Struct, summary: "Represents player query in Sand's cardinality-aware typed entity query model." }
register_entity_api! { path: "sand::entity::PlayerQuery::distance_range", aliases: ["sand::entity::PlayerQueries::distance_range", "sand::entity::SinglePlayerQuery::distance_range", "sand::prelude::PlayerQueries::distance_range", "sand::prelude::PlayerQuery::distance_range", "sand::prelude::SinglePlayerQuery::distance_range"], kind: Method, summary: "Configures or performs distance range for the typed query entity API." }
register_entity_api! { path: "sand::entity::PlayerQuery::each", aliases: ["sand::entity::PlayerQueries::each", "sand::entity::SinglePlayerQuery::each", "sand::prelude::PlayerQueries::each", "sand::prelude::PlayerQuery::each", "sand::prelude::SinglePlayerQuery::each"], kind: Method, summary: "Configures or performs each for the typed query entity API." }
register_entity_api! { path: "sand::entity::PlayerQuery::get", aliases: ["sand::entity::PlayerQueries::get", "sand::entity::SinglePlayerQuery::get", "sand::prelude::PlayerQueries::get", "sand::prelude::PlayerQuery::get", "sand::prelude::SinglePlayerQuery::get"], kind: Method, summary: "Configures or performs get for the typed query entity API." }
register_entity_api! { path: "sand::entity::PlayerQuery::limit", aliases: ["sand::entity::PlayerQueries::limit", "sand::entity::SinglePlayerQuery::limit", "sand::prelude::PlayerQueries::limit", "sand::prelude::PlayerQuery::limit", "sand::prelude::SinglePlayerQuery::limit"], kind: Method, summary: "Configures or performs limit for the typed query entity API." }
register_entity_api! { path: "sand::entity::PlayerQuery::nearest", aliases: ["sand::entity::PlayerQueries::nearest", "sand::entity::SinglePlayerQuery::nearest", "sand::prelude::PlayerQueries::nearest", "sand::prelude::PlayerQuery::nearest", "sand::prelude::SinglePlayerQuery::nearest"], kind: Method, summary: "Configures or performs nearest for the typed query entity API." }
register_entity_api! { path: "sand::entity::PlayerQuery::players", aliases: ["sand::entity::PlayerQueries::players", "sand::entity::SinglePlayerQuery::players", "sand::prelude::PlayerQueries::players", "sand::prelude::PlayerQuery::players", "sand::prelude::SinglePlayerQuery::players"], kind: Method, summary: "Configures or performs players for the typed query entity API." }
register_entity_api! { path: "sand::entity::PlayerQuery::selector", aliases: ["sand::entity::PlayerQueries::selector", "sand::entity::SinglePlayerQuery::selector", "sand::prelude::PlayerQueries::selector", "sand::prelude::PlayerQuery::selector", "sand::prelude::SinglePlayerQuery::selector"], kind: Method, summary: "Returns selector for the typed query entity API." }
register_entity_api! { path: "sand::entity::PlayerQuery::sort", aliases: ["sand::entity::PlayerQueries::sort", "sand::entity::SinglePlayerQuery::sort", "sand::prelude::PlayerQueries::sort", "sand::prelude::PlayerQuery::sort", "sand::prelude::SinglePlayerQuery::sort"], kind: Method, summary: "Configures or performs sort for the typed query entity API." }
register_entity_api! { path: "sand::entity::PlayerQuery::tag", aliases: ["sand::entity::PlayerQueries::tag", "sand::entity::SinglePlayerQuery::tag", "sand::prelude::PlayerQueries::tag", "sand::prelude::PlayerQuery::tag", "sand::prelude::SinglePlayerQuery::tag"], kind: Method, summary: "Configures or performs tag for the typed query entity API." }
register_entity_api! { path: "sand::entity::PlayerQuery::within_blocks", aliases: ["sand::entity::PlayerQueries::within_blocks", "sand::entity::SinglePlayerQuery::within_blocks", "sand::prelude::PlayerQueries::within_blocks", "sand::prelude::PlayerQuery::within_blocks", "sand::prelude::SinglePlayerQuery::within_blocks"], kind: Method, summary: "Configures or performs within blocks for the typed query entity API." }
register_entity_api! { path: "sand::entity::PlayerQuery::without_tag", aliases: ["sand::entity::PlayerQueries::without_tag", "sand::entity::SinglePlayerQuery::without_tag", "sand::prelude::PlayerQueries::without_tag", "sand::prelude::PlayerQuery::without_tag", "sand::prelude::SinglePlayerQuery::without_tag"], kind: Method, summary: "Configures or performs without tag for the typed query entity API." }
register_entity_api! { path: "sand::entity::SingleEntityQuery", aliases: ["sand::prelude::SingleEntityQuery"], kind: TypeAlias, summary: "Represents single entity query in Sand's cardinality-aware typed entity query model." }
register_entity_api! { path: "sand::entity::SinglePlayerQuery", aliases: ["sand::prelude::SinglePlayerQuery"], kind: TypeAlias, summary: "Represents single player query in Sand's cardinality-aware typed entity query model." }
register_entity_api! { path: "sand::entity::Relation", aliases: ["sand::prelude::Relation"], kind: Enum, summary: "Represents relation for typed traversal of Minecraft entity relationships." }
register_entity_api! { path: "sand::entity::Relation::Attacker", aliases: ["sand::prelude::Relation::Attacker"], kind: Variant, summary: "Selects the attacker semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::Relation::Controller", aliases: ["sand::prelude::Relation::Controller"], kind: Variant, summary: "Selects the controller semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::Relation::Leasher", aliases: ["sand::prelude::Relation::Leasher"], kind: Variant, summary: "Selects the leasher semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::Relation::Origin", aliases: ["sand::prelude::Relation::Origin"], kind: Variant, summary: "Selects the origin semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::Relation::Owner", aliases: ["sand::prelude::Relation::Owner"], kind: Variant, summary: "Selects the owner semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::Relation::Passengers", aliases: ["sand::prelude::Relation::Passengers"], kind: Variant, summary: "Selects the passengers semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::Relation::Target", aliases: ["sand::prelude::Relation::Target"], kind: Variant, summary: "Selects the target semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::Relation::Vehicle", aliases: ["sand::prelude::Relation::Vehicle"], kind: Variant, summary: "Selects the vehicle semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::Relation::check_supported", aliases: ["sand::prelude::Relation::check_supported"], kind: Method, summary: "Checks check supported for the typed relation entity API." }
register_entity_api! { path: "sand::entity::Relation::keyword", aliases: ["sand::prelude::Relation::keyword"], kind: Method, summary: "Configures or performs keyword for the typed relation entity API." }
register_entity_api! { path: "sand::entity::RelationQuery", aliases: ["sand::prelude::RelationQuery"], kind: Struct, summary: "Represents relation query for typed traversal of Minecraft entity relationships." }
register_entity_api! { path: "sand::entity::RelationQuery::each", aliases: ["sand::prelude::RelationQuery::each"], kind: Method, summary: "Configures or performs each for the typed relation entity API." }
register_entity_api! { path: "sand::entity::RelationQuery::if_player", aliases: ["sand::prelude::RelationQuery::if_player"], kind: Method, summary: "Configures or performs if player for the typed relation entity API." }
register_entity_api! { path: "sand::entity::RelationQuery::if_present", aliases: ["sand::prelude::RelationQuery::if_present"], kind: Method, summary: "Configures or performs if present for the typed relation entity API." }
register_entity_api! { path: "sand::entity::RelationQuery::relation", aliases: ["sand::prelude::RelationQuery::relation"], kind: Method, summary: "Configures or performs relation for the typed relation entity API." }
register_entity_api! { path: "sand::entity::EntityCooldown", aliases: ["sand::prelude::EntityCooldown"], kind: Struct, summary: "Represents entity cooldown in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityCooldown::new", aliases: ["sand::prelude::EntityCooldown::new"], kind: Method, summary: "Constructs new for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityCooldown::ready", aliases: ["sand::prelude::EntityCooldown::ready"], kind: Method, summary: "Configures or performs ready for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityCooldownAccessor", aliases: [], kind: Struct, summary: "Represents entity cooldown accessor in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityCooldownAccessor::ready", aliases: [], kind: Method, summary: "Configures or performs ready for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityCooldownAccessor::start", aliases: [], kind: Method, summary: "Configures or performs start for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityEnum", aliases: ["sand::prelude::EntityEnum"], kind: Struct, summary: "Represents entity enum in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityEnum::is", aliases: ["sand::prelude::EntityEnum::is"], kind: Method, summary: "Configures or performs is for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityEnum::new", aliases: ["sand::prelude::EntityEnum::new"], kind: Method, summary: "Constructs new for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityEnumAccessor", aliases: [], kind: Struct, summary: "Represents entity enum accessor in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityEnumAccessor::is", aliases: [], kind: Method, summary: "Configures or performs is for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityEnumAccessor::set", aliases: [], kind: Method, summary: "Configures or performs set for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityEnumValue", aliases: ["sand::prelude::EntityEnumValue"], kind: Trait, summary: "Represents entity enum value in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityEnumValue::ENCODINGS", aliases: ["sand::prelude::EntityEnumValue::ENCODINGS"], kind: AssociatedConst, summary: "Provides the canonical encodings metadata for this typed entity abstraction." }
register_entity_api! { path: "sand::entity::EntityEnumValue::encode", aliases: ["sand::prelude::EntityEnumValue::encode"], kind: TraitMethod, summary: "Defines how an entity type supplies encode to Sand's typed model." }
register_entity_api! { path: "sand::entity::EntityFlag", aliases: ["sand::prelude::EntityFlag"], kind: Struct, summary: "Represents entity flag in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityFlag::is_disabled", aliases: ["sand::prelude::EntityFlag::is_disabled"], kind: Method, summary: "Checks is disabled for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityFlag::is_enabled", aliases: ["sand::prelude::EntityFlag::is_enabled"], kind: Method, summary: "Checks is enabled for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityFlag::new", aliases: ["sand::prelude::EntityFlag::new"], kind: Method, summary: "Constructs new for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityFlagAccessor", aliases: [], kind: Struct, summary: "Represents entity flag accessor in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityFlagAccessor::disable", aliases: [], kind: Method, summary: "Configures or performs disable for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityFlagAccessor::enable", aliases: [], kind: Method, summary: "Configures or performs enable for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityFlagAccessor::is_disabled", aliases: [], kind: Method, summary: "Checks is disabled for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityFlagAccessor::is_enabled", aliases: [], kind: Method, summary: "Checks is enabled for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityScore", aliases: ["sand::prelude::EntityScore"], kind: Struct, summary: "Represents entity score in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityScore::__new", aliases: ["sand::prelude::EntityScore::__new"], kind: Method, summary: "Constructs new for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityScore::matches", aliases: ["sand::prelude::EntityScore::matches"], kind: Method, summary: "Configures or performs matches for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityScore::new", aliases: ["sand::prelude::EntityScore::new"], kind: Method, summary: "Constructs new for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityScoreAccessor", aliases: [], kind: Struct, summary: "Represents entity score accessor in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityScoreAccessor::add", aliases: [], kind: Method, summary: "Configures or performs add for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityScoreAccessor::get", aliases: [], kind: Method, summary: "Configures or performs get for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityScoreAccessor::matches", aliases: [], kind: Method, summary: "Configures or performs matches for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityScoreAccessor::set", aliases: [], kind: Method, summary: "Configures or performs set for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityScoreAccessor::subtract", aliases: [], kind: Method, summary: "Configures or performs subtract for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityState", aliases: ["sand::prelude::EntityState"], kind: Trait, summary: "Represents entity state in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityState::schema", aliases: ["sand::prelude::EntityState::schema"], kind: TraitMethod, summary: "Defines how an entity type supplies schema to Sand's typed model." }
register_entity_api! { path: "sand::entity::EntityStateField", aliases: ["sand::prelude::EntityStateField"], kind: Trait, summary: "Represents entity state field in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityStateField::Accessor", aliases: ["sand::prelude::EntityStateField::Accessor"], kind: AssociatedType, summary: "Names the accessor type used by this typed entity abstraction." }
register_entity_api! { path: "sand::entity::EntityStateField::bind", aliases: ["sand::prelude::EntityStateField::bind"], kind: TraitMethod, summary: "Defines how an entity type supplies bind to Sand's typed model." }
register_entity_api! { path: "sand::entity::EntityStateField::bind_to", aliases: ["sand::prelude::EntityStateField::bind_to"], kind: TraitMethod, summary: "Defines how an entity type supplies bind to to Sand's typed model." }
register_entity_api! { path: "sand::entity::EntityStateField::descriptor", aliases: ["sand::prelude::EntityStateField::descriptor"], kind: TraitMethod, summary: "Defines how an entity type supplies descriptor to Sand's typed model." }
register_entity_api! { path: "sand::entity::EntityStateField::dirty_objective", aliases: ["sand::prelude::EntityStateField::dirty_objective"], kind: TraitMethod, summary: "Defines how an entity type supplies dirty objective to Sand's typed model." }
register_entity_api! { path: "sand::entity::EntityStateField::objective", aliases: ["sand::prelude::EntityStateField::objective"], kind: TraitMethod, summary: "Defines how an entity type supplies objective to Sand's typed model." }
register_entity_api! { path: "sand::entity::EntityTimer", aliases: ["sand::prelude::EntityTimer"], kind: Struct, summary: "Represents entity timer in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityTimer::elapsed", aliases: ["sand::prelude::EntityTimer::elapsed"], kind: Method, summary: "Configures or performs elapsed for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityTimer::new", aliases: ["sand::prelude::EntityTimer::new"], kind: Method, summary: "Constructs new for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityTimerAccessor", aliases: [], kind: Struct, summary: "Represents entity timer accessor in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EntityTimerAccessor::elapsed", aliases: [], kind: Method, summary: "Configures or performs elapsed for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityTimerAccessor::start", aliases: [], kind: Method, summary: "Configures or performs start for the typed state entity API." }
register_entity_api! { path: "sand::entity::EntityTimerAccessor::tick", aliases: [], kind: Method, summary: "Configures or performs tick for the typed state entity API." }
register_entity_api! { path: "sand::entity::EnumEncoding", aliases: ["sand::prelude::EnumEncoding"], kind: Struct, summary: "Represents enum encoding in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::EnumEncoding::name", aliases: ["sand::prelude::EnumEncoding::name"], kind: Field, summary: "Carries the name value required by this typed entity case." }
register_entity_api! { path: "sand::entity::EnumEncoding::score", aliases: ["sand::prelude::EnumEncoding::score"], kind: Field, summary: "Carries the score value required by this typed entity case." }
register_entity_api! { path: "sand::entity::StateFieldDescriptor", aliases: ["sand::prelude::StateFieldDescriptor"], kind: Struct, summary: "Represents state field descriptor in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::StateFieldDescriptor::bounds", aliases: ["sand::prelude::StateFieldDescriptor::bounds"], kind: Field, summary: "Carries the bounds value required by this typed entity case." }
register_entity_api! { path: "sand::entity::StateFieldDescriptor::default", aliases: ["sand::prelude::StateFieldDescriptor::default"], kind: Field, summary: "Carries the default value required by this typed entity case." }
register_entity_api! { path: "sand::entity::StateFieldDescriptor::kind", aliases: ["sand::prelude::StateFieldDescriptor::kind"], kind: Field, summary: "Carries the kind value required by this typed entity case." }
register_entity_api! { path: "sand::entity::StateFieldDescriptor::name", aliases: ["sand::prelude::StateFieldDescriptor::name"], kind: Field, summary: "Carries the name value required by this typed entity case." }
register_entity_api! { path: "sand::entity::StateFieldDescriptor::new", aliases: ["sand::prelude::StateFieldDescriptor::new"], kind: Method, summary: "Constructs new for the typed state entity API." }
register_entity_api! { path: "sand::entity::StateFieldKind", aliases: ["sand::prelude::StateFieldKind"], kind: Enum, summary: "Represents state field kind in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::StateFieldKind::Cooldown", aliases: ["sand::prelude::StateFieldKind::Cooldown"], kind: Variant, summary: "Selects the cooldown semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::StateFieldKind::Dirty", aliases: ["sand::prelude::StateFieldKind::Dirty"], kind: Variant, summary: "Selects the dirty semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::StateFieldKind::Enum", aliases: ["sand::prelude::StateFieldKind::Enum"], kind: Variant, summary: "Selects the enum semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::StateFieldKind::Enum::0", aliases: ["sand::prelude::StateFieldKind::Enum::0"], kind: Field, summary: "Carries the 0 value required by this typed entity case." }
register_entity_api! { path: "sand::entity::StateFieldKind::Flag", aliases: ["sand::prelude::StateFieldKind::Flag"], kind: Variant, summary: "Selects the flag semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::StateFieldKind::Score", aliases: ["sand::prelude::StateFieldKind::Score"], kind: Variant, summary: "Selects the score semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::StateFieldKind::Timer", aliases: ["sand::prelude::StateFieldKind::Timer"], kind: Variant, summary: "Selects the timer semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::StateFieldKind::Version", aliases: ["sand::prelude::StateFieldKind::Version"], kind: Variant, summary: "Selects the version semantic case for this typed entity definition." }
register_entity_api! { path: "sand::entity::StatePredicate", aliases: ["sand::prelude::StatePredicate"], kind: Struct, summary: "Represents state predicate in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::StatePredicate::condition", aliases: ["sand::prelude::StatePredicate::condition"], kind: Method, summary: "Configures or performs condition for the typed state entity API." }
register_entity_api! { path: "sand::entity::StatePredicate::objective", aliases: ["sand::prelude::StatePredicate::objective"], kind: Method, summary: "Returns objective for the typed state entity API." }
register_entity_api! { path: "sand::entity::StatePredicate::selector_range", aliases: ["sand::prelude::StatePredicate::selector_range"], kind: Method, summary: "Returns selector range for the typed state entity API." }
register_entity_api! { path: "sand::entity::StateSchema", aliases: ["sand::prelude::StateSchema"], kind: Struct, summary: "Represents state schema in scoreboard-backed typed entity state." }
register_entity_api! { path: "sand::entity::StateSchema::fields", aliases: ["sand::prelude::StateSchema::fields"], kind: Field, summary: "Carries the fields value required by this typed entity case." }
register_entity_api! { path: "sand::entity::StateSchema::id", aliases: ["sand::prelude::StateSchema::id"], kind: Method, summary: "Returns id for the typed state entity API." }
register_entity_api! { path: "sand::entity::StateSchema::name", aliases: ["sand::prelude::StateSchema::name"], kind: Field, summary: "Carries the name value required by this typed entity case." }
register_entity_api! { path: "sand::entity::StateSchema::namespace", aliases: ["sand::prelude::StateSchema::namespace"], kind: Field, summary: "Carries the namespace value required by this typed entity case." }
register_entity_api! { path: "sand::entity::StateSchema::validate", aliases: ["sand::prelude::StateSchema::validate"], kind: Method, summary: "Configures or performs validate for the typed state entity API." }
register_entity_api! { path: "sand::entity::StateSchema::version", aliases: ["sand::prelude::StateSchema::version"], kind: Field, summary: "Carries the version value required by this typed entity case." }
// END ENTITY API CONTRACTS

// BEGIN STATE API CONTRACTS
register_state_api! { path: "sand::state::Ticks", aliases: ["sand::prelude::Ticks"], kind: Struct, summary: "Represents ticks in Sand's typed time state model." }
register_state_api! { path: "sand::state::Ticks::as_seconds", aliases: ["sand::prelude::Ticks::as_seconds"], kind: Method, summary: "Configures or performs as seconds for typed time state." }
register_state_api! { path: "sand::state::Ticks::get", aliases: ["sand::prelude::Ticks::get"], kind: Method, summary: "Configures or performs get for typed time state." }
register_state_api! { path: "sand::state::Ticks::minutes", aliases: ["sand::prelude::Ticks::minutes"], kind: Method, summary: "Configures or performs minutes for typed time state." }
register_state_api! { path: "sand::state::Ticks::new", aliases: ["sand::prelude::Ticks::new"], kind: Method, summary: "Configures or performs new for typed time state." }
register_state_api! { path: "sand::state::Ticks::seconds", aliases: ["sand::prelude::Ticks::seconds"], kind: Method, summary: "Configures or performs seconds for typed time state." }
register_state_api! { path: "sand::state::Cooldown", aliases: ["sand::prelude::Cooldown"], kind: Struct, summary: "Represents cooldown in Sand's typed cooldown state model." }
register_state_api! { path: "sand::state::Cooldown::active", aliases: ["sand::prelude::Cooldown::active"], kind: Method, summary: "Builds a typed condition for active state." }
register_state_api! { path: "sand::state::Cooldown::define", aliases: ["sand::prelude::Cooldown::define"], kind: Method, summary: "Configures or performs define for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::duration", aliases: ["sand::prelude::Cooldown::duration"], kind: Method, summary: "Configures or performs duration for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::expired", aliases: ["sand::prelude::Cooldown::expired"], kind: Method, summary: "Builds a typed condition for expired state." }
register_state_api! { path: "sand::state::Cooldown::guard", aliases: ["sand::prelude::Cooldown::guard"], kind: Method, summary: "Configures or performs guard for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::guard_active", aliases: ["sand::prelude::Cooldown::guard_active"], kind: Method, summary: "Configures or performs guard active for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::guard_ready", aliases: ["sand::prelude::Cooldown::guard_ready"], kind: Method, summary: "Configures or performs guard ready for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::new", aliases: ["sand::prelude::Cooldown::new"], kind: Method, summary: "Configures or performs new for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::objective_name", aliases: ["sand::prelude::Cooldown::objective_name"], kind: Method, summary: "Configures or performs objective name for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::ready", aliases: ["sand::prelude::Cooldown::ready"], kind: Method, summary: "Builds a typed condition for ready state." }
register_state_api! { path: "sand::state::Cooldown::reset_for", aliases: ["sand::prelude::Cooldown::reset_for"], kind: Method, summary: "Configures or performs reset for for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::start", aliases: ["sand::prelude::Cooldown::start"], kind: Method, summary: "Configures or performs start for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::start_for", aliases: ["sand::prelude::Cooldown::start_for"], kind: Method, summary: "Configures or performs start for for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::stop", aliases: ["sand::prelude::Cooldown::stop"], kind: Method, summary: "Configures or performs stop for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::tick", aliases: ["sand::prelude::Cooldown::tick"], kind: Method, summary: "Configures or performs tick for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::tick_all_players", aliases: ["sand::prelude::Cooldown::tick_all_players"], kind: Method, summary: "Configures or performs tick all players for typed cooldown state." }
register_state_api! { path: "sand::state::Cooldown::try_active", aliases: ["sand::prelude::Cooldown::try_active"], kind: Method, summary: "Validates and performs active with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Cooldown::try_expired", aliases: ["sand::prelude::Cooldown::try_expired"], kind: Method, summary: "Validates and performs expired with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Cooldown::try_guard", aliases: ["sand::prelude::Cooldown::try_guard"], kind: Method, summary: "Validates and performs guard with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Cooldown::try_guard_active", aliases: ["sand::prelude::Cooldown::try_guard_active"], kind: Method, summary: "Validates and performs guard active with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Cooldown::try_guard_ready", aliases: ["sand::prelude::Cooldown::try_guard_ready"], kind: Method, summary: "Validates and performs guard ready with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Cooldown::try_ready", aliases: ["sand::prelude::Cooldown::try_ready"], kind: Method, summary: "Validates and performs ready with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Cooldown::try_reset_for", aliases: ["sand::prelude::Cooldown::try_reset_for"], kind: Method, summary: "Validates and performs reset for with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Cooldown::try_start", aliases: ["sand::prelude::Cooldown::try_start"], kind: Method, summary: "Validates and performs start with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Cooldown::try_start_for", aliases: ["sand::prelude::Cooldown::try_start_for"], kind: Method, summary: "Validates and performs start for with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Cooldown::try_stop", aliases: ["sand::prelude::Cooldown::try_stop"], kind: Method, summary: "Validates and performs stop with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Cooldown::try_tick", aliases: ["sand::prelude::Cooldown::try_tick"], kind: Method, summary: "Validates and performs tick with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Flag", aliases: ["sand::prelude::Flag"], kind: Struct, summary: "Represents flag in Sand's typed flag state model." }
register_state_api! { path: "sand::state::Flag::clear", aliases: ["sand::prelude::Flag::clear"], kind: Method, summary: "Configures or performs clear for typed flag state." }
register_state_api! { path: "sand::state::Flag::define", aliases: ["sand::prelude::Flag::define"], kind: Method, summary: "Configures or performs define for typed flag state." }
register_state_api! { path: "sand::state::Flag::disable", aliases: ["sand::prelude::Flag::disable"], kind: Method, summary: "Configures or performs disable for typed flag state." }
register_state_api! { path: "sand::state::Flag::enable", aliases: ["sand::prelude::Flag::enable"], kind: Method, summary: "Configures or performs enable for typed flag state." }
register_state_api! { path: "sand::state::Flag::init_false", aliases: ["sand::prelude::Flag::init_false"], kind: Method, summary: "Configures or performs init false for typed flag state." }
register_state_api! { path: "sand::state::Flag::init_true", aliases: ["sand::prelude::Flag::init_true"], kind: Method, summary: "Configures or performs init true for typed flag state." }
register_state_api! { path: "sand::state::Flag::new", aliases: ["sand::prelude::Flag::new"], kind: Method, summary: "Configures or performs new for typed flag state." }
register_state_api! { path: "sand::state::Flag::objective_name", aliases: ["sand::prelude::Flag::objective_name"], kind: Method, summary: "Configures or performs objective name for typed flag state." }
register_state_api! { path: "sand::state::Flag::of", aliases: ["sand::prelude::Flag::of"], kind: Method, summary: "Configures or performs of for typed flag state." }
register_state_api! { path: "sand::state::Flag::set", aliases: ["sand::prelude::Flag::set"], kind: Method, summary: "Configures or performs set for typed flag state." }
register_state_api! { path: "sand::state::Flag::toggle", aliases: ["sand::prelude::Flag::toggle"], kind: Method, summary: "Configures or performs toggle for typed flag state." }
register_state_api! { path: "sand::state::Flag::try_clear", aliases: ["sand::prelude::Flag::try_clear"], kind: Method, summary: "Validates and performs clear with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Flag::try_disable", aliases: ["sand::prelude::Flag::try_disable"], kind: Method, summary: "Validates and performs disable with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Flag::try_enable", aliases: ["sand::prelude::Flag::try_enable"], kind: Method, summary: "Validates and performs enable with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Flag::try_init_false", aliases: ["sand::prelude::Flag::try_init_false"], kind: Method, summary: "Validates and performs init false with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Flag::try_init_true", aliases: ["sand::prelude::Flag::try_init_true"], kind: Method, summary: "Validates and performs init true with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Flag::try_of", aliases: ["sand::prelude::Flag::try_of"], kind: Method, summary: "Validates and performs of with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Flag::try_set", aliases: ["sand::prelude::Flag::try_set"], kind: Method, summary: "Validates and performs set with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Flag::try_toggle", aliases: ["sand::prelude::Flag::try_toggle"], kind: Method, summary: "Validates and performs toggle with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Flag::try_unless_true", aliases: ["sand::prelude::Flag::try_unless_true"], kind: Method, summary: "Validates and performs unless true with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Flag::try_when_false", aliases: ["sand::prelude::Flag::try_when_false"], kind: Method, summary: "Validates and performs when false with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Flag::try_when_true", aliases: ["sand::prelude::Flag::try_when_true"], kind: Method, summary: "Validates and performs when true with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Flag::unless_true", aliases: ["sand::prelude::Flag::unless_true"], kind: Method, summary: "Builds a typed condition for unless true state." }
register_state_api! { path: "sand::state::Flag::when_false", aliases: ["sand::prelude::Flag::when_false"], kind: Method, summary: "Builds a typed condition for when false state." }
register_state_api! { path: "sand::state::Flag::when_true", aliases: ["sand::prelude::Flag::when_true"], kind: Method, summary: "Builds a typed condition for when true state." }
register_state_api! { path: "sand::state::FlagRef", aliases: ["sand::prelude::FlagRef"], kind: Struct, summary: "Represents flag ref in Sand's typed flag state model." }
register_state_api! { path: "sand::state::FlagRef::is_false", aliases: ["sand::prelude::FlagRef::is_false"], kind: Method, summary: "Builds a typed condition for is false state." }
register_state_api! { path: "sand::state::FlagRef::is_not_true", aliases: ["sand::prelude::FlagRef::is_not_true"], kind: Method, summary: "Builds a typed condition for is not true state." }
register_state_api! { path: "sand::state::FlagRef::is_set", aliases: ["sand::prelude::FlagRef::is_set"], kind: Method, summary: "Builds a typed condition for is set state." }
register_state_api! { path: "sand::state::FlagRef::is_true", aliases: ["sand::prelude::FlagRef::is_true"], kind: Method, summary: "Builds a typed condition for is true state." }
register_state_api! { path: "sand::state::FlagRef::is_unset", aliases: ["sand::prelude::FlagRef::is_unset"], kind: Method, summary: "Builds a typed condition for is unset state." }
register_state_api! { path: "sand::state::FlowTransitionBuilder", aliases: [], kind: Struct, summary: "Represents flow transition builder in Sand's typed flow state model." }
register_state_api! { path: "sand::state::FlowTransitionBuilder::done", aliases: [], kind: Method, summary: "Configures or performs done for typed flow state." }
register_state_api! { path: "sand::state::FlowTransitionBuilder::priority", aliases: [], kind: Method, summary: "Configures or performs priority for typed flow state." }
register_state_api! { path: "sand::state::FlowTransitionBuilder::when", aliases: [], kind: Method, summary: "Configures or performs when for typed flow state." }
register_state_api! { path: "sand::state::IntoStateCommands", aliases: ["sand::prelude::IntoStateCommands"], kind: Trait, summary: "Represents into state commands in Sand's typed flow state model." }
register_state_api! { path: "sand::state::IntoStateCommands::into_state_commands", aliases: ["sand::prelude::IntoStateCommands::into_state_commands"], kind: TraitMethod, summary: "Defines how a typed state supplies into state commands." }
register_state_api! { path: "sand::state::StateFlow", aliases: ["sand::prelude::StateFlow"], kind: Struct, summary: "Represents state flow in Sand's typed flow state model." }
register_state_api! { path: "sand::state::StateFlow::for_subjects", aliases: ["sand::prelude::StateFlow::for_subjects"], kind: Method, summary: "Configures or performs for subjects for typed flow state." }
register_state_api! { path: "sand::state::StateFlow::named", aliases: ["sand::prelude::StateFlow::named"], kind: Method, summary: "Configures or performs named for typed flow state." }
register_state_api! { path: "sand::state::StateFlow::on_enter", aliases: ["sand::prelude::StateFlow::on_enter"], kind: Method, summary: "Configures or performs on enter for typed flow state." }
register_state_api! { path: "sand::state::StateFlow::on_exit", aliases: ["sand::prelude::StateFlow::on_exit"], kind: Method, summary: "Configures or performs on exit for typed flow state." }
register_state_api! { path: "sand::state::StateFlow::on_tick", aliases: ["sand::prelude::StateFlow::on_tick"], kind: Method, summary: "Configures or performs on tick for typed flow state." }
register_state_api! { path: "sand::state::StateFlow::on_tick_every", aliases: ["sand::prelude::StateFlow::on_tick_every"], kind: Method, summary: "Configures or performs on tick every for typed flow state." }
register_state_api! { path: "sand::state::StateFlow::players", aliases: ["sand::prelude::StateFlow::players"], kind: Method, summary: "Configures or performs players for typed flow state." }
register_state_api! { path: "sand::state::StateFlow::register", aliases: ["sand::prelude::StateFlow::register"], kind: Method, summary: "Configures or performs register for typed flow state." }
register_state_api! { path: "sand::state::StateFlow::transition", aliases: ["sand::prelude::StateFlow::transition"], kind: Method, summary: "Configures or performs transition for typed flow state." }
register_state_api! { path: "sand::state::StateTransitionBuilder", aliases: [], kind: Struct, summary: "Represents state transition builder in Sand's typed flow state model." }
register_state_api! { path: "sand::state::StateTransitionBuilder::from", aliases: [], kind: Method, summary: "Configures or performs from for typed flow state." }
register_state_api! { path: "sand::state::StateTransitionBuilder::to", aliases: [], kind: Method, summary: "Configures or performs to for typed flow state." }
register_state_api! { path: "sand::state::StateTransitionBuilder::when", aliases: [], kind: Method, summary: "Configures or performs when for typed flow state." }
register_state_api! { path: "sand::state::ScoreConst", aliases: [], kind: Struct, summary: "Represents score const in Sand's typed score state model." }
register_state_api! { path: "sand::state::ScoreConst::new", aliases: [], kind: Method, summary: "Configures or performs new for typed score state." }
register_state_api! { path: "sand::state::ScoreConst::ref_", aliases: [], kind: Method, summary: "Configures or performs ref  for typed score state." }
register_state_api! { path: "sand::state::ScoreConstants", aliases: [], kind: Struct, summary: "Represents score constants in Sand's typed score state model." }
register_state_api! { path: "sand::state::ScoreConstants::i32", aliases: [], kind: Method, summary: "Configures or performs i32 for typed score state." }
register_state_api! { path: "sand::state::ScoreConstants::new", aliases: [], kind: Method, summary: "Configures or performs new for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr", aliases: [], kind: Struct, summary: "Represents score expr in Sand's typed score state model." }
register_state_api! { path: "sand::state::ScoreExpr::div", aliases: [], kind: Method, summary: "Configures or performs div for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::eq", aliases: [], kind: Method, summary: "Configures or performs eq for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::eq_score", aliases: [], kind: Method, summary: "Configures or performs eq score for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::gt", aliases: [], kind: Method, summary: "Configures or performs gt for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::gt_score", aliases: [], kind: Method, summary: "Configures or performs gt score for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::gte", aliases: [], kind: Method, summary: "Configures or performs gte for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::gte_score", aliases: [], kind: Method, summary: "Configures or performs gte score for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::lt", aliases: [], kind: Method, summary: "Configures or performs lt for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::lt_score", aliases: [], kind: Method, summary: "Configures or performs lt score for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::lte", aliases: [], kind: Method, summary: "Configures or performs lte for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::lte_score", aliases: [], kind: Method, summary: "Configures or performs lte score for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::matches", aliases: [], kind: Method, summary: "Configures or performs matches for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::max", aliases: [], kind: Method, summary: "Configures or performs max for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::min", aliases: [], kind: Method, summary: "Configures or performs min for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::minus", aliases: [], kind: Method, summary: "Configures or performs minus for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::modulo", aliases: [], kind: Method, summary: "Configures or performs modulo for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::mul", aliases: [], kind: Method, summary: "Configures or performs mul for typed score state." }
register_state_api! { path: "sand::state::ScoreExpr::plus", aliases: [], kind: Method, summary: "Configures or performs plus for typed score state." }
register_state_api! { path: "sand::state::ScoreOperand", aliases: [], kind: Struct, summary: "Represents score operand in Sand's typed score state model." }
register_state_api! { path: "sand::state::ScoreOperation", aliases: [], kind: Enum, summary: "Represents score operation in Sand's typed score state model." }
register_state_api! { path: "sand::state::ScoreOperation::Add", aliases: [], kind: Variant, summary: "Selects the add operation or policy for typed gameplay state." }
register_state_api! { path: "sand::state::ScoreOperation::Assign", aliases: [], kind: Variant, summary: "Selects the assign operation or policy for typed gameplay state." }
register_state_api! { path: "sand::state::ScoreOperation::Div", aliases: [], kind: Variant, summary: "Selects the div operation or policy for typed gameplay state." }
register_state_api! { path: "sand::state::ScoreOperation::Max", aliases: [], kind: Variant, summary: "Selects the max operation or policy for typed gameplay state." }
register_state_api! { path: "sand::state::ScoreOperation::Min", aliases: [], kind: Variant, summary: "Selects the min operation or policy for typed gameplay state." }
register_state_api! { path: "sand::state::ScoreOperation::Mod", aliases: [], kind: Variant, summary: "Selects the mod operation or policy for typed gameplay state." }
register_state_api! { path: "sand::state::ScoreOperation::Mul", aliases: [], kind: Variant, summary: "Selects the mul operation or policy for typed gameplay state." }
register_state_api! { path: "sand::state::ScoreOperation::Sub", aliases: [], kind: Variant, summary: "Selects the sub operation or policy for typed gameplay state." }
register_state_api! { path: "sand::state::ScoreOperation::Swap", aliases: [], kind: Variant, summary: "Selects the swap operation or policy for typed gameplay state." }
register_state_api! { path: "sand::state::ScoreOperation::as_str", aliases: [], kind: Method, summary: "Configures or performs as str for typed score state." }
register_state_api! { path: "sand::state::ScoreRef", aliases: ["sand::prelude::ScoreRef"], kind: Struct, summary: "Represents score ref in Sand's typed score state model." }
register_state_api! { path: "sand::state::ScoreRef::add_score", aliases: ["sand::prelude::ScoreRef::add_score"], kind: Method, summary: "Configures or performs add score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::assign", aliases: ["sand::prelude::ScoreRef::assign"], kind: Method, summary: "Configures or performs assign for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::between", aliases: ["sand::prelude::ScoreRef::between"], kind: Method, summary: "Configures or performs between for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::clamp_score", aliases: ["sand::prelude::ScoreRef::clamp_score"], kind: Method, summary: "Configures or performs clamp score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::div_score", aliases: ["sand::prelude::ScoreRef::div_score"], kind: Method, summary: "Configures or performs div score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::eq", aliases: ["sand::prelude::ScoreRef::eq"], kind: Method, summary: "Configures or performs eq for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::eq_score", aliases: ["sand::prelude::ScoreRef::eq_score"], kind: Method, summary: "Configures or performs eq score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::expr", aliases: ["sand::prelude::ScoreRef::expr"], kind: Method, summary: "Configures or performs expr for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::gt", aliases: ["sand::prelude::ScoreRef::gt"], kind: Method, summary: "Configures or performs gt for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::gt_score", aliases: ["sand::prelude::ScoreRef::gt_score"], kind: Method, summary: "Configures or performs gt score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::gte", aliases: ["sand::prelude::ScoreRef::gte"], kind: Method, summary: "Configures or performs gte for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::gte_score", aliases: ["sand::prelude::ScoreRef::gte_score"], kind: Method, summary: "Configures or performs gte score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::is_nonzero", aliases: ["sand::prelude::ScoreRef::is_nonzero"], kind: Method, summary: "Builds a typed condition for is nonzero state." }
register_state_api! { path: "sand::state::ScoreRef::is_zero", aliases: ["sand::prelude::ScoreRef::is_zero"], kind: Method, summary: "Builds a typed condition for is zero state." }
register_state_api! { path: "sand::state::ScoreRef::lt", aliases: ["sand::prelude::ScoreRef::lt"], kind: Method, summary: "Configures or performs lt for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::lt_score", aliases: ["sand::prelude::ScoreRef::lt_score"], kind: Method, summary: "Configures or performs lt score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::lte", aliases: ["sand::prelude::ScoreRef::lte"], kind: Method, summary: "Configures or performs lte for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::lte_score", aliases: ["sand::prelude::ScoreRef::lte_score"], kind: Method, summary: "Configures or performs lte score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::matches", aliases: ["sand::prelude::ScoreRef::matches"], kind: Method, summary: "Configures or performs matches for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::max_score", aliases: ["sand::prelude::ScoreRef::max_score"], kind: Method, summary: "Configures or performs max score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::min_score", aliases: ["sand::prelude::ScoreRef::min_score"], kind: Method, summary: "Configures or performs min score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::mod_score", aliases: ["sand::prelude::ScoreRef::mod_score"], kind: Method, summary: "Configures or performs mod score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::mul_score", aliases: ["sand::prelude::ScoreRef::mul_score"], kind: Method, summary: "Configures or performs mul score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::ne", aliases: ["sand::prelude::ScoreRef::ne"], kind: Method, summary: "Configures or performs ne for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::ne_score", aliases: ["sand::prelude::ScoreRef::ne_score"], kind: Method, summary: "Configures or performs ne score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::negative", aliases: ["sand::prelude::ScoreRef::negative"], kind: Method, summary: "Configures or performs negative for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::operand", aliases: ["sand::prelude::ScoreRef::operand"], kind: Method, summary: "Configures or performs operand for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::outside", aliases: ["sand::prelude::ScoreRef::outside"], kind: Method, summary: "Configures or performs outside for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::positive", aliases: ["sand::prelude::ScoreRef::positive"], kind: Method, summary: "Configures or performs positive for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::safe_divide", aliases: ["sand::prelude::ScoreRef::safe_divide"], kind: Method, summary: "Configures or performs safe divide for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::saturating_add", aliases: ["sand::prelude::ScoreRef::saturating_add"], kind: Method, summary: "Configures or performs saturating add for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::saturating_sub", aliases: ["sand::prelude::ScoreRef::saturating_sub"], kind: Method, summary: "Configures or performs saturating sub for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::scale_percent", aliases: ["sand::prelude::ScoreRef::scale_percent"], kind: Method, summary: "Configures or performs scale percent for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::set_percent", aliases: ["sand::prelude::ScoreRef::set_percent"], kind: Method, summary: "Configures or performs set percent for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::set_ratio", aliases: ["sand::prelude::ScoreRef::set_ratio"], kind: Method, summary: "Configures or performs set ratio for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::store_into", aliases: ["sand::prelude::ScoreRef::store_into"], kind: Method, summary: "Configures or performs store into for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::sub_score", aliases: ["sand::prelude::ScoreRef::sub_score"], kind: Method, summary: "Configures or performs sub score for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::swap", aliases: ["sand::prelude::ScoreRef::swap"], kind: Method, summary: "Configures or performs swap for typed score state." }
register_state_api! { path: "sand::state::ScoreRef::try_between", aliases: ["sand::prelude::ScoreRef::try_between"], kind: Method, summary: "Validates and performs between with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::ScoreRef::try_gt", aliases: ["sand::prelude::ScoreRef::try_gt"], kind: Method, summary: "Validates and performs gt with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::ScoreRef::try_lt", aliases: ["sand::prelude::ScoreRef::try_lt"], kind: Method, summary: "Validates and performs lt with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::ScoreRef::try_matches", aliases: ["sand::prelude::ScoreRef::try_matches"], kind: Method, summary: "Validates and performs matches with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::ScoreVar", aliases: ["sand::prelude::ScoreVar"], kind: Struct, summary: "Represents score var in Sand's typed score state model." }
register_state_api! { path: "sand::state::ScoreVar::add", aliases: ["sand::prelude::ScoreVar::add"], kind: Method, summary: "Configures or performs add for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::clamp", aliases: ["sand::prelude::ScoreVar::clamp"], kind: Method, summary: "Configures or performs clamp for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::copy_from", aliases: ["sand::prelude::ScoreVar::copy_from"], kind: Method, summary: "Configures or performs copy from for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::copy_to", aliases: ["sand::prelude::ScoreVar::copy_to"], kind: Method, summary: "Configures or performs copy to for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::copy_within", aliases: ["sand::prelude::ScoreVar::copy_within"], kind: Method, summary: "Configures or performs copy within for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::define", aliases: ["sand::prelude::ScoreVar::define"], kind: Method, summary: "Configures or performs define for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::init", aliases: ["sand::prelude::ScoreVar::init"], kind: Method, summary: "Configures or performs init for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::is_nonzero", aliases: ["sand::prelude::ScoreVar::is_nonzero"], kind: Method, summary: "Builds a typed condition for is nonzero state." }
register_state_api! { path: "sand::state::ScoreVar::is_zero", aliases: ["sand::prelude::ScoreVar::is_zero"], kind: Method, summary: "Builds a typed condition for is zero state." }
register_state_api! { path: "sand::state::ScoreVar::max_op", aliases: ["sand::prelude::ScoreVar::max_op"], kind: Method, summary: "Configures or performs max op for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::min_op", aliases: ["sand::prelude::ScoreVar::min_op"], kind: Method, summary: "Configures or performs min op for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::negative", aliases: ["sand::prelude::ScoreVar::negative"], kind: Method, summary: "Configures or performs negative for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::new", aliases: ["sand::prelude::ScoreVar::new"], kind: Method, summary: "Configures or performs new for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::objective_name", aliases: ["sand::prelude::ScoreVar::objective_name"], kind: Method, summary: "Configures or performs objective name for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::of", aliases: ["sand::prelude::ScoreVar::of"], kind: Method, summary: "Configures or performs of for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::positive", aliases: ["sand::prelude::ScoreVar::positive"], kind: Method, summary: "Configures or performs positive for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::remove", aliases: ["sand::prelude::ScoreVar::remove"], kind: Method, summary: "Configures or performs remove for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::reset", aliases: ["sand::prelude::ScoreVar::reset"], kind: Method, summary: "Configures or performs reset for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::set", aliases: ["sand::prelude::ScoreVar::set"], kind: Method, summary: "Configures or performs set for typed score state." }
register_state_api! { path: "sand::state::ScoreVar::try_add", aliases: ["sand::prelude::ScoreVar::try_add"], kind: Method, summary: "Validates and performs add with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::ScoreVar::try_clamp", aliases: ["sand::prelude::ScoreVar::try_clamp"], kind: Method, summary: "Validates and performs clamp with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::ScoreVar::try_copy_within", aliases: ["sand::prelude::ScoreVar::try_copy_within"], kind: Method, summary: "Validates and performs copy within with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::ScoreVar::try_init", aliases: ["sand::prelude::ScoreVar::try_init"], kind: Method, summary: "Validates and performs init with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::ScoreVar::try_of", aliases: ["sand::prelude::ScoreVar::try_of"], kind: Method, summary: "Validates and performs of with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::ScoreVar::try_remove", aliases: ["sand::prelude::ScoreVar::try_remove"], kind: Method, summary: "Validates and performs remove with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::ScoreVar::try_reset", aliases: ["sand::prelude::ScoreVar::try_reset"], kind: Method, summary: "Validates and performs reset with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::ScoreVar::try_set", aliases: ["sand::prelude::ScoreVar::try_set"], kind: Method, summary: "Validates and performs set with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Timer", aliases: ["sand::prelude::Timer"], kind: Struct, summary: "Represents timer in Sand's typed timer state model." }
register_state_api! { path: "sand::state::Timer::active", aliases: ["sand::prelude::Timer::active"], kind: Method, summary: "Builds a typed condition for active state." }
register_state_api! { path: "sand::state::Timer::define", aliases: ["sand::prelude::Timer::define"], kind: Method, summary: "Configures or performs define for typed timer state." }
register_state_api! { path: "sand::state::Timer::duration", aliases: ["sand::prelude::Timer::duration"], kind: Method, summary: "Configures or performs duration for typed timer state." }
register_state_api! { path: "sand::state::Timer::expired", aliases: ["sand::prelude::Timer::expired"], kind: Method, summary: "Builds a typed condition for expired state." }
register_state_api! { path: "sand::state::Timer::guard_active", aliases: ["sand::prelude::Timer::guard_active"], kind: Method, summary: "Configures or performs guard active for typed timer state." }
register_state_api! { path: "sand::state::Timer::new", aliases: ["sand::prelude::Timer::new"], kind: Method, summary: "Configures or performs new for typed timer state." }
register_state_api! { path: "sand::state::Timer::objective_name", aliases: ["sand::prelude::Timer::objective_name"], kind: Method, summary: "Configures or performs objective name for typed timer state." }
register_state_api! { path: "sand::state::Timer::reset", aliases: ["sand::prelude::Timer::reset"], kind: Method, summary: "Configures or performs reset for typed timer state." }
register_state_api! { path: "sand::state::Timer::start", aliases: ["sand::prelude::Timer::start"], kind: Method, summary: "Configures or performs start for typed timer state." }
register_state_api! { path: "sand::state::Timer::tick", aliases: ["sand::prelude::Timer::tick"], kind: Method, summary: "Configures or performs tick for typed timer state." }
register_state_api! { path: "sand::state::Timer::tick_all_players", aliases: ["sand::prelude::Timer::tick_all_players"], kind: Method, summary: "Configures or performs tick all players for typed timer state." }
register_state_api! { path: "sand::state::Timer::try_active", aliases: ["sand::prelude::Timer::try_active"], kind: Method, summary: "Validates and performs active with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Timer::try_expired", aliases: ["sand::prelude::Timer::try_expired"], kind: Method, summary: "Validates and performs expired with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Timer::try_guard_active", aliases: ["sand::prelude::Timer::try_guard_active"], kind: Method, summary: "Validates and performs guard active with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Timer::try_reset", aliases: ["sand::prelude::Timer::try_reset"], kind: Method, summary: "Validates and performs reset with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Timer::try_start", aliases: ["sand::prelude::Timer::try_start"], kind: Method, summary: "Validates and performs start with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::Timer::try_tick", aliases: ["sand::prelude::Timer::try_tick"], kind: Method, summary: "Validates and performs tick with a typed Minecraft score holder." }
register_state_api! { path: "sand::state::GameState", aliases: ["sand::prelude::GameState"], kind: Struct, summary: "Represents game state in Sand's typed typed state state model." }
register_state_api! { path: "sand::state::GameState::default_score", aliases: ["sand::prelude::GameState::default_score"], kind: Method, summary: "Configures or performs default score for typed typed state state." }
register_state_api! { path: "sand::state::GameState::define", aliases: ["sand::prelude::GameState::define"], kind: Method, summary: "Configures or performs define for typed typed state state." }
register_state_api! { path: "sand::state::GameState::new", aliases: ["sand::prelude::GameState::new"], kind: Method, summary: "Configures or performs new for typed typed state state." }
register_state_api! { path: "sand::state::GameState::objective_name", aliases: ["sand::prelude::GameState::objective_name"], kind: Method, summary: "Configures or performs objective name for typed typed state state." }
register_state_api! { path: "sand::state::GameState::of", aliases: ["sand::prelude::GameState::of"], kind: Method, summary: "Configures or performs of for typed typed state state." }
register_state_api! { path: "sand::state::GameState::with_default_score", aliases: ["sand::prelude::GameState::with_default_score"], kind: Method, summary: "Configures or performs with default score for typed typed state state." }
register_state_api! { path: "sand::state::GameStateRef", aliases: ["sand::prelude::GameStateRef"], kind: Struct, summary: "Represents game state ref in Sand's typed typed state state model." }
register_state_api! { path: "sand::state::GameStateRef::clear", aliases: ["sand::prelude::GameStateRef::clear"], kind: Method, summary: "Configures or performs clear for typed typed state state." }
register_state_api! { path: "sand::state::GameStateRef::is", aliases: ["sand::prelude::GameStateRef::is"], kind: Method, summary: "Configures or performs is for typed typed state state." }
register_state_api! { path: "sand::state::GameStateRef::is_not", aliases: ["sand::prelude::GameStateRef::is_not"], kind: Method, summary: "Builds a typed condition for is not state." }
register_state_api! { path: "sand::state::GameStateRef::reset", aliases: ["sand::prelude::GameStateRef::reset"], kind: Method, summary: "Configures or performs reset for typed typed state state." }
register_state_api! { path: "sand::state::GameStateRef::selector", aliases: ["sand::prelude::GameStateRef::selector"], kind: Method, summary: "Configures or performs selector for typed typed state state." }
register_state_api! { path: "sand::state::GameStateRef::set", aliases: ["sand::prelude::GameStateRef::set"], kind: Method, summary: "Configures or performs set for typed typed state state." }
register_state_api! { path: "sand::state::GameStateRef::transition", aliases: ["sand::prelude::GameStateRef::transition"], kind: Method, summary: "Configures or performs transition for typed typed state state." }
register_state_api! { path: "sand::state::TypedGameState", aliases: ["sand::prelude::TypedGameState"], kind: Trait, summary: "Represents typed game state in Sand's typed typed state state model." }
register_state_api! { path: "sand::state::TypedGameState::from_score", aliases: ["sand::prelude::TypedGameState::from_score"], kind: TraitMethod, summary: "Defines how a typed state supplies from score." }
register_state_api! { path: "sand::state::TypedGameState::to_score", aliases: ["sand::prelude::TypedGameState::to_score"], kind: TraitMethod, summary: "Defines how a typed state supplies to score." }
// END STATE API CONTRACTS
register_participant_api! { path: "sand::participant", aliases: [], kind: Module, summary: "Provides typed event participant roles, observation plans, references, and snapshots." }
register_participant_api! { path: "sand::participant::BoundedItemSnapshot", aliases: ["sand::prelude::BoundedItemSnapshot"], kind: Struct, summary: "Represents bounded item snapshot in Sand event participant transport." }
register_participant_api! { path: "sand::participant::BoundedItemSnapshot::components_path", aliases: ["sand::prelude::BoundedItemSnapshot::components_path"], kind: Method, summary: "Provides components path for typed event participant handling." }
register_participant_api! { path: "sand::participant::BoundedItemSnapshot::count_path", aliases: ["sand::prelude::BoundedItemSnapshot::count_path"], kind: Method, summary: "Provides count path for typed event participant handling." }
register_participant_api! { path: "sand::participant::BoundedItemSnapshot::id_path", aliases: ["sand::prelude::BoundedItemSnapshot::id_path"], kind: Method, summary: "Provides id path for typed event participant handling." }
register_participant_api! { path: "sand::participant::BoundedItemSnapshot::is_absent", aliases: ["sand::prelude::BoundedItemSnapshot::is_absent"], kind: Method, summary: "Builds or evaluates the is absent participant state." }
register_participant_api! { path: "sand::participant::BoundedItemSnapshot::is_present", aliases: ["sand::prelude::BoundedItemSnapshot::is_present"], kind: Method, summary: "Builds or evaluates the is present participant state." }
register_participant_api! { path: "sand::participant::BoundedItemSnapshot::item_path", aliases: ["sand::prelude::BoundedItemSnapshot::item_path"], kind: Method, summary: "Provides item path for typed event participant handling." }
register_participant_api! { path: "sand::participant::BoundedItemSnapshot::reliability", aliases: ["sand::prelude::BoundedItemSnapshot::reliability"], kind: Method, summary: "Provides reliability for typed event participant handling." }
register_participant_api! { path: "sand::participant::BoundedItemSnapshot::reset_commands", aliases: ["sand::prelude::BoundedItemSnapshot::reset_commands"], kind: Method, summary: "Returns the commands that clear this captured participant state after its lifecycle ends." }
register_participant_api! { path: "sand::participant::BoundedItemSnapshot::source_kind", aliases: ["sand::prelude::BoundedItemSnapshot::source_kind"], kind: Method, summary: "Provides source kind for typed event participant handling." }
register_participant_api! { path: "sand::participant::CorrelatedEntityObservation", aliases: [], kind: Struct, summary: "Represents correlated entity observation in Sand event participant transport." }
register_participant_api! { path: "sand::participant::CorrelatedEntityObservation::cleanup_commands", aliases: [], kind: Method, summary: "Returns the commands that clear this captured participant state after its lifecycle ends." }
register_participant_api! { path: "sand::participant::CorrelatedEntityObservation::evidence", aliases: [], kind: Method, summary: "Provides evidence for typed event participant handling." }
register_participant_api! { path: "sand::participant::CorrelatedEntityObservation::is_absent", aliases: [], kind: Method, summary: "Builds or evaluates the is absent participant state." }
register_participant_api! { path: "sand::participant::CorrelatedEntityObservation::is_present", aliases: [], kind: Method, summary: "Builds or evaluates the is present participant state." }
register_participant_api! { path: "sand::participant::CorrelatedEntityObservation::participant", aliases: [], kind: Method, summary: "Provides participant for typed event participant handling." }
register_participant_api! { path: "sand::participant::CorrelatedEntityObservation::role", aliases: [], kind: Method, summary: "Provides role for typed event participant handling." }
register_participant_api! { path: "sand::participant::CorrelationEvidence", aliases: [], kind: Struct, summary: "Represents correlation evidence in Sand event participant transport." }
register_participant_api! { path: "sand::participant::CorrelationEvidence::ATTACKER_RELATION", aliases: [], kind: Variant, summary: "Selects the attacker relation participant semantic." }
register_participant_api! { path: "sand::participant::CorrelationEvidence::min_version", aliases: [], kind: Field, summary: "Carries the min version value for this participant result." }
register_participant_api! { path: "sand::participant::CorrelationEvidence::source", aliases: [], kind: Field, summary: "Carries the source value for this participant result." }
register_participant_api! { path: "sand::participant::CorrelationSource", aliases: [], kind: Enum, summary: "Classifies correlation source for typed event participant handling." }
register_participant_api! { path: "sand::participant::CorrelationSource::AttackerRelation", aliases: [], kind: Variant, summary: "Selects the attacker relation participant semantic." }
register_participant_api! { path: "sand::participant::DuplicateParticipantRole", aliases: [], kind: Enum, summary: "Classifies duplicate participant role for typed event participant handling." }
register_participant_api! { path: "sand::participant::DuplicateParticipantRole::Entity", aliases: [], kind: Variant, summary: "Selects the entity participant semantic." }
register_participant_api! { path: "sand::participant::DuplicateParticipantRole::Entity::0", aliases: [], kind: Field, summary: "Carries the 0 value for this participant result." }
register_participant_api! { path: "sand::participant::DuplicateParticipantRole::Item", aliases: [], kind: Variant, summary: "Selects the item participant semantic." }
register_participant_api! { path: "sand::participant::DuplicateParticipantRole::Item::0", aliases: [], kind: Field, summary: "Carries the 0 value for this participant result." }
register_participant_api! { path: "sand::participant::EntityParticipant", aliases: [], kind: Struct, summary: "Represents entity participant in Sand event participant transport." }
register_participant_api! { path: "sand::participant::EntityParticipant::correlated", aliases: [], kind: Method, summary: "Provides correlated for typed event participant handling." }
register_participant_api! { path: "sand::participant::EntityParticipant::execute_at", aliases: [], kind: Method, summary: "Runs a command at the correlated participant entity." }
register_participant_api! { path: "sand::participant::EntityParticipant::inferred", aliases: [], kind: Method, summary: "Provides inferred for typed event participant handling." }
register_participant_api! { path: "sand::participant::EntityParticipant::lifetime", aliases: [], kind: Method, summary: "Provides lifetime for typed event participant handling." }
register_participant_api! { path: "sand::participant::EntityParticipant::reliability", aliases: [], kind: Method, summary: "Provides reliability for typed event participant handling." }
register_participant_api! { path: "sand::participant::EntityParticipant::require", aliases: [], kind: Method, summary: "Checks that the participant evidence meets the requested reliability." }
register_participant_api! { path: "sand::participant::EntityParticipant::require_exact", aliases: [], kind: Method, summary: "Rejects participant evidence that is not exact for this handler operation." }
register_participant_api! { path: "sand::participant::EntityParticipant::role", aliases: [], kind: Method, summary: "Provides role for typed event participant handling." }
register_participant_api! { path: "sand::participant::EntityParticipant::selector", aliases: [], kind: Method, summary: "Provides selector for typed event participant handling." }
register_participant_api! { path: "sand::participant::EntityParticipant::subject", aliases: [], kind: Method, summary: "Provides subject for typed event participant handling." }
register_participant_api! { path: "sand::participant::EntityParticipantRole", aliases: ["sand::prelude::EntityParticipantRole"], kind: Enum, summary: "Classifies entity participant role for typed event participant handling." }
register_participant_api! { path: "sand::participant::EntityParticipantRole::Actor", aliases: ["sand::prelude::EntityParticipantRole::Actor"], kind: Variant, summary: "Selects the actor participant semantic." }
register_participant_api! { path: "sand::participant::EntityParticipantRole::Attacker", aliases: ["sand::prelude::EntityParticipantRole::Attacker"], kind: Variant, summary: "Selects the attacker participant semantic." }
register_participant_api! { path: "sand::participant::EntityParticipantRole::DirectAttacker", aliases: ["sand::prelude::EntityParticipantRole::DirectAttacker"], kind: Variant, summary: "Selects the direct attacker participant semantic." }
register_participant_api! { path: "sand::participant::EntityParticipantRole::InteractedEntity", aliases: ["sand::prelude::EntityParticipantRole::InteractedEntity"], kind: Variant, summary: "Selects the interacted entity participant semantic." }
register_participant_api! { path: "sand::participant::EntityParticipantRole::Killer", aliases: ["sand::prelude::EntityParticipantRole::Killer"], kind: Variant, summary: "Selects the killer participant semantic." }
register_participant_api! { path: "sand::participant::EntityParticipantRole::Projectile", aliases: ["sand::prelude::EntityParticipantRole::Projectile"], kind: Variant, summary: "Selects the projectile participant semantic." }
register_participant_api! { path: "sand::participant::EntityParticipantRole::ProjectileOwner", aliases: ["sand::prelude::EntityParticipantRole::ProjectileOwner"], kind: Variant, summary: "Selects the projectile owner participant semantic." }
register_participant_api! { path: "sand::participant::EntityParticipantRole::Subject", aliases: ["sand::prelude::EntityParticipantRole::Subject"], kind: Variant, summary: "Selects the subject participant semantic." }
register_participant_api! { path: "sand::participant::EntityParticipantRole::Target", aliases: ["sand::prelude::EntityParticipantRole::Target"], kind: Variant, summary: "Selects the target participant semantic." }
register_participant_api! { path: "sand::participant::EntityParticipantRole::Victim", aliases: ["sand::prelude::EntityParticipantRole::Victim"], kind: Variant, summary: "Selects the victim participant semantic." }
register_participant_api! { path: "sand::participant::EventParticipantPlan", aliases: ["sand::prelude::EventParticipantPlan"], kind: Struct, summary: "Represents event participant plan in Sand event participant transport." }
register_participant_api! { path: "sand::participant::EventParticipantPlan::inherit_entity", aliases: ["sand::prelude::EventParticipantPlan::inherit_entity"], kind: Method, summary: "Borrows entity from the named parent during the supported event lifecycle." }
register_participant_api! { path: "sand::participant::EventParticipantPlan::inherit_item", aliases: ["sand::prelude::EventParticipantPlan::inherit_item"], kind: Method, summary: "Borrows item from the named parent during the supported event lifecycle." }
register_participant_api! { path: "sand::participant::EventParticipantPlan::inherit_item_within", aliases: ["sand::prelude::EventParticipantPlan::inherit_item_within"], kind: Method, summary: "Borrows item within from the named parent during the supported event lifecycle." }
register_participant_api! { path: "sand::participant::EventParticipantPlan::is_empty", aliases: ["sand::prelude::EventParticipantPlan::is_empty"], kind: Method, summary: "Builds or evaluates the is empty participant state." }
register_participant_api! { path: "sand::participant::EventParticipantPlan::new", aliases: ["sand::prelude::EventParticipantPlan::new"], kind: Method, summary: "Provides new for typed event participant handling." }
register_participant_api! { path: "sand::participant::EventParticipantPlan::none", aliases: ["sand::prelude::EventParticipantPlan::none"], kind: Method, summary: "Provides none for typed event participant handling." }
register_participant_api! { path: "sand::participant::EventParticipantPlan::observe_correlated_attacker", aliases: ["sand::prelude::EventParticipantPlan::observe_correlated_attacker"], kind: Method, summary: "Declares correlated attacker capture for the event participant plan." }
register_participant_api! { path: "sand::participant::EventParticipantPlan::observe_correlated_killer", aliases: ["sand::prelude::EventParticipantPlan::observe_correlated_killer"], kind: Method, summary: "Declares correlated killer capture for the event participant plan." }
register_participant_api! { path: "sand::participant::EventParticipantPlan::observe_held_item", aliases: ["sand::prelude::EventParticipantPlan::observe_held_item"], kind: Method, summary: "Declares held item capture for the event participant plan." }
register_participant_api! { path: "sand::participant::EventParticipantPlan::observe_weapon", aliases: ["sand::prelude::EventParticipantPlan::observe_weapon"], kind: Method, summary: "Declares weapon capture for the event participant plan." }
register_participant_api! { path: "sand::participant::EventParticipantPlan::validate", aliases: ["sand::prelude::EventParticipantPlan::validate"], kind: Method, summary: "Provides validate for typed event participant handling." }
register_participant_api! { path: "sand::participant::EventParticipantPlanError", aliases: [], kind: Enum, summary: "Classifies event participant plan error for typed event participant handling." }
register_participant_api! { path: "sand::participant::EventParticipantPlanError::DuplicateRole", aliases: [], kind: Variant, summary: "Selects the duplicate role participant semantic." }
register_participant_api! { path: "sand::participant::EventParticipantPlanError::DuplicateRole::0", aliases: [], kind: Field, summary: "Carries the 0 value for this participant result." }
register_participant_api! { path: "sand::participant::EventParticipantPlanError::Observation", aliases: [], kind: Variant, summary: "Selects the observation participant semantic." }
register_participant_api! { path: "sand::participant::EventParticipantPlanError::Observation::0", aliases: [], kind: Field, summary: "Carries the 0 value for this participant result." }
register_participant_api! { path: "sand::participant::EventParticipantPlanError::Snapshot", aliases: [], kind: Variant, summary: "Selects the snapshot participant semantic." }
register_participant_api! { path: "sand::participant::EventParticipantPlanError::Snapshot::0", aliases: [], kind: Field, summary: "Carries the 0 value for this participant result." }
register_participant_api! { path: "sand::participant::ItemEvidenceQualifier", aliases: [], kind: Enum, summary: "Classifies item evidence qualifier for typed event participant handling." }
register_participant_api! { path: "sand::participant::ItemEvidenceQualifier::CapturedAtFirstSandControl", aliases: [], kind: Variant, summary: "Selects the captured at first sand control participant semantic." }
register_participant_api! { path: "sand::participant::ItemEvidenceQualifier::CapturedBeforeVanillaMutation", aliases: [], kind: Variant, summary: "Selects the captured before vanilla mutation participant semantic." }
register_participant_api! { path: "sand::participant::ItemParticipantRole", aliases: ["sand::item::ItemRole", "sand::item::snapshot::ItemRole", "sand::prelude::ItemParticipantRole"], kind: Enum, summary: "Classifies item participant role for typed event participant handling." }
register_participant_api! { path: "sand::participant::ItemParticipantRole::Ammunition", aliases: ["sand::item::ItemRole::Ammunition", "sand::item::snapshot::ItemRole::Ammunition", "sand::prelude::ItemParticipantRole::Ammunition"], kind: Variant, summary: "Selects the ammunition participant semantic." }
register_participant_api! { path: "sand::participant::ItemParticipantRole::DroppedItem", aliases: ["sand::item::ItemRole::DroppedItem", "sand::item::snapshot::ItemRole::DroppedItem", "sand::prelude::ItemParticipantRole::DroppedItem"], kind: Variant, summary: "Selects the dropped item participant semantic." }
register_participant_api! { path: "sand::participant::ItemParticipantRole::EquippedItem", aliases: ["sand::item::ItemRole::EquippedItem", "sand::item::snapshot::ItemRole::EquippedItem", "sand::prelude::ItemParticipantRole::EquippedItem"], kind: Variant, summary: "Selects the equipped item participant semantic." }
register_participant_api! { path: "sand::participant::ItemParticipantRole::ProjectileItem", aliases: ["sand::item::ItemRole::ProjectileItem", "sand::item::snapshot::ItemRole::ProjectileItem", "sand::prelude::ItemParticipantRole::ProjectileItem"], kind: Variant, summary: "Selects the projectile item participant semantic." }
register_participant_api! { path: "sand::participant::ItemParticipantRole::Tool", aliases: ["sand::item::ItemRole::Tool", "sand::item::snapshot::ItemRole::Tool", "sand::prelude::ItemParticipantRole::Tool"], kind: Variant, summary: "Selects the tool participant semantic." }
register_participant_api! { path: "sand::participant::ItemParticipantRole::UsedItem", aliases: ["sand::item::ItemRole::UsedItem", "sand::item::snapshot::ItemRole::UsedItem", "sand::prelude::ItemParticipantRole::UsedItem"], kind: Variant, summary: "Selects the used item participant semantic." }
register_participant_api! { path: "sand::participant::ItemParticipantRole::Weapon", aliases: ["sand::item::ItemRole::Weapon", "sand::item::snapshot::ItemRole::Weapon", "sand::prelude::ItemParticipantRole::Weapon"], kind: Variant, summary: "Selects the weapon participant semantic." }
register_participant_api! { path: "sand::participant::LocationParticipantRole", aliases: [], kind: Enum, summary: "Classifies location participant role for typed event participant handling." }
register_participant_api! { path: "sand::participant::LocationParticipantRole::EventBlock", aliases: [], kind: Variant, summary: "Selects the event block participant semantic." }
register_participant_api! { path: "sand::participant::ObservationError", aliases: [], kind: Enum, summary: "Classifies observation error for typed event participant handling." }
register_participant_api! { path: "sand::participant::ObservationError::UnsupportedVersion", aliases: [], kind: Variant, summary: "Selects the unsupported version participant semantic." }
register_participant_api! { path: "sand::participant::ObservationError::UnsupportedVersion::evidence", aliases: [], kind: Field, summary: "Carries the evidence value for this participant result." }
register_participant_api! { path: "sand::participant::ObservationError::UnsupportedVersion::role", aliases: [], kind: Field, summary: "Carries the role value for this participant result." }
register_participant_api! { path: "sand::participant::ObservationError::UnsupportedVersion::target_version", aliases: [], kind: Field, summary: "Carries the target version value for this participant result." }
register_participant_api! { path: "sand::participant::ObservationSchema", aliases: [], kind: Struct, summary: "Represents observation schema in Sand event participant transport." }
register_participant_api! { path: "sand::participant::ObservationSchema::new", aliases: [], kind: Method, summary: "Provides new for typed event participant handling." }
register_participant_api! { path: "sand::participant::ObservationSchema::storage", aliases: [], kind: Method, summary: "Provides storage for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantAvailability", aliases: ["sand::prelude::ParticipantAvailability"], kind: Enum, summary: "Classifies participant availability for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantAvailability::Available", aliases: ["sand::prelude::ParticipantAvailability::Available"], kind: Variant, summary: "Selects the available participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantAvailability::Available::0", aliases: ["sand::prelude::ParticipantAvailability::Available::0"], kind: Field, summary: "Carries the 0 value for this participant result." }
register_participant_api! { path: "sand::participant::ParticipantAvailability::Unavailable", aliases: ["sand::prelude::ParticipantAvailability::Unavailable"], kind: Variant, summary: "Selects the unavailable participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantAvailability::Unavailable::0", aliases: ["sand::prelude::ParticipantAvailability::Unavailable::0"], kind: Field, summary: "Carries the 0 value for this participant result." }
register_participant_api! { path: "sand::participant::ParticipantAvailability::available", aliases: ["sand::prelude::ParticipantAvailability::available"], kind: Method, summary: "Provides available for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantAvailability::is_available", aliases: ["sand::prelude::ParticipantAvailability::is_available"], kind: Method, summary: "Builds or evaluates the is available participant state." }
register_participant_api! { path: "sand::participant::ParticipantAvailability::map", aliases: ["sand::prelude::ParticipantAvailability::map"], kind: Method, summary: "Provides map for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantAvailability::reason", aliases: ["sand::prelude::ParticipantAvailability::reason"], kind: Method, summary: "Provides reason for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantBuilder", aliases: ["sand::prelude::ParticipantBuilder"], kind: Struct, summary: "Represents participant builder in Sand event participant transport." }
register_participant_api! { path: "sand::participant::ParticipantBuilder::build", aliases: ["sand::prelude::ParticipantBuilder::build"], kind: Method, summary: "Provides build for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantBuilder::inherit_entity", aliases: ["sand::prelude::ParticipantBuilder::inherit_entity"], kind: Method, summary: "Borrows entity from the named parent during the supported event lifecycle." }
register_participant_api! { path: "sand::participant::ParticipantBuilder::inherit_item", aliases: ["sand::prelude::ParticipantBuilder::inherit_item"], kind: Method, summary: "Borrows item from the named parent during the supported event lifecycle." }
register_participant_api! { path: "sand::participant::ParticipantBuilder::new", aliases: ["sand::prelude::ParticipantBuilder::new"], kind: Method, summary: "Provides new for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantBuilder::observe_entity", aliases: ["sand::prelude::ParticipantBuilder::observe_entity"], kind: Method, summary: "Declares entity capture for the event participant plan." }
register_participant_api! { path: "sand::participant::ParticipantBuilder::observe_item", aliases: ["sand::prelude::ParticipantBuilder::observe_item"], kind: Method, summary: "Declares item capture for the event participant plan." }
register_participant_api! { path: "sand::participant::ParticipantHand", aliases: ["sand::prelude::ParticipantHand"], kind: Enum, summary: "Classifies participant hand for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantHand::MainHand", aliases: ["sand::prelude::ParticipantHand::MainHand"], kind: Variant, summary: "Selects the main hand participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantHand::OffHand", aliases: ["sand::prelude::ParticipantHand::OffHand"], kind: Variant, summary: "Selects the off hand participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantLifetime", aliases: [], kind: Enum, summary: "Classifies participant lifetime for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantLifetime::BoundedWindow", aliases: [], kind: Variant, summary: "Selects the bounded window participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantLifetime::EventCycle", aliases: [], kind: Variant, summary: "Selects the event cycle participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantLifetime::Invocation", aliases: [], kind: Variant, summary: "Selects the invocation participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantLifetime::SynchronousDescendants", aliases: [], kind: Variant, summary: "Selects the synchronous descendants participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantLifetime::covers", aliases: [], kind: Method, summary: "Provides covers for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantReliability", aliases: ["sand::prelude::ParticipantReliability"], kind: Enum, summary: "Classifies participant reliability for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantReliability::Correlated", aliases: ["sand::prelude::ParticipantReliability::Correlated"], kind: Variant, summary: "Selects the correlated participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantReliability::Exact", aliases: ["sand::prelude::ParticipantReliability::Exact"], kind: Variant, summary: "Selects the exact participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantReliability::ExactSnapshot", aliases: ["sand::prelude::ParticipantReliability::ExactSnapshot"], kind: Variant, summary: "Selects the exact snapshot participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantReliability::Inferred", aliases: ["sand::prelude::ParticipantReliability::Inferred"], kind: Variant, summary: "Selects the inferred participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantReliability::Unavailable", aliases: ["sand::prelude::ParticipantReliability::Unavailable"], kind: Variant, summary: "Selects the unavailable participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantReliability::meets", aliases: ["sand::prelude::ParticipantReliability::meets"], kind: Method, summary: "Provides meets for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantReliabilityError", aliases: [], kind: Struct, summary: "Represents participant reliability error in Sand event participant transport." }
register_participant_api! { path: "sand::participant::ParticipantReliabilityError::requested", aliases: [], kind: Field, summary: "Carries the requested value for this participant result." }
register_participant_api! { path: "sand::participant::ParticipantReliabilityError::role", aliases: [], kind: Field, summary: "Carries the role value for this participant result." }
register_participant_api! { path: "sand::participant::ParticipantReliabilityError::supplied", aliases: [], kind: Field, summary: "Carries the supplied value for this participant result." }
register_participant_api! { path: "sand::participant::ParticipantUnavailableReason", aliases: ["sand::prelude::ParticipantUnavailableReason"], kind: Enum, summary: "Classifies participant unavailable reason for typed event participant handling." }
register_participant_api! { path: "sand::participant::ParticipantUnavailableReason::AmbiguousCandidates", aliases: ["sand::prelude::ParticipantUnavailableReason::AmbiguousCandidates"], kind: Variant, summary: "Selects the ambiguous candidates participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantUnavailableReason::CorrelationExpired", aliases: ["sand::prelude::ParticipantUnavailableReason::CorrelationExpired"], kind: Variant, summary: "Selects the correlation expired participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantUnavailableReason::ItemSourceAlreadyMutated", aliases: ["sand::prelude::ParticipantUnavailableReason::ItemSourceAlreadyMutated"], kind: Variant, summary: "Selects the item source already mutated participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantUnavailableReason::LifetimeExpired", aliases: ["sand::prelude::ParticipantUnavailableReason::LifetimeExpired"], kind: Variant, summary: "Selects the lifetime expired participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantUnavailableReason::NoMatchingObservation", aliases: ["sand::prelude::ParticipantUnavailableReason::NoMatchingObservation"], kind: Variant, summary: "Selects the no matching observation participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantUnavailableReason::NotApplicable", aliases: ["sand::prelude::ParticipantUnavailableReason::NotApplicable"], kind: Variant, summary: "Selects the not applicable participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantUnavailableReason::NotSuppliedByTrigger", aliases: ["sand::prelude::ParticipantUnavailableReason::NotSuppliedByTrigger"], kind: Variant, summary: "Selects the not supplied by trigger participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantUnavailableReason::UnsupportedBackend", aliases: ["sand::prelude::ParticipantUnavailableReason::UnsupportedBackend"], kind: Variant, summary: "Selects the unsupported backend participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantUnavailableReason::UnsupportedVersion", aliases: ["sand::prelude::ParticipantUnavailableReason::UnsupportedVersion"], kind: Variant, summary: "Selects the unsupported version participant semantic." }
register_participant_api! { path: "sand::participant::ParticipantUnavailableReason::description", aliases: ["sand::prelude::ParticipantUnavailableReason::description"], kind: Method, summary: "Provides description for typed event participant handling." }
register_participant_api! { path: "sand::participant::PlayerParticipant", aliases: [], kind: Struct, summary: "Represents player participant in Sand event participant transport." }
register_participant_api! { path: "sand::participant::PlayerParticipant::lifetime", aliases: [], kind: Method, summary: "Provides lifetime for typed event participant handling." }
register_participant_api! { path: "sand::participant::PlayerParticipant::reliability", aliases: [], kind: Method, summary: "Provides reliability for typed event participant handling." }
register_participant_api! { path: "sand::participant::PlayerParticipant::require", aliases: [], kind: Method, summary: "Checks that the participant evidence meets the requested reliability." }
register_participant_api! { path: "sand::participant::PlayerParticipant::require_exact", aliases: [], kind: Method, summary: "Rejects participant evidence that is not exact for this handler operation." }
register_participant_api! { path: "sand::participant::PlayerParticipant::role", aliases: [], kind: Method, summary: "Provides role for typed event participant handling." }
register_participant_api! { path: "sand::participant::PlayerParticipant::selector", aliases: [], kind: Method, summary: "Provides selector for typed event participant handling." }
register_participant_api! { path: "sand::participant::PlayerParticipant::subject", aliases: [], kind: Method, summary: "Provides subject for typed event participant handling." }
register_participant_api! { path: "sand::participant::observe_correlated_attacker", aliases: [], kind: Function, summary: "Captures the triggering attacker through Minecraft correlation for typed handler access." }
// END PARTICIPANT API CONTRACTS
register_text_api! { path: "sand::text::ChatColor", aliases: ["sand::cmd::ChatColor", "sand::command::ChatColor", "sand::prelude::ChatColor", "sand::prelude::cmd::ChatColor"], kind: Enum, summary: "Defines chat color values for structured Minecraft text." }
register_text_api! { path: "sand::text::ChatColor::Aqua", aliases: ["sand::cmd::ChatColor::Aqua", "sand::command::ChatColor::Aqua", "sand::prelude::ChatColor::Aqua", "sand::prelude::cmd::ChatColor::Aqua"], kind: Variant, summary: "Selects the aqua Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::Black", aliases: ["sand::cmd::ChatColor::Black", "sand::command::ChatColor::Black", "sand::prelude::ChatColor::Black", "sand::prelude::cmd::ChatColor::Black"], kind: Variant, summary: "Selects the black Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::Blue", aliases: ["sand::cmd::ChatColor::Blue", "sand::command::ChatColor::Blue", "sand::prelude::ChatColor::Blue", "sand::prelude::cmd::ChatColor::Blue"], kind: Variant, summary: "Selects the blue Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::DarkAqua", aliases: ["sand::cmd::ChatColor::DarkAqua", "sand::command::ChatColor::DarkAqua", "sand::prelude::ChatColor::DarkAqua", "sand::prelude::cmd::ChatColor::DarkAqua"], kind: Variant, summary: "Selects the dark aqua Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::DarkBlue", aliases: ["sand::cmd::ChatColor::DarkBlue", "sand::command::ChatColor::DarkBlue", "sand::prelude::ChatColor::DarkBlue", "sand::prelude::cmd::ChatColor::DarkBlue"], kind: Variant, summary: "Selects the dark blue Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::DarkGray", aliases: ["sand::cmd::ChatColor::DarkGray", "sand::command::ChatColor::DarkGray", "sand::prelude::ChatColor::DarkGray", "sand::prelude::cmd::ChatColor::DarkGray"], kind: Variant, summary: "Selects the dark gray Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::DarkGreen", aliases: ["sand::cmd::ChatColor::DarkGreen", "sand::command::ChatColor::DarkGreen", "sand::prelude::ChatColor::DarkGreen", "sand::prelude::cmd::ChatColor::DarkGreen"], kind: Variant, summary: "Selects the dark green Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::DarkPurple", aliases: ["sand::cmd::ChatColor::DarkPurple", "sand::command::ChatColor::DarkPurple", "sand::prelude::ChatColor::DarkPurple", "sand::prelude::cmd::ChatColor::DarkPurple"], kind: Variant, summary: "Selects the dark purple Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::DarkRed", aliases: ["sand::cmd::ChatColor::DarkRed", "sand::command::ChatColor::DarkRed", "sand::prelude::ChatColor::DarkRed", "sand::prelude::cmd::ChatColor::DarkRed"], kind: Variant, summary: "Selects the dark red Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::Gold", aliases: ["sand::cmd::ChatColor::Gold", "sand::command::ChatColor::Gold", "sand::prelude::ChatColor::Gold", "sand::prelude::cmd::ChatColor::Gold"], kind: Variant, summary: "Selects the gold Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::Gray", aliases: ["sand::cmd::ChatColor::Gray", "sand::command::ChatColor::Gray", "sand::prelude::ChatColor::Gray", "sand::prelude::cmd::ChatColor::Gray"], kind: Variant, summary: "Selects the gray Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::Green", aliases: ["sand::cmd::ChatColor::Green", "sand::command::ChatColor::Green", "sand::prelude::ChatColor::Green", "sand::prelude::cmd::ChatColor::Green"], kind: Variant, summary: "Selects the green Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::LightPurple", aliases: ["sand::cmd::ChatColor::LightPurple", "sand::command::ChatColor::LightPurple", "sand::prelude::ChatColor::LightPurple", "sand::prelude::cmd::ChatColor::LightPurple"], kind: Variant, summary: "Selects the light purple Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::Red", aliases: ["sand::cmd::ChatColor::Red", "sand::command::ChatColor::Red", "sand::prelude::ChatColor::Red", "sand::prelude::cmd::ChatColor::Red"], kind: Variant, summary: "Selects the red Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::White", aliases: ["sand::cmd::ChatColor::White", "sand::command::ChatColor::White", "sand::prelude::ChatColor::White", "sand::prelude::cmd::ChatColor::White"], kind: Variant, summary: "Selects the white Minecraft text behavior." }
register_text_api! { path: "sand::text::ChatColor::Yellow", aliases: ["sand::cmd::ChatColor::Yellow", "sand::command::ChatColor::Yellow", "sand::prelude::ChatColor::Yellow", "sand::prelude::cmd::ChatColor::Yellow"], kind: Variant, summary: "Selects the yellow Minecraft text behavior." }
register_text_api! { path: "sand::text::ClickEvent", aliases: ["sand::cmd::ClickEvent", "sand::command::ClickEvent", "sand::prelude::ClickEvent", "sand::prelude::cmd::ClickEvent"], kind: Enum, summary: "Defines click event values for structured Minecraft text." }
register_text_api! { path: "sand::text::ClickEvent::ChangePage", aliases: ["sand::cmd::ClickEvent::ChangePage", "sand::command::ClickEvent::ChangePage", "sand::prelude::ClickEvent::ChangePage", "sand::prelude::cmd::ClickEvent::ChangePage"], kind: Variant, summary: "Selects the change page Minecraft text behavior." }
register_text_api! { path: "sand::text::ClickEvent::ChangePage::0", aliases: ["sand::cmd::ClickEvent::ChangePage::0", "sand::command::ClickEvent::ChangePage::0", "sand::prelude::ClickEvent::ChangePage::0", "sand::prelude::cmd::ClickEvent::ChangePage::0"], kind: Field, summary: "Carries the 0 payload serialized for this text event." }
register_text_api! { path: "sand::text::ClickEvent::CopyToClipboard", aliases: ["sand::cmd::ClickEvent::CopyToClipboard", "sand::command::ClickEvent::CopyToClipboard", "sand::prelude::ClickEvent::CopyToClipboard", "sand::prelude::cmd::ClickEvent::CopyToClipboard"], kind: Variant, summary: "Selects the copy to clipboard Minecraft text behavior." }
register_text_api! { path: "sand::text::ClickEvent::CopyToClipboard::0", aliases: ["sand::cmd::ClickEvent::CopyToClipboard::0", "sand::command::ClickEvent::CopyToClipboard::0", "sand::prelude::ClickEvent::CopyToClipboard::0", "sand::prelude::cmd::ClickEvent::CopyToClipboard::0"], kind: Field, summary: "Carries the 0 payload serialized for this text event." }
register_text_api! { path: "sand::text::ClickEvent::OpenUrl", aliases: ["sand::cmd::ClickEvent::OpenUrl", "sand::command::ClickEvent::OpenUrl", "sand::prelude::ClickEvent::OpenUrl", "sand::prelude::cmd::ClickEvent::OpenUrl"], kind: Variant, summary: "Selects the open url Minecraft text behavior." }
register_text_api! { path: "sand::text::ClickEvent::OpenUrl::0", aliases: ["sand::cmd::ClickEvent::OpenUrl::0", "sand::command::ClickEvent::OpenUrl::0", "sand::prelude::ClickEvent::OpenUrl::0", "sand::prelude::cmd::ClickEvent::OpenUrl::0"], kind: Field, summary: "Carries the 0 payload serialized for this text event." }
register_text_api! { path: "sand::text::ClickEvent::RunCommand", aliases: ["sand::cmd::ClickEvent::RunCommand", "sand::command::ClickEvent::RunCommand", "sand::prelude::ClickEvent::RunCommand", "sand::prelude::cmd::ClickEvent::RunCommand"], kind: Variant, summary: "Selects the run command Minecraft text behavior." }
register_text_api! { path: "sand::text::ClickEvent::RunCommand::0", aliases: ["sand::cmd::ClickEvent::RunCommand::0", "sand::command::ClickEvent::RunCommand::0", "sand::prelude::ClickEvent::RunCommand::0", "sand::prelude::cmd::ClickEvent::RunCommand::0"], kind: Field, summary: "Carries the 0 payload serialized for this text event." }
register_text_api! { path: "sand::text::ClickEvent::SuggestCommand", aliases: ["sand::cmd::ClickEvent::SuggestCommand", "sand::command::ClickEvent::SuggestCommand", "sand::prelude::ClickEvent::SuggestCommand", "sand::prelude::cmd::ClickEvent::SuggestCommand"], kind: Variant, summary: "Selects the suggest command Minecraft text behavior." }
register_text_api! { path: "sand::text::ClickEvent::SuggestCommand::0", aliases: ["sand::cmd::ClickEvent::SuggestCommand::0", "sand::command::ClickEvent::SuggestCommand::0", "sand::prelude::ClickEvent::SuggestCommand::0", "sand::prelude::cmd::ClickEvent::SuggestCommand::0"], kind: Field, summary: "Carries the 0 payload serialized for this text event." }
register_text_api! { path: "sand::text::EntityHoverId", aliases: ["sand::cmd::EntityHoverId", "sand::command::EntityHoverId", "sand::prelude::EntityHoverId", "sand::prelude::cmd::EntityHoverId"], kind: Struct, summary: "Represents entity hover id in a structured Minecraft text component." }
register_text_api! { path: "sand::text::EntityHoverId::parse", aliases: ["sand::cmd::EntityHoverId::parse", "sand::command::EntityHoverId::parse", "sand::prelude::EntityHoverId::parse", "sand::prelude::cmd::EntityHoverId::parse"], kind: Method, summary: "Builds or validates parse on a structured Minecraft text component." }
register_text_api! { path: "sand::text::HoverEvent", aliases: ["sand::cmd::HoverEvent", "sand::command::HoverEvent", "sand::prelude::HoverEvent", "sand::prelude::cmd::HoverEvent"], kind: Enum, summary: "Defines hover event values for structured Minecraft text." }
register_text_api! { path: "sand::text::HoverEvent::ShowEntity", aliases: ["sand::cmd::HoverEvent::ShowEntity", "sand::command::HoverEvent::ShowEntity", "sand::prelude::HoverEvent::ShowEntity", "sand::prelude::cmd::HoverEvent::ShowEntity"], kind: Variant, summary: "Selects the show entity Minecraft text behavior." }
register_text_api! { path: "sand::text::HoverEvent::ShowEntity::entity_type", aliases: ["sand::cmd::HoverEvent::ShowEntity::entity_type", "sand::command::HoverEvent::ShowEntity::entity_type", "sand::prelude::HoverEvent::ShowEntity::entity_type", "sand::prelude::cmd::HoverEvent::ShowEntity::entity_type"], kind: Field, summary: "Carries the entity type payload serialized for this text event." }
register_text_api! { path: "sand::text::HoverEvent::ShowEntity::id", aliases: ["sand::cmd::HoverEvent::ShowEntity::id", "sand::command::HoverEvent::ShowEntity::id", "sand::prelude::HoverEvent::ShowEntity::id", "sand::prelude::cmd::HoverEvent::ShowEntity::id"], kind: Field, summary: "Carries the id payload serialized for this text event." }
register_text_api! { path: "sand::text::HoverEvent::ShowEntity::name", aliases: ["sand::cmd::HoverEvent::ShowEntity::name", "sand::command::HoverEvent::ShowEntity::name", "sand::prelude::HoverEvent::ShowEntity::name", "sand::prelude::cmd::HoverEvent::ShowEntity::name"], kind: Field, summary: "Carries the name payload serialized for this text event." }
register_text_api! { path: "sand::text::HoverEvent::ShowItem", aliases: ["sand::cmd::HoverEvent::ShowItem", "sand::command::HoverEvent::ShowItem", "sand::prelude::HoverEvent::ShowItem", "sand::prelude::cmd::HoverEvent::ShowItem"], kind: Variant, summary: "Selects the show item Minecraft text behavior." }
register_text_api! { path: "sand::text::HoverEvent::ShowItem::count", aliases: ["sand::cmd::HoverEvent::ShowItem::count", "sand::command::HoverEvent::ShowItem::count", "sand::prelude::HoverEvent::ShowItem::count", "sand::prelude::cmd::HoverEvent::ShowItem::count"], kind: Field, summary: "Carries the count payload serialized for this text event." }
register_text_api! { path: "sand::text::HoverEvent::ShowItem::id", aliases: ["sand::cmd::HoverEvent::ShowItem::id", "sand::command::HoverEvent::ShowItem::id", "sand::prelude::HoverEvent::ShowItem::id", "sand::prelude::cmd::HoverEvent::ShowItem::id"], kind: Field, summary: "Carries the id payload serialized for this text event." }
register_text_api! { path: "sand::text::HoverEvent::ShowText", aliases: ["sand::cmd::HoverEvent::ShowText", "sand::command::HoverEvent::ShowText", "sand::prelude::HoverEvent::ShowText", "sand::prelude::cmd::HoverEvent::ShowText"], kind: Variant, summary: "Selects the show text Minecraft text behavior." }
register_text_api! { path: "sand::text::HoverEvent::ShowText::0", aliases: ["sand::cmd::HoverEvent::ShowText::0", "sand::command::HoverEvent::ShowText::0", "sand::prelude::HoverEvent::ShowText::0", "sand::prelude::cmd::HoverEvent::ShowText::0"], kind: Field, summary: "Carries the 0 payload serialized for this text event." }
register_text_api! { path: "sand::text::IntoTextEntityType", aliases: ["sand::cmd::IntoTextEntityType", "sand::command::IntoTextEntityType", "sand::prelude::IntoTextEntityType", "sand::prelude::cmd::IntoTextEntityType"], kind: Trait, summary: "Converts typed entity identifiers for Minecraft hover text." }
register_text_api! { path: "sand::text::IntoTextEntityType::into_text_entity_type", aliases: ["sand::cmd::IntoTextEntityType::into_text_entity_type", "sand::command::IntoTextEntityType::into_text_entity_type", "sand::prelude::IntoTextEntityType::into_text_entity_type", "sand::prelude::cmd::IntoTextEntityType::into_text_entity_type"], kind: TraitMethod, summary: "Converts the typed entity identifier into its validated text-component form." }
register_text_api! { path: "sand::text::Text", aliases: ["sand::cmd::Text", "sand::command::Text", "sand::prelude::Text", "sand::prelude::cmd::Text"], kind: Struct, summary: "Represents text in a structured Minecraft text component." }
register_text_api! { path: "sand::text::Text::new", aliases: ["sand::cmd::Text::new", "sand::command::Text::new", "sand::prelude::Text::new", "sand::prelude::cmd::Text::new"], kind: Method, summary: "Builds or validates new on a structured Minecraft text component." }
register_text_api! { path: "sand::text::Text::raw_json", aliases: ["sand::cmd::Text::raw_json", "sand::command::Text::raw_json", "sand::prelude::Text::raw_json", "sand::prelude::cmd::Text::raw_json"], kind: Method, summary: "Provides the explicit untyped raw json escape hatch for prevalidated Minecraft JSON or identifiers." }
register_text_api! { path: "sand::text::TextComponent", aliases: ["sand::cmd::TextComponent", "sand::command::TextComponent", "sand::prelude::TextComponent", "sand::prelude::cmd::TextComponent"], kind: Struct, summary: "Represents text component in a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::aqua", aliases: ["sand::cmd::TextComponent::aqua", "sand::command::TextComponent::aqua", "sand::prelude::TextComponent::aqua", "sand::prelude::cmd::TextComponent::aqua"], kind: Method, summary: "Builds or validates aqua on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::black", aliases: ["sand::cmd::TextComponent::black", "sand::command::TextComponent::black", "sand::prelude::TextComponent::black", "sand::prelude::cmd::TextComponent::black"], kind: Method, summary: "Builds or validates black on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::blue", aliases: ["sand::cmd::TextComponent::blue", "sand::command::TextComponent::blue", "sand::prelude::TextComponent::blue", "sand::prelude::cmd::TextComponent::blue"], kind: Method, summary: "Builds or validates blue on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::bold", aliases: ["sand::cmd::TextComponent::bold", "sand::command::TextComponent::bold", "sand::prelude::TextComponent::bold", "sand::prelude::cmd::TextComponent::bold"], kind: Method, summary: "Builds or validates bold on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::click_change_page", aliases: ["sand::cmd::TextComponent::click_change_page", "sand::command::TextComponent::click_change_page", "sand::prelude::TextComponent::click_change_page", "sand::prelude::cmd::TextComponent::click_change_page"], kind: Method, summary: "Attaches the change page click action to this Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::click_copy", aliases: ["sand::cmd::TextComponent::click_copy", "sand::command::TextComponent::click_copy", "sand::prelude::TextComponent::click_copy", "sand::prelude::cmd::TextComponent::click_copy"], kind: Method, summary: "Attaches the copy click action to this Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::click_open_url", aliases: ["sand::cmd::TextComponent::click_open_url", "sand::command::TextComponent::click_open_url", "sand::prelude::TextComponent::click_open_url", "sand::prelude::cmd::TextComponent::click_open_url"], kind: Method, summary: "Attaches the open url click action to this Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::click_run_command", aliases: ["sand::cmd::TextComponent::click_run_command", "sand::command::TextComponent::click_run_command", "sand::prelude::TextComponent::click_run_command", "sand::prelude::cmd::TextComponent::click_run_command"], kind: Method, summary: "Attaches the run command click action to this Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::click_suggest_command", aliases: ["sand::cmd::TextComponent::click_suggest_command", "sand::command::TextComponent::click_suggest_command", "sand::prelude::TextComponent::click_suggest_command", "sand::prelude::cmd::TextComponent::click_suggest_command"], kind: Method, summary: "Attaches the suggest command click action to this Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::color", aliases: ["sand::cmd::TextComponent::color", "sand::command::TextComponent::color", "sand::prelude::TextComponent::color", "sand::prelude::cmd::TextComponent::color"], kind: Method, summary: "Builds or validates color on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::color_hex", aliases: ["sand::cmd::TextComponent::color_hex", "sand::command::TextComponent::color_hex", "sand::prelude::TextComponent::color_hex", "sand::prelude::cmd::TextComponent::color_hex"], kind: Method, summary: "Builds or validates color hex on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::dark_aqua", aliases: ["sand::cmd::TextComponent::dark_aqua", "sand::command::TextComponent::dark_aqua", "sand::prelude::TextComponent::dark_aqua", "sand::prelude::cmd::TextComponent::dark_aqua"], kind: Method, summary: "Builds or validates dark aqua on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::dark_blue", aliases: ["sand::cmd::TextComponent::dark_blue", "sand::command::TextComponent::dark_blue", "sand::prelude::TextComponent::dark_blue", "sand::prelude::cmd::TextComponent::dark_blue"], kind: Method, summary: "Builds or validates dark blue on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::dark_gray", aliases: ["sand::cmd::TextComponent::dark_gray", "sand::command::TextComponent::dark_gray", "sand::prelude::TextComponent::dark_gray", "sand::prelude::cmd::TextComponent::dark_gray"], kind: Method, summary: "Builds or validates dark gray on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::dark_green", aliases: ["sand::cmd::TextComponent::dark_green", "sand::command::TextComponent::dark_green", "sand::prelude::TextComponent::dark_green", "sand::prelude::cmd::TextComponent::dark_green"], kind: Method, summary: "Builds or validates dark green on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::dark_purple", aliases: ["sand::cmd::TextComponent::dark_purple", "sand::command::TextComponent::dark_purple", "sand::prelude::TextComponent::dark_purple", "sand::prelude::cmd::TextComponent::dark_purple"], kind: Method, summary: "Builds or validates dark purple on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::dark_red", aliases: ["sand::cmd::TextComponent::dark_red", "sand::command::TextComponent::dark_red", "sand::prelude::TextComponent::dark_red", "sand::prelude::cmd::TextComponent::dark_red"], kind: Method, summary: "Builds or validates dark red on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::font", aliases: ["sand::cmd::TextComponent::font", "sand::command::TextComponent::font", "sand::prelude::TextComponent::font", "sand::prelude::cmd::TextComponent::font"], kind: Method, summary: "Builds or validates font on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::gold", aliases: ["sand::cmd::TextComponent::gold", "sand::command::TextComponent::gold", "sand::prelude::TextComponent::gold", "sand::prelude::cmd::TextComponent::gold"], kind: Method, summary: "Builds or validates gold on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::gray", aliases: ["sand::cmd::TextComponent::gray", "sand::command::TextComponent::gray", "sand::prelude::TextComponent::gray", "sand::prelude::cmd::TextComponent::gray"], kind: Method, summary: "Builds or validates gray on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::green", aliases: ["sand::cmd::TextComponent::green", "sand::command::TextComponent::green", "sand::prelude::TextComponent::green", "sand::prelude::cmd::TextComponent::green"], kind: Method, summary: "Builds or validates green on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::hover_entity", aliases: ["sand::cmd::TextComponent::hover_entity", "sand::command::TextComponent::hover_entity", "sand::prelude::TextComponent::hover_entity", "sand::prelude::cmd::TextComponent::hover_entity"], kind: Method, summary: "Attaches the entity hover content to this Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::hover_entity_raw", aliases: ["sand::cmd::TextComponent::hover_entity_raw", "sand::command::TextComponent::hover_entity_raw", "sand::prelude::TextComponent::hover_entity_raw", "sand::prelude::cmd::TextComponent::hover_entity_raw"], kind: Method, summary: "Provides the explicit untyped hover entity raw escape hatch for prevalidated Minecraft JSON or identifiers." }
register_text_api! { path: "sand::text::TextComponent::hover_entity_with_id", aliases: ["sand::cmd::TextComponent::hover_entity_with_id", "sand::command::TextComponent::hover_entity_with_id", "sand::prelude::TextComponent::hover_entity_with_id", "sand::prelude::cmd::TextComponent::hover_entity_with_id"], kind: Method, summary: "Attaches the entity with id hover content to this Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::hover_item", aliases: ["sand::cmd::TextComponent::hover_item", "sand::command::TextComponent::hover_item", "sand::prelude::TextComponent::hover_item", "sand::prelude::cmd::TextComponent::hover_item"], kind: Method, summary: "Attaches the item hover content to this Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::hover_item_raw", aliases: ["sand::cmd::TextComponent::hover_item_raw", "sand::command::TextComponent::hover_item_raw", "sand::prelude::TextComponent::hover_item_raw", "sand::prelude::cmd::TextComponent::hover_item_raw"], kind: Method, summary: "Provides the explicit untyped hover item raw escape hatch for prevalidated Minecraft JSON or identifiers." }
register_text_api! { path: "sand::text::TextComponent::hover_item_with_count", aliases: ["sand::cmd::TextComponent::hover_item_with_count", "sand::command::TextComponent::hover_item_with_count", "sand::prelude::TextComponent::hover_item_with_count", "sand::prelude::cmd::TextComponent::hover_item_with_count"], kind: Method, summary: "Attaches the item with count hover content to this Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::hover_text", aliases: ["sand::cmd::TextComponent::hover_text", "sand::command::TextComponent::hover_text", "sand::prelude::TextComponent::hover_text", "sand::prelude::cmd::TextComponent::hover_text"], kind: Method, summary: "Attaches the text hover content to this Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::insertion", aliases: ["sand::cmd::TextComponent::insertion", "sand::command::TextComponent::insertion", "sand::prelude::TextComponent::insertion", "sand::prelude::cmd::TextComponent::insertion"], kind: Method, summary: "Builds or validates insertion on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::italic", aliases: ["sand::cmd::TextComponent::italic", "sand::command::TextComponent::italic", "sand::prelude::TextComponent::italic", "sand::prelude::cmd::TextComponent::italic"], kind: Method, summary: "Builds or validates italic on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::keybind", aliases: ["sand::cmd::TextComponent::keybind", "sand::command::TextComponent::keybind", "sand::prelude::TextComponent::keybind", "sand::prelude::cmd::TextComponent::keybind"], kind: Method, summary: "Builds or validates keybind on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::light_purple", aliases: ["sand::cmd::TextComponent::light_purple", "sand::command::TextComponent::light_purple", "sand::prelude::TextComponent::light_purple", "sand::prelude::cmd::TextComponent::light_purple"], kind: Method, summary: "Builds or validates light purple on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::literal", aliases: ["sand::cmd::TextComponent::literal", "sand::command::TextComponent::literal", "sand::prelude::TextComponent::literal", "sand::prelude::cmd::TextComponent::literal"], kind: Method, summary: "Builds or validates literal on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::obfuscated", aliases: ["sand::cmd::TextComponent::obfuscated", "sand::command::TextComponent::obfuscated", "sand::prelude::TextComponent::obfuscated", "sand::prelude::cmd::TextComponent::obfuscated"], kind: Method, summary: "Builds or validates obfuscated on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::red", aliases: ["sand::cmd::TextComponent::red", "sand::command::TextComponent::red", "sand::prelude::TextComponent::red", "sand::prelude::cmd::TextComponent::red"], kind: Method, summary: "Builds or validates red on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::score", aliases: ["sand::cmd::TextComponent::score", "sand::command::TextComponent::score", "sand::prelude::TextComponent::score", "sand::prelude::cmd::TextComponent::score"], kind: Method, summary: "Builds or validates score on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::selector", aliases: ["sand::cmd::TextComponent::selector", "sand::command::TextComponent::selector", "sand::prelude::TextComponent::selector", "sand::prelude::cmd::TextComponent::selector"], kind: Method, summary: "Builds or validates selector on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::selector_raw", aliases: ["sand::cmd::TextComponent::selector_raw", "sand::command::TextComponent::selector_raw", "sand::prelude::TextComponent::selector_raw", "sand::prelude::cmd::TextComponent::selector_raw"], kind: Method, summary: "Provides the explicit untyped selector raw escape hatch for prevalidated Minecraft JSON or identifiers." }
register_text_api! { path: "sand::text::TextComponent::selector_typed", aliases: ["sand::cmd::TextComponent::selector_typed", "sand::command::TextComponent::selector_typed", "sand::prelude::TextComponent::selector_typed", "sand::prelude::cmd::TextComponent::selector_typed"], kind: Method, summary: "Builds or validates selector typed on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::strikethrough", aliases: ["sand::cmd::TextComponent::strikethrough", "sand::command::TextComponent::strikethrough", "sand::prelude::TextComponent::strikethrough", "sand::prelude::cmd::TextComponent::strikethrough"], kind: Method, summary: "Builds or validates strikethrough on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::then", aliases: ["sand::cmd::TextComponent::then", "sand::command::TextComponent::then", "sand::prelude::TextComponent::then", "sand::prelude::cmd::TextComponent::then"], kind: Method, summary: "Builds or validates then on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::translate", aliases: ["sand::cmd::TextComponent::translate", "sand::command::TextComponent::translate", "sand::prelude::TextComponent::translate", "sand::prelude::cmd::TextComponent::translate"], kind: Method, summary: "Builds or validates translate on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::translate_with", aliases: ["sand::cmd::TextComponent::translate_with", "sand::command::TextComponent::translate_with", "sand::prelude::TextComponent::translate_with", "sand::prelude::cmd::TextComponent::translate_with"], kind: Method, summary: "Builds or validates translate with on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::try_to_json_value", aliases: ["sand::cmd::TextComponent::try_to_json_value", "sand::command::TextComponent::try_to_json_value", "sand::prelude::TextComponent::try_to_json_value", "sand::prelude::cmd::TextComponent::try_to_json_value"], kind: Method, summary: "Builds or validates try to json value on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::underlined", aliases: ["sand::cmd::TextComponent::underlined", "sand::command::TextComponent::underlined", "sand::prelude::TextComponent::underlined", "sand::prelude::cmd::TextComponent::underlined"], kind: Method, summary: "Builds or validates underlined on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::validate_at_path", aliases: ["sand::cmd::TextComponent::validate_at_path", "sand::command::TextComponent::validate_at_path", "sand::prelude::TextComponent::validate_at_path", "sand::prelude::cmd::TextComponent::validate_at_path"], kind: Method, summary: "Builds or validates validate at path on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::white", aliases: ["sand::cmd::TextComponent::white", "sand::command::TextComponent::white", "sand::prelude::TextComponent::white", "sand::prelude::cmd::TextComponent::white"], kind: Method, summary: "Builds or validates white on a structured Minecraft text component." }
register_text_api! { path: "sand::text::TextComponent::yellow", aliases: ["sand::cmd::TextComponent::yellow", "sand::command::TextComponent::yellow", "sand::prelude::TextComponent::yellow", "sand::prelude::cmd::TextComponent::yellow"], kind: Method, summary: "Builds or validates yellow on a structured Minecraft text component." }
// END TEXT API CONTRACTS
register_data_api! { path: "sand::data::BlockNbt", aliases: ["sand::prelude::BlockNbt", "sand::state::BlockNbt"], kind: Struct, summary: "Represents block nbt in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::BlockNbt::path", aliases: ["sand::prelude::BlockNbt::path", "sand::state::BlockNbt::path"], kind: Method, summary: "Builds or resolves path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::BlockNbt::pos", aliases: ["sand::prelude::BlockNbt::pos", "sand::state::BlockNbt::pos"], kind: Method, summary: "Builds or resolves pos in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::DataCommand", aliases: ["sand::cmd::DataCommand", "sand::command::DataCommand", "sand::prelude::DataCommand", "sand::prelude::cmd::DataCommand", "sand::state::DataCommand"], kind: Enum, summary: "Defines data command for typed Minecraft NBT and data commands." }
register_data_api! { path: "sand::data::DataCommand::Get", aliases: ["sand::cmd::DataCommand::Get", "sand::command::DataCommand::Get", "sand::prelude::DataCommand::Get", "sand::prelude::cmd::DataCommand::Get", "sand::state::DataCommand::Get"], kind: Variant, summary: "Selects the get NBT or data-command operation." }
register_data_api! { path: "sand::data::DataCommand::Get::scale", aliases: ["sand::cmd::DataCommand::Get::scale", "sand::command::DataCommand::Get::scale", "sand::prelude::DataCommand::Get::scale", "sand::prelude::cmd::DataCommand::Get::scale", "sand::state::DataCommand::Get::scale"], kind: Field, summary: "Carries the scale payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataCommand::Get::source", aliases: ["sand::cmd::DataCommand::Get::source", "sand::command::DataCommand::Get::source", "sand::prelude::DataCommand::Get::source", "sand::prelude::cmd::DataCommand::Get::source", "sand::state::DataCommand::Get::source"], kind: Field, summary: "Carries the source payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataCommand::Merge", aliases: ["sand::cmd::DataCommand::Merge", "sand::command::DataCommand::Merge", "sand::prelude::DataCommand::Merge", "sand::prelude::cmd::DataCommand::Merge", "sand::state::DataCommand::Merge"], kind: Variant, summary: "Selects the merge NBT or data-command operation." }
register_data_api! { path: "sand::data::DataCommand::Merge::target", aliases: ["sand::cmd::DataCommand::Merge::target", "sand::command::DataCommand::Merge::target", "sand::prelude::DataCommand::Merge::target", "sand::prelude::cmd::DataCommand::Merge::target", "sand::state::DataCommand::Merge::target"], kind: Field, summary: "Carries the target payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataCommand::Merge::value", aliases: ["sand::cmd::DataCommand::Merge::value", "sand::command::DataCommand::Merge::value", "sand::prelude::DataCommand::Merge::value", "sand::prelude::cmd::DataCommand::Merge::value", "sand::state::DataCommand::Merge::value"], kind: Field, summary: "Carries the value payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataCommand::Modify", aliases: ["sand::cmd::DataCommand::Modify", "sand::command::DataCommand::Modify", "sand::prelude::DataCommand::Modify", "sand::prelude::cmd::DataCommand::Modify", "sand::state::DataCommand::Modify"], kind: Variant, summary: "Selects the modify NBT or data-command operation." }
register_data_api! { path: "sand::data::DataCommand::Modify::operation", aliases: ["sand::cmd::DataCommand::Modify::operation", "sand::command::DataCommand::Modify::operation", "sand::prelude::DataCommand::Modify::operation", "sand::prelude::cmd::DataCommand::Modify::operation", "sand::state::DataCommand::Modify::operation"], kind: Field, summary: "Carries the operation payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataCommand::Modify::source", aliases: ["sand::cmd::DataCommand::Modify::source", "sand::command::DataCommand::Modify::source", "sand::prelude::DataCommand::Modify::source", "sand::prelude::cmd::DataCommand::Modify::source", "sand::state::DataCommand::Modify::source"], kind: Field, summary: "Carries the source payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataCommand::Modify::target", aliases: ["sand::cmd::DataCommand::Modify::target", "sand::command::DataCommand::Modify::target", "sand::prelude::DataCommand::Modify::target", "sand::prelude::cmd::DataCommand::Modify::target", "sand::state::DataCommand::Modify::target"], kind: Field, summary: "Carries the target payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataCommand::Remove", aliases: ["sand::cmd::DataCommand::Remove", "sand::command::DataCommand::Remove", "sand::prelude::DataCommand::Remove", "sand::prelude::cmd::DataCommand::Remove", "sand::state::DataCommand::Remove"], kind: Variant, summary: "Selects the remove NBT or data-command operation." }
register_data_api! { path: "sand::data::DataCommand::Remove::target", aliases: ["sand::cmd::DataCommand::Remove::target", "sand::command::DataCommand::Remove::target", "sand::prelude::DataCommand::Remove::target", "sand::prelude::cmd::DataCommand::Remove::target", "sand::state::DataCommand::Remove::target"], kind: Field, summary: "Carries the target payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataCommand::contains", aliases: ["sand::cmd::DataCommand::contains", "sand::command::DataCommand::contains", "sand::prelude::DataCommand::contains", "sand::prelude::cmd::DataCommand::contains", "sand::state::DataCommand::contains"], kind: Method, summary: "Builds or resolves contains in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::DataCommand::try_render", aliases: ["sand::cmd::DataCommand::try_render", "sand::command::DataCommand::try_render", "sand::prelude::DataCommand::try_render", "sand::prelude::cmd::DataCommand::try_render", "sand::state::DataCommand::try_render"], kind: Method, summary: "Builds or resolves try render in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::DataModifyOperation", aliases: ["sand::cmd::DataModifyOperation", "sand::command::DataModifyOperation", "sand::prelude::cmd::DataModifyOperation"], kind: Enum, summary: "Defines data modify operation for typed Minecraft NBT and data commands." }
register_data_api! { path: "sand::data::DataModifyOperation::Append", aliases: ["sand::cmd::DataModifyOperation::Append", "sand::command::DataModifyOperation::Append", "sand::prelude::cmd::DataModifyOperation::Append"], kind: Variant, summary: "Selects the append NBT or data-command operation." }
register_data_api! { path: "sand::data::DataModifyOperation::Insert", aliases: ["sand::cmd::DataModifyOperation::Insert", "sand::command::DataModifyOperation::Insert", "sand::prelude::cmd::DataModifyOperation::Insert"], kind: Variant, summary: "Selects the insert NBT or data-command operation." }
register_data_api! { path: "sand::data::DataModifyOperation::Insert::0", aliases: ["sand::cmd::DataModifyOperation::Insert::0", "sand::command::DataModifyOperation::Insert::0", "sand::prelude::cmd::DataModifyOperation::Insert::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataModifyOperation::Merge", aliases: ["sand::cmd::DataModifyOperation::Merge", "sand::command::DataModifyOperation::Merge", "sand::prelude::cmd::DataModifyOperation::Merge"], kind: Variant, summary: "Selects the merge NBT or data-command operation." }
register_data_api! { path: "sand::data::DataModifyOperation::Prepend", aliases: ["sand::cmd::DataModifyOperation::Prepend", "sand::command::DataModifyOperation::Prepend", "sand::prelude::cmd::DataModifyOperation::Prepend"], kind: Variant, summary: "Selects the prepend NBT or data-command operation." }
register_data_api! { path: "sand::data::DataModifyOperation::Set", aliases: ["sand::cmd::DataModifyOperation::Set", "sand::command::DataModifyOperation::Set", "sand::prelude::cmd::DataModifyOperation::Set"], kind: Variant, summary: "Selects the set NBT or data-command operation." }
register_data_api! { path: "sand::data::DataSource", aliases: ["sand::cmd::DataSource", "sand::command::DataSource", "sand::prelude::cmd::DataSource"], kind: Enum, summary: "Defines data source for typed Minecraft NBT and data commands." }
register_data_api! { path: "sand::data::DataSource::From", aliases: ["sand::cmd::DataSource::From", "sand::command::DataSource::From", "sand::prelude::cmd::DataSource::From"], kind: Variant, summary: "Selects the from NBT or data-command operation." }
register_data_api! { path: "sand::data::DataSource::From::0", aliases: ["sand::cmd::DataSource::From::0", "sand::command::DataSource::From::0", "sand::prelude::cmd::DataSource::From::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataSource::String", aliases: ["sand::cmd::DataSource::String", "sand::command::DataSource::String", "sand::prelude::cmd::DataSource::String"], kind: Variant, summary: "Selects the string NBT or data-command operation." }
register_data_api! { path: "sand::data::DataSource::String::0", aliases: ["sand::cmd::DataSource::String::0", "sand::command::DataSource::String::0", "sand::prelude::cmd::DataSource::String::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataSource::Value", aliases: ["sand::cmd::DataSource::Value", "sand::command::DataSource::Value", "sand::prelude::cmd::DataSource::Value"], kind: Variant, summary: "Selects the value NBT or data-command operation." }
register_data_api! { path: "sand::data::DataSource::Value::0", aliases: ["sand::cmd::DataSource::Value::0", "sand::command::DataSource::Value::0", "sand::prelude::cmd::DataSource::Value::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataTarget", aliases: ["sand::cmd::DataTarget", "sand::command::DataTarget", "sand::prelude::cmd::DataTarget"], kind: Enum, summary: "Defines data target for typed Minecraft NBT and data commands." }
register_data_api! { path: "sand::data::DataTarget::Block", aliases: ["sand::cmd::DataTarget::Block", "sand::command::DataTarget::Block", "sand::data::NbtLocation::Block", "sand::prelude::NbtLocation::Block", "sand::prelude::cmd::DataTarget::Block", "sand::state::NbtLocation::Block"], kind: Variant, summary: "Selects the block NBT or data-command operation." }
register_data_api! { path: "sand::data::DataTarget::Block::0", aliases: ["sand::cmd::DataTarget::Block::0", "sand::command::DataTarget::Block::0", "sand::data::NbtLocation::Block::0", "sand::prelude::NbtLocation::Block::0", "sand::prelude::cmd::DataTarget::Block::0", "sand::state::NbtLocation::Block::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataTarget::Entity", aliases: ["sand::cmd::DataTarget::Entity", "sand::command::DataTarget::Entity", "sand::data::NbtLocation::Entity", "sand::prelude::NbtLocation::Entity", "sand::prelude::cmd::DataTarget::Entity", "sand::state::NbtLocation::Entity"], kind: Variant, summary: "Selects the entity NBT or data-command operation." }
register_data_api! { path: "sand::data::DataTarget::Entity::0", aliases: ["sand::cmd::DataTarget::Entity::0", "sand::command::DataTarget::Entity::0", "sand::data::NbtLocation::Entity::0", "sand::prelude::NbtLocation::Entity::0", "sand::prelude::cmd::DataTarget::Entity::0", "sand::state::NbtLocation::Entity::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataTarget::Storage", aliases: ["sand::cmd::DataTarget::Storage", "sand::command::DataTarget::Storage", "sand::data::NbtLocation::Storage", "sand::prelude::NbtLocation::Storage", "sand::prelude::cmd::DataTarget::Storage", "sand::state::NbtLocation::Storage"], kind: Variant, summary: "Selects the storage NBT or data-command operation." }
register_data_api! { path: "sand::data::DataTarget::Storage::0", aliases: ["sand::cmd::DataTarget::Storage::0", "sand::command::DataTarget::Storage::0", "sand::data::NbtLocation::Storage::0", "sand::prelude::NbtLocation::Storage::0", "sand::prelude::cmd::DataTarget::Storage::0", "sand::state::NbtLocation::Storage::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::DataTarget::block", aliases: ["sand::cmd::DataTarget::block", "sand::command::DataTarget::block", "sand::data::NbtLocation::block", "sand::prelude::NbtLocation::block", "sand::prelude::cmd::DataTarget::block", "sand::state::NbtLocation::block"], kind: Method, summary: "Builds or resolves block in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::DataTarget::entity", aliases: ["sand::cmd::DataTarget::entity", "sand::command::DataTarget::entity", "sand::data::NbtLocation::entity", "sand::prelude::NbtLocation::entity", "sand::prelude::cmd::DataTarget::entity", "sand::state::NbtLocation::entity"], kind: Method, summary: "Builds or resolves entity in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::DataTarget::merge", aliases: ["sand::cmd::DataTarget::merge", "sand::command::DataTarget::merge", "sand::data::NbtLocation::merge", "sand::prelude::NbtLocation::merge", "sand::prelude::cmd::DataTarget::merge", "sand::state::NbtLocation::merge"], kind: Method, summary: "Builds the typed Minecraft data modification for merge." }
register_data_api! { path: "sand::data::DataTarget::path", aliases: ["sand::cmd::DataTarget::path", "sand::command::DataTarget::path", "sand::data::NbtLocation::path", "sand::prelude::NbtLocation::path", "sand::prelude::cmd::DataTarget::path", "sand::state::NbtLocation::path"], kind: Method, summary: "Builds or resolves path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::DataTarget::storage", aliases: ["sand::cmd::DataTarget::storage", "sand::command::DataTarget::storage", "sand::data::NbtLocation::storage", "sand::prelude::NbtLocation::storage", "sand::prelude::cmd::DataTarget::storage", "sand::state::NbtLocation::storage"], kind: Method, summary: "Builds or resolves storage in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::DataTarget::typed_path", aliases: ["sand::cmd::DataTarget::typed_path", "sand::command::DataTarget::typed_path", "sand::data::NbtLocation::typed_path", "sand::prelude::NbtLocation::typed_path", "sand::prelude::cmd::DataTarget::typed_path", "sand::state::NbtLocation::typed_path"], kind: Method, summary: "Builds or resolves typed path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::EntityNbt", aliases: ["sand::prelude::EntityNbt", "sand::state::EntityNbt"], kind: Struct, summary: "Represents entity nbt in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::EntityNbt::path", aliases: ["sand::prelude::EntityNbt::path", "sand::state::EntityNbt::path"], kind: Method, summary: "Builds or resolves path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::EntityNbt::target", aliases: ["sand::prelude::EntityNbt::target", "sand::state::EntityNbt::target"], kind: Method, summary: "Builds or resolves target in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::Nbt", aliases: ["sand::cmd::Nbt", "sand::command::Nbt", "sand::prelude::Nbt", "sand::prelude::cmd::Nbt", "sand::state::Nbt"], kind: Struct, summary: "Represents nbt in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::Nbt::block", aliases: ["sand::cmd::Nbt::block", "sand::command::Nbt::block", "sand::prelude::Nbt::block", "sand::prelude::cmd::Nbt::block", "sand::state::Nbt::block"], kind: Method, summary: "Builds or resolves block in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::Nbt::entity", aliases: ["sand::cmd::Nbt::entity", "sand::command::Nbt::entity", "sand::prelude::Nbt::entity", "sand::prelude::cmd::Nbt::entity", "sand::state::Nbt::entity"], kind: Method, summary: "Builds or resolves entity in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::Nbt::storage", aliases: ["sand::cmd::Nbt::storage", "sand::command::Nbt::storage", "sand::prelude::Nbt::storage", "sand::prelude::cmd::Nbt::storage", "sand::state::Nbt::storage"], kind: Method, summary: "Builds or resolves storage in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtCompound", aliases: ["sand::cmd::NbtCompound", "sand::command::NbtCompound", "sand::data::SnbtCompound", "sand::prelude::NbtCompound", "sand::prelude::SnbtCompound", "sand::prelude::cmd::NbtCompound", "sand::state::SnbtCompound"], kind: Struct, summary: "Represents nbt compound in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtCompound::field", aliases: ["sand::cmd::NbtCompound::field", "sand::command::NbtCompound::field", "sand::data::SnbtCompound::field", "sand::prelude::NbtCompound::field", "sand::prelude::SnbtCompound::field", "sand::prelude::cmd::NbtCompound::field", "sand::state::SnbtCompound::field"], kind: Method, summary: "Builds or resolves field in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtCompound::insert", aliases: ["sand::cmd::NbtCompound::insert", "sand::command::NbtCompound::insert", "sand::data::SnbtCompound::insert", "sand::prelude::NbtCompound::insert", "sand::prelude::SnbtCompound::insert", "sand::prelude::cmd::NbtCompound::insert", "sand::state::SnbtCompound::insert"], kind: Method, summary: "Builds the typed Minecraft data modification for insert." }
register_data_api! { path: "sand::data::NbtCompound::is_empty", aliases: ["sand::cmd::NbtCompound::is_empty", "sand::command::NbtCompound::is_empty", "sand::data::SnbtCompound::is_empty", "sand::prelude::NbtCompound::is_empty", "sand::prelude::SnbtCompound::is_empty", "sand::prelude::cmd::NbtCompound::is_empty", "sand::state::SnbtCompound::is_empty"], kind: Method, summary: "Builds or resolves is empty in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtCompound::new", aliases: ["sand::cmd::NbtCompound::new", "sand::command::NbtCompound::new", "sand::data::SnbtCompound::new", "sand::prelude::NbtCompound::new", "sand::prelude::SnbtCompound::new", "sand::prelude::cmd::NbtCompound::new", "sand::state::SnbtCompound::new"], kind: Method, summary: "Builds or resolves new in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtLocation", aliases: ["sand::prelude::NbtLocation", "sand::state::NbtLocation"], kind: TypeAlias, summary: "Names nbt location as the canonical typed NBT location." }
register_data_api! { path: "sand::data::NbtPath", aliases: ["sand::cmd::NbtPath", "sand::command::NbtPath", "sand::prelude::NbtPath", "sand::prelude::cmd::NbtPath", "sand::state::NbtPath"], kind: Struct, summary: "Represents nbt path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtPath::as_str", aliases: ["sand::cmd::NbtPath::as_str", "sand::command::NbtPath::as_str", "sand::prelude::NbtPath::as_str", "sand::prelude::cmd::NbtPath::as_str", "sand::state::NbtPath::as_str"], kind: Method, summary: "Builds or resolves as str in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtPath::field", aliases: ["sand::cmd::NbtPath::field", "sand::command::NbtPath::field", "sand::prelude::NbtPath::field", "sand::prelude::cmd::NbtPath::field", "sand::state::NbtPath::field"], kind: Method, summary: "Builds or resolves field in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtPath::index", aliases: ["sand::cmd::NbtPath::index", "sand::command::NbtPath::index", "sand::prelude::NbtPath::index", "sand::prelude::cmd::NbtPath::index", "sand::state::NbtPath::index"], kind: Method, summary: "Builds or resolves index in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtPath::is_raw", aliases: ["sand::cmd::NbtPath::is_raw", "sand::command::NbtPath::is_raw", "sand::prelude::NbtPath::is_raw", "sand::prelude::cmd::NbtPath::is_raw", "sand::state::NbtPath::is_raw"], kind: Method, summary: "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility." }
register_data_api! { path: "sand::data::NbtPath::key", aliases: ["sand::cmd::NbtPath::key", "sand::command::NbtPath::key", "sand::prelude::NbtPath::key", "sand::prelude::cmd::NbtPath::key", "sand::state::NbtPath::key"], kind: Method, summary: "Builds or resolves key in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtPath::new", aliases: ["sand::cmd::NbtPath::new", "sand::command::NbtPath::new", "sand::prelude::NbtPath::new", "sand::prelude::cmd::NbtPath::new", "sand::state::NbtPath::new"], kind: Method, summary: "Builds or resolves new in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtPath::raw", aliases: ["sand::cmd::NbtPath::raw", "sand::command::NbtPath::raw", "sand::prelude::NbtPath::raw", "sand::prelude::cmd::NbtPath::raw", "sand::state::NbtPath::raw"], kind: Method, summary: "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility." }
register_data_api! { path: "sand::data::NbtPath::root", aliases: ["sand::cmd::NbtPath::root", "sand::command::NbtPath::root", "sand::prelude::NbtPath::root", "sand::prelude::cmd::NbtPath::root", "sand::state::NbtPath::root"], kind: Method, summary: "Builds or resolves root in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtRef", aliases: ["sand::cmd::NbtRef", "sand::command::NbtRef", "sand::prelude::NbtRef", "sand::prelude::cmd::NbtRef", "sand::state::NbtRef"], kind: Struct, summary: "Represents nbt ref in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtRef::append", aliases: ["sand::cmd::NbtRef::append", "sand::command::NbtRef::append", "sand::prelude::NbtRef::append", "sand::prelude::cmd::NbtRef::append", "sand::state::NbtRef::append"], kind: Method, summary: "Builds the typed Minecraft data modification for append." }
register_data_api! { path: "sand::data::NbtRef::append_from", aliases: ["sand::cmd::NbtRef::append_from", "sand::command::NbtRef::append_from", "sand::prelude::NbtRef::append_from", "sand::prelude::cmd::NbtRef::append_from", "sand::state::NbtRef::append_from"], kind: Method, summary: "Builds the typed Minecraft data modification for append from." }
register_data_api! { path: "sand::data::NbtRef::as_str", aliases: ["sand::cmd::NbtRef::as_str", "sand::command::NbtRef::as_str", "sand::prelude::NbtRef::as_str", "sand::prelude::cmd::NbtRef::as_str", "sand::state::NbtRef::as_str"], kind: Method, summary: "Builds or resolves as str in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtRef::copy_from", aliases: ["sand::cmd::NbtRef::copy_from", "sand::command::NbtRef::copy_from", "sand::prelude::NbtRef::copy_from", "sand::prelude::cmd::NbtRef::copy_from", "sand::state::NbtRef::copy_from"], kind: Method, summary: "Builds the typed Minecraft data modification for copy from." }
register_data_api! { path: "sand::data::NbtRef::field", aliases: ["sand::cmd::NbtRef::field", "sand::command::NbtRef::field", "sand::prelude::NbtRef::field", "sand::prelude::cmd::NbtRef::field", "sand::state::NbtRef::field"], kind: Method, summary: "Builds or resolves field in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtRef::get", aliases: ["sand::cmd::NbtRef::get", "sand::command::NbtRef::get", "sand::prelude::NbtRef::get", "sand::prelude::cmd::NbtRef::get", "sand::state::NbtRef::get"], kind: Method, summary: "Builds the typed Minecraft data query for get." }
register_data_api! { path: "sand::data::NbtRef::get_scaled", aliases: ["sand::cmd::NbtRef::get_scaled", "sand::command::NbtRef::get_scaled", "sand::prelude::NbtRef::get_scaled", "sand::prelude::cmd::NbtRef::get_scaled", "sand::state::NbtRef::get_scaled"], kind: Method, summary: "Builds the typed Minecraft data query for get scaled." }
register_data_api! { path: "sand::data::NbtRef::index", aliases: ["sand::cmd::NbtRef::index", "sand::command::NbtRef::index", "sand::prelude::NbtRef::index", "sand::prelude::cmd::NbtRef::index", "sand::state::NbtRef::index"], kind: Method, summary: "Builds or resolves index in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtRef::insert", aliases: ["sand::cmd::NbtRef::insert", "sand::command::NbtRef::insert", "sand::prelude::NbtRef::insert", "sand::prelude::cmd::NbtRef::insert", "sand::state::NbtRef::insert"], kind: Method, summary: "Builds the typed Minecraft data modification for insert." }
register_data_api! { path: "sand::data::NbtRef::insert_from", aliases: ["sand::cmd::NbtRef::insert_from", "sand::command::NbtRef::insert_from", "sand::prelude::NbtRef::insert_from", "sand::prelude::cmd::NbtRef::insert_from", "sand::state::NbtRef::insert_from"], kind: Method, summary: "Builds the typed Minecraft data modification for insert from." }
register_data_api! { path: "sand::data::NbtRef::key", aliases: ["sand::cmd::NbtRef::key", "sand::command::NbtRef::key", "sand::prelude::NbtRef::key", "sand::prelude::cmd::NbtRef::key", "sand::state::NbtRef::key"], kind: Method, summary: "Builds or resolves key in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtRef::location", aliases: ["sand::cmd::NbtRef::location", "sand::command::NbtRef::location", "sand::prelude::NbtRef::location", "sand::prelude::cmd::NbtRef::location", "sand::state::NbtRef::location"], kind: Method, summary: "Builds or resolves location in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtRef::merge", aliases: ["sand::cmd::NbtRef::merge", "sand::command::NbtRef::merge", "sand::prelude::NbtRef::merge", "sand::prelude::cmd::NbtRef::merge", "sand::state::NbtRef::merge"], kind: Method, summary: "Builds the typed Minecraft data modification for merge." }
register_data_api! { path: "sand::data::NbtRef::merge_from", aliases: ["sand::cmd::NbtRef::merge_from", "sand::command::NbtRef::merge_from", "sand::prelude::NbtRef::merge_from", "sand::prelude::cmd::NbtRef::merge_from", "sand::state::NbtRef::merge_from"], kind: Method, summary: "Builds the typed Minecraft data modification for merge from." }
register_data_api! { path: "sand::data::NbtRef::new", aliases: ["sand::cmd::NbtRef::new", "sand::command::NbtRef::new", "sand::prelude::NbtRef::new", "sand::prelude::cmd::NbtRef::new", "sand::state::NbtRef::new"], kind: Method, summary: "Builds or resolves new in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtRef::path_value", aliases: ["sand::cmd::NbtRef::path_value", "sand::command::NbtRef::path_value", "sand::prelude::NbtRef::path_value", "sand::prelude::cmd::NbtRef::path_value", "sand::state::NbtRef::path_value"], kind: Method, summary: "Builds or resolves path value in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtRef::prepend", aliases: ["sand::cmd::NbtRef::prepend", "sand::command::NbtRef::prepend", "sand::prelude::NbtRef::prepend", "sand::prelude::cmd::NbtRef::prepend", "sand::state::NbtRef::prepend"], kind: Method, summary: "Builds the typed Minecraft data modification for prepend." }
register_data_api! { path: "sand::data::NbtRef::prepend_from", aliases: ["sand::cmd::NbtRef::prepend_from", "sand::command::NbtRef::prepend_from", "sand::prelude::NbtRef::prepend_from", "sand::prelude::cmd::NbtRef::prepend_from", "sand::state::NbtRef::prepend_from"], kind: Method, summary: "Builds the typed Minecraft data modification for prepend from." }
register_data_api! { path: "sand::data::NbtRef::remove", aliases: ["sand::cmd::NbtRef::remove", "sand::command::NbtRef::remove", "sand::prelude::NbtRef::remove", "sand::prelude::cmd::NbtRef::remove", "sand::state::NbtRef::remove"], kind: Method, summary: "Builds the typed Minecraft data modification for remove." }
register_data_api! { path: "sand::data::NbtRef::set", aliases: ["sand::cmd::NbtRef::set", "sand::command::NbtRef::set", "sand::prelude::NbtRef::set", "sand::prelude::cmd::NbtRef::set", "sand::state::NbtRef::set"], kind: Method, summary: "Builds the typed Minecraft data modification for set." }
register_data_api! { path: "sand::data::NbtRef::set_bool", aliases: ["sand::cmd::NbtRef::set_bool", "sand::command::NbtRef::set_bool", "sand::prelude::NbtRef::set_bool", "sand::prelude::cmd::NbtRef::set_bool", "sand::state::NbtRef::set_bool"], kind: Method, summary: "Builds the typed Minecraft data modification for set bool." }
register_data_api! { path: "sand::data::NbtRef::set_int", aliases: ["sand::cmd::NbtRef::set_int", "sand::command::NbtRef::set_int", "sand::prelude::NbtRef::set_int", "sand::prelude::cmd::NbtRef::set_int", "sand::state::NbtRef::set_int"], kind: Method, summary: "Builds the typed Minecraft data modification for set int." }
register_data_api! { path: "sand::data::NbtRef::set_raw", aliases: ["sand::cmd::NbtRef::set_raw", "sand::command::NbtRef::set_raw", "sand::prelude::NbtRef::set_raw", "sand::prelude::cmd::NbtRef::set_raw", "sand::state::NbtRef::set_raw"], kind: Method, summary: "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility." }
register_data_api! { path: "sand::data::NbtRef::set_string", aliases: ["sand::cmd::NbtRef::set_string", "sand::command::NbtRef::set_string", "sand::prelude::NbtRef::set_string", "sand::prelude::cmd::NbtRef::set_string", "sand::state::NbtRef::set_string"], kind: Method, summary: "Builds the typed Minecraft data modification for set string." }
register_data_api! { path: "sand::data::NbtRef::set_string_from", aliases: ["sand::cmd::NbtRef::set_string_from", "sand::command::NbtRef::set_string_from", "sand::prelude::NbtRef::set_string_from", "sand::prelude::cmd::NbtRef::set_string_from", "sand::state::NbtRef::set_string_from"], kind: Method, summary: "Builds the typed Minecraft data modification for set string from." }
register_data_api! { path: "sand::data::NbtRef::set_value", aliases: ["sand::cmd::NbtRef::set_value", "sand::command::NbtRef::set_value", "sand::prelude::NbtRef::set_value", "sand::prelude::cmd::NbtRef::set_value", "sand::state::NbtRef::set_value"], kind: Method, summary: "Builds the typed Minecraft data modification for set value." }
register_data_api! { path: "sand::data::NbtRef::storage", aliases: ["sand::cmd::NbtRef::storage", "sand::command::NbtRef::storage", "sand::prelude::NbtRef::storage", "sand::prelude::cmd::NbtRef::storage", "sand::state::NbtRef::storage"], kind: Method, summary: "Builds or resolves storage in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtRef::typed_field", aliases: ["sand::cmd::NbtRef::typed_field", "sand::command::NbtRef::typed_field", "sand::prelude::NbtRef::typed_field", "sand::prelude::cmd::NbtRef::typed_field", "sand::state::NbtRef::typed_field"], kind: Method, summary: "Builds or resolves typed field in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtTarget", aliases: ["sand::cmd::NbtTarget", "sand::command::NbtTarget", "sand::prelude::NbtTarget", "sand::prelude::cmd::NbtTarget", "sand::state::NbtTarget"], kind: Struct, summary: "Represents nbt target in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtTarget::location", aliases: ["sand::cmd::NbtTarget::location", "sand::command::NbtTarget::location", "sand::prelude::NbtTarget::location", "sand::prelude::cmd::NbtTarget::location", "sand::state::NbtTarget::location"], kind: Method, summary: "Builds or resolves location in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtTarget::merge", aliases: ["sand::cmd::NbtTarget::merge", "sand::command::NbtTarget::merge", "sand::prelude::NbtTarget::merge", "sand::prelude::cmd::NbtTarget::merge", "sand::state::NbtTarget::merge"], kind: Method, summary: "Builds the typed Minecraft data modification for merge." }
register_data_api! { path: "sand::data::NbtTarget::new", aliases: ["sand::cmd::NbtTarget::new", "sand::command::NbtTarget::new", "sand::prelude::NbtTarget::new", "sand::prelude::cmd::NbtTarget::new", "sand::state::NbtTarget::new"], kind: Method, summary: "Builds or resolves new in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtTarget::path", aliases: ["sand::cmd::NbtTarget::path", "sand::command::NbtTarget::path", "sand::prelude::NbtTarget::path", "sand::prelude::cmd::NbtTarget::path", "sand::state::NbtTarget::path"], kind: Method, summary: "Builds or resolves path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtTarget::typed_path", aliases: ["sand::cmd::NbtTarget::typed_path", "sand::command::NbtTarget::typed_path", "sand::prelude::NbtTarget::typed_path", "sand::prelude::cmd::NbtTarget::typed_path", "sand::state::NbtTarget::typed_path"], kind: Method, summary: "Builds or resolves typed path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtValue", aliases: ["sand::cmd::NbtValue", "sand::command::NbtValue", "sand::data::SnbtValue", "sand::prelude::SnbtValue", "sand::prelude::cmd::NbtValue", "sand::state::SnbtValue"], kind: Enum, summary: "Defines nbt value for typed Minecraft NBT and data commands." }
register_data_api! { path: "sand::data::NbtValue::Bool", aliases: ["sand::cmd::NbtValue::Bool", "sand::command::NbtValue::Bool", "sand::data::SnbtValue::Bool", "sand::prelude::SnbtValue::Bool", "sand::prelude::cmd::NbtValue::Bool", "sand::state::SnbtValue::Bool"], kind: Variant, summary: "Selects the bool NBT or data-command operation." }
register_data_api! { path: "sand::data::NbtValue::Bool::0", aliases: ["sand::cmd::NbtValue::Bool::0", "sand::command::NbtValue::Bool::0", "sand::data::SnbtValue::Bool::0", "sand::prelude::SnbtValue::Bool::0", "sand::prelude::cmd::NbtValue::Bool::0", "sand::state::SnbtValue::Bool::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::NbtValue::Byte", aliases: ["sand::cmd::NbtValue::Byte", "sand::command::NbtValue::Byte", "sand::data::SnbtValue::Byte", "sand::prelude::SnbtValue::Byte", "sand::prelude::cmd::NbtValue::Byte", "sand::state::SnbtValue::Byte"], kind: Variant, summary: "Selects the byte NBT or data-command operation." }
register_data_api! { path: "sand::data::NbtValue::Byte::0", aliases: ["sand::cmd::NbtValue::Byte::0", "sand::command::NbtValue::Byte::0", "sand::data::SnbtValue::Byte::0", "sand::prelude::SnbtValue::Byte::0", "sand::prelude::cmd::NbtValue::Byte::0", "sand::state::SnbtValue::Byte::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::NbtValue::Compound", aliases: ["sand::cmd::NbtValue::Compound", "sand::command::NbtValue::Compound", "sand::data::SnbtValue::Compound", "sand::prelude::SnbtValue::Compound", "sand::prelude::cmd::NbtValue::Compound", "sand::state::SnbtValue::Compound"], kind: Variant, summary: "Selects the compound NBT or data-command operation." }
register_data_api! { path: "sand::data::NbtValue::Compound::0", aliases: ["sand::cmd::NbtValue::Compound::0", "sand::command::NbtValue::Compound::0", "sand::data::SnbtValue::Compound::0", "sand::prelude::SnbtValue::Compound::0", "sand::prelude::cmd::NbtValue::Compound::0", "sand::state::SnbtValue::Compound::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::NbtValue::Double", aliases: ["sand::cmd::NbtValue::Double", "sand::command::NbtValue::Double", "sand::data::SnbtValue::Double", "sand::prelude::SnbtValue::Double", "sand::prelude::cmd::NbtValue::Double", "sand::state::SnbtValue::Double"], kind: Variant, summary: "Selects the double NBT or data-command operation." }
register_data_api! { path: "sand::data::NbtValue::Double::0", aliases: ["sand::cmd::NbtValue::Double::0", "sand::command::NbtValue::Double::0", "sand::data::SnbtValue::Double::0", "sand::prelude::SnbtValue::Double::0", "sand::prelude::cmd::NbtValue::Double::0", "sand::state::SnbtValue::Double::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::NbtValue::Float", aliases: ["sand::cmd::NbtValue::Float", "sand::command::NbtValue::Float", "sand::data::SnbtValue::Float", "sand::prelude::SnbtValue::Float", "sand::prelude::cmd::NbtValue::Float", "sand::state::SnbtValue::Float"], kind: Variant, summary: "Selects the float NBT or data-command operation." }
register_data_api! { path: "sand::data::NbtValue::Float::0", aliases: ["sand::cmd::NbtValue::Float::0", "sand::command::NbtValue::Float::0", "sand::data::SnbtValue::Float::0", "sand::prelude::SnbtValue::Float::0", "sand::prelude::cmd::NbtValue::Float::0", "sand::state::SnbtValue::Float::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::NbtValue::Int", aliases: ["sand::cmd::NbtValue::Int", "sand::command::NbtValue::Int", "sand::data::SnbtValue::Int", "sand::prelude::SnbtValue::Int", "sand::prelude::cmd::NbtValue::Int", "sand::state::SnbtValue::Int"], kind: Variant, summary: "Selects the int NBT or data-command operation." }
register_data_api! { path: "sand::data::NbtValue::Int::0", aliases: ["sand::cmd::NbtValue::Int::0", "sand::command::NbtValue::Int::0", "sand::data::SnbtValue::Int::0", "sand::prelude::SnbtValue::Int::0", "sand::prelude::cmd::NbtValue::Int::0", "sand::state::SnbtValue::Int::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::NbtValue::List", aliases: ["sand::cmd::NbtValue::List", "sand::command::NbtValue::List", "sand::data::SnbtValue::List", "sand::prelude::SnbtValue::List", "sand::prelude::cmd::NbtValue::List", "sand::state::SnbtValue::List"], kind: Variant, summary: "Selects the list NBT or data-command operation." }
register_data_api! { path: "sand::data::NbtValue::List::0", aliases: ["sand::cmd::NbtValue::List::0", "sand::command::NbtValue::List::0", "sand::data::SnbtValue::List::0", "sand::prelude::SnbtValue::List::0", "sand::prelude::cmd::NbtValue::List::0", "sand::state::SnbtValue::List::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::NbtValue::Long", aliases: ["sand::cmd::NbtValue::Long", "sand::command::NbtValue::Long", "sand::data::SnbtValue::Long", "sand::prelude::SnbtValue::Long", "sand::prelude::cmd::NbtValue::Long", "sand::state::SnbtValue::Long"], kind: Variant, summary: "Selects the long NBT or data-command operation." }
register_data_api! { path: "sand::data::NbtValue::Long::0", aliases: ["sand::cmd::NbtValue::Long::0", "sand::command::NbtValue::Long::0", "sand::data::SnbtValue::Long::0", "sand::prelude::SnbtValue::Long::0", "sand::prelude::cmd::NbtValue::Long::0", "sand::state::SnbtValue::Long::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::NbtValue::Raw", aliases: ["sand::cmd::NbtValue::Raw", "sand::command::NbtValue::Raw", "sand::data::SnbtValue::Raw", "sand::prelude::SnbtValue::Raw", "sand::prelude::cmd::NbtValue::Raw", "sand::state::SnbtValue::Raw"], kind: Variant, summary: "Selects the raw NBT or data-command operation." }
register_data_api! { path: "sand::data::NbtValue::Raw::0", aliases: ["sand::cmd::NbtValue::Raw::0", "sand::command::NbtValue::Raw::0", "sand::data::SnbtValue::Raw::0", "sand::prelude::SnbtValue::Raw::0", "sand::prelude::cmd::NbtValue::Raw::0", "sand::state::SnbtValue::Raw::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::NbtValue::Short", aliases: ["sand::cmd::NbtValue::Short", "sand::command::NbtValue::Short", "sand::data::SnbtValue::Short", "sand::prelude::SnbtValue::Short", "sand::prelude::cmd::NbtValue::Short", "sand::state::SnbtValue::Short"], kind: Variant, summary: "Selects the short NBT or data-command operation." }
register_data_api! { path: "sand::data::NbtValue::Short::0", aliases: ["sand::cmd::NbtValue::Short::0", "sand::command::NbtValue::Short::0", "sand::data::SnbtValue::Short::0", "sand::prelude::SnbtValue::Short::0", "sand::prelude::cmd::NbtValue::Short::0", "sand::state::SnbtValue::Short::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::NbtValue::String", aliases: ["sand::cmd::NbtValue::String", "sand::command::NbtValue::String", "sand::data::SnbtValue::String", "sand::prelude::SnbtValue::String", "sand::prelude::cmd::NbtValue::String", "sand::state::SnbtValue::String"], kind: Variant, summary: "Selects the string NBT or data-command operation." }
register_data_api! { path: "sand::data::NbtValue::String::0", aliases: ["sand::cmd::NbtValue::String::0", "sand::command::NbtValue::String::0", "sand::data::SnbtValue::String::0", "sand::prelude::SnbtValue::String::0", "sand::prelude::cmd::NbtValue::String::0", "sand::state::SnbtValue::String::0"], kind: Field, summary: "Carries the 0 payload for this typed NBT operation." }
register_data_api! { path: "sand::data::NbtValue::compound", aliases: ["sand::cmd::NbtValue::compound", "sand::command::NbtValue::compound", "sand::data::SnbtValue::compound", "sand::prelude::SnbtValue::compound", "sand::prelude::cmd::NbtValue::compound", "sand::state::SnbtValue::compound"], kind: Method, summary: "Builds or resolves compound in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtValue::list", aliases: ["sand::cmd::NbtValue::list", "sand::command::NbtValue::list", "sand::data::SnbtValue::list", "sand::prelude::SnbtValue::list", "sand::prelude::cmd::NbtValue::list", "sand::state::SnbtValue::list"], kind: Method, summary: "Builds or resolves list in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::NbtValue::raw", aliases: ["sand::cmd::NbtValue::raw", "sand::command::NbtValue::raw", "sand::data::SnbtValue::raw", "sand::prelude::SnbtValue::raw", "sand::prelude::cmd::NbtValue::raw", "sand::state::SnbtValue::raw"], kind: Method, summary: "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility." }
register_data_api! { path: "sand::data::StorageField", aliases: ["sand::prelude::StorageField", "sand::state::StorageField"], kind: Struct, summary: "Represents storage field in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageField::append", aliases: ["sand::prelude::StorageField::append", "sand::state::StorageField::append"], kind: Method, summary: "Builds the typed Minecraft data modification for append." }
register_data_api! { path: "sand::data::StorageField::copy_from", aliases: ["sand::prelude::StorageField::copy_from", "sand::state::StorageField::copy_from"], kind: Method, summary: "Builds the typed Minecraft data modification for copy from." }
register_data_api! { path: "sand::data::StorageField::copy_from_entity", aliases: ["sand::prelude::StorageField::copy_from_entity", "sand::state::StorageField::copy_from_entity"], kind: Method, summary: "Builds the typed Minecraft data modification for copy from entity." }
register_data_api! { path: "sand::data::StorageField::copy_from_path", aliases: ["sand::prelude::StorageField::copy_from_path", "sand::state::StorageField::copy_from_path"], kind: Method, summary: "Builds the typed Minecraft data modification for copy from path." }
register_data_api! { path: "sand::data::StorageField::exists", aliases: ["sand::prelude::StorageField::exists", "sand::state::StorageField::exists"], kind: Method, summary: "Builds the typed Minecraft data query for exists." }
register_data_api! { path: "sand::data::StorageField::field_name", aliases: ["sand::prelude::StorageField::field_name", "sand::state::StorageField::field_name"], kind: Method, summary: "Builds or resolves field name in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageField::field_path", aliases: ["sand::prelude::StorageField::field_path", "sand::state::StorageField::field_path"], kind: Method, summary: "Builds or resolves field path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageField::full_path", aliases: ["sand::prelude::StorageField::full_path", "sand::state::StorageField::full_path"], kind: Method, summary: "Builds or resolves full path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageField::get", aliases: ["sand::prelude::StorageField::get", "sand::state::StorageField::get"], kind: Method, summary: "Builds the typed Minecraft data query for get." }
register_data_api! { path: "sand::data::StorageField::get_scaled", aliases: ["sand::prelude::StorageField::get_scaled", "sand::state::StorageField::get_scaled"], kind: Method, summary: "Builds the typed Minecraft data query for get scaled." }
register_data_api! { path: "sand::data::StorageField::location", aliases: ["sand::prelude::StorageField::location", "sand::state::StorageField::location"], kind: Method, summary: "Builds or resolves location in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageField::merge", aliases: ["sand::prelude::StorageField::merge", "sand::state::StorageField::merge"], kind: Method, summary: "Builds the typed Minecraft data modification for merge." }
register_data_api! { path: "sand::data::StorageField::new", aliases: ["sand::prelude::StorageField::new", "sand::state::StorageField::new"], kind: Method, summary: "Builds or resolves new in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageField::path", aliases: ["sand::prelude::StorageField::path", "sand::state::StorageField::path"], kind: Method, summary: "Builds or resolves path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageField::remove", aliases: ["sand::prelude::StorageField::remove", "sand::state::StorageField::remove"], kind: Method, summary: "Builds the typed Minecraft data modification for remove." }
register_data_api! { path: "sand::data::StorageField::root_path", aliases: ["sand::prelude::StorageField::root_path", "sand::state::StorageField::root_path"], kind: Method, summary: "Builds or resolves root path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageField::set", aliases: ["sand::prelude::StorageField::set", "sand::state::StorageField::set"], kind: Method, summary: "Builds the typed Minecraft data modification for set." }
register_data_api! { path: "sand::data::StorageField::set_raw_snbt", aliases: ["sand::prelude::StorageField::set_raw_snbt", "sand::state::StorageField::set_raw_snbt"], kind: Method, summary: "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility." }
register_data_api! { path: "sand::data::StorageField::set_value", aliases: ["sand::prelude::StorageField::set_value", "sand::state::StorageField::set_value"], kind: Method, summary: "Builds the typed Minecraft data modification for set value." }
register_data_api! { path: "sand::data::StorageField::storage", aliases: ["sand::prelude::StorageField::storage", "sand::state::StorageField::storage"], kind: Method, summary: "Builds or resolves storage in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageLocation", aliases: ["sand::prelude::StorageLocation", "sand::state::StorageLocation"], kind: Struct, summary: "Represents storage location in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageLocation::as_resource_location", aliases: ["sand::prelude::StorageLocation::as_resource_location", "sand::state::StorageLocation::as_resource_location"], kind: Method, summary: "Builds or resolves as resource location in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageLocation::new", aliases: ["sand::prelude::StorageLocation::new", "sand::state::StorageLocation::new"], kind: Method, summary: "Builds or resolves new in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageLocation::parse", aliases: ["sand::prelude::StorageLocation::parse", "sand::state::StorageLocation::parse"], kind: Method, summary: "Builds or resolves parse in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageSchema", aliases: ["sand::prelude::StorageSchema", "sand::state::StorageSchema"], kind: Struct, summary: "Represents storage schema in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageSchema::exists", aliases: ["sand::prelude::StorageSchema::exists", "sand::state::StorageSchema::exists"], kind: Method, summary: "Builds the typed Minecraft data query for exists." }
register_data_api! { path: "sand::data::StorageSchema::field", aliases: ["sand::prelude::StorageSchema::field", "sand::state::StorageSchema::field"], kind: Method, summary: "Builds or resolves field in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageSchema::get", aliases: ["sand::prelude::StorageSchema::get", "sand::state::StorageSchema::get"], kind: Method, summary: "Builds the typed Minecraft data query for get." }
register_data_api! { path: "sand::data::StorageSchema::location", aliases: ["sand::prelude::StorageSchema::location", "sand::state::StorageSchema::location"], kind: Method, summary: "Builds or resolves location in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageSchema::merge", aliases: ["sand::prelude::StorageSchema::merge", "sand::state::StorageSchema::merge"], kind: Method, summary: "Builds the typed Minecraft data modification for merge." }
register_data_api! { path: "sand::data::StorageSchema::new", aliases: ["sand::prelude::StorageSchema::new", "sand::state::StorageSchema::new"], kind: Method, summary: "Builds or resolves new in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageSchema::path", aliases: ["sand::prelude::StorageSchema::path", "sand::state::StorageSchema::path"], kind: Method, summary: "Builds or resolves path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageSchema::remove", aliases: ["sand::prelude::StorageSchema::remove", "sand::state::StorageSchema::remove"], kind: Method, summary: "Builds the typed Minecraft data modification for remove." }
register_data_api! { path: "sand::data::StorageSchema::root_path", aliases: ["sand::prelude::StorageSchema::root_path", "sand::state::StorageSchema::root_path"], kind: Method, summary: "Builds or resolves root path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageSchema::set", aliases: ["sand::prelude::StorageSchema::set", "sand::state::StorageSchema::set"], kind: Method, summary: "Builds the typed Minecraft data modification for set." }
register_data_api! { path: "sand::data::StorageSchema::set_raw_snbt", aliases: ["sand::prelude::StorageSchema::set_raw_snbt", "sand::state::StorageSchema::set_raw_snbt"], kind: Method, summary: "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility." }
register_data_api! { path: "sand::data::StorageSchema::storage", aliases: ["sand::prelude::StorageSchema::storage", "sand::state::StorageSchema::storage"], kind: Method, summary: "Builds or resolves storage in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageVar", aliases: ["sand::prelude::StorageVar", "sand::state::StorageVar"], kind: Struct, summary: "Represents storage var in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageVar::as_path", aliases: ["sand::prelude::StorageVar::as_path", "sand::state::StorageVar::as_path"], kind: Method, summary: "Builds or resolves as path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageVar::copy_from", aliases: ["sand::prelude::StorageVar::copy_from", "sand::state::StorageVar::copy_from"], kind: Method, summary: "Builds the typed Minecraft data modification for copy from." }
register_data_api! { path: "sand::data::StorageVar::exists", aliases: ["sand::prelude::StorageVar::exists", "sand::state::StorageVar::exists"], kind: Method, summary: "Builds the typed Minecraft data query for exists." }
register_data_api! { path: "sand::data::StorageVar::get", aliases: ["sand::prelude::StorageVar::get", "sand::state::StorageVar::get"], kind: Method, summary: "Builds the typed Minecraft data query for get." }
register_data_api! { path: "sand::data::StorageVar::get_scaled", aliases: ["sand::prelude::StorageVar::get_scaled", "sand::state::StorageVar::get_scaled"], kind: Method, summary: "Builds the typed Minecraft data query for get scaled." }
register_data_api! { path: "sand::data::StorageVar::new", aliases: ["sand::prelude::StorageVar::new", "sand::state::StorageVar::new"], kind: Method, summary: "Builds or resolves new in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageVar::path", aliases: ["sand::prelude::StorageVar::path", "sand::state::StorageVar::path"], kind: Method, summary: "Builds or resolves path in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::StorageVar::remove", aliases: ["sand::prelude::StorageVar::remove", "sand::state::StorageVar::remove"], kind: Method, summary: "Builds the typed Minecraft data modification for remove." }
register_data_api! { path: "sand::data::StorageVar::set_bool", aliases: ["sand::prelude::StorageVar::set_bool", "sand::state::StorageVar::set_bool"], kind: Method, summary: "Builds the typed Minecraft data modification for set bool." }
register_data_api! { path: "sand::data::StorageVar::set_double", aliases: ["sand::prelude::StorageVar::set_double", "sand::state::StorageVar::set_double"], kind: Method, summary: "Builds the typed Minecraft data modification for set double." }
register_data_api! { path: "sand::data::StorageVar::set_float", aliases: ["sand::prelude::StorageVar::set_float", "sand::state::StorageVar::set_float"], kind: Method, summary: "Builds the typed Minecraft data modification for set float." }
register_data_api! { path: "sand::data::StorageVar::set_int", aliases: ["sand::prelude::StorageVar::set_int", "sand::state::StorageVar::set_int"], kind: Method, summary: "Builds the typed Minecraft data modification for set int." }
register_data_api! { path: "sand::data::StorageVar::set_long", aliases: ["sand::prelude::StorageVar::set_long", "sand::state::StorageVar::set_long"], kind: Method, summary: "Builds the typed Minecraft data modification for set long." }
register_data_api! { path: "sand::data::StorageVar::set_raw_snbt", aliases: ["sand::prelude::StorageVar::set_raw_snbt", "sand::state::StorageVar::set_raw_snbt"], kind: Method, summary: "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility." }
register_data_api! { path: "sand::data::StorageVar::set_string", aliases: ["sand::prelude::StorageVar::set_string", "sand::state::StorageVar::set_string"], kind: Method, summary: "Builds the typed Minecraft data modification for set string." }
register_data_api! { path: "sand::data::StorageVar::set_value", aliases: ["sand::prelude::StorageVar::set_value", "sand::state::StorageVar::set_value"], kind: Method, summary: "Builds the typed Minecraft data modification for set value." }
register_data_api! { path: "sand::data::StorageVar::storage", aliases: ["sand::prelude::StorageVar::storage", "sand::state::StorageVar::storage"], kind: Method, summary: "Builds or resolves storage in the typed NBT and command-storage model." }
register_data_api! { path: "sand::data::UntypedNbt", aliases: ["sand::cmd::UntypedNbt", "sand::command::UntypedNbt", "sand::prelude::UntypedNbt", "sand::prelude::cmd::UntypedNbt", "sand::state::UntypedNbt"], kind: Struct, summary: "Represents untyped nbt in the typed NBT and command-storage model." }
// END DATA API CONTRACTS
register_systems_api! { path: "sand::systems::cooldowns", aliases: [], kind: Module, summary: "Provides the feature-gated cooldowns gameplay system." }
register_systems_api! { path: "sand::systems::cooldowns::register_cooldown", aliases: [], kind: Function, summary: "Registers a typed cooldown for automatic per-tick decrementing." }
register_systems_api! { path: "sand::systems::damage", aliases: [], kind: Module, summary: "Provides the feature-gated damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageThreshold", aliases: ["sand::prelude::DamageThreshold"], kind: Enum, summary: "Defines typed damage threshold values for damage tracking." }
register_systems_api! { path: "sand::systems::damage::DamageThreshold::Hearts", aliases: ["sand::prelude::DamageThreshold::Hearts"], kind: Variant, summary: "Selects the hearts damage threshold representation." }
register_systems_api! { path: "sand::systems::damage::DamageThreshold::Hearts::0", aliases: ["sand::prelude::DamageThreshold::Hearts::0"], kind: Field, summary: "Configures the 0 value used by this gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageThreshold::RawStat", aliases: ["sand::prelude::DamageThreshold::RawStat"], kind: Variant, summary: "Selects the raw stat damage threshold representation." }
register_systems_api! { path: "sand::systems::damage::DamageThreshold::RawStat::0", aliases: ["sand::prelude::DamageThreshold::RawStat::0"], kind: Field, summary: "Configures the 0 value used by this gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageThreshold::hearts", aliases: ["sand::prelude::DamageThreshold::hearts"], kind: Method, summary: "Configures or performs hearts for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageThreshold::raw_stat", aliases: ["sand::prelude::DamageThreshold::raw_stat"], kind: Method, summary: "Configures or performs raw stat for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageThreshold::to_raw_stat", aliases: ["sand::prelude::DamageThreshold::to_raw_stat"], kind: Method, summary: "Configures or performs to raw stat for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageThreshold::try_hearts", aliases: ["sand::prelude::DamageThreshold::try_hearts"], kind: Method, summary: "Configures or performs try hearts for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageThreshold::try_raw_stat", aliases: ["sand::prelude::DamageThreshold::try_raw_stat"], kind: Method, summary: "Configures or performs try raw stat for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageTracker", aliases: ["sand::prelude::DamageTracker"], kind: Struct, summary: "Configures damage tracker in the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::clear_recent_damage", aliases: ["sand::prelude::DamageTracker::clear_recent_damage"], kind: Method, summary: "Configures or performs clear recent damage for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::current_damage_at_least", aliases: ["sand::prelude::DamageTracker::current_damage_at_least"], kind: Method, summary: "Configures or performs current damage at least for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::current_damage_raw", aliases: ["sand::prelude::DamageTracker::current_damage_raw"], kind: Method, summary: "Configures or performs current damage raw for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::damaged_this_tick", aliases: ["sand::prelude::DamageTracker::damaged_this_tick"], kind: Method, summary: "Builds the typed condition for damaged this tick." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::define", aliases: ["sand::prelude::DamageTracker::define"], kind: Method, summary: "Configures or performs define for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::hurt_within", aliases: ["sand::prelude::DamageTracker::hurt_within"], kind: Method, summary: "Builds the typed condition for hurt within." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::last_damage_at_least", aliases: ["sand::prelude::DamageTracker::last_damage_at_least"], kind: Method, summary: "Configures or performs last damage at least for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::last_damage_raw", aliases: ["sand::prelude::DamageTracker::last_damage_raw"], kind: Method, summary: "Configures or performs last damage raw for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::not_damaged_this_tick", aliases: ["sand::prelude::DamageTracker::not_damaged_this_tick"], kind: Method, summary: "Builds the typed condition for not damaged this tick." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::not_hurt_for", aliases: ["sand::prelude::DamageTracker::not_hurt_for"], kind: Method, summary: "Builds the typed condition for not hurt for." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::tick", aliases: ["sand::prelude::DamageTracker::tick"], kind: Method, summary: "Configures or performs tick for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::tick_players", aliases: ["sand::prelude::DamageTracker::tick_players"], kind: Method, summary: "Configures or performs tick players for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::tick_raw", aliases: ["sand::prelude::DamageTracker::tick_raw"], kind: Method, summary: "Configures or performs tick raw for the damage gameplay system." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::ticks_since_hurt", aliases: ["sand::prelude::DamageTracker::ticks_since_hurt"], kind: Method, summary: "Builds the typed condition for ticks since hurt." }
register_systems_api! { path: "sand::systems::damage::DamageTracker::was_hurt", aliases: ["sand::prelude::DamageTracker::was_hurt"], kind: Method, summary: "Builds the typed condition for was hurt." }
register_systems_api! { path: "sand::systems::damage::recently_damaged", aliases: ["sand::prelude::recently_damaged"], kind: Function, summary: "Builds the condition that detects a recently damaged entity." }
register_systems_api! { path: "sand::systems::entities", aliases: [], kind: Module, summary: "Provides the feature-gated entities gameplay system." }
register_systems_api! { path: "sand::systems::entities::InteractSize", aliases: [], kind: Struct, summary: "Configures interact size in the entities gameplay system." }
register_systems_api! { path: "sand::systems::entities::InteractSize::height", aliases: [], kind: Field, summary: "Configures the height value used by this gameplay system." }
register_systems_api! { path: "sand::systems::entities::InteractSize::width", aliases: [], kind: Field, summary: "Configures the width value used by this gameplay system." }
register_systems_api! { path: "sand::systems::entities::Interactable", aliases: [], kind: Struct, summary: "Configures interactable in the entities gameplay system." }
register_systems_api! { path: "sand::systems::entities::Interactable::advancement", aliases: [], kind: Method, summary: "Configures or performs advancement for the entities gameplay system." }
register_systems_api! { path: "sand::systems::entities::Interactable::advancement_with", aliases: [], kind: Method, summary: "Configures or performs advancement with for the entities gameplay system." }
register_systems_api! { path: "sand::systems::entities::Interactable::new", aliases: [], kind: Method, summary: "Configures or performs new for the entities gameplay system." }
register_systems_api! { path: "sand::systems::entities::Interactable::response", aliases: [], kind: Method, summary: "Configures or performs response for the entities gameplay system." }
register_systems_api! { path: "sand::systems::entities::Interactable::size", aliases: [], kind: Method, summary: "Configures or performs size for the entities gameplay system." }
register_systems_api! { path: "sand::systems::entities::Interactable::summon_at", aliases: [], kind: Method, summary: "Builds commands that summon the configured interaction entity for at." }
register_systems_api! { path: "sand::systems::entities::Interactable::summon_here", aliases: [], kind: Method, summary: "Builds commands that summon the configured interaction entity for here." }
register_systems_api! { path: "sand::systems::entities::Interactable::tag", aliases: [], kind: Method, summary: "Configures or performs tag for the entities gameplay system." }
register_systems_api! { path: "sand::systems::inventory", aliases: [], kind: Module, summary: "Provides the feature-gated inventory gameplay system." }
register_systems_api! { path: "sand::systems::inventory::ClearBuilder", aliases: [], kind: Struct, summary: "Configures clear builder in the inventory gameplay system." }
register_systems_api! { path: "sand::systems::inventory::ClearBuilder::amount", aliases: [], kind: Method, summary: "Configures or performs amount for the inventory gameplay system." }
register_systems_api! { path: "sand::systems::inventory::HasItemCheck", aliases: [], kind: Struct, summary: "Configures has item check in the inventory gameplay system." }
register_systems_api! { path: "sand::systems::inventory::HasItemCheck::in_any_slot", aliases: [], kind: Method, summary: "Builds the typed condition for in any slot." }
register_systems_api! { path: "sand::systems::inventory::HasItemCheck::in_any_weapon", aliases: [], kind: Method, summary: "Builds the typed condition for in any weapon." }
register_systems_api! { path: "sand::systems::inventory::HasItemCheck::in_armor", aliases: [], kind: Method, summary: "Builds the typed condition for in armor." }
register_systems_api! { path: "sand::systems::inventory::HasItemCheck::in_hotbar", aliases: [], kind: Method, summary: "Builds the typed condition for in hotbar." }
register_systems_api! { path: "sand::systems::inventory::HasItemCheck::in_inventory", aliases: [], kind: Method, summary: "Builds the typed condition for in inventory." }
register_systems_api! { path: "sand::systems::inventory::HasItemCheck::in_mainhand", aliases: [], kind: Method, summary: "Builds the typed condition for in mainhand." }
register_systems_api! { path: "sand::systems::inventory::HasItemCheck::in_offhand", aliases: [], kind: Method, summary: "Builds the typed condition for in offhand." }
register_systems_api! { path: "sand::systems::inventory::HasItemCheck::in_slot", aliases: [], kind: Method, summary: "Builds the typed condition for in slot." }
register_systems_api! { path: "sand::systems::inventory::HasItemCheck::not_anywhere", aliases: [], kind: Method, summary: "Builds the typed condition for not anywhere." }
register_systems_api! { path: "sand::systems::inventory::HasItemCheck::not_in_slot", aliases: [], kind: Method, summary: "Builds the typed condition for not in slot." }
register_systems_api! { path: "sand::systems::inventory::InventorySystem", aliases: [], kind: Struct, summary: "Configures inventory system in the inventory gameplay system." }
register_systems_api! { path: "sand::systems::inventory::InventorySystem::clear_item", aliases: [], kind: Method, summary: "Configures or performs clear item for the inventory gameplay system." }
register_systems_api! { path: "sand::systems::inventory::InventorySystem::clear_slot", aliases: [], kind: Method, summary: "Configures or performs clear slot for the inventory gameplay system." }
register_systems_api! { path: "sand::systems::inventory::InventorySystem::for_entity", aliases: [], kind: Method, summary: "Configures or performs for entity for the inventory gameplay system." }
register_systems_api! { path: "sand::systems::inventory::InventorySystem::give", aliases: [], kind: Method, summary: "Configures or performs give for the inventory gameplay system." }
register_systems_api! { path: "sand::systems::inventory::InventorySystem::has", aliases: [], kind: Method, summary: "Configures or performs has for the inventory gameplay system." }
register_systems_api! { path: "sand::systems::inventory::InventorySystem::has_in", aliases: [], kind: Method, summary: "Configures or performs has in for the inventory gameplay system." }
register_systems_api! { path: "sand::systems::inventory::InventorySystem::replace", aliases: [], kind: Method, summary: "Configures or performs replace for the inventory gameplay system." }
register_systems_api! { path: "sand::systems::inventory::InventorySystem::replace_count", aliases: [], kind: Method, summary: "Configures or performs replace count for the inventory gameplay system." }
register_systems_api! { path: "sand::systems::movement", aliases: [], kind: Module, summary: "Provides the feature-gated movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Launch", aliases: [], kind: Struct, summary: "Configures launch in the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Launch::amount", aliases: [], kind: Method, summary: "Configures or performs amount for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Launch::build", aliases: [], kind: Method, summary: "Builds the Minecraft commands for this configured gameplay effect." }
register_systems_api! { path: "sand::systems::movement::Launch::new", aliases: [], kind: Method, summary: "Configures or performs new for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Launch::targets", aliases: [], kind: Method, summary: "Configures or performs targets for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Launch::with_targets", aliases: [], kind: Method, summary: "Configures or performs with targets for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::PushAway", aliases: [], kind: Struct, summary: "Configures push away in the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::PushAway::build", aliases: [], kind: Method, summary: "Builds the Minecraft commands for this configured gameplay effect." }
register_systems_api! { path: "sand::systems::movement::PushAway::lift", aliases: [], kind: Method, summary: "Configures or performs lift for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::PushAway::new", aliases: [], kind: Method, summary: "Configures or performs new for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::PushAway::source", aliases: [], kind: Method, summary: "Configures or performs source for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::PushAway::strength", aliases: [], kind: Method, summary: "Configures or performs strength for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::PushAway::targets", aliases: [], kind: Method, summary: "Configures or performs targets for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Slow", aliases: [], kind: Struct, summary: "Configures slow in the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Slow::amount", aliases: [], kind: Method, summary: "Configures or performs amount for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Slow::amplifier", aliases: [], kind: Method, summary: "Configures or performs amplifier for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Slow::build", aliases: [], kind: Method, summary: "Builds the Minecraft commands for this configured gameplay effect." }
register_systems_api! { path: "sand::systems::movement::Slow::duration", aliases: [], kind: Method, summary: "Configures or performs duration for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Slow::new", aliases: [], kind: Method, summary: "Configures or performs new for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Slow::target", aliases: [], kind: Method, summary: "Configures or performs target for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Slow::targets", aliases: [], kind: Method, summary: "Configures or performs targets for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::Slow::with_target", aliases: [], kind: Method, summary: "Configures or performs with target for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::SpeedBoost", aliases: [], kind: Struct, summary: "Configures speed boost in the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::SpeedBoost::amount", aliases: [], kind: Method, summary: "Configures or performs amount for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::SpeedBoost::amplifier", aliases: [], kind: Method, summary: "Configures or performs amplifier for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::SpeedBoost::build", aliases: [], kind: Method, summary: "Builds the Minecraft commands for this configured gameplay effect." }
register_systems_api! { path: "sand::systems::movement::SpeedBoost::duration", aliases: [], kind: Method, summary: "Configures or performs duration for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::SpeedBoost::new", aliases: [], kind: Method, summary: "Configures or performs new for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::SpeedBoost::target", aliases: [], kind: Method, summary: "Configures or performs target for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::SpeedBoost::target_many", aliases: [], kind: Method, summary: "Configures or performs target many for the movement gameplay system." }
register_systems_api! { path: "sand::systems::movement::SpeedBoost::with_target", aliases: [], kind: Method, summary: "Configures or performs with target for the movement gameplay system." }
register_systems_api! { path: "sand::systems::player_data", aliases: [], kind: Module, summary: "Provides the feature-gated player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::CooldownField", aliases: ["sand::prelude::CooldownField"], kind: Struct, summary: "Configures cooldown field in the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::CooldownField::new", aliases: ["sand::prelude::CooldownField::new"], kind: Method, summary: "Configures or performs new for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::CooldownField::of", aliases: ["sand::prelude::CooldownField::of"], kind: Method, summary: "Configures or performs of for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::CooldownField::value", aliases: ["sand::prelude::CooldownField::value"], kind: Method, summary: "Configures or performs value for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::CooldownFieldRef", aliases: ["sand::prelude::CooldownFieldRef"], kind: Struct, summary: "Configures cooldown field ref in the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::CooldownFieldRef::active", aliases: ["sand::prelude::CooldownFieldRef::active"], kind: Method, summary: "Builds the typed condition for active." }
register_systems_api! { path: "sand::systems::player_data::CooldownFieldRef::ready", aliases: ["sand::prelude::CooldownFieldRef::ready"], kind: Method, summary: "Builds the typed condition for ready." }
register_systems_api! { path: "sand::systems::player_data::CooldownFieldRef::start", aliases: ["sand::prelude::CooldownFieldRef::start"], kind: Method, summary: "Configures or performs start for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::CooldownFieldRef::stop", aliases: ["sand::prelude::CooldownFieldRef::stop"], kind: Method, summary: "Configures or performs stop for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::FlagField", aliases: ["sand::prelude::FlagField"], kind: Struct, summary: "Configures flag field in the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::FlagField::default", aliases: ["sand::prelude::FlagField::default"], kind: Method, summary: "Configures or performs default for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::FlagField::default_value", aliases: ["sand::prelude::FlagField::default_value"], kind: Method, summary: "Configures or performs default value for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::FlagField::new", aliases: ["sand::prelude::FlagField::new"], kind: Method, summary: "Configures or performs new for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::FlagField::of", aliases: ["sand::prelude::FlagField::of"], kind: Method, summary: "Configures or performs of for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::FlagField::value", aliases: ["sand::prelude::FlagField::value"], kind: Method, summary: "Configures or performs value for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::GameStateField", aliases: ["sand::prelude::GameStateField"], kind: Struct, summary: "Configures game state field in the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::GameStateField::new", aliases: ["sand::prelude::GameStateField::new"], kind: Method, summary: "Configures or performs new for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::GameStateField::of", aliases: ["sand::prelude::GameStateField::of"], kind: Method, summary: "Configures or performs of for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::GameStateField::value", aliases: ["sand::prelude::GameStateField::value"], kind: Method, summary: "Configures or performs value for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::GameStateField::with_default_score", aliases: ["sand::prelude::GameStateField::with_default_score"], kind: Method, summary: "Configures or performs with default score for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::GlobalStorageField", aliases: ["sand::prelude::GlobalStorageField"], kind: Struct, summary: "Configures global storage field in the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::GlobalStorageField::nbt", aliases: ["sand::prelude::GlobalStorageField::nbt"], kind: Method, summary: "Configures or performs nbt for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::GlobalStorageField::new", aliases: ["sand::prelude::GlobalStorageField::new"], kind: Method, summary: "Configures or performs new for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::GlobalStorageField::value", aliases: ["sand::prelude::GlobalStorageField::value"], kind: Method, summary: "Configures or performs value for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema", aliases: ["sand::prelude::PlayerDataSchema"], kind: Struct, summary: "Configures player data schema in the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::cooldown", aliases: ["sand::prelude::PlayerDataSchema::cooldown", "sand::prelude::PlayerSchema::cooldown"], kind: Method, summary: "Configures or performs cooldown for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::cooldown_field", aliases: ["sand::prelude::PlayerDataSchema::cooldown_field", "sand::prelude::PlayerSchema::cooldown_field"], kind: Method, summary: "Configures or performs cooldown field for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::define_all", aliases: ["sand::prelude::PlayerDataSchema::define_all", "sand::prelude::PlayerSchema::define_all"], kind: Method, summary: "Configures or performs define all for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::flag", aliases: ["sand::prelude::PlayerDataSchema::flag", "sand::prelude::PlayerSchema::flag"], kind: Method, summary: "Configures or performs flag for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::flag_field", aliases: ["sand::prelude::PlayerDataSchema::flag_field", "sand::prelude::PlayerSchema::flag_field"], kind: Method, summary: "Configures or performs flag field for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::game_state", aliases: ["sand::prelude::PlayerDataSchema::game_state", "sand::prelude::PlayerSchema::game_state"], kind: Method, summary: "Configures or performs game state for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::global_storage_field", aliases: ["sand::prelude::PlayerDataSchema::global_storage_field", "sand::prelude::PlayerSchema::global_storage_field"], kind: Method, summary: "Configures or performs global storage field for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::has_storage", aliases: ["sand::prelude::PlayerDataSchema::has_storage", "sand::prelude::PlayerSchema::has_storage"], kind: Method, summary: "Configures or performs has storage for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::init_player", aliases: ["sand::prelude::PlayerDataSchema::init_player", "sand::prelude::PlayerSchema::init_player"], kind: Method, summary: "Configures or performs init player for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::name", aliases: ["sand::prelude::PlayerDataSchema::name", "sand::prelude::PlayerSchema::name"], kind: Method, summary: "Configures or performs name for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::new", aliases: ["sand::prelude::PlayerDataSchema::new", "sand::prelude::PlayerSchema::new"], kind: Method, summary: "Configures or performs new for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::score", aliases: ["sand::prelude::PlayerDataSchema::score", "sand::prelude::PlayerSchema::score"], kind: Method, summary: "Configures or performs score for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::score_field", aliases: ["sand::prelude::PlayerDataSchema::score_field", "sand::prelude::PlayerSchema::score_field"], kind: Method, summary: "Configures or performs score field for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::scoreboard_field_count", aliases: ["sand::prelude::PlayerDataSchema::scoreboard_field_count", "sand::prelude::PlayerSchema::scoreboard_field_count"], kind: Method, summary: "Configures or performs scoreboard field count for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::storage", aliases: ["sand::prelude::PlayerDataSchema::storage", "sand::prelude::PlayerSchema::storage"], kind: Method, summary: "Configures or performs storage for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::storage_locations", aliases: ["sand::prelude::PlayerDataSchema::storage_locations", "sand::prelude::PlayerSchema::storage_locations"], kind: Method, summary: "Configures or performs storage locations for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::timer", aliases: ["sand::prelude::PlayerDataSchema::timer", "sand::prelude::PlayerSchema::timer"], kind: Method, summary: "Configures or performs timer for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::timer_field", aliases: ["sand::prelude::PlayerDataSchema::timer_field", "sand::prelude::PlayerSchema::timer_field"], kind: Method, summary: "Configures or performs timer field for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::PlayerDataSchema::try_init_player", aliases: ["sand::prelude::PlayerDataSchema::try_init_player", "sand::prelude::PlayerSchema::try_init_player"], kind: Method, summary: "Configures or performs try init player for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::ScoreField", aliases: ["sand::prelude::ScoreField"], kind: Struct, summary: "Configures score field in the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::ScoreField::default", aliases: ["sand::prelude::ScoreField::default"], kind: Method, summary: "Configures or performs default for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::ScoreField::default_value", aliases: ["sand::prelude::ScoreField::default_value"], kind: Method, summary: "Configures or performs default value for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::ScoreField::new", aliases: ["sand::prelude::ScoreField::new"], kind: Method, summary: "Configures or performs new for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::ScoreField::of", aliases: ["sand::prelude::ScoreField::of"], kind: Method, summary: "Configures or performs of for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::ScoreField::value", aliases: ["sand::prelude::ScoreField::value"], kind: Method, summary: "Configures or performs value for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::TimerField", aliases: ["sand::prelude::TimerField"], kind: Struct, summary: "Configures timer field in the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::TimerField::new", aliases: ["sand::prelude::TimerField::new"], kind: Method, summary: "Configures or performs new for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::TimerField::of", aliases: ["sand::prelude::TimerField::of"], kind: Method, summary: "Configures or performs of for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::TimerField::value", aliases: ["sand::prelude::TimerField::value"], kind: Method, summary: "Configures or performs value for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::TimerFieldRef", aliases: ["sand::prelude::TimerFieldRef"], kind: Struct, summary: "Configures timer field ref in the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::TimerFieldRef::active", aliases: ["sand::prelude::TimerFieldRef::active"], kind: Method, summary: "Builds the typed condition for active." }
register_systems_api! { path: "sand::systems::player_data::TimerFieldRef::expired", aliases: ["sand::prelude::TimerFieldRef::expired"], kind: Method, summary: "Builds the typed condition for expired." }
register_systems_api! { path: "sand::systems::player_data::TimerFieldRef::reset", aliases: ["sand::prelude::TimerFieldRef::reset"], kind: Method, summary: "Configures or performs reset for the player data gameplay system." }
register_systems_api! { path: "sand::systems::player_data::TimerFieldRef::start", aliases: ["sand::prelude::TimerFieldRef::start"], kind: Method, summary: "Configures or performs start for the player data gameplay system." }
// END SYSTEMS API CONTRACTS
