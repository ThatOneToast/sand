#![forbid(unsafe_code)]

//! # Sand — Minecraft datapacks in type-safe Rust
//!
//! Sand lets you author complete Minecraft Java datapacks — functions,
//! commands, events, custom items, recipes, advancements, loot tables,
//! predicates, dialogs, and more — as ordinary Rust code, then compile them
//! to a datapack with the `sand` CLI.
//!
//! This crate is the only dependency a datapack project needs:
//!
//! ```toml
//! [dependencies]
//! sand = "0.1"
//! ```
//!
//! ```rust
//! use sand::prelude::*;
//!
//! #[function]
//! fn hello() {
//!     cmd::say("Hello from Sand");
//! }
//!
//! // `#[function]`-tagged functions return the commands they emit, so
//! // ordinary Rust tests can assert on generated output directly:
//! assert_eq!(hello(), vec!["say Hello from Sand"]);
//! ```
//!
//! # Where to look
//!
//! - [`prelude`] — the common authoring vocabulary; `use sand::prelude::*`
//!   covers ordinary datapack development.
//! - Topic modules ([`mod@event`], [`mod@item`], [`state`], [`command`],
//!   [`mod@component`], [`entity`], [`data`], [`text`], [`version`], [`vfx`]) —
//!   the full supported surface for less common needs.
//! - [`advanced`] — supported custom, version-aware export integration
//!   for framework integrations.
//! - `__private` is macro/compiler wiring only and carries no compatibility
//!   promise; nothing in it is part of the authoring API.
//!
//! # Execution-context expectations
//!
//! Attribute macros like `#[function]`, `#[datapack_component]`, `#[on_event]`, and
//! `#[custom_item]` register their targets with Sand's `inventory`-based collector at
//! program load, and the bodies they wrap are only meaningful when compiled
//! and exported through `sand build` (or `sand_export`, the binary that
//! `sand build` generates for your project). Calling a `#[function]`-tagged
//! Rust function directly (e.g. from a unit test) just returns the
//! `Vec<String>` of Minecraft commands it would emit — useful for asserting
//! on generated command output, as `examples/book_project` does — but the
//! function is only wired into the datapack's actual `.mcfunction` files
//! through the export pipeline.

extern crate self as sand;

mod api_contracts;

// ── Procedural macros ─────────────────────────────────────────────────────────

pub use sand_macros::state_lifecycle;
pub use sand_macros::system;
/// Derives scoped state schemas and stable finite-enum encodings.
///
/// `State` validates schema metadata, generates concrete bound views, and
/// registers scope-aware lifecycle metadata; `EntityStateEnum` maps fieldless
/// enum variants to scoreboard integers.
pub use sand_macros::{EntityStateEnum, State, StateBundle, StateEnum, StateQuery};
/// `#[function]`, `#[datapack_component]`, `#[on_event]`, `#[custom_item]`, `#[armor_event]`,
/// `#[schedule]`, and `run_fn!` — the attribute and function-like macros that
/// turn ordinary Rust functions into datapack functions, lifecycle hooks
/// (`Load`/`Tick`/`Tag`), typed event handlers, custom items with generated
/// predicates, armor equip/unequip watchers, and self-scheduling routines.
/// Re-exported here so authors never depend on the `sand-macros` proc-macro
/// crate directly — `use sand::prelude::*` (or these paths) is the only
/// import needed. See each macro's own docs for attribute syntax and
/// generated code; `#[function]`/`#[datapack_component]`/`#[on_event]` bodies are only
/// meaningful when compiled through `sand build`.
pub use sand_macros::{
    armor_event, custom_item, datapack_component, entity_archetype, function, on_event, run_fn,
    schedule,
};

/// Defines the authoritative contract for a supported Sand public API item.
pub use sand_macros::api;

/// `#[derive(SandStorage)]` — generates a typed [`data::StorageSchema`] and
/// one [`data::StorageField`] accessor per struct field, from a plain Rust
/// struct annotated with `#[sand(storage = "namespace:id", root = "nbt.path")]`.
/// The typed way to declare a datapack's NBT storage shape without hand-
/// writing storage IDs or NBT paths at each call site. See the macro's own
/// docs for the generated API and `#[sand(path = "...")]` field overrides.
pub use sand_macros::SandStorage;

/// `hud_bar!`, `hud_element!`, and `texture!` — declarative resource-pack
/// authoring macros for custom HUD bars/elements and referenced textures.
/// Only available with the `resourcepack` feature, and only useful alongside
/// [`resourcepack`] (the `sand-resourcepack` crate), which provides the types
/// these macros construct.
#[cfg(feature = "resourcepack")]
pub use sand_macros::{hud_bar, hud_element, texture};

// ── Declarative macros (defined in the implementation crate) ─────────────────

/// `all!`/`any!` compose typed [`condition::Condition`]s (all-of / any-of);
/// `mcfunction!` builds a `Vec<String>` of commands from semicolon-separated
/// expressions. These expression macros are defined in the implementation
/// crate and re-exported here so `sand::` is the only path authors need.
pub use sand_core::{all, any, mcfunction};

// ── Prelude ───────────────────────────────────────────────────────────────────

pub mod prelude;

// ── Topic modules ─────────────────────────────────────────────────────────────

/// Typed command builders: `execute` chains, selectors (`Selector`,
/// `EntityTargets`), scoreboard operations, effects, sounds, particles,
/// block/NBT operations, and free functions like `cmd::say`/`cmd::tellraw`.
/// Reach for this when the [`prelude`] doesn't already have the command
/// builder you need, or when you want to name the module explicitly (e.g. in
/// generic code taking `impl Fn() -> Vec<String>`). Every command builder
/// implements `Display`, so `.to_string()` (or letting `mcfunction!`/
/// `#[function]` collect it) produces the literal Minecraft command text.
pub use sand_core::cmd as command;

