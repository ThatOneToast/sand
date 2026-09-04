//! Typed scoreboard-backed state for entity, living-entity, player, and global
//! schemas.
//!
//! Entity and living-entity state normally binds to the entity currently
//! executing as `@s`. Its accessors may mark deterministic hidden source-dirty
//! objectives so an archetype can reconcile only outputs that depend on the
//! changed field. State survives chunk unloading with the entity, but generated
//! timers and observers do not run while the entity is unloaded.
//!
//! Player state binds to the current player without dirty tracking. Global
//! state binds to a deterministic fake-player score holder, also without dirty
//! tracking. Accessors for every scope emit Minecraft commands; they do not
//! mutate Rust memory.
//!
//! Scoreboard persistence is deliberate: arbitrary custom top-level entity NBT
//! is not a reliable persistence mechanism in Minecraft 26.2.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};

use sand_commands::selector::ScoreRange as SelectorScoreRange;

use crate::condition::{Condition, ScoreRange};
use crate::entity::diagnostic::EntityDiagnostic;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EnumEncoding",
    aliases = ["sand::prelude::EnumEncoding"],
    module = "sand::entity",
    summary = "One enum variant's stable scoreboard encoding.",
    context = "One enum variant's stable scoreboard encoding. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EnumEncoding;",
    fields(name = "Rust-facing variant name used by diagnostics.", score = "Integer stored in the scoreboard."),
)]
/// One enum variant's stable scoreboard encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumEncoding {
    /// Rust-facing variant name used by diagnostics.
    pub name: &'static str,
    /// Integer stored in the scoreboard.
    pub score: i32,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityEnumValue",
    aliases = ["sand::prelude::EntityEnumValue"],
    module = "sand::entity",
    summary = "A finite typed value stored by [`EntityEnum`]. `#[derive(EntityStateEnum)]` is the normal implementation path. Manual implementations remain supported for established wire formats.",
    context = "A finite typed value stored by [`EntityEnum`]. `#[derive(EntityStateEnum)]` is the normal implementation path. Manual implementations remain supported for established wire formats. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityEnumValue;",
)]
/// A finite typed value stored by [`EntityEnum`].
///
/// `#[derive(EntityStateEnum)]` is the normal implementation path. Manual
/// implementations remain supported for established wire formats.
pub trait EntityEnumValue: Copy + Eq + fmt::Debug + 'static {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityEnumValue::ENCODINGS",
        aliases = ["sand::prelude::EntityEnumValue::ENCODINGS"],
        module = "sand::entity",
        kind = "associated_const",
        summary = "Complete stable encoding table.",
        context = "Complete stable encoding table. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        example = "use sand::entity::EntityEnumValue;",
    )]
    /// Complete stable encoding table.
    const ENCODINGS: &'static [EnumEncoding];

    /// Convert a value to its declared score.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityEnumValue::encode",
        aliases = ["sand::prelude::EntityEnumValue::encode"],
        module = "sand::entity",
        summary = "Convert a value to its declared score.",
        context = "Convert a value to its declared score. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `i32` value produced to convert a value to its declared score.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::entity::EntityEnumValue>(entity_enum_value_value: T)  {\n    let encode = entity_enum_value_value.encode();\n}",
    )]
    fn encode(self) -> i32;
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::StateFieldKind",
    aliases = ["sand::prelude::StateFieldKind"],
    module = "sand::entity",
    summary = "Persistence and runtime behavior of a state field.",
    context = "Persistence and runtime behavior of a state field. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::StateFieldKind;",
    variants(Cooldown = "Reusable countdown that is ready at zero.", Dirty = "Generated source/output dirty bit.", Enum = "Finite enum encoded without string matching.", Fixed = "Fixed-point score encoded with the carried positive scale.", Flag = "Boolean encoded as zero or one.", Score = "Signed integer or fixed-point score.", Timer = "Elapsed/countdown timer.", Version = "Schema or archetype version."),
    variant_fields(Enum = ["Finite enum encoded without string matching."], Fixed = ["Number of scoreboard units representing one whole value."]),
)]
/// Persistence and runtime behavior of a state field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StateFieldKind {
    /// Signed integer or fixed-point score.
    Score,
    /// Fixed-point score encoded with the carried positive scale.
    Fixed(#[doc = "Number of scoreboard units representing one whole value."] i32),
    /// Boolean encoded as zero or one.
    Flag,
    /// Finite enum encoded without string matching.
    Enum(#[doc = "Finite enum encoded without string matching."] &'static [EnumEncoding]),
    /// Elapsed/countdown timer.
    Timer,
    /// Reusable countdown that is ready at zero.
    Cooldown,
    /// Schema or archetype version.
    Version,
    /// Generated source/output dirty bit.
    Dirty,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::StateFieldDescriptor",
    aliases = ["sand::prelude::StateFieldDescriptor"],
    module = "sand::entity",
    summary = "Static metadata for one schema field.",
    context = "Static metadata for one schema field. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::StateFieldDescriptor;",
    fields(bounds = "Optional inclusive bounds.", default = "Initial score assigned only when the value is missing.", kind = "Storage/behavior family.", name = "Rust-facing field name."),
)]
/// Static metadata for one schema field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateFieldDescriptor {
    /// Rust-facing field name.
    pub name: &'static str,
    /// Storage/behavior family.
    pub kind: StateFieldKind,
    /// Initial score assigned only when the value is missing.
    pub default: i32,
    /// Optional inclusive bounds.
    pub bounds: Option<(i32, i32)>,
}

impl StateFieldDescriptor {
    /// Construct field metadata.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StateFieldDescriptor::new",
        aliases = ["sand::prelude::StateFieldDescriptor::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct field metadata.",
        context = "Construct field metadata. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(name = "`name` is used when constructing field metadata.", kind = "`kind` is used when constructing field metadata.", default = "`default` is used when constructing field metadata.", bounds = "`bounds` is used when constructing field metadata."),
        returns = "A `StateFieldDescriptor` representing field metadata.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: & 'static str, kind: sand::entity::StateFieldKind, default: i32, bounds: Option < (i32 , i32) >)  {\n    let state_field_descriptor = sand::entity::StateFieldDescriptor::new(name, kind, default, bounds);\n}",
    )]
    #[must_use]
    pub const fn new(
        name: &'static str,
        kind: StateFieldKind,
        default: i32,
        bounds: Option<(i32, i32)>,
    ) -> Self {
        Self {
            name,
            kind,
            default,
            bounds,
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::StateSchema",
    aliases = ["sand::prelude::StateSchema"],
    module = "sand::entity",
    summary = "Complete metadata for a typed state schema.",
    context = "Complete metadata for a typed state schema. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::StateSchema;",
    fields(fields = "Fields in source declaration order.", name = "Schema name within the namespace.", namespace = "Namespace used in generated logical names.", version = "Current version; zero is reserved for an uninitialized entity."),
)]
/// Complete metadata for a typed state schema.
#[derive(Debug, Clone, Copy)]
pub struct StateSchema {
    /// Namespace used in generated logical names.
    pub namespace: &'static str,
    /// Schema name within the namespace.
    pub name: &'static str,
    /// Current version; zero is reserved for an uninitialized entity.
    pub version: u32,
    /// Fields in source declaration order.
    pub fields: &'static [StateFieldDescriptor],
}

impl StateSchema {
    /// Logical `namespace:name` identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StateSchema::id",
        aliases = ["sand::prelude::StateSchema::id"],
        module = "sand::entity",
        kind = "method",
        summary = "Logical `namespace:name` identifier.",
        context = "Logical `namespace:name` identifier. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to logical `namespace:name` identifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(state_schema_value: &sand::entity::StateSchema)  {\n    let id = state_schema_value.id();\n}",
    )]
    #[must_use]
    pub fn id(&self) -> String {
        format!("{}:{}", self.namespace, self.name)
    }

    /// Validate field names, bounds, resolved objective names, and enum encodings.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StateSchema::validate",
        aliases = ["sand::prelude::StateSchema::validate"],
        module = "sand::entity",
        kind = "method",
        summary = "Validate field names, bounds, resolved objective names, and enum encodings.",
        context = "Validate field names, bounds, resolved objective names, and enum encodings. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "On success, the value produced to validate field names, bounds, resolved objective names, and enum encodings; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(state_schema_value: &sand::entity::StateSchema)  {\n    let validate = state_schema_value.validate();\n}",
    )]
    pub fn validate(&self) -> Result<(), EntityDiagnostic> {
        let schema = self.id();
        let mut names = BTreeSet::new();
        let mut objectives = BTreeMap::<String, &'static str>::new();
        for field in self.fields {
            if !names.insert(field.name) {
                return Err(EntityDiagnostic::DuplicateStateField {
                    schema,
                    field: field.name.to_string(),
                    detail: "field name is declared more than once".into(),
                });
            }
            if let Some((min, max)) = field.bounds
                && min > max
            {
                return Err(EntityDiagnostic::InvalidRange {
                    schema,
                    field: field.name.into(),
                    range: format!("{min}..={max}"),
                });
            }
            if let StateFieldKind::Enum(encodings) = field.kind {
                validate_enum_encodings(&schema, field.name, encodings)?;
            }
            let objective = objective_name(self.namespace, self.name, field.name);
            if let Some(previous) = objectives.insert(objective.clone(), field.name) {
                return Err(EntityDiagnostic::DuplicateStateField {
                    schema,
                    field: field.name.into(),
                    detail: format!(
                        "objective `{objective}` collides with `{previous}` after Minecraft's 16-character limit"
                    ),
                });
            }
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityState",
    aliases = ["sand::prelude::EntityState"],
    module = "sand::entity",
    summary = "A type-level State schema. The derive macro generates this implementation and associated typed field constants. Manual implementations must return stable immutable metadata.",
    context = "A type-level State schema. The derive macro generates this implementation and associated typed field constants. Manual implementations must return stable immutable metadata. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityState;",
)]
/// A type-level State schema.
///
/// The derive macro generates this implementation and associated typed field
/// constants. Manual implementations must return stable immutable metadata.
pub trait EntityState: 'static {
    /// Return this schema's metadata.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityState::schema",
        aliases = ["sand::prelude::EntityState::schema"],
        module = "sand::entity",
        summary = "Return this schema's metadata.",
        context = "Return this schema's metadata. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "Return this schema's metadata.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::entity::EntityState>()  {\n    let schema = <T as sand::entity::EntityState>::schema();\n}",
    )]
    fn schema() -> StateSchema;

    /// Storage-backed fields owned by this component.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityState::data_fields", kind = "trait_method",
        aliases = ["sand::prelude::EntityState::data_fields"],
        module = "sand::entity",
        summary = "Storage-backed fields owned by this component.",
        context = "Storage-backed fields owned by this component. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& 'static [StateDataFieldDescriptor]` value produced to storage-backed fields owned by this component.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::entity::EntityState>()  {\n    let data_fields = <T as sand::entity::EntityState>::data_fields();\n}",
    )]
    fn data_fields() -> &'static [StateDataFieldDescriptor] {
        &[]
    }
}

/// Compiler metadata for one component-owned typed storage path.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateDataFieldDescriptor {
    pub storage: &'static str,
    pub path: &'static str,
    pub default_snbt: &'static str,
    pub keyed: bool,
}

impl StateDataFieldDescriptor {
    #[doc(hidden)]
    pub const fn new(
        storage: &'static str,
        path: &'static str,
        default_snbt: &'static str,
    ) -> Self {
        Self {
            storage,
            path,
            default_snbt,
            keyed: false,
        }
    }

    #[doc(hidden)]
    pub const fn keyed(
        storage: &'static str,
        path: &'static str,
        default_snbt: &'static str,
    ) -> Self {
        Self {
            storage,
            path,
            default_snbt,
            keyed: true,
        }
    }
}

