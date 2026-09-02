//! Runtime event handle API for enabling/disabling/resetting events.

use crate::condition::{Condition, ScoreRange};
use crate::function::SAND_LOCAL_NS;
use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::OnceLock;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::handle::EventHandle",
    aliases = ["sand::prelude::EventHandle"],
    module = "sand::event",
    summary = "Runtime handle for enabling, disabling, and resetting an advancement-backed event.",
    context = "Runtime handle for enabling, disabling, and resetting an advancement-backed event. The generic parameter `E` is a marker that binds the handle to a specific event type. No trait bound is required — the objective name is derived lazily from `E`'s fully-qualified Rust type name via [`std::any::type_name`]. The scoreboard objective is `__ev_<8-hex-chars>` where the hash input is the fully-qualified Rust type name of `E` (e.g. `arcane_pack::events::AteGoldenAppleEvent`).  This is stable within a compilation but may change if the type is moved to a different module.",
    minecraft = "The scoreboard objective is `__ev_<8-hex-chars>` where the hash input is the fully-qualified Rust type name of `E` (e.g. `arcane_pack::events::AteGoldenAppleEvent`).  This is stable within a compilation but may change if the type is moved to a different module.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::handle::EventHandle;",
)]
/// Runtime handle for enabling, disabling, and resetting an advancement-backed event.
///
/// The generic parameter `E` is a marker that binds the handle to a specific
/// event type. No trait bound is required — the objective name is derived lazily
/// from `E`'s fully-qualified Rust type name via [`std::any::type_name`].
///
/// # Example
///
/// ```rust,ignore
/// use sand_core::event::handle::EventHandle;
///
/// static GOLDEN_APPLE: EventHandle<AteGoldenAppleEvent> = EventHandle::new();
///
/// #[datapack_component(Load)]
/// pub fn load() {
///     GOLDEN_APPLE.define();
/// }
///
/// #[on_event]
/// pub fn on_death(_: OnDeath) {
///     GOLDEN_APPLE.disable("@s");
/// }
/// ```
///
/// # Objective naming
///
/// The scoreboard objective is `__ev_<8-hex-chars>` where the hash input is
/// the fully-qualified Rust type name of `E` (e.g.
/// `arcane_pack::events::AteGoldenAppleEvent`).  This is stable within a
/// compilation but may change if the type is moved to a different module.
pub struct EventHandle<E> {
    /// Lazily-initialised scoreboard objective name.
    objective: OnceLock<String>,
    /// Lazily-resolved advancement sentinel path (`__sand_local:<path>`).
    adv_path: OnceLock<String>,
    /// Variance: `fn() -> E` keeps the handle `Sync` even for non-`Sync` `E`,
    /// since no `E` value is ever stored.
    _marker: PhantomData<fn() -> E>,
}