/// Same module as [`command`]; kept under its conventional short name because
/// generated code and examples call helpers as `cmd::say(...)`. Both paths
/// point at the identical module — use whichever reads better at the call
/// site.
pub use sand_core::cmd;

/// The typed event model: [`event::Event`], `AdvancementEvent` (custom
/// advancement-backed triggers), vanilla event markers, and the trigger
/// builders (`InventoryChangedTrigger`, `RecipeUnlockedTrigger`, …) used to
/// describe when an event fires. Use this module when defining your own
/// advancement-backed event type or reading its handler context
/// (`event.player()`); ordinary `#[on_event]` handlers for built-in vanilla
/// events usually only need the handler parameter type, exported from here
/// (e.g. `sand::event::vanilla::OnDeath`) as shown in the crate-level example.
pub use sand_core::event;

/// The event graph/dispatch surface backing `#[on_event]` and tick-driven custom
/// events: `SandEvent`, `SandEventDispatch` (tick/chain/after-any/after-all
/// dispatch composition), and vanilla event marker types
/// (`PlayerSprintEvent`, etc.) usable as dispatch parents. Use this module
/// when composing a custom event out of another event's detection logic
/// (`SandEventDispatch::chain::<Parent>()`) instead of writing a fresh
/// tick-poll condition from scratch.
pub use sand_core::events;

/// Custom items: `CustomItem` (the builder passed to `#[custom_item]`), item stack
/// component types, item matchers/predicates, and item location helpers.
/// Use this when building or matching custom items outside a `#[custom_item]`
/// function body — for example, constructing an `ItemPredicate` to gate an
/// event or `execute if items` check.
pub use sand_core::item;

/// Typed live inventory locations. [`inventory::ItemLocation`] is the canonical model;
/// the entity/block factory handles in this module only construct locations.
/// NBT reads and snapshots share [`data::NbtRef`], while live mutation and
/// matching use vanilla `/item` and `execute if items`.
#[api(
    path = "sand::inventory",
    module = "sand",
    summary = "Constructs typed live inventory and equipment locations.",
    context = "Inventory locations distinguish live Minecraft slots from NBT snapshots and make slot bounds and entity-versus-block ownership explicit.",
    minecraft = "Renders vanilla item-command locations and execute-if-items targets for entities and block containers.",
    use_when = ["Mutating or matching a live inventory slot", "Addressing bounded player, entity, or container slots"],
    avoid_when = ["Reading an offline snapshot or arbitrary NBT path"],
    example = "use sand::prelude::*;\nlet inventory = ItemLocation::entity(Selector::self_());\nlet slot = inventory.hotbar(0).unwrap();"
)]
pub mod inventory {
    pub use sand_core::item::{
        BlockInventory, ContainerIndex, EnderChestIndex, EntityInventory, EntityInventorySlot,
        HotbarIndex, InventoryIndex, ItemLocation, ItemLocationError, MainInventoryIndex,
    };
}

/// Predicate resources and the typed conditions used to build them.
///
/// This is the canonical predicate API path. Prelude and component-module
/// appearances are aliases of these same contracts.
#[api(
    path = "sand::predicate",
    module = "sand",
    summary = "Builds reusable Minecraft predicate resources from typed conditions.",
    context = "Predicates package vanilla loot-condition logic under a namespaced identifier so commands and other generated resources can reference the same condition consistently.",
    minecraft = "Generates JSON resources under data/<namespace>/predicate and evaluates them only when referenced by Minecraft.",
    use_when = ["Sharing entity, equipment, item, or location checks", "Referencing a condition from commands or generated resources"],
    avoid_when = ["Tracking mutable runtime state", "Performing scoreboard arithmetic"],
    example = "use sand::predicate::{Predicate, PredicateRoot};"
)]
pub mod predicate {
    pub use sand_components::{
        BlockPredicate, DamagePredicate, DamageSourcePredicate, DistancePredicate, EffectPredicate,
        EntityEquipment, EntityFlags, EntityPredicate, EntityPredicateTarget, FloatRange, IntRange,
        ItemPredicate, LocationPredicate, Predicate, PredicateId, PredicateRoot, WeatherPredicate,
    };
}

/// State implementation primitives and typed storage/NBT schemas. Ordinary
/// authoring uses [`State`] with `#[state(...)]`; this module remains available
/// for deliberate helpers over typed score and storage representations.
#[api(path = "sand::state", module = "sand", summary = "Provides typed scoreboard, timer, storage, and derived State primitives.", context = "This module contains the vocabulary beneath derived schemas and explicit low-level state operations.", minecraft = "Operations render scoreboard and data-storage commands against persistent datapack state locations.", use_when = ["Building typed state operations", "Defining a derived State schema"], avoid_when = ["Unvalidated raw commands can be replaced by typed state APIs"], example = "use sand::state::*;")]
pub mod state {
    pub use sand_core::state::lifecycle::{
        StateCleanup, StateInit, StateLifecycle, StateMigrate, StateProvision, StateReconcile,
        StateTick,
    };
    pub use sand_core::state::{
        BlockNbt, Cooldown, DataCommand, EntityNbt, Flag, FlagRef, FlowTransitionBuilder,
        GameState, GameStateRef, IntoStateCommands, Nbt, NbtLocation, NbtPath, NbtRef, NbtTarget,
        ScoreConst, ScoreConstants, ScoreExpr, ScoreOperand, ScoreOperation, ScoreRef, ScoreVar,
        SnbtCompound, SnbtValue, StateFlow, StateTransitionBuilder, StorageField, StorageLocation,
        StorageSchema, StorageVar, Ticks, Timer, TypedGameState, UntypedNbt,
    };
}