/// Compiler-facing composition contract implemented by generated State
/// components and State bundles.
#[doc(hidden)]
pub trait StateBundleMember: 'static {
    /// Concrete holder-bound view generated for this member.
    type Bound;
    /// Hidden owner-scope proof used to reject incompatible bundles.
    type Scope: StateScopeMarker;

    /// Compile-time component tree used to reject nested query contradictions.
    const COMPONENT_TREE: StateBundleTree;

    /// Bind every nested component to one execution-scoped holder.
    fn bind_member(holder: &'static str) -> Self::Bound;

    /// Bind every nested global component to its own singleton holder.
    fn bind_global_member() -> Self::Bound;

    /// Attach all unique nested components in declaration order.
    fn attach_member(holder: &'static str) -> Vec<String>;

    /// Attach every nested global component through its singleton holder.
    fn attach_global_member() -> Vec<String>;

    /// Detach all unique nested components in reverse declaration order.
    fn detach_member(holder: &'static str) -> Vec<String>;

    /// Detach every nested global component through its singleton holder.
    fn detach_global_member() -> Vec<String>;

    /// Resolved presence objectives and accepted component versions for query filtering.
    fn presence_requirements() -> Vec<(String, u32)>;
}

/// A State component or nested bundle that can participate in archetype composition.
///
/// Implementations are generated by `State` and `StateBundle`; authors use
/// this capability through [`crate::entity::EntityArchetype::components`].
///
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::StateComposition",
    aliases = ["sand::prelude::StateComposition"],
    module = "sand::entity",
    summary = "A State component or nested bundle that can participate in archetype composition.",
    context = "A State component or nested bundle that can participate in archetype composition. Implementations are generated by `State` and `StateBundle`; authors use this capability through [`sand::entity::EntityArchetype::components`].",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::StateComposition;",
)]
pub trait StateComposition: 'static {
    /// Returns flattened component identities for conflict detection and deduplication.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StateComposition::composition_identities",
        aliases = ["sand::prelude::StateComposition::composition_identities"],
        module = "sand::entity",
        summary = "Returns flattened component identities for conflict detection and deduplication.",
        context = "Returns flattened component identities for conflict detection and deduplication. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "Returns flattened component identities for conflict detection and deduplication.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::entity::StateComposition>()  {\n    let values = <T as sand::entity::StateComposition>::composition_identities();\n}",
    )]
    fn composition_identities() -> Vec<(String, u32)>;
    /// Lowers idempotent canonical attachment for this component composition.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StateComposition::composition_attach",
        aliases = ["sand::prelude::StateComposition::composition_attach"],
        module = "sand::entity",
        summary = "Lowers idempotent canonical attachment for this component composition.",
        context = "Lowers idempotent canonical attachment for this component composition. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(holder = "`holder` is used to lower idempotent canonical attachment for this component composition."),
        returns = "The ordered values produced to lower idempotent canonical attachment for this component composition.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::entity::StateComposition>(holder: & 'static str)  {\n    let values = <T as sand::entity::StateComposition>::composition_attach(holder);\n}",
    )]
    fn composition_attach(holder: &'static str) -> Vec<String>;
    /// Lowers ownership-safe canonical detachment in reverse composition order.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StateComposition::composition_detach",
        aliases = ["sand::prelude::StateComposition::composition_detach"],
        module = "sand::entity",
        summary = "Lowers ownership-safe canonical detachment in reverse composition order.",
        context = "Lowers ownership-safe canonical detachment in reverse composition order. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(holder = "`holder` is used to lower ownership-safe canonical detachment in reverse composition order."),
        returns = "The ordered values produced to lower ownership-safe canonical detachment in reverse composition order.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::entity::StateComposition>(holder: & 'static str)  {\n    let values = <T as sand::entity::StateComposition>::composition_detach(holder);\n}",
    )]
    fn composition_detach(holder: &'static str) -> Vec<String>;
}

impl<T> StateComposition for T
where
    T: StateBundleMember,
    T::Scope: ArchetypeStateScope,
{
    fn composition_identities() -> Vec<(String, u32)> {
        T::presence_requirements()
    }

    fn composition_attach(holder: &'static str) -> Vec<String> {
        T::attach_member(holder)
    }

    fn composition_detach(holder: &'static str) -> Vec<String> {
        T::detach_member(holder)
    }
}

/// A generated bundle whose members are all global State components.
///
/// Importing Sand's prelude makes these associated operations available on a
/// global bundle. Each member keeps its own deterministic singleton holder.
///
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::GlobalStateBundleOperations",
    aliases = ["sand::prelude::GlobalStateBundleOperations"],
    module = "sand::entity",
    summary = "Provides singleton-holder operations for a generated bundle made entirely from global State components.",
    context = "A generated bundle whose members are all global State components. The StateBundle derive implements this extension automatically when every flattened member has global scope; importing the prelude makes its associated operations available on the bundle type.",
    minecraft = "Every member retains its own deterministic fake-player holder, objective identities, version marker, typed storage path, and lifecycle instead of sharing the current executor or allocating bundle storage.",
    use_when = ["Grouping several global State resources behind one concrete named view", "Attaching or detaching a global resource composition in deterministic order"],
    avoid_when = ["Any member is player, entity, or living scoped", "A component should share the current executor rather than its global singleton"],
    example = "use sand::entity::GlobalStateBundleOperations;",
)]
pub trait GlobalStateBundleOperations: StateBundleMember {
    /// Bind every component in this bundle to its own global holder.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::GlobalStateBundleOperations::global", kind = "trait_method",
        aliases = ["sand::prelude::GlobalStateBundleOperations::global"],
        module = "sand::entity",
        summary = "Binds every component in a global State bundle to its own deterministic singleton holder.",
        context = "Bind every component in this bundle to its own global holder. The returned concrete bundle view preserves named-field completion while each nested component chooses the same holder as its generated component-level global method.",
        minecraft = "Later field operations address each component's existing fake-player score holder or global command-storage path; no command is emitted merely by binding the view.",
        use_when = ["Reading or mutating several related global resources through one named view"],
        avoid_when = ["Binding a scoped owner represented by the current executor; use bundle on instead"],
        returns = "The concrete nested bundle view with every component bound to its singleton holder.",
        example = "let world = WorldResources::global();",
    )]
    fn global() -> Self::Bound {
        Self::bind_global_member()
    }

    /// Attach every unique component in this global bundle.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::GlobalStateBundleOperations::attach_global", kind = "trait_method",
        aliases = ["sand::prelude::GlobalStateBundleOperations::attach_global"],
        module = "sand::entity",
        summary = "Attaches every unique component in a global State bundle through its singleton holder.",
        context = "Attach every unique component in this global bundle. Nested bundles flatten to the canonical component lifecycle; repeated members are deduplicated without merging component versions or ownership.",
        minecraft = "Emits idempotent initialization and migration commands in declaration order, publishing each component presence/version marker only after its owned values are ready.",
        use_when = ["Explicitly attaching a named composition of global State resources"],
        avoid_when = ["Provisioning objectives manually", "Resetting already initialized global progress"],
        returns = "Lifecycle commands for every unique global component in declaration order.",
        example = "let commands = WorldResources::attach_global();",
    )]
    fn attach_global() -> Vec<String> {
        Self::attach_global_member()
    }

    /// Detach every unique component in reverse bundle order.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::GlobalStateBundleOperations::detach_global", kind = "trait_method",
        aliases = ["sand::prelude::GlobalStateBundleOperations::detach_global"],
        module = "sand::entity",
        summary = "Detaches every unique component in reverse global bundle order without touching unrelated state.",
        context = "Detach every unique component in reverse bundle order. The bundle delegates cleanup to each canonical component lifecycle and never removes shared objectives, another component's values, or external scores.",
        minecraft = "Emits cleanup and owned-value removal against each component's deterministic holder in reverse composition order.",
        use_when = ["Explicitly removing a complete global resource composition"],
        avoid_when = ["Removing only one member; call that component's detach method", "Deleting shared scoreboard objectives"],
        returns = "Ownership-safe cleanup commands in reverse component order.",
        example = "let commands = WorldResources::detach_global();",
    )]
    fn detach_global() -> Vec<String> {
        Self::detach_global_member()
    }
}

impl<T> GlobalStateBundleOperations for T
where
    T: StateBundleMember,
    T::Scope: GlobalStateBundleScope,
{
}

/// Hidden type-level owner-scope marker implemented by generated State components.
#[doc(hidden)]
pub trait StateScopeMarker: 'static {}