impl<E> EventHandle<E> {
    /// Create a typed event handle bound to event type `E`.
    ///
    /// The scoreboard objective name is derived from the Rust type name of `E`
    /// the first time any method on this handle is called — no string required.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::handle::EventHandle::new",
        aliases = ["sand::prelude::EventHandle::new"],
        module = "sand::event",
        kind = "method",
        summary = "Create a typed event handle bound to event type `E`.",
        context = "Create a typed event handle bound to event type `E`. The scoreboard objective name is derived from the Rust type name of `E` the first time any method on this handle is called — no string required.",
        minecraft = "The scoreboard objective name is derived from the Rust type name of `E` the first time any method on this handle is called — no string required.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "A newly constructed `EventHandle` configured to create a typed event handle bound to event type `E`.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E: 'static>()  {\n    let event_handle = sand::event::handle::EventHandle ::< E >::new();\n}",
    )]
    pub const fn new() -> Self {
        Self {
            objective: OnceLock::new(),
            adv_path: OnceLock::new(),
            _marker: PhantomData,
        }
    }

    /// `scoreboard objectives add <obj> dummy` — register the objective.
    ///
    /// Call this in your `#[datapack_component(Load)]` function.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::handle::EventHandle::define",
        aliases = ["sand::prelude::EventHandle::define"],
        module = "sand::event",
        kind = "method",
        summary = "`scoreboard objectives add <obj> dummy` — register the objective.",
        context = "`scoreboard objectives add <obj> dummy` — register the objective. Call this in your `#[datapack_component(Load)]` function.",
        minecraft = "Call this in your `#[datapack_component(Load)]` function.",
        use_when = ["Call this in your `#[datapack_component(Load)]` function."],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The string value produced to emit the documented `scoreboard objectives add <obj> dummy` — register the objective form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E: 'static>(event_handle_value: &sand::event::handle::EventHandle < E >)  {\n    let define = event_handle_value.define();\n}",
    )]
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective_name())
    }

    /// Build a [`Condition`] that checks whether this event is enabled for `@s`.
    ///
    /// Inject into your event's `guard()` implementation to honour the handle.
    ///
    /// ```rust,ignore
    /// fn guard() -> Option<Condition> {
    ///     Some(GOLDEN_APPLE.condition().and(MANA.of("@s").lt(100)))
    /// }
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::handle::EventHandle::condition",
        aliases = ["sand::prelude::EventHandle::condition"],
        module = "sand::event",
        kind = "method",
        summary = "Build a [`Condition`] that checks whether this event is enabled for `@s`.",
        context = "Build a [`Condition`] that checks whether this event is enabled for `@s`. Inject into your event's `guard()` implementation to honour the handle.",
        minecraft = "Checks whether `@s` has score 1 in the handle's derived scoreboard objective; it does not inspect advancement grant state.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `Condition` value produced to build a [`Condition`] that checks whether this event is enabled for `@s`.",
        example = "fn guard() -> Option<Condition> {\nSome(GOLDEN_APPLE.condition().and(MANA.of(\"@s\").lt(100)))\n}",
    )]
    pub fn condition(&self) -> Condition {
        Condition::score(
            "@s".into(),
            self.objective_name().to_string(),
            ScoreRange::Eq(1),
        )
    }

    /// Command to enable this event for the given selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::handle::EventHandle::enable",
        aliases = ["sand::prelude::EventHandle::enable"],
        module = "sand::event",
        kind = "method",
        summary = "Command to enable this event for the given selector.",
        context = "Command to enable this event for the given selector. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Renders `scoreboard players set <selector> <derived-objective> 1`; the event guard must use condition to honor this flag.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(selector = "`selector` provides the Minecraft target selection used to command to enable this event for the given selector."),
        returns = "The rendered Minecraft command text produced to command to enable this event for the given selector.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate<E: 'static>(event_handle_value: &sand::event::handle::EventHandle < E >, selector: impl std::fmt::Display)  {\n    let command = event_handle_value.enable(selector);\n}",
    )]
    pub fn enable(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {selector} {} 1",
            self.objective_name()
        )
    }

    /// Command to disable this event for the given selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::handle::EventHandle::disable",
        aliases = ["sand::prelude::EventHandle::disable"],
        module = "sand::event",
        kind = "method",
        summary = "Command to disable this event for the given selector.",
        context = "Command to disable this event for the given selector. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Renders `scoreboard players set <selector> <derived-objective> 0`; it does not revoke or re-arm the event advancement.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(selector = "`selector` provides the Minecraft target selection used to command to disable this event for the given selector."),
        returns = "The rendered Minecraft command text produced to command to disable this event for the given selector.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate<E: 'static>(event_handle_value: &sand::event::handle::EventHandle < E >, selector: impl std::fmt::Display)  {\n    let command = event_handle_value.disable(selector);\n}",
    )]
    pub fn disable(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {selector} {} 0",
            self.objective_name()
        )
    }

    /// Revoke (re-arm) the advancement for this event.
    ///
    /// Emits `advancement revoke <selector> only <ns>:<path>`.  The advancement
    /// resource location comes from the event registration produced by
    /// `#[on_event]`; Sand replaces its project-namespace sentinel while
    /// exporting the datapack.
    ///
    /// Requires `E: 'static` for the `TypeId` lookup.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::handle::EventHandle::revoke",
        aliases = ["sand::prelude::EventHandle::revoke"],
        module = "sand::event",
        kind = "method",
        summary = "Revoke (re-arm) the advancement for this event. Emits `advancement revoke <selector> only <ns>:<path>`.  The advancement resource location comes from the event registration produced by `#[on_event]`; Sand replaces its project-namespace sentinel while exporting the datapack.",
        context = "Revoke (re-arm) the advancement for this event. Emits `advancement revoke <selector> only <ns>:<path>`.  The advancement resource location comes from the event registration produced by `#[on_event]`; Sand replaces its project-namespace sentinel while exporting the datapack. Requires `E: 'static` for the `TypeId` lookup.",
        minecraft = "Emits `advancement revoke <selector> only <ns>:<path>`.  The advancement resource location comes from the event registration produced by `#[on_event]`; Sand replaces its project-namespace sentinel while exporting the datapack.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(selector = "`selector` provides the Minecraft target selection used to revoke (re-arm) the advancement for this event. Emits `advancement revoke <selector> only <ns>:<path>`. The advancement resource location comes from the event registration produced by `#[on_event]`; Sand replaces its project-namespace sentinel while exporting the datapack."),
        returns = "The string value produced to revoke (re-arm) the advancement for this event. Emits `advancement revoke <selector> only <ns>:<path>`. The advancement resource location comes from the event registration produced by `#[on_event]`; Sand replaces its project-namespace sentinel while exporting the datapack.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate<E: 'static>(event_handle_value: &sand::event::handle::EventHandle < E >, selector: impl std::fmt::Display) where E : 'static {\n    let revoke = event_handle_value.revoke(selector);\n}",
    )]
    pub fn revoke(&self, selector: impl std::fmt::Display) -> String
    where
        E: 'static,
    {
        format!(
            "advancement revoke {selector} only {}",
            self.adv_sentinel::<E>()
        )
    }

    /// Alias for [`revoke`](EventHandle::revoke).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::handle::EventHandle::reset",
        aliases = ["sand::prelude::EventHandle::reset"],
        module = "sand::event",
        kind = "method",
        summary = "Alias for [`revoke`](EventHandle::revoke).",
        context = "Alias for [`revoke`](EventHandle::revoke). This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It emits the same advancement revoke operation as revoke.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(selector = "`selector` provides the Minecraft target selection used to use alias for [`revoke`](EventHandle::revoke)."),
        returns = "The string value produced to use alias for [`revoke`](EventHandle::revoke).",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate<E: 'static>(event_handle_value: &sand::event::handle::EventHandle < E >, selector: impl std::fmt::Display) where E : 'static {\n    let reset = event_handle_value.reset(selector);\n}",
    )]
    pub fn reset(&self, selector: impl std::fmt::Display) -> String
    where
        E: 'static,
    {
        self.revoke(selector)
    }

    /// Grant the advancement for this event (manually fire the trigger logic).
    ///
    /// Emits `advancement grant <selector> only <ns>:<path>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::handle::EventHandle::grant",
        aliases = ["sand::prelude::EventHandle::grant"],
        module = "sand::event",
        kind = "method",
        summary = "Grant the advancement for this event (manually fire the trigger logic).",
        context = "Grant the advancement for this event (manually fire the trigger logic). Emits `advancement grant <selector> only <ns>:<path>`.",
        minecraft = "Grant affects Minecraft's advancement state and is useful for deliberate one-shot lifecycle control.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(selector = "`selector` provides the Minecraft target selection used to grant the advancement for this event (manually fire the trigger logic)."),
        returns = "The string value produced to grant the advancement for this event (manually fire the trigger logic).",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate<E: 'static>(event_handle_value: &sand::event::handle::EventHandle < E >, selector: impl std::fmt::Display) where E : 'static {\n    let grant = event_handle_value.grant(selector);\n}",
    )]
    pub fn grant(&self, selector: impl std::fmt::Display) -> String
    where
        E: 'static,
    {
        format!(
            "advancement grant {selector} only {}",
            self.adv_sentinel::<E>()
        )
    }

    pub(crate) fn objective_name(&self) -> &str {
        self.objective.get_or_init(|| {
            let h = stable_hash(std::any::type_name::<E>());
            format!("__ev_{h}")
        })
    }

    /// Look up the advancement path from the `EventPathEntry` inventory and
    /// return a local sentinel `__sand_local:<path>` for namespace resolution.
    fn adv_sentinel<F: 'static>(&self) -> &str {
        self.adv_path.get_or_init(|| {
            use crate::inventory;
            let tid = TypeId::of::<F>();
            for entry in inventory::iter::<crate::function::EventPathEntry>() {
                if entry.type_id == tid {
                    return format!("{SAND_LOCAL_NS}:{}", entry.path);
                }
            }
            // Fallback: no EventPathEntry found (e.g. tick-poll event or unregistered type).
            // Use type-name hash so the command is at least stable.
            format!(
                "{SAND_LOCAL_NS}:__unknown_{}",
                stable_hash(std::any::type_name::<F>())
            )
        })
    }
}