/// Typed entity-bound state, archetypes, curves, native properties, and
/// execution-scoped entity contexts.
///
/// State persists per entity in scoreboards. Archetype scans see loaded
/// chunks only, pause while an entity is unloaded, and resume after load.
/// Every generated helper binds its entity to `@s`; `EntityContext` never
/// represents durable cross-tick identity.
///
/// ```
/// use sand::prelude::*;
///
/// #[derive(State)]
/// #[state(namespace = "demo", scope = living, name = "zombie", version = 1)]
/// struct Mob {
///     #[state(default = 1, min = 1, max = 100)]
///     level: EntityScore<i32>,
///     #[state(default = 20, min = 1, max = 1000)]
///     max_health: EntityScore<i32>,
/// }
///
/// #[entity_archetype]
/// fn zombie() -> EntityArchetype<ZombieKind, Mob> {
///     EntityArchetype::new(ResourceLocation::new("demo", "zombie").unwrap())
///         .adopt(Adoption::natural_and_external().every(Ticks::new(5)))
///         .derive(
///             EntityDerivation::new(
///                 "health",
///                 Mob::max_health,
///                 StatCurve::linear(StatCurve::state(Mob::level), 2.0, 18.0),
///             ),
///         )
///         .health(HealthBinding::new(Mob::max_health))
/// }
/// ```
#[api(
    path = "sand::entity",
    module = "sand",
    summary = "Models typed entity state, queries, archetypes, and execution contexts.",
    context = "The entity module centralizes author-facing models for persistent entity data and the temporary selector-bound context in which generated commands run.",
    minecraft = "Generates selector-driven commands, scoreboard and NBT state operations, and optional entity-archetype lifecycle functions.",
    use_when = ["Querying or mutating Minecraft entities through typed APIs", "Declaring entity-specific state or an archetype"],
    avoid_when = ["Addressing a one-off command token already covered by the command module", "Representing a durable entity identity outside an execution context"],
    example = "let entities = sand::entity::EntityQueries::entities();"
)]
pub mod entity {
    pub use sand_core::entity::{
        Adoption, AdoptionSource, AnyEntity, AttributeBinding, AttributeModifierBinding,
        CurrentHealthSync, CurveEvaluationError, CurveInputs, DEFAULT_FIXED_POINT_SCALE, Data,
        DerivedScoreEncoding, EffectBinding, EntityAction, EntityArchetype, EntityContext,
        EntityCooldown, EntityCooldownAccessor, EntityDerivation, EntityDiagnostic, EntityEnum,
        EntityEnumAccessor, EntityEnumValue, EntityEventId, EntityFlag, EntityFlagAccessor,
        EntityKind, EntityNbtBinding, EntityNbtProperty, EntityNbtType, EntityNbtValue,
        EntityQueries, EntityQuery, EntityScope, EntityScore, EntityScoreAccessor, EntityState,
        EntityStateField, EntityTag, EntityTeam, EntityText, EntityTextSegment, EntityTimer,
        EntityTimerAccessor, EntityTransition, EntityTransitionField, EnumEncoding,
        EquipmentBinding, FixedPoint, FixedScore, FixedScoreAccessor, FixedScoreValue, FixedValue,
        GlobalStateBundleOperations, HealthBinding, HealthResizePolicy, KeyedData, KnownEntityKind,
        LivingEntityKind, MarkerKind, Migration, MutableLivingEntityKind, NameBinding,
        NumericPropertySource, NumericStateField, NumericStateSource, OverflowPolicy,
        OwnershipPolicy, PlayerContext, PlayerKind, PlayerQueries, PlayerQuery, PropertyNameError,
        RawEntityProperty, RawEntityStateField, RawPropertyAccess, RawStateBackend,
        ReconcilePolicy, RefreshPolicy, Relation, RelationQuery, RoundingPolicy,
        SafeEntityDataWriteKind, ScopedEntityRef, Score, SingleEntityQuery, SinglePlayerQuery,
        SpecialEntityPolicy, StatCurve, StateComposition, StateFieldDescriptor, StateFieldKind,
        StatePredicate, StateQueryOperations, StateSchema, TagBinding, TeamBinding,
        ThresholdDirection, ZombieKind,
    };
}