/// Compile-time component identity tree retained across nested bundles.
#[doc(hidden)]
pub enum StateBundleTree {
    Component(&'static str),
    Bundle(&'static [StateBundleTree]),
}

/// Returns whether two component or bundle trees contain a shared component.
#[doc(hidden)]
pub const fn state_bundle_trees_overlap(left: &StateBundleTree, right: &StateBundleTree) -> bool {
    match (left, right) {
        (StateBundleTree::Component(left), StateBundleTree::Component(right)) => {
            const_str_eq(left, right)
        }
        (StateBundleTree::Bundle(items), other) | (other, StateBundleTree::Bundle(items)) => {
            let mut index = 0;
            while index < items.len() {
                if state_bundle_trees_overlap(&items[index], other) {
                    return true;
                }
                index += 1;
            }
            false
        }
    }
}

const fn const_str_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

/// Proof that an entity context is valid for a bundle owner scope.
#[doc(hidden)]
pub trait StateBundleTarget<K: crate::entity::EntityKind>: StateScopeMarker {}

impl StateBundleTarget<crate::entity::PlayerKind> for PlayerStateScope {}
impl<K: crate::entity::EntityKind> StateBundleTarget<K> for EntityStateScope {}
impl<K: crate::entity::LivingEntityKind> StateBundleTarget<K> for LivingStateScope {}

/// Proof that a bundle consists entirely of global components.
#[doc(hidden)]
pub trait GlobalStateBundleScope: StateScopeMarker {}

/// Scope proof for components that may be composed into an entity archetype.
#[doc(hidden)]
pub trait ArchetypeStateScope: StateScopeMarker {}

/// Hidden proof that two generated component scopes are identical.
#[doc(hidden)]
pub trait SameStateScope<Rhs: StateScopeMarker>: StateScopeMarker {}

impl<T: StateScopeMarker> SameStateScope<T> for T {}

#[doc(hidden)]
pub struct PlayerStateScope;
#[doc(hidden)]
pub struct EntityStateScope;
#[doc(hidden)]
pub struct LivingStateScope;
#[doc(hidden)]
pub struct GlobalStateScope;

impl StateScopeMarker for PlayerStateScope {}
impl StateScopeMarker for EntityStateScope {}
impl StateScopeMarker for LivingStateScope {}
impl StateScopeMarker for GlobalStateScope {}
impl GlobalStateBundleScope for GlobalStateScope {}
impl ArchetypeStateScope for EntityStateScope {}
impl ArchetypeStateScope for LivingStateScope {}

/// Construct the typed exact-version predicate used by generated State queries.
#[doc(hidden)]
pub fn state_presence_predicate(objective: String, version: u32) -> StatePredicate {
    exact_predicate(objective, version as i32)
}

/// Build the canonical idempotent attachment sequence for a component.
///
/// This is compiler wiring used by `#[derive(State)]`. Dependencies are
/// provisioned by the generated load function; attachment initializes only
/// missing values and publishes the component version last.
#[doc(hidden)]
pub fn state_attach_commands<S: EntityState>(
    holder: &'static str,
    presence_logical: &'static str,
    suppression_logical: &'static str,
    clear_suppression: bool,
) -> Vec<String> {
    let schema = S::schema();
    let presence = sand_commands::ObjectiveName::logical(presence_logical)
        .as_str()
        .to_owned();
    let suppression = sand_commands::ObjectiveName::logical(suppression_logical)
        .as_str()
        .to_owned();
    let mut commands = Vec::new();
    if clear_suppression {
        commands.push(format!("scoreboard players reset {holder} {suppression}"));
    }
    for field in schema.fields {
        let objective = objective_name(schema.namespace, schema.name, field.name);
        commands.push(format!(
            "execute unless score {holder} {objective} matches -2147483648.. run scoreboard players set {holder} {objective} {}",
            field.default
        ));
    }
    for field in S::data_fields() {
        commands.extend(state_data_initialize_commands(*field));
    }
    for command in crate::state::registry::initialize_hook_commands::<S>(holder) {
        commands.push(format!(
            "execute unless score {holder} {presence} matches 1.. run {command}"
        ));
    }
    for (from, to) in crate::state::registry::migration_steps::<S>() {
        for command in crate::state::registry::migrate_hook_commands::<S>(holder, from, to) {
            commands.push(format!(
                "execute if score {holder} {presence} matches {from} run {command}"
            ));
        }
        commands.push(format!(
            "execute if score {holder} {presence} matches {from} run scoreboard players set {holder} {presence} {to}"
        ));
    }
    commands.push(format!(
        "execute unless score {holder} {presence} matches 1.. run scoreboard players set {holder} {presence} {}",
        schema.version
    ));
    commands
}

/// Build ownership-safe component detachment commands.
#[doc(hidden)]
pub fn state_detach_commands<S: EntityState>(
    holder: &'static str,
    presence_logical: &'static str,
    suppression_logical: &'static str,
    track_dirty: bool,
    suppress_observation: bool,
) -> Vec<String> {
    let schema = S::schema();
    let presence = sand_commands::ObjectiveName::logical(presence_logical)
        .as_str()
        .to_owned();
    let suppression = sand_commands::ObjectiveName::logical(suppression_logical)
        .as_str()
        .to_owned();
    let mut commands = Vec::new();
    for command in crate::state::registry::cleanup_hook_commands::<S>(holder) {
        commands.push(format!(
            "execute if score {holder} {presence} matches 1.. run {command}"
        ));
    }
    for field in schema.fields {
        let objective = objective_name(schema.namespace, schema.name, field.name);
        commands.push(format!("scoreboard players reset {holder} {objective}"));
        if track_dirty {
            commands.push(format!(
                "scoreboard players reset {holder} {}",
                dirty_name(schema.namespace, schema.name, field.name)
            ));
        }
    }
    if track_dirty {
        commands.push(format!(
            "scoreboard players reset {holder} {}",
            component_dirty_name(schema.namespace, schema.name)
        ));
    }
    for field in S::data_fields() {
        commands.extend(state_data_remove_commands(*field));
    }
    commands.push(format!("scoreboard players reset {holder} {presence}"));
    if suppress_observation {
        commands.push(format!("scoreboard players set {holder} {suppression} 1"));
    }
    commands
}

/// Test whether a component is attached at its current version.
#[doc(hidden)]
pub fn state_attached_condition<S: EntityState>(
    holder: &'static str,
    presence_logical: &'static str,
) -> Condition {
    let presence = sand_commands::ObjectiveName::logical(presence_logical)
        .as_str()
        .to_owned();
    Condition::score(
        holder.to_owned(),
        presence,
        ScoreRange::Eq(S::schema().version as i32),
    )
}

/// Resolve a logical State objective through the canonical objective rules.
#[doc(hidden)]
pub fn resolve_state_objective(logical: &str) -> String {
    sand_commands::ObjectiveName::logical(logical)
        .as_str()
        .to_owned()
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityStateField",
    aliases = ["sand::prelude::EntityStateField"],
    module = "sand::entity",
    summary = "Shared behavior implemented by all typed state field handles.",
    context = "Shared behavior implemented by all typed state field handles. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityStateField;",
)]
/// Shared behavior implemented by all typed state field handles.
pub trait EntityStateField: Copy + 'static {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityStateField::Accessor",
        aliases = ["sand::prelude::EntityStateField::Accessor"],
        module = "sand::entity",
        kind = "associated_type",
        summary = "Accessor returned by [`sand::entity::EntityContext::state`].",
        context = "Accessor returned by [`sand::entity::EntityContext::state`]. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        example = "use sand::entity::EntityStateField;",
    )]
    /// Accessor returned by [`crate::entity::EntityContext::state`].
    type Accessor;

    /// Static field metadata.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityStateField::descriptor",
        aliases = ["sand::prelude::EntityStateField::descriptor"],
        module = "sand::entity",
        summary = "Static field metadata.",
        context = "Static field metadata. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `StateFieldDescriptor` value produced to static field metadata.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::entity::EntityStateField>(entity_state_field_value: T)  {\n    let descriptor = entity_state_field_value.descriptor();\n}",
    )]
    fn descriptor(self) -> StateFieldDescriptor;

    /// Resolved scoreboard objective, at most 16 characters.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityStateField::objective",
        aliases = ["sand::prelude::EntityStateField::objective"],
        module = "sand::entity",
        summary = "Resolved scoreboard objective, at most 16 characters.",
        context = "Resolved scoreboard objective, at most 16 characters. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to resolved scoreboard objective, at most 16 characters.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::entity::EntityStateField>(entity_state_field_value: T)  {\n    let objective = entity_state_field_value.objective();\n}",
    )]
    fn objective(self) -> String;

    /// Hidden source-dirty objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityStateField::dirty_objective",
        aliases = ["sand::prelude::EntityStateField::dirty_objective"],
        module = "sand::entity",
        summary = "Hidden source-dirty objective.",
        context = "Hidden source-dirty objective. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to hidden source-dirty objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::entity::EntityStateField>(entity_state_field_value: T)  {\n    let dirty_objective = entity_state_field_value.dirty_objective();\n}",
    )]
    fn dirty_objective(self) -> String;

    /// Bind this field to the current executor.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityStateField::bind", kind = "trait_method",
        aliases = ["sand::prelude::EntityStateField::bind"],
        module = "sand::entity",
        summary = "Bind this field to the current executor.",
        context = "Bind this field to the current executor. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `Self :: Accessor` value produced to bind this field to the current executor.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::entity::EntityStateField>(entity_state_field_value: T)  {\n    let bind = entity_state_field_value.bind();\n}",
    )]
    fn bind(self) -> Self::Accessor {
        self.bind_to("@s", true)
    }

    /// Bind this field to an explicit typed score holder.
    ///
    /// `track_dirty` is enabled for entity/living state whose archetype
    /// reconciliation consumes the auxiliary objective. Player/global state
    /// passes `false` because its lifecycle owns only the primary objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityStateField::bind_to",
        aliases = ["sand::prelude::EntityStateField::bind_to"],
        module = "sand::entity",
        summary = "Bind this field to an explicit typed score holder.",
        context = "Bind this field to an explicit typed score holder. `track_dirty` is enabled for entity/living state whose archetype reconciliation consumes the auxiliary objective. Player/global state passes `false` because its lifecycle owns only the primary objective.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(holder = "`holder` provides the holder used when binding this field to an explicit typed score holder.", track_dirty = "`track_dirty` is enabled for entity/living state whose archetype reconciliation consumes the auxiliary objective. Player/global state passes `false` because its lifecycle owns only the primary objective."),
        returns = "The `Self :: Accessor` value produced to bind this field to an explicit typed score holder.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::entity::EntityStateField>(entity_state_field_value: T, holder: & 'static str, track_dirty: bool)  {\n    let bind_to = entity_state_field_value.bind_to(holder, track_dirty);\n}",
    )]
    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor;
}

trait ComponentDirtyField: EntityStateField {
    fn component_dirty_objective(self) -> String;
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::StatePredicate",
    aliases = ["sand::prelude::StatePredicate"],
    module = "sand::entity",
    summary = "Typed score predicate consumable by [`sand::command::Target`].",
    context = "Typed score predicate consumable by [`sand::command::Target`]. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::StatePredicate;",
)]
/// Typed score predicate consumable by [`sand_commands::Target`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatePredicate {
    pub(crate) objective: String,
    pub(crate) selector_range: SelectorScoreRange,
    condition_range: ScoreRange,
}

impl StatePredicate {
    /// Generated scoreboard objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatePredicate::objective",
        aliases = ["sand::prelude::StatePredicate::objective"],
        module = "sand::entity",
        kind = "method",
        summary = "Generated scoreboard objective.",
        context = "Generated scoreboard objective. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to generated scoreboard objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate(state_predicate_value: &sand::entity::StatePredicate)  {\n    let objective = state_predicate_value.objective();\n}",
    )]
    #[must_use]
    pub fn objective(&self) -> &str {
        &self.objective
    }

    /// Vanilla selector range used when this predicate constrains adoption or
    /// a [`sand_commands::Target`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatePredicate::selector_range",
        aliases = ["sand::prelude::StatePredicate::selector_range"],
        module = "sand::entity",
        kind = "method",
        summary = "Vanilla selector range used when this predicate constrains adoption or an [`sand::command::Target`].",
        context = "Vanilla selector range used when this predicate constrains adoption or an [`sand::command::Target`]. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to vanilla selector range used when this predicate constrains adoption or an [`sand::command::Target`].",
        example = "use sand::prelude::*;\n\nfn demonstrate(state_predicate_value: &sand::entity::StatePredicate)  {\n    let selector_range = state_predicate_value.selector_range();\n}",
    )]
    #[must_use]
    pub fn selector_range(&self) -> String {
        self.selector_range.to_string()
    }

    /// Turn this predicate into an `@s` condition.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatePredicate::condition",
        aliases = ["sand::prelude::StatePredicate::condition"],
        module = "sand::entity",
        kind = "method",
        summary = "Turn this predicate into an `@s` condition.",
        context = "Turn this predicate into an `@s` condition. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `Condition` value produced to turn this predicate into an `@s` condition.",
        example = "use sand::prelude::*;\n\nfn demonstrate(state_predicate_value: &sand::entity::StatePredicate)  {\n    let condition = state_predicate_value.condition();\n}",
    )]
    #[must_use]
    pub fn condition(&self) -> Condition {
        Condition::score(
            "@s".into(),
            self.objective.clone(),
            self.condition_range.clone(),
        )
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityScore",
    aliases = ["sand::prelude::EntityScore"],
    module = "sand::entity",
    summary = "Typed signed entity score.",
    context = "Typed signed entity score. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityScore;",
)]
/// Typed signed entity score.
#[derive(Debug, PartialEq, Eq)]
pub struct EntityScore<T = i32> {
    namespace: &'static str,
    schema: &'static str,
    descriptor: StateFieldDescriptor,
    _marker: PhantomData<fn() -> T>,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::Score",
    aliases = ["sand::prelude::Score"],
    module = "sand::entity",
    summary = "Canonical integer field marker for `#[derive(State)]` declarations.",
    context = "Canonical integer field marker for `#[derive(State)]` declarations. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::Score;",
)]
/// Canonical integer field marker for `#[derive(State)]` declarations.
pub type Score = EntityScore<i32>;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::FixedScore",
    aliases = ["sand::prelude::FixedScore"],
    module = "sand::entity",
    summary = "Canonical decimal field marker for `#[derive(State)]` declarations.",
    context = "Canonical decimal field marker for `#[derive(State)]` declarations. Values are stored as signed 32-bit scoreboard units. The generated schema records a positive scale (1,000 by default), rounds decimal inputs to the nearest unit with exact halves away from zero, then saturates at the field bounds and the scoreboard range.",
    minecraft = "Values are stored as signed 32-bit scoreboard units. The generated schema records a positive scale (1,000 by default), rounds decimal inputs to the nearest unit with exact halves away from zero, then saturates at the field bounds and the scoreboard range.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::FixedScore;",
)]
/// Canonical decimal field marker for `#[derive(State)]` declarations.
///
/// Values are stored as signed 32-bit scoreboard units. The generated schema
/// records a positive scale (1,000 by default), rounds decimal inputs to the
/// nearest unit with exact halves away from zero, then saturates at the field
/// bounds and the scoreboard range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedScore {
    namespace: &'static str,
    schema: &'static str,
    descriptor: StateFieldDescriptor,
    scale: i32,
}

impl FixedScore {
    /// Construct a fixed-point field; normally generated by `State`.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn __new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        scale: i32,
        default: i32,
        bounds: Option<(i32, i32)>,
    ) -> Self {
        Self {
            namespace,
            schema,
            descriptor: StateFieldDescriptor::new(
                name,
                StateFieldKind::Fixed(scale),
                default,
                bounds,
            ),
            scale,
        }
    }

    /// Number of scoreboard units representing one whole value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedScore::scale",
        aliases = ["sand::prelude::FixedScore::scale"],
        module = "sand::entity",
        kind = "method",
        summary = "Number of scoreboard units representing one whole value.",
        context = "Number of scoreboard units representing one whole value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `i32` value produced to number of scoreboard units representing one whole value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_score_value: sand::entity::FixedScore)  {\n    let scale = fixed_score_value.scale();\n}",
    )]
    #[must_use]
    pub const fn scale(self) -> i32 {
        self.scale
    }

    /// Match a decimal range after applying this field's encoding.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedScore::matches",
        aliases = ["sand::prelude::FixedScore::matches"],
        module = "sand::entity",
        kind = "method",
        summary = "Match a decimal range after applying this field's encoding.",
        context = "Match a decimal range after applying this field's encoding. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(range = "`range` provides the accepted numeric range used to match a decimal range after applying this field's encoding."),
        returns = "On success, the value produced to match a decimal range after applying this field's encoding; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_score_value: sand::entity::FixedScore, range: impl std::ops::RangeBounds < f64 >)  {\n    let matches = fixed_score_value.matches(range);\n}",
    )]
    pub fn matches(self, range: impl RangeBounds<f64>) -> Result<StatePredicate, EntityDiagnostic> {
        let start = match range.start_bound() {
            Bound::Included(value) => Bound::Included(self.encode(*value)),
            Bound::Excluded(value) => Bound::Excluded(self.encode(*value)),
            Bound::Unbounded => Bound::Unbounded,
        };
        let end = match range.end_bound() {
            Bound::Included(value) => Bound::Included(self.encode(*value)),
            Bound::Excluded(value) => Bound::Excluded(self.encode(*value)),
            Bound::Unbounded => Bound::Unbounded,
        };
        predicate_for_range(
            self.objective(),
            format!("{}:{}", self.namespace, self.schema),
            self.descriptor.name,
            (start, end),
        )
    }

    fn encode(self, value: f64) -> i32 {
        encode_fixed(value, self.scale, self.descriptor.bounds)
    }
}

