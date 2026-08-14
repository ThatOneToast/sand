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