/// Typed participant context (#230): reliability, availability, roles,
/// lifetime, typed handles (`EntityParticipant`, `PlayerParticipant`),
/// declarative observation plans (`EventParticipantPlan`,
/// [`ParticipantBuilder`](sand_core::participant::ParticipantBuilder)), and
/// the correlated-observation backend (`observe_correlated_attacker`).
///
/// Accessors (`Event<E>::entity`/`.item`/`.attacker`/`.victim`/`.weapon`, and
/// the equivalent bare-`SandEvent` accessors via
/// [`events::SandEventParticipants`]) return the typed participant directly
/// (#273) — reach for this module when declaring a plan
/// (`SandEvent::participants`/`AdvancementEvent::participants`, built with
/// [`ParticipantBuilder`](sand_core::participant::ParticipantBuilder)) or
/// working with a typed handle (`EntityParticipant::selector()`).
///
/// ```rust,ignore
/// use sand::prelude::*;
///
/// #[on_event]
/// fn on_hit(event: Event<EntityDamagePlayerEvent>) {
///     let attacker = event.attacker();
///     // build commands against attacker.selector()
/// }
/// ```
/// Typed event participants and the observation plans that make them
/// available to handlers.
///
/// Compiler capture records and transport bookkeeping remain behind Sand's
/// implementation boundary; this module exposes only the semantic roles,
/// lifetimes, references, snapshots, and plans used by datapack authors.
#[api(path = "sand::participant", module = "sand", summary = "Provides typed event participant roles, observation plans, references, and snapshots.", context = "Participants are available only when an event plan declares real observation or valid same-cycle inheritance.", minecraft = "Entity relationships use execute relations, while item snapshots use bounded Sand-owned command storage.", use_when = ["Declaring or reading a participant guaranteed by an event plan"], avoid_when = ["Assuming a participant remains live beyond its declared lifetime"], example = "use sand::participant::*;")]
pub mod participant {
    pub use sand_core::participant::{
        BoundedItemSnapshot, CorrelatedEntityObservation, CorrelationEvidence, CorrelationSource,
        DuplicateParticipantRole, EntityParticipant, EntityParticipantRole, EventParticipantPlan,
        EventParticipantPlanError, ItemEvidenceQualifier, ItemParticipantRole,
        LocationParticipantRole, ObservationError, ObservationSchema, ParticipantAvailability,
        ParticipantBuilder, ParticipantHand, ParticipantLifetime, ParticipantReliability,
        ParticipantReliabilityError, ParticipantUnavailableReason, PlayerParticipant,
        observe_correlated_attacker,
    };
}

/// Datapack component builders: advancements, recipes (shaped/shapeless/
/// smithing/stonecutting), loot tables, predicates, item modifiers, tags,
/// dialogs, and enchantments. Functions returning one of these types and
/// annotated `#[datapack_component]` (e.g. `examples/book_project`'s
/// `trailhead_dialog()`, which returns `Dialog`) are exported as generated
/// JSON resources. Most individual builder types (`Advancement`,
/// `LootTable`, `Dialog`, …) are also re-exported from the [`prelude`].
#[api(path = "sand::component", module = "sand", summary = "Builds datapack JSON components such as recipes, loot tables, tags, dialogs, and advancements.", context = "Component builders model vanilla data resources as typed Rust values that #[datapack_component] registers for export.", minecraft = "Serializes each component into its corresponding version-aware datapack JSON resource.", use_when = ["Constructing a datapack JSON resource"], avoid_when = ["Emitting one command in a function body"], example = "use sand::component::*;")]
pub mod component {
    pub use sand_components::advancement::InventorySlotsPredicate;
    pub use sand_components::dialog::{
        Dialog, DialogAction, DialogBody, DialogButton, DialogItemRef, DialogKind, DialogTag,
        DialogText, IntoDialogRef,
    };
    pub use sand_components::{
        BannerPattern, Biome, BiomeEffects, CarverFloatRange, CarvingStep, CaveCarverConfig,
        ChatDecoration, ChatDecorationParameter, ChatStyle, ChatType, ConfiguredCarver,
        ConfiguredFeature, CustomData, DensityFunction, DensityFunctionBinaryOp,
        DensityFunctionExpr, DensityFunctionUnaryOp, Dimension, DimensionType, EnchantmentEntry,
        IntoItemStack, IntoRecipeItemId, ItemComponent, ItemStackComponents, LootText,
        MonsterSpawnLightLevel, Noise, OreConfig, OreTarget, PlacedFeature, PotionContents,
        RawComponent, RawJson, RawSnbt, Result, RuleTest, SandError, SpawnCondition,
        StatusEffectInstance, StructureTemplate, SuspiciousStewEffect, TagEntry, TagRegistry,
        TemperatureModifier, TypedTag,
    };
    pub use sand_core::components::*;
}

/// Typed conditions used by `execute`, event guards, and grouped branches.
/// [`condition::Condition`] is an opaque expression tree: construct it through
/// typed score, selector, predicate, NBT, or item APIs, then compose it with
/// `all!`, `any!`, `!`, or the methods on `Condition`. See [`execute_when`] for
/// the `if_`/`unless`/`when` grouped-branch API.
#[api(
    path = "sand::condition",
    module = "sand",
    summary = "Composes typed Minecraft conditions without exposing Sand's lowering representation.",
    context = "Conditions are shared by execute commands, event guards, state checks, and grouped branches, so one typed expression model keeps their boolean behavior consistent.",
    minecraft = "Lowers condition trees into one or more execute-if or execute-unless clause plans, distributing nested alternatives when required.",
    use_when = ["Combining typed score, entity, predicate, NBT, or item checks", "Passing a reusable guard to execute or event APIs"],
    avoid_when = ["Choosing Rust generation-time control flow", "Hand-writing execute syntax that an existing typed condition represents"],
    example = "use sand::prelude::*;\nlet ready = Condition::entity(Selector::self_().tag(\"ready\"));"
)]
pub mod condition {
    pub use sand_core::condition::*;
}