impl EntityStateField for FixedScore {
    type Accessor = FixedScoreAccessor;

    fn descriptor(self) -> StateFieldDescriptor {
        self.descriptor
    }

    fn objective(self) -> String {
        objective_name(self.namespace, self.schema, self.descriptor.name)
    }

    fn dirty_objective(self) -> String {
        dirty_name(self.namespace, self.schema, self.descriptor.name)
    }

    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor {
        FixedScoreAccessor {
            field: self,
            holder,
            track_dirty,
        }
    }
}

impl ComponentDirtyField for FixedScore {
    fn component_dirty_objective(self) -> String {
        component_dirty_name(self.namespace, self.schema)
    }
}

/// A fixed-point field bound to one generated scoreboard holder.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::FixedScoreAccessor",
    aliases = ["sand::prelude::FixedScoreAccessor"],
    module = "sand::entity",
    summary = "A fixed-point field bound to one generated scoreboard holder.",
    context = "A fixed-point field bound to one generated scoreboard holder. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::FixedScoreAccessor;",
)]
#[derive(Debug, Clone, Copy)]
pub struct FixedScoreAccessor {
    field: FixedScore,
    holder: &'static str,
    track_dirty: bool,
}

impl FixedScoreAccessor {
    /// Return the raw scoreboard view together with its declared scale.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedScoreAccessor::get",
        aliases = ["sand::prelude::FixedScoreAccessor::get"],
        module = "sand::entity",
        kind = "method",
        summary = "Return the raw scoreboard view together with its declared scale.",
        context = "Return the raw scoreboard view together with its declared scale. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "Return the raw scoreboard view together with its declared scale.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_score_accessor_value: sand::entity::FixedScoreAccessor)  {\n    let get = fixed_score_accessor_value.get();\n}",
    )]
    #[must_use]
    pub fn get(self) -> FixedScoreValue {
        FixedScoreValue {
            score: EntityScoreValue {
                objective: self.field.objective(),
                holder: self.holder,
            },
            scale: self.field.scale,
        }
    }

    /// Assign a decimal value using the field's deterministic encoding.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedScoreAccessor::set",
        aliases = ["sand::prelude::FixedScoreAccessor::set"],
        module = "sand::entity",
        kind = "method",
        summary = "Assign a decimal value using the field's deterministic encoding.",
        context = "Assign a decimal value using the field's deterministic encoding. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to assign a decimal value using the field's deterministic encoding."),
        returns = "The ordered values produced to assign a decimal value using the field's deterministic encoding.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_score_accessor_value: sand::entity::FixedScoreAccessor, value: f64)  {\n    let values = fixed_score_accessor_value.set(value);\n}",
    )]
    #[must_use]
    pub fn set(self, value: f64) -> Vec<String> {
        mutation(
            self.field,
            self.holder,
            self.track_dirty,
            "set",
            self.field.encode(value),
        )
    }

    /// Add a decimal value using the field's deterministic encoding.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedScoreAccessor::add",
        aliases = ["sand::prelude::FixedScoreAccessor::add"],
        module = "sand::entity",
        kind = "method",
        summary = "Add a decimal value using the field's deterministic encoding.",
        context = "Add a decimal value using the field's deterministic encoding. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to add a decimal value using the field's deterministic encoding."),
        returns = "The ordered values produced to add a decimal value using the field's deterministic encoding.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_score_accessor_value: sand::entity::FixedScoreAccessor, value: f64)  {\n    let values = fixed_score_accessor_value.add(value);\n}",
    )]
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, value: f64) -> Vec<String> {
        mutation(
            self.field,
            self.holder,
            self.track_dirty,
            "add",
            encode_fixed(value, self.field.scale, None),
        )
    }

    /// Subtract a decimal value using the field's deterministic encoding.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedScoreAccessor::subtract",
        aliases = ["sand::prelude::FixedScoreAccessor::subtract"],
        module = "sand::entity",
        kind = "method",
        summary = "Subtract a decimal value using the field's deterministic encoding.",
        context = "Subtract a decimal value using the field's deterministic encoding. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to subtract a decimal value using the field's deterministic encoding."),
        returns = "The ordered values produced to subtract a decimal value using the field's deterministic encoding.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_score_accessor_value: sand::entity::FixedScoreAccessor, value: f64)  {\n    let values = fixed_score_accessor_value.subtract(value);\n}",
    )]
    #[must_use]
    pub fn subtract(self, value: f64) -> Vec<String> {
        mutation(
            self.field,
            self.holder,
            self.track_dirty,
            "remove",
            encode_fixed(value, self.field.scale, None),
        )
    }

    /// Build a condition over a decimal range.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedScoreAccessor::matches",
        aliases = ["sand::prelude::FixedScoreAccessor::matches"],
        module = "sand::entity",
        kind = "method",
        summary = "Build a condition over a decimal range.",
        context = "Build a condition over a decimal range. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(range = "`range` provides the accepted numeric range used to build a condition over a decimal range."),
        returns = "On success, the value produced to build a condition over a decimal range; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_score_accessor_value: sand::entity::FixedScoreAccessor, range: impl std::ops::RangeBounds < f64 >)  {\n    let matches = fixed_score_accessor_value.matches(range);\n}",
    )]
    pub fn matches(self, range: impl RangeBounds<f64>) -> Result<Condition, EntityDiagnostic> {
        let predicate = self.field.matches(range)?;
        Ok(Condition::score(
            self.holder.to_string(),
            predicate.objective,
            predicate.condition_range,
        ))
    }
}

/// Read-only fixed-point score view. Minecraft still stores integer units;
/// `scale()` tells display or data-copying code how to interpret them.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::FixedScoreValue",
    aliases = ["sand::prelude::FixedScoreValue"],
    module = "sand::entity",
    summary = "Read-only fixed-point score view. Minecraft still stores integer units; `scale()` tells display or data-copying code how to interpret them.",
    context = "Read-only fixed-point score view. Minecraft still stores integer units; `scale()` tells display or data-copying code how to interpret them. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::FixedScoreValue;",
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedScoreValue {
    score: EntityScoreValue,
    scale: i32,
}

impl FixedScoreValue {
    /// Emit the underlying scoreboard read command.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedScoreValue::command",
        aliases = ["sand::prelude::FixedScoreValue::command"],
        module = "sand::entity",
        kind = "method",
        summary = "Emit the underlying scoreboard read command.",
        context = "Emit the underlying scoreboard read command. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The rendered Minecraft command text produced to emit the underlying scoreboard read command.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_score_value_value: &sand::entity::FixedScoreValue)  {\n    let command = fixed_score_value_value.command();\n}",
    )]
    #[must_use]
    pub fn command(&self) -> String {
        self.score.command()
    }

    /// Number of stored units per whole value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedScoreValue::scale",
        aliases = ["sand::prelude::FixedScoreValue::scale"],
        module = "sand::entity",
        kind = "method",
        summary = "Number of stored units per whole value.",
        context = "Number of stored units per whole value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `i32` value produced to number of stored units per whole value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_score_value_value: &sand::entity::FixedScoreValue)  {\n    let scale = fixed_score_value_value.scale();\n}",
    )]
    #[must_use]
    pub const fn scale(&self) -> i32 {
        self.scale
    }

    /// Resolved objective name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedScoreValue::objective",
        aliases = ["sand::prelude::FixedScoreValue::objective"],
        module = "sand::entity",
        kind = "method",
        summary = "Resolved objective name.",
        context = "Resolved objective name. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to resolved objective name.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_score_value_value: &sand::entity::FixedScoreValue)  {\n    let objective = fixed_score_value_value.objective();\n}",
    )]
    #[must_use]
    pub fn objective(&self) -> &str {
        self.score.objective()
    }
}

impl<T> Copy for EntityScore<T> {}
impl<T> Clone for EntityScore<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> EntityScore<T> {
    /// Construct a score field; normally generated by `State`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityScore::new",
        aliases = ["sand::entity::Score::new", "sand::prelude::EntityScore::new", "sand::prelude::Score::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct a score field; normally generated by `State`.",
        context = "Construct a score field; normally generated by `State`. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(namespace = "`namespace` is used when constructing a score field; normally generated by `State`.", schema = "`schema` is used when constructing a score field; normally generated by `State`.", name = "`name` is used when constructing a score field; normally generated by `State`.", default = "`default` is used when constructing a score field; normally generated by `State`.", bounds = "`bounds` is used when constructing a score field; normally generated by `State`."),
        returns = "An `EntityScore` representing a score field; normally generated by `State`.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : 'static>(namespace: & 'static str, schema: & 'static str, name: & 'static str, default: i32, bounds: Option < (i32 , i32) >)  {\n    let entity_score = sand::entity::EntityScore ::< T >::new(namespace, schema, name, default, bounds);\n}",
    )]
    #[must_use]
    pub const fn new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        default: i32,
        bounds: Option<(i32, i32)>,
    ) -> Self {
        Self::__new(
            namespace,
            schema,
            name,
            StateFieldKind::Score,
            default,
            bounds,
        )
    }

    /// Compiler-facing constructor for version and dirty score fields.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn __new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        kind: StateFieldKind,
        default: i32,
        bounds: Option<(i32, i32)>,
    ) -> Self {
        Self {
            namespace,
            schema,
            descriptor: StateFieldDescriptor::new(name, kind, default, bounds),
            _marker: PhantomData,
        }
    }

    /// Match an inclusive/open Rust range without handwritten selector maps.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityScore::matches",
        aliases = ["sand::entity::Score::matches", "sand::prelude::EntityScore::matches", "sand::prelude::Score::matches"],
        module = "sand::entity",
        kind = "method",
        summary = "Match an inclusive/open Rust range without handwritten selector maps.",
        context = "Match an inclusive/open Rust range without handwritten selector maps. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(range = "`range` provides the accepted numeric range used to match an inclusive/open Rust range without handwritten selector maps."),
        returns = "On success, the value produced to match an inclusive/open Rust range without handwritten selector maps; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : 'static>(entity_score_value: sand::entity::EntityScore < T >, range: impl std::ops::RangeBounds < i32 >)  {\n    let matches = entity_score_value.matches(range);\n}",
    )]
    pub fn matches(self, range: impl RangeBounds<i32>) -> Result<StatePredicate, EntityDiagnostic> {
        predicate_for_range(
            self.objective(),
            format!("{}:{}", self.namespace, self.schema),
            self.descriptor.name,
            range,
        )
    }
}

impl<T: 'static> EntityStateField for EntityScore<T> {
    type Accessor = EntityScoreAccessor<T>;
    fn descriptor(self) -> StateFieldDescriptor {
        self.descriptor
    }
    fn objective(self) -> String {
        objective_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn dirty_objective(self) -> String {
        dirty_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor {
        EntityScoreAccessor {
            field: self,
            holder,
            track_dirty,
        }
    }
}

impl<T: 'static> ComponentDirtyField for EntityScore<T> {
    fn component_dirty_objective(self) -> String {
        component_dirty_name(self.namespace, self.schema)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityScoreAccessor",
    module = "sand::entity",
    summary = "An [`EntityScore`] bound to its schema-selected score holder.",
    context = "An [`EntityScore`] bound to its schema-selected score holder. Entity/living schemas normally use the current executor (`@s`), player schemas use the current player (`@s`), and global schemas use their deterministic fake-player holder.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityScoreAccessor;",
)]
/// An [`EntityScore`] bound to its schema-selected score holder.
///
/// Entity/living schemas normally use the current executor (`@s`), player
/// schemas use the current player (`@s`), and global schemas use their
/// deterministic fake-player holder.
#[derive(Debug, Clone, Copy)]
pub struct EntityScoreAccessor<T = i32> {
    field: EntityScore<T>,
    holder: &'static str,
    track_dirty: bool,
}