impl<E> Default for EventHandle<E> {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: EventHandle<E> never stores an E value; the OnceLock<String> is
// inherently Sync.  The PhantomData<fn() -> E> is Sync regardless of E.
unsafe impl<E> Sync for EventHandle<E> {}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Stable FNV-1a 64-bit hash, first 8 hex chars.
fn stable_hash(s: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let prime: u64 = 0x0000_0100_0000_01b3;
    for b in s.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(prime);
    }
    format!("{hash:016x}")[..8].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeEventA;
    struct FakeEventB;

    #[test]
    fn typed_handle_define_emits_objective() {
        let handle: EventHandle<FakeEventA> = EventHandle::new();
        let cmd = handle.define();
        assert!(cmd.starts_with("scoreboard objectives add __ev_"), "{cmd}");
    }

    #[test]
    fn typed_handle_enable_disable() {
        let handle: EventHandle<FakeEventA> = EventHandle::new();
        let enable = handle.enable("@s");
        let disable = handle.disable("@s");
        assert!(enable.contains("@s") && enable.ends_with("1"), "{enable}");
        assert!(
            disable.contains("@s") && disable.ends_with("0"),
            "{disable}"
        );
        let obj = handle.objective_name().to_string();
        assert!(enable.contains(&obj));
        assert!(disable.contains(&obj));
    }

    #[test]
    fn typed_handle_condition() {
        let handle: EventHandle<FakeEventA> = EventHandle::new();
        let cond = handle.condition();
        let cmd_str = format!("{cond:?}");
        assert!(cmd_str.contains("__ev_"), "{cmd_str}");
    }

    #[test]
    fn different_event_types_get_different_objectives() {
        let h1: EventHandle<FakeEventA> = EventHandle::new();
        let h2: EventHandle<FakeEventB> = EventHandle::new();
        assert_ne!(h1.objective_name(), h2.objective_name());
    }
}