/// Grouped-branch `execute` composition: `if_(condition)`, `unless(condition)`,
/// and `when(condition)`, each returning a builder with `.then_all(...)`
/// (and, for `if_`, `.else_all(...)`) that accepts command lists built with
/// `mcfunction!`. Use this instead of hand-writing parallel `execute if`/
/// `execute unless` command pairs.
#[api(
    path = "sand::execute_when",
    module = "sand",
    summary = "Builds typed conditional command branches with explicit evaluation semantics.",
    context = "The branch builders distinguish one-time grouped evaluation from per-command condition checks so state-changing command sequences behave predictably.",
    minecraft = "Emits execute-if or execute-unless commands and registers grouped arms as generated helper functions in the datapack.",
    use_when = ["Running commands under a typed Condition", "Expressing an if/else command branch"],
    avoid_when = ["A typed command builder already exposes the required conditional form", "Rust control flow is being used only to decide what code to generate"],
    example = "use sand::prelude::*;\nlet ready = Condition::entity(Selector::self_().tag(\"ready\"));\nlet commands = when(ready).then_one(\"say ready\");"
)]
pub mod execute_when {
    pub use sand_core::execute_when::*;
}

/// Typed references to Minecraft resources owned by a datapack.
#[api(
    path = "sand::resource_ref",
    module = "sand",
    summary = "Groups typed identifiers for Minecraft resources referenced by Sand APIs.",
    context = "Resource-kind-specific IDs prevent a function, predicate, dialog, or generated data resource from being passed where a different Minecraft resource kind is required.",
    minecraft = "Each ID serializes as the validated namespace:path location Minecraft uses to find its corresponding datapack resource.",
    use_when = ["Connecting one Sand resource or command to another by identity", "Validating a datapack resource location before export"],
    avoid_when = ["Building the JSON payload of the resource itself", "Passing an unchecked namespace:path string to a typed API"],
    example = "let dialog = sand::resource_ref::DialogId::local(\"welcome\");"
)]
pub mod resource_ref {
    pub use sand_core::resource_ref::{
        AdvancementId, DialogId, FunctionId, LootTableId, PredicateId, RecipeId,
    };
}

/// Typed identifiers for Minecraft registries, including custom/modded IDs
/// and generated vanilla status-effect and potion enums.
///
/// This is the canonical owner for registry-wide identifiers. The prelude
/// reexports the same types for convenience, but prelude curation does not
/// define their stable API identity.
#[api(
    path = "sand::registry",
    module = "sand",
    summary = "Groups typed identifiers for Minecraft registry entries.",
    context = "Registry-specific wrappers prevent identifiers for different Minecraft registries from being mixed while retaining custom namespace:path support.",
    minecraft = "Each wrapper serializes as the validated resource location used by its corresponding Minecraft registry.",
    use_when = ["Passing a custom or modded registry entry to a typed Sand API", "Naming a registry entry that has no generated vanilla enum variant"],
    avoid_when = ["A resource-file identity belongs in sand::resource_ref", "Passing an unchecked namespace:path string"],
    example = "let item = sand::registry::ItemId::minecraft(\"diamond\").unwrap();"
)]
pub mod registry {
    pub use sand_core::{
        BiomeId, BlockId, ChickenVariantId, ConfiguredCarverId, ConfiguredFeatureId, CowVariantId,
        DamageTypeId, DensityFunctionId, DimensionId, DimensionTypeId, EffectId,
        EnchantmentEffectComponentId, EnchantmentId, EntityTypeId, EquipmentModelId, ItemId,
        NoiseId, PigVariantId, PotionId, PotionRegistryId, ProcessorListId, RandomSequenceId,
        SoundEventId, StatusEffectId, StructureId, StructureSetId, StructureTemplateId,
        StructureTypeId, TemplatePoolId, TradeSetId, VillagerTradeId,
    };
}

/// Typed Minecraft-version parsing, resolved pack formats, and capability
/// checks. Most datapacks select a target through `sand.toml`; use this module
/// when a reusable authoring system must make an explicit version-aware choice.
#[api(
    path = "sand::version",
    module = "sand",
    summary = "Models Minecraft target versions and their verified capabilities.",
    context = "A resolved profile keeps pack formats and feature gates consistent instead of scattering release-number comparisons through author code.",
    minecraft = "Selects the pack metadata and data-driven features valid for the target Minecraft Java Edition release.",
    use_when = ["Checking whether authored content needs a Minecraft capability", "Inspecting the pack formats selected for a target release"],
    avoid_when = ["Driving Sand's generated export wiring directly", "Passing an unvalidated version string between APIs"],
    example = "let version = sand::version::MinecraftVersion::parse(\"1.21.4\").unwrap();"
)]
pub mod version {
    pub use sand_core::component::{ComponentFeature, VersionCaps};
    pub use sand_core::version::{
        LATEST_KNOWN, MinecraftVersion, PackMetadata, VersionError, VersionFeature, VersionProfile,
    };
}

/// Particle/sound VFX sequencing: `Vfx`, `VfxParticle`, `VfxSound`, and the
/// `VfxStep` enum used to build a reusable, composable effect
/// (`Vfx::new(name).particle(...).sound(...)`) that emits its commands with
/// `.play_at(selector)`.
pub use sand_core::vfx;