impl<T: 'static> EntityScoreAccessor<T> {
    /// Return a readable typed score view.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityScoreAccessor::get",
        module = "sand::entity",
        kind = "method",
        summary = "Return a readable typed score view.",
        context = "Return a readable typed score view. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "Return a readable typed score view.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : 'static>(entity_score_accessor_value: sand::entity::EntityScoreAccessor < T >)  {\n    let get = entity_score_accessor_value.get();\n}",
    )]
    #[must_use]
    pub fn get(self) -> EntityScoreValue {
        EntityScoreValue {
            objective: self.field.objective(),
            holder: self.holder,
        }
    }

    /// Assign a value, marking the source dirty for archetype-bound state.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityScoreAccessor::set",
        module = "sand::entity",
        kind = "method",
        summary = "Assign a value, marking the source dirty for archetype-bound state.",
        context = "Assign a value, marking the source dirty for archetype-bound state. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to assign a value, marking the source dirty for archetype-bound state."),
        returns = "The ordered values produced to assign a value, marking the source dirty for archetype-bound state.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : 'static>(entity_score_accessor_value: sand::entity::EntityScoreAccessor < T >, value: i32)  {\n    let values = entity_score_accessor_value.set(value);\n}",
    )]
    #[must_use]
    pub fn set(self, value: i32) -> Vec<String> {
        mutation(self.field, self.holder, self.track_dirty, "set", value)
    }

    /// Add a value, marking the source dirty for archetype-bound state.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityScoreAccessor::add",
        module = "sand::entity",
        kind = "method",
        summary = "Add a value, marking the source dirty for archetype-bound state.",
        context = "Add a value, marking the source dirty for archetype-bound state. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to add a value, marking the source dirty for archetype-bound state."),
        returns = "The ordered values produced to add a value, marking the source dirty for archetype-bound state.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : 'static>(entity_score_accessor_value: sand::entity::EntityScoreAccessor < T >, value: i32)  {\n    let values = entity_score_accessor_value.add(value);\n}",
    )]
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, value: i32) -> Vec<String> {
        mutation(self.field, self.holder, self.track_dirty, "add", value)
    }

    /// Subtract a value, marking the source dirty for archetype-bound state.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityScoreAccessor::subtract",
        module = "sand::entity",
        kind = "method",
        summary = "Subtract a value, marking the source dirty for archetype-bound state.",
        context = "Subtract a value, marking the source dirty for archetype-bound state. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to subtract a value, marking the source dirty for archetype-bound state."),
        returns = "The ordered values produced to subtract a value, marking the source dirty for archetype-bound state.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : 'static>(entity_score_accessor_value: sand::entity::EntityScoreAccessor < T >, value: i32)  {\n    let values = entity_score_accessor_value.subtract(value);\n}",
    )]
    #[must_use]
    pub fn subtract(self, value: i32) -> Vec<String> {
        mutation(self.field, self.holder, self.track_dirty, "remove", value)
    }

    /// Build a typed condition over this score.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityScoreAccessor::matches",
        module = "sand::entity",
        kind = "method",
        summary = "Build a typed condition over this score.",
        context = "Build a typed condition over this score. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(range = "`range` provides the accepted numeric range used to build a typed condition over this score."),
        returns = "On success, the value produced to build a typed condition over this score; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : 'static>(entity_score_accessor_value: sand::entity::EntityScoreAccessor < T >, range: impl std::ops::RangeBounds < i32 >)  {\n    let matches = entity_score_accessor_value.matches(range);\n}",
    )]
    pub fn matches(self, range: impl RangeBounds<i32>) -> Result<Condition, EntityDiagnostic> {
        let predicate = predicate_for_range(
            self.field.objective(),
            format!("{}:{}", self.field.namespace, self.field.schema),
            self.field.descriptor.name,
            range,
        )?;
        Ok(Condition::score(
            self.holder.to_string(),
            predicate.objective,
            predicate.condition_range,
        ))
    }
}

/// Read-only view of the current entity's score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityScoreValue {
    objective: String,
    holder: &'static str,
}

impl EntityScoreValue {
    /// Emit `scoreboard players get <holder> <objective>`.
    #[must_use]
    pub fn command(&self) -> String {
        format!("scoreboard players get {} {}", self.holder, self.objective)
    }

    /// Resolved objective name.
    #[must_use]
    pub fn objective(&self) -> &str {
        &self.objective
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityFlag",
    aliases = ["sand::prelude::EntityFlag"],
    module = "sand::entity",
    summary = "Typed boolean entity field.",
    context = "Typed boolean entity field. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityFlag;",
)]
/// Typed boolean entity field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityFlag {
    namespace: &'static str,
    schema: &'static str,
    descriptor: StateFieldDescriptor,
}

impl EntityFlag {
    /// Construct a flag field; normally generated by `State`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityFlag::new",
        aliases = ["sand::prelude::EntityFlag::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct a flag field; normally generated by `State`.",
        context = "Construct a flag field; normally generated by `State`. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(namespace = "`namespace` is used when constructing a flag field; normally generated by `State`.", schema = "`schema` is used when constructing a flag field; normally generated by `State`.", name = "`name` is used when constructing a flag field; normally generated by `State`.", default = "`default` provides the switch that enables or disables the behavior used to construct a flag field; normally generated by `State`."),
        returns = "An `EntityFlag` representing a flag field; normally generated by `State`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(namespace: & 'static str, schema: & 'static str, name: & 'static str, default: bool)  {\n    let entity_flag = sand::entity::EntityFlag::new(namespace, schema, name, default);\n}",
    )]
    #[must_use]
    pub const fn new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        default: bool,
    ) -> Self {
        Self {
            namespace,
            schema,
            descriptor: StateFieldDescriptor::new(
                name,
                StateFieldKind::Flag,
                default as i32,
                Some((0, 1)),
            ),
        }
    }

    /// Predicate for an enabled flag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityFlag::is_enabled",
        aliases = ["sand::prelude::EntityFlag::is_enabled"],
        module = "sand::entity",
        kind = "method",
        summary = "Predicate for an enabled flag.",
        context = "Predicate for an enabled flag. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `StatePredicate` value produced to predicate for an enabled flag.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_flag_value: sand::entity::EntityFlag)  {\n    let is_enabled = entity_flag_value.is_enabled();\n}",
    )]
    #[must_use]
    pub fn is_enabled(self) -> StatePredicate {
        exact_predicate(self.objective(), 1)
    }

    /// Predicate for a disabled flag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityFlag::is_disabled",
        aliases = ["sand::prelude::EntityFlag::is_disabled"],
        module = "sand::entity",
        kind = "method",
        summary = "Predicate for a disabled flag.",
        context = "Predicate for a disabled flag. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `StatePredicate` value produced to predicate for a disabled flag.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_flag_value: sand::entity::EntityFlag)  {\n    let is_disabled = entity_flag_value.is_disabled();\n}",
    )]
    #[must_use]
    pub fn is_disabled(self) -> StatePredicate {
        exact_predicate(self.objective(), 0)
    }
}

impl EntityStateField for EntityFlag {
    type Accessor = EntityFlagAccessor;
    fn descriptor(self) -> StateFieldDescriptor {
        self.descriptor
    }
    fn objective(self) -> String {
        objective_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn dirty_objective(self) -> String {
        dirty_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor {
        EntityFlagAccessor {
            field: self,
            holder,
            track_dirty,
        }
    }
}

impl ComponentDirtyField for EntityFlag {
    fn component_dirty_objective(self) -> String {
        component_dirty_name(self.namespace, self.schema)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityFlagAccessor",
    module = "sand::entity",
    summary = "An [`EntityFlag`] bound to its schema-selected score holder.",
    context = "An [`EntityFlag`] bound to its schema-selected score holder. Entity/living schemas normally use the current executor (`@s`), player schemas use the current player (`@s`), and global schemas use their deterministic fake-player holder.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityFlagAccessor;",
)]
/// An [`EntityFlag`] bound to its schema-selected score holder.
///
/// Entity/living schemas normally use the current executor (`@s`), player
/// schemas use the current player (`@s`), and global schemas use their
/// deterministic fake-player holder.
#[derive(Debug, Clone, Copy)]
pub struct EntityFlagAccessor {
    field: EntityFlag,
    holder: &'static str,
    track_dirty: bool,
}

impl EntityFlagAccessor {
    /// Set the flag to one.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityFlagAccessor::enable",
        module = "sand::entity",
        kind = "method",
        summary = "Set the flag to one.",
        context = "Set the flag to one. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The ordered values produced to set the flag to one.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_flag_accessor_value: sand::entity::EntityFlagAccessor)  {\n    let values = entity_flag_accessor_value.enable();\n}",
    )]
    #[must_use]
    pub fn enable(self) -> Vec<String> {
        mutation(self.field, self.holder, self.track_dirty, "set", 1)
    }
    /// Set the flag to zero.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityFlagAccessor::disable",
        module = "sand::entity",
        kind = "method",
        summary = "Set the flag to zero.",
        context = "Set the flag to zero. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The ordered values produced to set the flag to zero.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_flag_accessor_value: sand::entity::EntityFlagAccessor)  {\n    let values = entity_flag_accessor_value.disable();\n}",
    )]
    #[must_use]
    pub fn disable(self) -> Vec<String> {
        mutation(self.field, self.holder, self.track_dirty, "set", 0)
    }
    /// Condition: enabled.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityFlagAccessor::is_enabled",
        module = "sand::entity",
        kind = "method",
        summary = "Condition: enabled.",
        context = "Condition: enabled. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `Condition` value produced to condition enabled.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_flag_accessor_value: sand::entity::EntityFlagAccessor)  {\n    let is_enabled = entity_flag_accessor_value.is_enabled();\n}",
    )]
    #[must_use]
    pub fn is_enabled(self) -> Condition {
        holder_condition(self.holder, self.field.objective(), ScoreRange::Eq(1))
    }
    /// Condition: disabled.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityFlagAccessor::is_disabled",
        module = "sand::entity",
        kind = "method",
        summary = "Condition: disabled.",
        context = "Condition: disabled. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `Condition` value produced to condition disabled.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_flag_accessor_value: sand::entity::EntityFlagAccessor)  {\n    let is_disabled = entity_flag_accessor_value.is_disabled();\n}",
    )]
    #[must_use]
    pub fn is_disabled(self) -> Condition {
        holder_condition(self.holder, self.field.objective(), ScoreRange::Eq(0))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityEnum",
    aliases = ["sand::prelude::EntityEnum"],
    module = "sand::entity",
    summary = "Typed finite entity enum.",
    context = "Typed finite entity enum. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityEnum;",
)]
/// Typed finite entity enum.
#[derive(Debug, PartialEq, Eq)]
pub struct EntityEnum<T: EntityEnumValue> {
    namespace: &'static str,
    schema: &'static str,
    descriptor: StateFieldDescriptor,
    _marker: PhantomData<fn() -> T>,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::Data",
    aliases = ["sand::prelude::Data"],
    module = "sand::entity",
    summary = "Canonical typed-data field marker. Data fields are lowered through Sand's typed storage backend by the State derive. The marker carries no runtime Rust value.",
    context = "Canonical typed-data field marker. Data fields are lowered through Sand's typed storage backend by the State derive. The marker carries no runtime Rust value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::Data;",
)]
/// Canonical typed-data field marker.
///
/// Data fields are lowered through Sand's typed storage backend by the State
/// derive. The marker carries no runtime Rust value.
#[derive(Debug, PartialEq, Eq)]
pub struct Data<T> {
    storage: &'static str,
    path: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Data<T> {}
impl<T> Clone for Data<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Data<T> {
    /// Constructs a generated component-owned typed storage handle.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Data::new",
        aliases = ["sand::prelude::Data::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Constructs a generated component-owned typed storage handle.",
        context = "Constructs a generated component-owned typed storage handle. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(storage = "`storage` is used when constructing a generated component-owned typed storage handle.", path = "`path` provides the typed resource identifier or location used to construct a generated component-owned typed storage handle."),
        returns = "A `Data` representing a generated component-owned typed storage handle.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage: & 'static str, path: & 'static str)  {\n    let data = sand::entity::Data ::< T >::new(storage, path);\n}",
    )]
    pub const fn new(storage: &'static str, path: &'static str) -> Self {
        Self {
            storage,
            path,
            _marker: PhantomData,
        }
    }