/// Build-time, typed world and server configuration (issue #317):
/// [`build::SandBuild`], [`build::BuildContext`]/[`build::BuildProfile`],
/// [`build::World`]/[`build::Dimensions`]/generator builders, and the
/// separate, host-only [`build::ServerConfig`].
///
/// A project's `sand.build.rs` script (compiled by `sand-cli` as the
/// `sand_build_world` binary — see `sand add worldbuild`) exposes
/// `fn build(ctx: &build::BuildContext) -> build::SandBuild` and is invoked
/// during `sand build`/`sand run` to select world generation and, for `sand
/// run` only, local dev-server settings.
///
/// **World vs. Server:** everything reachable from [`build::World`] lowers
/// into the exported datapack (🌍 works in singleplayer, LAN, realms, any
/// vanilla-compatible server). [`build::ServerConfig`] is a structurally
/// separate type: it configures only Sand's own local dev server via `sand
/// run` and is never written into `dist/<pack>/data/...` (🖥️ see its docs
/// for why — view distance, simulation distance, a difficulty *default*,
/// online-mode, and world-reset policy have no datapack representation at
/// all). See the "Build scripts" and "Server configuration" mdBook chapters.
#[api(
    registry = sand_api_contract,
    path = "sand::build",
    module = "sand::build",
    summary = "The build module exposes SandBuild, BuildContext/BuildProfile, World/Dimensions/generator builders, and the host-only ServerConfig for build-time world and server configuration.",
    context = "A project's sand.build.rs script returns a SandBuild built from this module's types; sand-cli lowers it to datapack resources during sand build and applies ServerConfig during sand run.",
    minecraft = "World-reachable types lower into data/<namespace>/... in the exported datapack; ServerConfig never does.",
    use_when = ["Authoring dimensions, world generation, spawn, border, gamerules, time, or weather", "Configuring sand run's local dev server"],
    avoid_when = ["Authoring ordinary function/component/event content; use the other topic modules"],
    example = "use sand::build::{BuildContext, BuildProfile, SandBuild, World};"
)]
pub mod build {
    pub use sand_core::build::{
        BiomeSource, BuildContext, BuildDiagnostic, BuildProfile, Difficulty, Dimension,
        DimensionSlot, DimensionType, Dimensions, FlatGenerator, FlatLayer, Generator,
        NoiseGenerator, NoiseSettingsRef, SandBuild, Seed, ServerConfig, Spawn, SpawnPlatform,
        TimeConfig, VanillaNoiseSettings, WeatherConfig, World, WorldBorder, WorldPreset,
        WorldResetPolicy, WorldResource, lower_world, run_and_print,
    };
}

/// Optional higher-level gameplay systems built from Sand's typed state,
/// event, entity, and inventory primitives.
///
/// Export registries, lifecycle bookkeeping, and generated tick-command
/// drains stay internal; each feature exposes only the semantic builder or
/// registration API a datapack author uses.
#[api(path = "sand::systems", module = "sand", summary = "Groups optional higher-level gameplay systems built from Sand's typed primitives.", context = "Each child module is feature-gated and exposes semantic authoring APIs rather than exporter bookkeeping.", minecraft = "Enabled systems emit their documented resources, lifecycle functions, and commands.", use_when = ["A built-in system matches the gameplay behavior the pack needs"], avoid_when = ["The pack needs different semantics from the documented system"], example = "use sand::systems;")]
pub mod systems {
    #[cfg(feature = "systems-damage")]
    #[sand_macros::api(path = "sand::systems::damage", module = "sand::systems", summary = "Provides the feature-gated damage gameplay system.", context = "This opt-in system composes State and events into typed damage tracking.", minecraft = "Emits damage scoreboards and tick reconciliation.", use_when = ["Tracking typed damage state"], avoid_when = ["Vanilla health alone is sufficient"], example = "use sand::systems::damage::*;", availability = ["Cargo feature: systems-damage"])]
    pub mod damage {
        pub use sand_core::systems::damage::{DamageThreshold, DamageTracker, recently_damaged};
    }

    #[cfg(feature = "systems-cooldowns")]
    #[sand_macros::api(path = "sand::systems::cooldowns", module = "sand::systems", summary = "Provides the feature-gated cooldown gameplay system.", context = "This opt-in system ticks registered typed cooldown fields.", minecraft = "Emits a tick path that decrements active scoreboard cooldowns.", use_when = ["Several cooldowns need shared ticking"], avoid_when = ["Another system owns cooldown timing"], example = "use sand::systems::cooldowns::*;", availability = ["Cargo feature: systems-cooldowns"])]
    pub mod cooldowns {
        pub use sand_core::systems::cooldowns::register_cooldown;
    }

    #[cfg(feature = "systems-lifecycle")]
    #[sand_macros::api(path = "sand::systems::lifecycle", module = "sand::systems", summary = "Provides feature-gated first-join and respawn helpers.", context = "These helpers package common player lifecycle transitions as typed command fragments.", minecraft = "Tests and updates lifecycle scoreboards around join and respawn behavior.", use_when = ["Building first-join or respawn flows"], avoid_when = ["A custom event already owns the lifecycle"], example = "use sand::systems::lifecycle::*;", availability = ["Cargo feature: systems-lifecycle"])]
    pub mod lifecycle {
        pub use sand_core::systems::lifecycle::{FirstJoinCommands, RespawnCommands};
    }

    #[cfg(feature = "systems-player-data")]
    #[sand_macros::api(path = "sand::systems::player_data", module = "sand::systems", summary = "Provides the feature-gated typed player-data schema system.", context = "Groups score, flag, timer, cooldown, and storage fields under a player schema.", minecraft = "Provisions backing scoreboards and storage paths for declared fields.", use_when = ["Modeling cohesive persistent player data"], avoid_when = ["A derived State schema already models the data"], example = "use sand::systems::player_data::*;", availability = ["Cargo feature: systems-player-data"])]
    pub mod player_data {
        pub use sand_core::systems::player_data::{
            CooldownField, CooldownFieldRef, FlagField, GameStateField, GlobalStorageField,
            PlayerDataSchema, PlayerSchema, ScoreField, TimerField, TimerFieldRef,
        };
    }

    #[cfg(feature = "systems-movement")]
    #[sand_macros::api(path = "sand::systems::movement", module = "sand::systems", summary = "Provides feature-gated typed movement effects.", context = "Builders describe pushes, launches, speed boosts, and slowing behavior.", minecraft = "Lowers movement intent to version-appropriate motion and effect commands.", use_when = ["Applying a built-in movement effect"], avoid_when = ["Direct teleportation is intended"], example = "use sand::systems::movement::*;", availability = ["Cargo feature: systems-movement"])]
    pub mod movement {
        pub use sand_core::systems::movement::{Launch, PushAway, Slow, SpeedBoost};
    }

    #[cfg(feature = "systems-inventory")]
    #[sand_macros::api(path = "sand::systems::inventory", module = "sand::systems", summary = "Provides the feature-gated inventory gameplay system.", context = "Exposes typed inventory checks and mutation builders.", minecraft = "Emits validated item predicates and inventory commands.", use_when = ["Checking or changing inventories through typed builders"], avoid_when = ["Manipulating unrelated entity NBT"], example = "use sand::systems::inventory::*;", availability = ["Cargo feature: systems-inventory"])]
    pub mod inventory {
        pub use sand_core::systems::inventory::{ClearBuilder, HasItemCheck, InventorySystem};
    }

    #[cfg(feature = "systems-entities")]
    #[sand_macros::api(path = "sand::systems::entities", module = "sand::systems", summary = "Provides the feature-gated interactable entity system.", context = "Packages entity setup for typed interaction targets.", minecraft = "Emits entity summon and interaction configuration commands.", use_when = ["Creating a built-in interactable entity"], avoid_when = ["A vanilla entity already has the required interaction"], example = "use sand::systems::entities::*;", availability = ["Cargo feature: systems-entities"])]
    pub mod entities {
        pub use sand_core::systems::entities::{InteractSize, Interactable};
    }
}

/// A validated `namespace:path` resource identifier, used throughout Sand
/// anywhere a datapack resource (function, advancement, item, tag, …) is
/// referenced by name. Construction is fallible and validates both segments
/// at call time: `ResourceLocation::new("trail", "grapple/execute").unwrap()`.
pub use sand_core::ResourceLocation;

/// A validated namespace owned by the datapack being generated.
pub use sand_components::PackNamespace;

/// Text components, chat colors, and click/hover events for `tellraw`,
/// titles, dialogs, and books. [`text::Text`] is the builder used everywhere
/// a chat component is needed (`Text::new("Hello").gold().bold(true)`); it
/// implements `Display`, so it renders directly to the JSON text component
/// Minecraft expects wherever a command takes one.
#[api(
    path = "sand::text",
    module = "sand",
    summary = "Provides typed Minecraft text components and interaction events.",
    context = "Minecraft text is structured JSON rather than an unvalidated display string; these builders preserve that structure across commands, dialogs, and books.",
    minecraft = "Renders JSON text components accepted by tellraw, titles, dialogs, books, and other vanilla text fields.",
    use_when = ["Formatting player-visible text", "Adding click or hover behavior"],
    avoid_when = ["Emitting a plain command token that is not a text component"],
    example = "let message = sand::text::Text::new(\"Ready\").gold();"
)]
pub mod text {
    pub use sand_core::prelude::{
        ChatColor, ClickEvent, EntityHoverId, HoverEvent, IntoTextEntityType, Text, TextComponent,
    };
}

/// Storage/NBT data authoring: SNBT values (`SnbtValue`, `SnbtCompound`),
/// command-storage locations (`StorageLocation`, `NbtLocation`, `NbtPath`),
/// and typed storage schemas (`StorageSchema`, `StorageField`,
/// `StorageVar` — also available from [`state`] since storage-backed values
/// are one kind of state). Use this module when working with NBT/storage
/// data directly rather than through a typed state wrapper.
#[api(
    path = "sand::data",
    module = "sand",
    summary = "Models typed NBT, SNBT, and command-storage locations.",
    context = "The data module is the focused API for persistent structured values that do not fit a scoreboard and for commands that read or mutate Minecraft NBT.",
    minecraft = "Generates data-command targets, validated NBT paths, SNBT values, and namespaced command-storage references.",
    use_when = ["Persisting structured datapack state", "Reading or modifying entity, block, or storage NBT"],
    avoid_when = ["A scoreboard-backed integer or flag is the simpler state model"],
    example = "use sand::data::{NbtPath, StorageLocation};"
)]
pub mod data {
    pub use sand_core::cmd::{DataModifyOperation, DataSource, DataTarget, NbtCompound, NbtValue};
    pub use sand_core::state::{
        BlockNbt, DataCommand, EntityNbt, Nbt, NbtLocation, NbtPath, NbtRef, NbtTarget,
        SnbtCompound, SnbtValue, StorageField, StorageLocation, StorageSchema, StorageVar,
        UntypedNbt,
    };
}