    /// Build a typed query for this component-owned storage value.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Data::get",
        aliases = ["sand::prelude::Data::get"],
        module = "sand::entity",
        kind = "method",
        summary = "Build a typed query for this component-owned storage value.",
        context = "Build a typed query for this component-owned storage value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to build a typed query for this component-owned storage value.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(data_value: sand::entity::Data < T >)  {\n    let get = data_value.get();\n}",
    )]
    pub fn get(self) -> String {
        crate::state::Nbt::storage(self.storage)
            .typed_path::<T>(self.path)
            .get()
            .to_string()
    }

    /// Replace this component-owned storage value.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Data::set",
        aliases = ["sand::prelude::Data::set"],
        module = "sand::entity",
        kind = "method",
        summary = "Replace this component-owned storage value.",
        context = "Replace this component-owned storage value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to replace this component-owned storage value."),
        returns = "The ordered values produced to replace this component-owned storage value.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(data_value: sand::entity::Data < T >, value: impl Into < sand::data::NbtValue >)  {\n    let values = data_value.set(value);\n}",
    )]
    pub fn set(self, value: impl Into<sand_commands::NbtValue>) -> Vec<String> {
        vec![
            crate::state::Nbt::storage(self.storage)
                .typed_path::<T>(self.path)
                .set(value)
                .to_string(),
        ]
    }

    /// Test whether this component-owned storage path currently exists.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Data::exists",
        aliases = ["sand::prelude::Data::exists"],
        module = "sand::entity",
        kind = "method",
        summary = "Test whether this component-owned storage path currently exists.",
        context = "Test whether this component-owned storage path currently exists. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `Condition` value produced to test whether this component-owned storage path currently exists.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(data_value: sand::entity::Data < T >)  {\n    let exists = data_value.exists();\n}",
    )]
    pub fn exists(self) -> Condition {
        Condition::nbt_exists(
            sand_commands::DataTarget::storage(self.storage),
            sand_commands::NbtPath::new(self.path),
        )
    }

    /// Remove this component-owned storage value.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Data::remove",
        aliases = ["sand::prelude::Data::remove"],
        module = "sand::entity",
        kind = "method",
        summary = "Remove this component-owned storage value.",
        context = "Remove this component-owned storage value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The ordered values produced to remove this component-owned storage value.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(data_value: sand::entity::Data < T >)  {\n    let values = data_value.remove();\n}",
    )]
    pub fn remove(self) -> Vec<String> {
        vec![format!(
            "data remove storage {} {}",
            self.storage, self.path
        )]
    }
}

/// A typed command-storage field keyed by the current entity or player's UUID.
///
/// The UUID lives in a shared owner record while the value remains under its
/// component-owned path. Every operation copies the four UUID integers into a
/// short-lived macro argument compound and then runs a generated helper. This
/// keeps values stable across unload/reload without writing custom top-level
/// entity NBT.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::KeyedData",
    aliases = ["sand::prelude::KeyedData"],
    module = "sand::entity",
    summary = "A typed command-storage field keyed by the current entity or player's UUID.",
    context = "A typed command-storage field keyed by the current entity or player's UUID. The UUID lives in a shared owner record while the value remains under its component-owned path. Every operation copies the four UUID integers into a short-lived macro argument compound and then runs a generated helper. This keeps values stable across unload/reload without writing custom top-level entity NBT.",
    minecraft = "The UUID lives in a shared owner record while the value remains under its component-owned path. Every operation copies the four UUID integers into a short-lived macro argument compound and then runs a generated helper. This keeps values stable across unload/reload without writing custom top-level entity NBT.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::KeyedData;",
)]
#[derive(Debug, PartialEq, Eq)]
pub struct KeyedData<T> {
    storage: &'static str,
    path: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for KeyedData<T> {}
impl<T> Clone for KeyedData<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> KeyedData<T> {
    /// Construct a generated UUID-keyed storage handle.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::KeyedData::new",
        aliases = ["sand::prelude::KeyedData::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct a generated UUID-keyed storage handle.",
        context = "Construct a generated UUID-keyed storage handle. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(storage = "`storage` is used when constructing a generated UUID-keyed storage handle.", path = "`path` provides the typed resource identifier or location used to construct a generated UUID-keyed storage handle."),
        returns = "A `KeyedData` representing a generated UUID-keyed storage handle.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage: & 'static str, path: & 'static str)  {\n    let keyed_data = sand::entity::KeyedData ::< T >::new(storage, path);\n}",
    )]
    #[must_use]
    pub const fn new(storage: &'static str, path: &'static str) -> Self {
        Self {
            storage,
            path,
            _marker: PhantomData,
        }
    }

    /// Read the current owner's value through a generated UUID-keyed helper.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::KeyedData::get",
        aliases = ["sand::prelude::KeyedData::get"],
        module = "sand::entity",
        kind = "method",
        summary = "Read the current owner's value through a generated UUID-keyed helper.",
        context = "Read the current owner's value through a generated UUID-keyed helper. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The ordered values produced to read the current owner's value through a generated UUID-keyed helper.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(keyed_data_value: sand::entity::KeyedData < T >)  {\n    let values = keyed_data_value.get();\n}",
    )]
    #[must_use]
    pub fn get(self) -> Vec<String> {
        keyed_data_call(
            self.storage,
            format!(
                "$data get storage {} {}",
                self.storage,
                keyed_path(self.path)
            ),
        )
    }

    /// Replace the current owner's component-owned value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::KeyedData::set",
        aliases = ["sand::prelude::KeyedData::set"],
        module = "sand::entity",
        kind = "method",
        summary = "Replace the current owner's component-owned value.",
        context = "Replace the current owner's component-owned value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to replace the current owner's component-owned value."),
        returns = "The ordered values produced to replace the current owner's component-owned value.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(keyed_data_value: sand::entity::KeyedData < T >, value: impl Into < sand::data::NbtValue >)  {\n    let values = keyed_data_value.set(value);\n}",
    )]
    #[must_use]
    pub fn set(self, value: impl Into<sand_commands::NbtValue>) -> Vec<String> {
        keyed_data_call(
            self.storage,
            format!(
                "$data modify storage {} {} set value {}",
                self.storage,
                keyed_path(self.path),
                value.into()
            ),
        )
    }

    /// Remove only the current owner's value for this component field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::KeyedData::remove",
        aliases = ["sand::prelude::KeyedData::remove"],
        module = "sand::entity",
        kind = "method",
        summary = "Remove only the current owner's value for this component field.",
        context = "Remove only the current owner's value for this component field. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The ordered values produced to remove only the current owner's value for this component field.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(keyed_data_value: sand::entity::KeyedData < T >)  {\n    let values = keyed_data_value.remove();\n}",
    )]
    #[must_use]
    pub fn remove(self) -> Vec<String> {
        keyed_data_call(
            self.storage,
            format!(
                "$data remove storage {} {}",
                self.storage,
                keyed_path(self.path)
            ),
        )
    }

    /// Run commands only when the current owner's value actually exists.
    ///
    /// This callback is evaluated by Minecraft at runtime. It deliberately is
    /// not a Rust `Option` or ordinary [`Condition`], because command-storage
    /// membership depends on the executing entity's UUID.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::KeyedData::if_present",
        aliases = ["sand::prelude::KeyedData::if_present"],
        module = "sand::entity",
        kind = "method",
        summary = "Run commands only when the current owner's value actually exists.",
        context = "Run commands only when the current owner's value actually exists. This callback is evaluated by Minecraft at runtime. It deliberately is not a Rust `Option` or ordinary [`Condition`], because command-storage membership depends on the executing entity's UUID.",
        minecraft = "This callback is evaluated by Minecraft at runtime. It deliberately is not a Rust `Option` or ordinary [`Condition`], because command-storage membership depends on the executing entity's UUID.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(body = "`body` provides the body used when running commands only when the current owner's value actually exists."),
        returns = "The ordered values produced to run commands only when the current owner's value actually exists.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(keyed_data_value: sand::entity::KeyedData < T >, body: impl FnOnce () -> Vec < String >)  {\n    let values = keyed_data_value.if_present(body);\n}",
    )]
    #[must_use]
    pub fn if_present(self, body: impl FnOnce() -> Vec<String>) -> Vec<String> {
        let body = body();
        if body.is_empty() {
            return Vec::new();
        }
        let callback = crate::function::register_dyn_fn_dedup("sand/state_data/body", body);
        keyed_data_call(
            self.storage,
            format!(
                "$execute if data storage {} {} run function __sand_local:{}",
                self.storage,
                keyed_path(self.path),
                callback
            ),
        )
    }
}

pub(crate) fn state_data_initialize_commands(field: StateDataFieldDescriptor) -> Vec<String> {
    if !field.keyed {
        return vec![format!(
            "execute unless data storage {} {} run data modify storage {} {} set value {}",
            field.storage, field.path, field.storage, field.path, field.default_snbt
        )];
    }
    let owner_path = keyed_owner_path();
    let value_path = keyed_path(field.path);
    keyed_data_call(
        field.storage,
        format!(
            "$execute unless data storage {0} {owner_path} run data modify storage {0} owners append value {{uuid:{1},components:{{}}}}\n$execute unless data storage {0} {value_path} run data modify storage {0} {value_path} set value {2}",
            field.storage,
            keyed_uuid(),
            field.default_snbt,
        ),
    )
}

fn state_data_remove_commands(field: StateDataFieldDescriptor) -> Vec<String> {
    if field.keyed {
        keyed_data_call(
            field.storage,
            format!(
                "$data remove storage {} {}",
                field.storage,
                keyed_path(field.path)
            ),
        )
    } else {
        vec![format!(
            "data remove storage {} {}",
            field.storage, field.path
        )]
    }
}

fn keyed_uuid() -> &'static str {
    "[I;$(u0),$(u1),$(u2),$(u3)]"
}

fn keyed_owner_path() -> String {
    format!("owners[{{uuid:{}}}]", keyed_uuid())
}

fn keyed_path(path: &str) -> String {
    format!("{}.{}", keyed_owner_path(), path)
}

fn keyed_data_call(storage: &str, macro_body: String) -> Vec<String> {
    let path = crate::function::register_dyn_fn_dedup(
        "sand/state_data/keyed",
        macro_body.lines().map(str::to_owned).collect(),
    );
    let args = "__sand_owner";
    let mut commands = (0..4)
        .map(|index| {
            format!(
                "data modify storage {storage} {args}.u{index} set from entity @s UUID[{index}]"
            )
        })
        .collect::<Vec<_>>();
    commands.push(format!(
        "function __sand_local:{path} with storage {storage} {args}"
    ));
    commands
}

impl<T: EntityEnumValue> Copy for EntityEnum<T> {}
impl<T: EntityEnumValue> Clone for EntityEnum<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: EntityEnumValue> EntityEnum<T> {
    /// Construct an enum field from its default encoded score.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityEnum::new",
        aliases = ["sand::prelude::EntityEnum::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct an enum field from its default encoded score.",
        context = "Construct an enum field from its default encoded score. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(namespace = "`namespace` is used when constructing an enum field from its default encoded score.", schema = "`schema` is used when constructing an enum field from its default encoded score.", name = "`name` is used when constructing an enum field from its default encoded score.", default_score = "`default_score` is used when constructing an enum field from its default encoded score."),
        returns = "An `EntityEnum` representing an enum field from its default encoded score.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::entity::EntityEnumValue + 'static>(namespace: & 'static str, schema: & 'static str, name: & 'static str, default_score: i32)  {\n    let entity_enum = sand::entity::EntityEnum ::< T >::new(namespace, schema, name, default_score);\n}",
    )]
    #[must_use]
    pub const fn new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        default_score: i32,
    ) -> Self {
        Self {
            namespace,
            schema,
            descriptor: StateFieldDescriptor::new(
                name,
                StateFieldKind::Enum(T::ENCODINGS),
                default_score,
                None,
            ),
            _marker: PhantomData,
        }
    }

    /// Predicate for exactly one variant.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityEnum::is",
        aliases = ["sand::prelude::EntityEnum::is"],
        module = "sand::entity",
        kind = "method",
        summary = "Predicate for exactly one variant.",
        context = "Predicate for exactly one variant. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to predicate for exactly one variant."),
        returns = "The `StatePredicate` value produced to predicate for exactly one variant.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::entity::EntityEnumValue + 'static>(entity_enum_value: sand::entity::EntityEnum < T >, value: T)  {\n    let is = entity_enum_value.is(value);\n}",
    )]
    #[must_use]
    pub fn is(self, value: T) -> StatePredicate {
        exact_predicate(self.objective(), value.encode())
    }
}

impl<T: EntityEnumValue> EntityStateField for EntityEnum<T> {
    type Accessor = EntityEnumAccessor<T>;
    fn descriptor(self) -> StateFieldDescriptor {
        self.descriptor
    }
    fn objective(self) -> String {
        objective_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn dirty_objective(self) -> String {
        dirty_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor {
        EntityEnumAccessor {
            field: self,
            holder,
            track_dirty,
        }
    }
}

impl<T: EntityEnumValue> ComponentDirtyField for EntityEnum<T> {
    fn component_dirty_objective(self) -> String {
        component_dirty_name(self.namespace, self.schema)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityEnumAccessor",
    module = "sand::entity",
    summary = "An [`EntityEnum`] bound to its schema-selected score holder.",
    context = "An [`EntityEnum`] bound to its schema-selected score holder. Entity/living schemas normally use the current executor (`@s`), player schemas use the current player (`@s`), and global schemas use their deterministic fake-player holder.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityEnumAccessor;",
)]
/// An [`EntityEnum`] bound to its schema-selected score holder.
///
/// Entity/living schemas normally use the current executor (`@s`), player
/// schemas use the current player (`@s`), and global schemas use their
/// deterministic fake-player holder.
#[derive(Debug, Clone, Copy)]
pub struct EntityEnumAccessor<T: EntityEnumValue> {
    field: EntityEnum<T>,
    holder: &'static str,
    track_dirty: bool,
}

impl<T: EntityEnumValue> EntityEnumAccessor<T> {
    /// Store a variant, marking the source dirty for archetype-bound state.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityEnumAccessor::set",
        module = "sand::entity",
        kind = "method",
        summary = "Store a variant, marking the source dirty for archetype-bound state.",
        context = "Store a variant, marking the source dirty for archetype-bound state. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to store a variant, marking the source dirty for archetype-bound state."),
        returns = "The ordered values produced to store a variant, marking the source dirty for archetype-bound state.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::entity::EntityEnumValue + 'static>(entity_enum_accessor_value: sand::entity::EntityEnumAccessor < T >, value: T)  {\n    let values = entity_enum_accessor_value.set(value);\n}",
    )]
    #[must_use]
    pub fn set(self, value: T) -> Vec<String> {
        mutation(
            self.field,
            self.holder,
            self.track_dirty,
            "set",
            value.encode(),
        )
    }
    /// Condition: current value equals `value`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityEnumAccessor::is",
        module = "sand::entity",
        kind = "method",
        summary = "Condition: current value equals `value`.",
        context = "Condition: current value equals `value`. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "Condition: current value equals `value`."),
        returns = "The `Condition` value produced to condition current value equals `value`.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::entity::EntityEnumValue + 'static>(entity_enum_accessor_value: sand::entity::EntityEnumAccessor < T >, value: T)  {\n    let is = entity_enum_accessor_value.is(value);\n}",
    )]
    #[must_use]
    pub fn is(self, value: T) -> Condition {
        holder_condition(
            self.holder,
            self.field.objective(),
            ScoreRange::Eq(value.encode()),
        )
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityTimer",
    aliases = ["sand::prelude::EntityTimer"],
    module = "sand::entity",
    summary = "Typed elapsed/countdown entity timer.",
    context = "Typed elapsed/countdown entity timer. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityTimer;",
)]
/// Typed elapsed/countdown entity timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityTimer {
    namespace: &'static str,
    schema: &'static str,
    descriptor: StateFieldDescriptor,
}

impl EntityTimer {
    /// Construct a non-negative timer field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTimer::new",
        aliases = ["sand::prelude::EntityTimer::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct a non-negative timer field.",
        context = "Construct a non-negative timer field. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(namespace = "`namespace` is used when constructing a non-negative timer field.", schema = "`schema` is used when constructing a non-negative timer field.", name = "`name` is used when constructing a non-negative timer field.", initial = "`initial` is used when constructing a non-negative timer field."),
        returns = "An `EntityTimer` representing a non-negative timer field.",
        example = "use sand::prelude::*;\n\nfn demonstrate(namespace: & 'static str, schema: & 'static str, name: & 'static str, initial: i32)  {\n    let entity_timer = sand::entity::EntityTimer::new(namespace, schema, name, initial);\n}",
    )]
    #[must_use]
    pub const fn new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        initial: i32,
    ) -> Self {
        Self {
            namespace,
            schema,
            descriptor: StateFieldDescriptor::new(
                name,
                StateFieldKind::Timer,
                initial,
                Some((0, i32::MAX)),
            ),
        }
    }

    /// Predicate: timer reached zero.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTimer::elapsed",
        aliases = ["sand::prelude::EntityTimer::elapsed"],
        module = "sand::entity",
        kind = "method",
        summary = "Predicate: timer reached zero.",
        context = "Predicate: timer reached zero. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `StatePredicate` value produced to predicate timer reached zero.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_timer_value: sand::entity::EntityTimer)  {\n    let elapsed = entity_timer_value.elapsed();\n}",
    )]
    #[must_use]
    pub fn elapsed(self) -> StatePredicate {
        exact_predicate(self.objective(), 0)
    }
}

impl EntityStateField for EntityTimer {
    type Accessor = EntityTimerAccessor;
    fn descriptor(self) -> StateFieldDescriptor {
        self.descriptor
    }
    fn objective(self) -> String {
        objective_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn dirty_objective(self) -> String {
        dirty_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor {
        EntityTimerAccessor {
            field: self,
            holder,
            track_dirty,
        }
    }
}

impl ComponentDirtyField for EntityTimer {
    fn component_dirty_objective(self) -> String {
        component_dirty_name(self.namespace, self.schema)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityTimerAccessor",
    module = "sand::entity",
    summary = "An [`EntityTimer`] bound to its schema-selected score holder.",
    context = "An [`EntityTimer`] bound to its schema-selected score holder. Entity/living schemas normally use the current executor (`@s`), player schemas use the current player (`@s`), and global schemas use their deterministic fake-player holder.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityTimerAccessor;",
)]
/// An [`EntityTimer`] bound to its schema-selected score holder.
///
/// Entity/living schemas normally use the current executor (`@s`), player
/// schemas use the current player (`@s`), and global schemas use their
/// deterministic fake-player holder.
#[derive(Debug, Clone, Copy)]
pub struct EntityTimerAccessor {
    field: EntityTimer,
    holder: &'static str,
    track_dirty: bool,
}

impl EntityTimerAccessor {
    /// Start a countdown.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTimerAccessor::start",
        module = "sand::entity",
        kind = "method",
        summary = "Start a countdown.",
        context = "Start a countdown. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(ticks = "`ticks` provides the Minecraft tick duration used to start a countdown."),
        returns = "The ordered values produced to start a countdown.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_timer_accessor_value: sand::entity::EntityTimerAccessor, ticks: sand::state::Ticks)  {\n    let values = entity_timer_accessor_value.start(ticks);\n}",
    )]
    #[must_use]
    pub fn start(self, ticks: crate::state::Ticks) -> Vec<String> {
        mutation(
            self.field,
            self.holder,
            self.track_dirty,
            "set",
            ticks.get().min(i32::MAX as u32) as i32,
        )
    }
    /// Decrement a positive timer for the bound score holder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTimerAccessor::tick",
        module = "sand::entity",
        kind = "method",
        summary = "Decrement a positive timer for the bound score holder.",
        context = "Decrement a positive timer for the bound score holder. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The ordered values produced to decrement a positive timer for the bound score holder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_timer_accessor_value: sand::entity::EntityTimerAccessor)  {\n    let values = entity_timer_accessor_value.tick();\n}",
    )]
    #[must_use]
    pub fn tick(self) -> Vec<String> {
        let decrement = format!(
            "execute if score {1} {0} matches 1.. run scoreboard players remove {1} {0} 1",
            self.field.objective(),
            self.holder,
        );
        if self.track_dirty {
            vec![
                format!(
                    "execute if score {2} {0} matches 1.. run scoreboard players set {2} {1} 1",
                    self.field.objective(),
                    self.field.dirty_objective(),
                    self.holder,
                ),
                format!(
                    "execute if score {2} {0} matches 1.. run scoreboard players set {2} {1} 1",
                    self.field.objective(),
                    self.field.component_dirty_objective(),
                    self.holder,
                ),
                decrement,
            ]
        } else {
            vec![decrement]
        }
    }
    /// Condition: timer reached zero.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTimerAccessor::elapsed",
        module = "sand::entity",
        kind = "method",
        summary = "Condition: timer reached zero.",
        context = "Condition: timer reached zero. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `Condition` value produced to condition timer reached zero.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_timer_accessor_value: sand::entity::EntityTimerAccessor)  {\n    let elapsed = entity_timer_accessor_value.elapsed();\n}",
    )]
    #[must_use]
    pub fn elapsed(self) -> Condition {
        holder_condition(self.holder, self.field.objective(), ScoreRange::Eq(0))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityCooldown",
    aliases = ["sand::prelude::EntityCooldown"],
    module = "sand::entity",
    summary = "Typed entity cooldown, ready at zero.",
    context = "Typed entity cooldown, ready at zero. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityCooldown;",
)]
/// Typed entity cooldown, ready at zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityCooldown(EntityTimer);

impl EntityCooldown {
    /// Construct a cooldown field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityCooldown::new",
        aliases = ["sand::prelude::EntityCooldown::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct a cooldown field.",
        context = "Construct a cooldown field. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(namespace = "`namespace` is used when constructing a cooldown field.", schema = "`schema` is used when constructing a cooldown field.", name = "`name` is used when constructing a cooldown field."),
        returns = "An `EntityCooldown` representing a cooldown field.",
        example = "use sand::prelude::*;\n\nfn demonstrate(namespace: & 'static str, schema: & 'static str, name: & 'static str)  {\n    let entity_cooldown = sand::entity::EntityCooldown::new(namespace, schema, name);\n}",
    )]
    #[must_use]
    pub const fn new(namespace: &'static str, schema: &'static str, name: &'static str) -> Self {
        Self(EntityTimer {
            namespace,
            schema,
            descriptor: StateFieldDescriptor::new(
                name,
                StateFieldKind::Cooldown,
                0,
                Some((0, i32::MAX)),
            ),
        })
    }

    /// Predicate: cooldown is ready.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityCooldown::ready",
        aliases = ["sand::prelude::EntityCooldown::ready"],
        module = "sand::entity",
        kind = "method",
        summary = "Predicate: cooldown is ready.",
        context = "Predicate: cooldown is ready. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `StatePredicate` value produced to predicate cooldown is ready.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_cooldown_value: sand::entity::EntityCooldown)  {\n    let ready = entity_cooldown_value.ready();\n}",
    )]
    #[must_use]
    pub fn ready(self) -> StatePredicate {
        exact_predicate(self.objective(), 0)
    }
}

impl EntityStateField for EntityCooldown {
    type Accessor = EntityCooldownAccessor;
    fn descriptor(self) -> StateFieldDescriptor {
        self.0.descriptor
    }
    fn objective(self) -> String {
        self.0.objective()
    }
    fn dirty_objective(self) -> String {
        self.0.dirty_objective()
    }
    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor {
        EntityCooldownAccessor {
            field: self,
            holder,
            track_dirty,
        }
    }
}