/// Generated vanilla Minecraft registry identifiers: `Item`, `Block`,
/// `EntityType`, and `SoundEvent` enums populated at build time from
/// Mojang's own data generator report for the configured Minecraft version
/// from the selected version provider catalog.
///
/// Use these directly wherever Sand asks for a vanilla identifier or entity
/// type — `vanilla::Item::Diamond`, `vanilla::Block::WhiteWool`,
/// `vanilla::EntityType::Marker` — including in [`entity::EntityQuery`],
/// `EntityTargets`/`Selector::entity_type`, and `cmd::summon`/`cmd::give`.
/// They convert into Sand's typed IDs (`ItemId`, `BlockId`, `EntityTypeId`)
/// via `From`/`Into` for cases that need the wrapper type directly (storage,
/// serialization, mixing with custom/external IDs).
///
/// For custom or modded/external identifiers not in this generated list, use
/// the typed `*Id` wrappers instead (`ItemId::minecraft`/`::custom`,
/// `BlockId`, `EntityTypeId`, ...) — see [`prelude`].
///
/// Not every generated Minecraft registry is exposed here: `sand-build`'s
/// codegen also lists `minecraft:worldgen/biome` and `minecraft:enchantment`
/// as candidate registries, but Mojang's data generator report does not
/// currently include either as a plain registry (biome and enchantment data
/// moved to datapack-authorable JSON), so no `Biome`/`Enchantment` enum is
/// actually generated — only `Item`, `Block`, `EntityType`, and `SoundEvent`
/// exist today. Effects, particles, and other registries without generated
/// data still require typed ID wrappers (e.g. `EffectId`) or raw resource
/// locations.
///
/// # Example
/// ```
/// use sand::prelude::*;
/// use sand::vanilla;
///
/// let wool: BlockId = vanilla::Block::WhiteWool.into();
/// let marker = EntityQuery::entities().entity_type(vanilla::EntityType::Marker);
/// ```
#[api(
    path = "sand::vanilla",
    aliases = ["sand::prelude::vanilla"],
    module = "sand",
    summary = "Exposes generated typed identifiers from Minecraft's registries.",
    context = "Registry enums make vanilla identifiers discoverable and typo-resistant while converting into Sand's canonical identifier wrappers.",
    minecraft = "Values map exactly to registry identifiers from Sand's verified Minecraft data-generator input.",
    use_when = ["Referencing a vanilla item, block, entity type, or sound", "Avoiding raw minecraft namespace strings"],
    avoid_when = ["Referencing custom or modded content that needs a typed custom identifier"],
    example = "let item: sand::prelude::ItemId = sand::vanilla::Item::Diamond.into();"
)]
pub mod vanilla {
    #[cfg(not(sand_placeholder_codegen))]
    pub use sand_core::generated::{Block, EntityType, Item, SoundEvent};
}

/// Supported low-level hook for framework integrators that need to drive the
/// version-aware component exporter themselves. Ordinary datapack authors
/// should use `sand build`; compiler wiring and raw escape-hatch types retain
/// their canonical topic-module paths rather than being duplicated here.
pub use sand_core::advanced;

/// Resource-pack authoring (HUD bars/elements, textures), re-exporting the
/// `sand-resourcepack` crate. Only available with the `resourcepack`
/// feature; pair with the [`hud_bar!`](crate::hud_bar),
/// [`hud_element!`](crate::hud_element), and [`texture!`](crate::texture)
/// macros, also feature-gated.
#[cfg(feature = "resourcepack")]
#[api(path = "sand::resourcepack", module = "sand", summary = "Provides optional typed resource-pack authoring APIs.", context = "The feature-gated module accompanies HUD and texture macros with typed asset registrations.", minecraft = "Writes resource-pack GUI textures, bitmap fonts, HUD definitions, and pack metadata.", use_when = ["Authoring client assets with a Sand datapack"], avoid_when = ["Building a datapack-only project"], example = "use sand::resourcepack::*;", availability = ["Cargo feature: resourcepack"])]
pub mod resourcepack {
    pub use sand_resourcepack::{
        AssetContent, AssetOutput, BarHandle, BarStat, BitmapFont, BitmapProvider, Color,
        ElementHandle, FontProvider, GenHudBar, GenHudElement, HudBar, HudElement, HudLayout,
        RawTexture, ResourcePackComponent, ResourcePackDescriptor, ResourcePackRecord, advance_x,
        bar_char, bar_text_json, element_char, element_text_json, export_resourcepack_json,
        resource_pack_format_for,
    };
}

// ── Macro/compiler wiring. Not public API. ────────────────────────────────────

#[doc(hidden)]
pub mod __private {
    //! Expansion targets for Sand's procedural macros and wiring for the
    //! compiler/export pipeline. Nothing here is a compatibility promise;
    //! paths exist solely so generated code can reach the implementation
    //! crate through the façade. See docs/architecture/adr-001.
    pub mod api_contract {
        pub use sand_api_contract::*;
        pub use sand_core::__private::GENERATED_API_PROVIDER_CATALOGS;
        include!(concat!(env!("OUT_DIR"), "/api_coverage.rs"));
    }
    pub mod entity {
        pub use sand_core::entity::archetype::{ArchetypeDefinition, EntityArchetypeDescriptor};
        pub use sand_core::entity::*;
    }
    pub use sand_core::__private::{
        ArchetypeStateScope, EntityStateScope, GlobalStateBundleScope, GlobalStateScope,
        LivingStateScope, PlayerStateScope, QueryableStateScope, SameStateScope, StateBundleMember,
        StateBundleTarget, StateBundleTree, StateDataFieldDescriptor, StateMigrationDescriptor,
        StateQueryHandle, StateQueryScope, StateQuerySpec, StateScopeMarker, StateSystemDescriptor,
        entity_score_new, event_dispatch_advancement, event_dispatch_chain, event_dispatch_tick,
        event_dispatch_tick_condition, event_dispatch_tracked, fixed_score_new,
        lower_state_query_current, lower_state_query_each, player_sneaking_tracked_source,
        resolve_state_objective, state_attach_commands, state_attached_condition,
        state_bundle_trees_overlap, state_detach_commands, state_presence_predicate,
    };
    pub use sand_core::entity::*;
    pub use sand_core::*;
    pub use sand_core::{cmd, condition, event, events, state};

    #[cfg(feature = "resourcepack")]
    pub use sand_resourcepack as rp;
}