impl ComponentDirtyField for EntityCooldown {
    fn component_dirty_objective(self) -> String {
        self.0.component_dirty_objective()
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityCooldownAccessor",
    module = "sand::entity",
    summary = "An [`EntityCooldown`] bound to its schema-selected score holder.",
    context = "An [`EntityCooldown`] bound to its schema-selected score holder. Entity/living schemas normally use the current executor (`@s`), player schemas use the current player (`@s`), and global schemas use their deterministic fake-player holder.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityCooldownAccessor;",
)]
/// An [`EntityCooldown`] bound to its schema-selected score holder.
///
/// Entity/living schemas normally use the current executor (`@s`), player
/// schemas use the current player (`@s`), and global schemas use their
/// deterministic fake-player holder.
#[derive(Debug, Clone, Copy)]
pub struct EntityCooldownAccessor {
    field: EntityCooldown,
    holder: &'static str,
    track_dirty: bool,
}

impl EntityCooldownAccessor {
    /// Start the cooldown.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityCooldownAccessor::start",
        module = "sand::entity",
        kind = "method",
        summary = "Start the cooldown.",
        context = "Start the cooldown. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(ticks = "`ticks` provides the Minecraft tick duration used to start the cooldown."),
        returns = "The ordered values produced to start the cooldown.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_cooldown_accessor_value: sand::entity::EntityCooldownAccessor, ticks: sand::state::Ticks)  {\n    let values = entity_cooldown_accessor_value.start(ticks);\n}",
    )]
    #[must_use]
    pub fn start(self, ticks: crate::state::Ticks) -> Vec<String> {
        mutation(
            self.field,
            self.holder,
            self.track_dirty,
            "set",
            ticks.get().min(i32::MAX as u32) as i32,
        )
    }
    /// Condition: ready.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityCooldownAccessor::ready",
        module = "sand::entity",
        kind = "method",
        summary = "Condition: ready.",
        context = "Condition: ready. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `Condition` value produced to condition ready.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_cooldown_accessor_value: sand::entity::EntityCooldownAccessor)  {\n    let ready = entity_cooldown_accessor_value.ready();\n}",
    )]
    #[must_use]
    pub fn ready(self) -> Condition {
        holder_condition(self.holder, self.field.objective(), ScoreRange::Eq(0))
    }
}

fn validate_enum_encodings(
    schema: &str,
    field: &str,
    encodings: &[EnumEncoding],
) -> Result<(), EntityDiagnostic> {
    let mut variants = BTreeSet::new();
    let mut scores = BTreeMap::new();
    for encoding in encodings {
        if !variants.insert(encoding.name) {
            return Err(EntityDiagnostic::InvalidEnumEncoding {
                schema: schema.into(),
                field: field.into(),
                detail: format!("variant `{}` is declared twice", encoding.name),
            });
        }
        if let Some(previous) = scores.insert(encoding.score, encoding.name) {
            return Err(EntityDiagnostic::InvalidEnumEncoding {
                schema: schema.into(),
                field: field.into(),
                detail: format!(
                    "`{previous}` and `{}` both encode as {}",
                    encoding.name, encoding.score
                ),
            });
        }
    }
    Ok(())
}

fn mutation<F: EntityStateField + ComponentDirtyField>(
    field: F,
    holder: &str,
    track_dirty: bool,
    operation: &str,
    value: i32,
) -> Vec<String> {
    let objective = field.objective();
    let mut commands = vec![format!(
        "scoreboard players {operation} {holder} {objective} {value}"
    )];
    if let Some((min, max)) = field.descriptor().bounds {
        if min != i32::MIN {
            commands.push(format!(
                "execute if score {holder} {objective} matches ..{} run scoreboard players set {holder} {objective} {min}",
                min - 1
            ));
        }
        if max != i32::MAX {
            commands.push(format!(
                "execute if score {holder} {objective} matches {}.. run scoreboard players set {holder} {objective} {max}",
                max + 1
            ));
        }
    }
    if track_dirty {
        commands.push(format!(
            "scoreboard players set {holder} {} 1",
            field.dirty_objective()
        ));
        commands.push(format!(
            "scoreboard players set {holder} {} 1",
            field.component_dirty_objective()
        ));
    }
    commands
}

fn encode_fixed(value: f64, scale: i32, bounds: Option<(i32, i32)>) -> i32 {
    let scaled = value * f64::from(scale);
    let encoded = if scaled.is_nan() {
        0
    } else if scaled >= f64::from(i32::MAX) {
        i32::MAX
    } else if scaled <= f64::from(i32::MIN) {
        i32::MIN
    } else {
        scaled.round() as i32
    };
    bounds.map_or(encoded, |(min, max)| encoded.clamp(min, max))
}

fn holder_condition(holder: &str, objective: String, range: ScoreRange) -> Condition {
    Condition::score(holder.to_string(), objective, range)
}

pub(crate) fn objective_name(namespace: &str, schema: &str, field: &str) -> String {
    sand_commands::ObjectiveName::logical(format!("{namespace}:{schema}.{field}"))
        .as_str()
        .to_string()
}

pub(crate) fn dirty_name(namespace: &str, schema: &str, field: &str) -> String {
    sand_commands::ObjectiveName::logical(format!("{namespace}:{schema}.{field}.dirty"))
        .as_str()
        .to_string()
}

pub(crate) fn component_dirty_name(namespace: &str, schema: &str) -> String {
    sand_commands::ObjectiveName::logical(format!("{namespace}:{schema}.reconcile_dirty"))
        .as_str()
        .to_owned()
}

fn exact_predicate(objective: String, value: i32) -> StatePredicate {
    StatePredicate {
        objective,
        selector_range: SelectorScoreRange::exact(value),
        condition_range: ScoreRange::Eq(value),
    }
}

fn predicate_for_range(
    objective: String,
    schema: String,
    field: &str,
    range: impl RangeBounds<i32>,
) -> Result<StatePredicate, EntityDiagnostic> {
    let min = match range.start_bound() {
        Bound::Included(value) => Some(*value),
        Bound::Excluded(value) => value.checked_add(1),
        Bound::Unbounded => None,
    };
    let max = match range.end_bound() {
        Bound::Included(value) => Some(*value),
        Bound::Excluded(value) => value.checked_sub(1),
        Bound::Unbounded => None,
    };
    if matches!(range.start_bound(), Bound::Excluded(&i32::MAX))
        || matches!(range.end_bound(), Bound::Excluded(&i32::MIN))
        || matches!((min, max), (Some(min), Some(max)) if min > max)
    {
        return Err(EntityDiagnostic::InvalidRange {
            schema,
            field: field.into(),
            range: format!("{min:?}..={max:?}"),
        });
    }
    let selector_range = match (min, max) {
        (Some(min), Some(max)) => SelectorScoreRange::between(min, max),
        (Some(min), None) => SelectorScoreRange::at_least(min),
        (None, Some(max)) => SelectorScoreRange::at_most(max),
        (None, None) => SelectorScoreRange::between(i32::MIN, i32::MAX),
    };
    let condition_range = match (min, max) {
        (Some(min), Some(max)) if min == max => ScoreRange::Eq(min),
        (min, max) => ScoreRange::Between(min, max),
    };
    Ok(StatePredicate {
        objective,
        selector_range,
        condition_range,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Phase {
        Calm,
        Enraged,
    }

    impl EntityEnumValue for Phase {
        const ENCODINGS: &'static [EnumEncoding] = &[
            EnumEncoding {
                name: "Calm",
                score: 0,
            },
            EnumEncoding {
                name: "Enraged",
                score: 2,
            },
        ];
        fn encode(self) -> i32 {
            match self {
                Self::Calm => 0,
                Self::Enraged => 2,
            }
        }
    }

    #[test]
    fn names_are_stable_bounded_and_separate_from_dirty() {
        let field = EntityScore::<i32>::new("rpg", "mob", "maximum_health", 20, None);
        assert_eq!(field.objective(), field.objective());
        assert!(field.objective().len() <= 16);
        assert_ne!(field.objective(), field.dirty_objective());
    }

    #[test]
    fn write_marks_field_and_component_reconciliation_dirty() {
        let field = EntityScore::<i32>::new("rpg", "mob", "level", 1, Some((1, 100)));
        let commands = field.bind().set(10);
        assert_eq!(
            commands,
            vec![
                format!("scoreboard players set @s {} 10", field.objective()),
                format!(
                    "execute if score @s {} matches ..0 run scoreboard players set @s {} 1",
                    field.objective(),
                    field.objective()
                ),
                format!(
                    "execute if score @s {} matches 101.. run scoreboard players set @s {} 100",
                    field.objective(),
                    field.objective()
                ),
                format!("scoreboard players set @s {} 1", field.dirty_objective()),
                format!(
                    "scoreboard players set @s {} 1",
                    component_dirty_name("rpg", "mob")
                ),
            ]
        );
    }

    #[test]
    fn fixed_scores_round_saturate_and_keep_their_scale() {
        let field = FixedScore::__new("rpg", "mob", "speed", 10, 13, Some((-20, 20)));
        let bound = field.bind();
        assert_eq!(field.scale(), 10);
        assert_eq!(bound.get().scale(), 10);
        assert!(bound.get().command().contains(&field.objective()));
        assert_eq!(
            bound.set(1.25)[0],
            format!("scoreboard players set @s {} 13", field.objective())
        );
        assert_eq!(
            bound.set(-1.25)[0],
            format!("scoreboard players set @s {} -13", field.objective())
        );
        assert_eq!(
            bound.set(f64::INFINITY)[0],
            format!("scoreboard players set @s {} 20", field.objective())
        );

        let positive = FixedScore::__new("rpg", "mob", "power", 100, 100, Some((100, 1_000)));
        let positive = positive.bind();
        assert_eq!(
            positive.add(0.05)[0],
            format!(
                "scoreboard players add @s {} 5",
                FixedScore::__new("rpg", "mob", "power", 100, 100, Some((100, 1_000))).objective()
            )
        );
        assert_eq!(
            positive.subtract(0.05)[0],
            format!(
                "scoreboard players remove @s {} 5",
                FixedScore::__new("rpg", "mob", "power", 100, 100, Some((100, 1_000))).objective()
            )
        );
    }

    #[test]
    fn keyed_data_uses_uuid_macro_storage_without_entity_nbt_writes() {
        let field = KeyedData::<i32>::new("rpg:state", "components.\"stats\".xp");
        let commands = field.set(7);
        assert_eq!(commands.len(), 5);
        for (index, command) in commands[..4].iter().enumerate() {
            assert_eq!(
                command,
                &format!(
                    "data modify storage rpg:state __sand_owner.u{index} set from entity @s UUID[{index}]"
                )
            );
        }
        assert!(commands[4].starts_with("function __sand_local:sand/state_data/keyed/"));
        assert!(commands[4].ends_with(" with storage rpg:state __sand_owner"));
        assert!(
            commands
                .iter()
                .all(|command| !command.contains("data modify entity"))
        );
        let helpers = crate::function::drain_dyn_fns();
        assert_eq!(helpers.len(), 1);
        assert!(helpers[0].1[0].contains("owners[{uuid:[I;$(u0),$(u1),$(u2),$(u3)]}]"));
    }

    #[test]
    fn enum_predicate_uses_declared_encoding() {
        let field = EntityEnum::new("rpg", "mob", "phase", Phase::Calm.encode());
        assert_eq!(field.is(Phase::Enraged).selector_range.to_string(), "2");
    }

    #[test]
    fn inverted_range_is_structured_error() {
        let field = EntityScore::<i32>::new("rpg", "mob", "level", 1, None);
        assert_eq!(
            field
                .matches(std::ops::RangeInclusive::new(20, 10))
                .unwrap_err()
                .code(),
            "SAND-ENTITY-RANGE"
        );
    }

    #[test]
    fn duplicate_enum_scores_are_rejected() {
        static BAD: &[EnumEncoding] = &[
            EnumEncoding {
                name: "A",
                score: 1,
            },
            EnumEncoding {
                name: "B",
                score: 1,
            },
        ];
        static FIELDS: &[StateFieldDescriptor] = &[StateFieldDescriptor::new(
            "phase",
            StateFieldKind::Enum(BAD),
            1,
            None,
        )];
        let error = StateSchema {
            namespace: "rpg",
            name: "bad",
            version: 1,
            fields: FIELDS,
        }
        .validate()
        .unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-ENUM");
    }
}
