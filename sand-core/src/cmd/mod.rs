//! Typed Minecraft command builders.
//!
//! Each Minecraft command (or family of commands) is represented as a Rust
//! struct or free function that serializes to the correct command string via
//! [`std::fmt::Display`]. All types implement the [`Command`] marker trait.
//!
//! String-building types are provided by [`sand_commands`] and re-exported
//! here. Sand-core-specific modules contain only datapack-level concepts.
//!
//! # Module layout
//!
//! | Source | Contents |
//! |---|---|
//! | `sand_commands` (re-exported) | All command builders: blocks, coordinates, execute, selectors, scoreboard, NBT, sound, display, inventory, particles … |
//! | `cooldown` | [`Cooldown`] — scoreboard-based ability cooldown timer |
//! | `data` | [`Storage`], [`StorageKind`] — named NBT namespaces; bridges to `Objective::load_from` via `From<&Storage> for String` |
//! | `fn_macros` | `macro_var`, `macro_line`, `function_with` — function macro utilities |
//!
//! # Example
//! ```rust,ignore
//! use sand_core::cmd::{self, Execute, Target};
//!
//! mcfunction! {
//!     cmd::give(Target::players(), "diamond_sword").count(1);
//!     cmd::kill(Target::entities().tag("enemy"));
//!     Execute::new()
//!         .as_(Target::players())
//!         .if_score_matches("@s", "playtime", "100..")
//!         .run(cmd::say("100 ticks!"));
//! }
//! ```
//!
//! # Handwritten helper audit ([#175](https://github.com/ThatOneToast/sand/issues/175))
//!
//! Unlike the generated `_generated` builders (which are checked against the
//! Minecraft command tree), the helpers below are handwritten and therefore
//! individually audited and classified:
//!
//! | Helper | Classification | Notes |
//! |---|---|---|
//! | [`call`] | validated-compatibility | typed `IntoFunctionRef` inputs are well-formed by construction; the `&str`/`String` escape hatch is validated by [`try_call`] |
//! | [`try_call`] | typed-canonical | validates the resolved resource location before returning |
//! | [`function`] | explicit-raw | interpolates `id` verbatim; prefer [`try_function`] |
//! | [`try_function`] | typed-canonical | validates `id` as a resource location |
//! | [`function_id`] | validated-compatibility | same shape as `call`; raw escape hatch validated by [`try_function_id`] |
//! | [`try_function_id`] | typed-canonical | validates the resolved resource location before returning |
//! | [`show_dialog`] | validated-compatibility | dialog side is always typed via `IntoDialogRef`; selector validated by [`try_show_dialog`] |
//! | [`try_show_dialog`] | typed-canonical | validates the target selector |
//! | [`tellraw`] | typed-canonical | routes through [`TextComponent`]/[`TextCommand`] |
//! | [`tellraw_raw`] | explicit-raw | target and JSON interpolated verbatim; prefer [`tellraw`] or [`try_tellraw_raw`] |
//! | [`try_tellraw_raw`] | validated-compatibility | validates the selector and that `json` parses as JSON syntax |
//! | [`give`] | validated-compatibility | typed [`IntoGiveItem`] inputs are well-formed by construction; the `&str`/`String` escape hatch is validated by [`try_give`] |
//! | [`try_give`] | typed-canonical | validates the selector and the item's resource-location shape |
//! | [`return_fail`] | typed-canonical | fixed, always-valid command text |
//! | [`return_cmd`] | typed-canonical | fixed command shape; `value` is a plain `i32` |
//! | [`raw`] | explicit-raw | deliberate escape hatch; constrained to one valid `.mcfunction` line by [`RawCommand`]'s [`Validate`]/[`RenderCommand`] impls (`try_build`/profile-aware rendering) |
//! | `fn_macros::function_with` | explicit-raw | interpolates `name` verbatim; prefer `fn_macros::try_function_with` or `fn_macros::call_with`/`try_call_with` |
//! | `fn_macros::try_function_with` | validated-compatibility | validates `name` as a resource location and the NBT source/path |
//! | `fn_macros::call_with`/`try_call_with` | typed-canonical | fully typed function + NBT reference path (#194) |
//! | `data::Storage` raw methods (`remove`, `get`, `get_scaled`, `contains`, `get_or_insert`, `merge`) | validated-compatibility | each has a `try_*` counterpart routing through [`DataTarget`]/NBT-path validation |
//! | `IntoFunctionRef` for `fn() -> Vec<String>` / function items | programmer-error panic (documented, not a `try_*` gap) | see the "unregistered function pointer" rationale on [`crate::function::IntoFunctionRef`] |

// ── Internal modules (sand-core-specific) ─────────────────────────────────────

mod cooldown;
mod data;
mod effect;
mod fn_macros;
mod typed_execute;

// ── Re-exports from sand-commands ─────────────────────────────────────────────

/// Command construction and the shared profile-aware validation boundary.
pub use sand_commands::{
    Build, CommandError, CommandProfile, CommandResult, EffectCommand, EffectDuration,
    IntoDamageTargets, IntoEntityType, RawCommand, RenderCommand, Validate,
};
pub(crate) use sand_commands::{Selector, SingleEntity, SingleTargetArgument, TargetArgument};

/// Trait for types resolving to a `function <id>` command.
pub use crate::function::IntoFunctionRef;

// Block placement
pub use sand_commands::{
    BlockState, CloneBlocks, CloneMaskMode, CloneMode, Fill, FillMode, SetBlock, SetBlockMode,
};
// Coordinate types
pub use sand_commands::{BlockPos, Coord, Rotation, Vec2, Vec3};
// Player display commands
pub use sand_commands::{
    Actionbar, Bossbar, BossbarColor, BossbarCommand, BossbarId, BossbarStyle, IntoBossbarId,
    Title, TitleTimes,
};
// Execute builder
pub use sand_commands::Execute;
// Execute argument types
pub use sand_commands::{Anchor, ItemSlot, NbtStoreKind, Swizzle};
// Inventory manipulation
pub use sand_commands::Inventory;
// Particle effects
pub use sand_commands::{
    IntoParticleId, Particle, ParticleBuilder, ParticleCommand, ParticleEffect, ParticleSpread,
};
// Entity/player targeting
pub use sand_commands::{
    Damage as DamageBuilder, DamageAmount, DamageKind, GameMode, ScoreRange, Target,
};
// Sound
pub use sand_commands::{IntoSoundEvent, Sound, SoundSource, StopSoundCommand};
// Text components
pub use sand_commands::{
    ChatColor, ClickEvent, EntityHoverId, HoverEvent, IntoTextEntityType, Text, TextCommand,
    TextComponent,
};
// NBT types — owned by sand-commands
pub use sand_commands::{
    DataCommand, DataModify, DataModifyOperation, DataSource, DataTarget, Nbt, NbtCompound,
    NbtPath, NbtRef, NbtTarget, NbtValue, UntypedNbt, data_modify,
};
// Scoreboard types — owned by sand-commands
// Note: &Storage satisfies Objective::load_from's `impl Into<String>` parameter
// via the `From<&Storage> for String` impl in mod data.
pub use sand_commands::{
    DisplaySlot, Objective, ObjectiveName, ScoreCmp, ScoreHolder, ScoreOp,
    ScoreboardPlayersOperation, scoreboard_players_operation,
};
// NOTE: sand_commands::builtins::* is intentionally NOT re-exported here because
// sand-core provides its own generated command builders (see _generated below)
// that would conflict. Use sand_commands directly for the free-function builders.

// ── Re-exports from internal modules ─────────────────────────────────────────
pub use cooldown::Cooldown;
// Storage and StorageKind are datapack concepts defined only in sand-core.
// All other NBT/scoreboard types come from sand-commands above.
pub use crate::vfx::{Vfx, VfxParticle, VfxParticleVisibility, VfxSound, VfxStep};
pub use data::{Storage, StorageKind};
pub use effect::{EffectGive, effect_clear, effect_clear_effect, effect_give, effect_give_raw};
pub use fn_macros::{
    FunctionMacroArg, FunctionMacroArgs, call_with, function_with, macro_line, macro_var,
    try_call_with, try_function_with, try_macro_var,
};
pub use typed_execute::{ConditionedExecute, ExecuteExt, TypedExecute};

/// Call a function by resolved reference.
///
/// Accepts registered `#[function]` pointers, [`FunctionId`](crate::resource_ref::FunctionId),
/// [`ResourceLocation`](crate::ResourceLocation), and raw path strings.
///
/// # Examples
///
/// ```rust,ignore
/// use sand_core::prelude::*;
///
/// // Local registered function pointer (requires `use IntoFunctionRef`)
/// cmd::call(ate_golden_apple);
///
/// // External function ref
/// cmd::call("other_pack:api/do_thing".parse::<FunctionId>().unwrap());
///
/// // Resource location
/// cmd::call(ResourceLocation::new("my_pack", "my_func").unwrap());
/// ```
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::call",
    aliases = ["sand::cmd::call", "sand::prelude::cmd::call"],
    module = "sand::command",
    summary = "Call a function by resolved reference. Accepts registered `#[function]` pointers, [`FunctionId`](sand::resource_ref::FunctionId), [`ResourceLocation`](sand::ResourceLocation), and raw path strings.",
    context = "Call a function by resolved reference. Accepts registered `#[function]` pointers, [`FunctionId`](sand::resource_ref::FunctionId), [`ResourceLocation`](sand::ResourceLocation), and raw path strings. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Call a function by resolved reference."],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(id = "`id` provides the typed resource identifier or location used to call a function by resolved reference. Accepts registered `#[function]` pointers, [`FunctionId`](sand::resource_ref::FunctionId), [`ResourceLocation`](sand::ResourceLocation), and raw path strings."),
    returns = "The string value produced to call a function by resolved reference. Accepts registered `#[function]` pointers, [`FunctionId`](sand::resource_ref::FunctionId), [`ResourceLocation`](sand::ResourceLocation), and raw path strings.",
    example = "use sand::prelude::*;\n// Local registered function pointer (requires `use IntoFunctionRef`)\ncmd::call(ate_golden_apple);\n// External function ref\ncmd::call(\"other_pack:api/do_thing\".parse::<FunctionId>().unwrap());\n// Resource location\ncmd::call(ResourceLocation::new(\"my_pack\", \"my_func\").unwrap());",
)]
pub fn call(id: impl crate::function::IntoFunctionRef) -> String {
    id.into_function_command()
}

/// Validated counterpart to [`call`].
///
/// [`crate::function::IntoFunctionRef`]'s registered-pointer, [`FunctionId`](crate::resource_ref::FunctionId),
/// and [`ResourceLocation`](crate::ResourceLocation) implementors are always
/// well-formed by construction, but the `&str`/`String` raw-path escape hatch
/// is not — this validates the resolved `namespace:path` resource location
/// (or the `__sand_local:path` sentinel used for not-yet-namespaced local
/// function pointers) before returning command text.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::try_call",
    aliases = ["sand::cmd::try_call", "sand::prelude::cmd::try_call"],
    module = "sand::command",
    summary = "Validated counterpart to [`call`]. [`sand::command::IntoFunctionRef`]'s registered-pointer, [`FunctionId`](sand::resource_ref::FunctionId), and [`ResourceLocation`](sand::ResourceLocation) implementors are always well-formed by construction, but the `&str`/`String` raw-path escape hatch is not — this validates the resolved `namespace:path` resource location (or the `__sand_local:path` sentinel used for not-yet-namespaced local function pointers) before returning command text.",
    context = "Validated counterpart to [`call`]. [`sand::command::IntoFunctionRef`]'s registered-pointer, [`FunctionId`](sand::resource_ref::FunctionId), and [`ResourceLocation`](sand::ResourceLocation) implementors are always well-formed by construction, but the `&str`/`String` raw-path escape hatch is not — this validates the resolved `namespace:path` resource location (or the `__sand_local:path` sentinel used for not-yet-namespaced local function pointers) before returning command text. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "[`sand::command::IntoFunctionRef`]'s registered-pointer, [`FunctionId`](sand::resource_ref::FunctionId), and [`ResourceLocation`](sand::ResourceLocation) implementors are always well-formed by construction, but the `&str`/`String` raw-path escape hatch is not — this validates the resolved `namespace:path` resource location (or the `__sand_local:path` sentinel used for not-yet-namespaced local function pointers) before returning command text.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(id = "`id` provides the typed resource identifier or location used to use validated counterpart to [`call`]. [`sand::command::IntoFunctionRef`]'s registered-pointer, [`FunctionId`](sand::resource_ref::FunctionId), and [`ResourceLocation`](sand::ResourceLocation) implementors are always well-formed by construction, but the `&str`/`String` raw-path escape hatch is not — this validates the resolved `namespace:path` resource location (or the `__sand_local:path` sentinel used for not-yet-namespaced local function pointers) before returning command text."),
    returns = "The `sand :: command :: CommandResult < String >` value produced to use validated counterpart to [`call`]. [`sand::command::IntoFunctionRef`]'s registered-pointer, [`FunctionId`](sand::resource_ref::FunctionId), and [`ResourceLocation`](sand::ResourceLocation) implementors are always well-formed by construction, but the `&str`/`String` raw-path escape hatch is not — this validates the resolved `namespace:path` resource location (or the `__sand_local:path` sentinel used for not-yet-namespaced local function pointers) before returning command text.",
    example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoFunctionRef)  {\n    let try_call = sand::command::try_call(id);\n}",
)]
pub fn try_call(id: impl crate::function::IntoFunctionRef) -> sand_commands::CommandResult<String> {
    let function_id = try_function_id(id)?;
    Ok(format!("function {function_id}"))
}

/// `function <namespace:path>` — run a datapack function by resource location.
///
/// Raw/unchecked: `id` is interpolated verbatim, with no resource-location
/// validation. This explicit fallback keeps the common function command
/// available even when generated vanilla command builders cannot be produced
/// in a local/CI build. Prefer [`call`] (registered typed function
/// references) or [`try_function`] (validated resource-location string) in
/// normal code — see [#175](https://github.com/ThatOneToast/sand/issues/175).
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::function",
    aliases = ["sand::cmd::function", "sand::prelude::cmd::function"],
    module = "sand::command",
    summary = "`function <namespace:path>` — run a datapack function by resource location.",
    context = "`function <namespace:path>` — run a datapack function by resource location. Raw/unchecked: `id` is interpolated verbatim, with no resource-location validation. This explicit fallback keeps the common function command available even when generated vanilla command builders cannot be produced in a local/CI build. Prefer [`call`] (registered typed function references) or [`try_function`] (validated resource-location string) in normal code — see [#175](https://github.com/ThatOneToast/sand/issues/175).",
    minecraft = "Raw/unchecked: `id` is interpolated verbatim, with no resource-location validation. This explicit fallback keeps the common function command available even when generated vanilla command builders cannot be produced in a local/CI build. Prefer [`call`] (registered typed function references) or [`try_function`] (validated resource-location string) in normal code — see [#175](https://github.com/ThatOneToast/sand/issues/175).",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(id = "Raw/unchecked: `id` is interpolated verbatim, with no resource-location validation. This explicit fallback keeps the common function command available even when generated vanilla command builders cannot be produced in a local/CI build. Prefer [`call`] (registered typed function references) or [`try_function`] (validated resource-location string) in normal code — see [#175](https://github.com/ThatOneToast/sand/issues/175)."),
    returns = "The string value produced to emit the documented `function <namespace:path>` — run a datapack function by resource location form.",
    example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(id: impl std::fmt::Display)  {\n    let function = sand::command::function(id);\n}",
)]
pub fn function(id: impl std::fmt::Display) -> String {
    format!("function {id}")
}

/// Validated counterpart to [`function`]: rejects an `id` that is not a
/// syntactically valid `namespace:path` resource location before returning
/// command text.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::try_function",
    aliases = ["sand::cmd::try_function", "sand::prelude::cmd::try_function"],
    module = "sand::command",
    summary = "Validated counterpart to [`function`]: rejects an `id` that is not a syntactically valid `namespace:path` resource location before returning command text.",
    context = "Validated counterpart to [`function`]: rejects an `id` that is not a syntactically valid `namespace:path` resource location before returning command text. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(id = "Validated counterpart to [`function`]: rejects an `id` that is not a syntactically valid `namespace:path` resource location before returning command text."),
    returns = "The `sand :: command :: CommandResult < String >` value produced to use validated counterpart to [`function`]: rejects an `id` that is not a syntactically valid `namespace:path` resource location before returning command text.",
    example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(id: impl std::fmt::Display)  {\n    let try_function = sand::command::try_function(id);\n}",
)]
pub fn try_function(id: impl std::fmt::Display) -> sand_commands::CommandResult<String> {
    let id = id.to_string();
    sand_commands::validate::resource_location_shape(&id, "cmd::try_function", "id")
        .map_err(|e| e.with_code("SAND-COMMAND-ARG-FUNCTION-ID"))?;
    Ok(format!("function {id}"))
}

/// Resolve a function identifier to its `namespace:path` resource location.
///
/// # Examples
///
/// ```rust,ignore
/// let loc = cmd::function_id(ate_golden_apple);
/// assert_eq!(loc, "powers:ate_golden_apple");
/// ```
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::function_id",
    aliases = ["sand::cmd::function_id", "sand::prelude::cmd::function_id"],
    module = "sand::command",
    summary = "Resolve a function identifier to its `namespace:path` resource location.",
    context = "Resolve a function identifier to its `namespace:path` resource location. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(id = "`id` provides the typed resource identifier or location used to resolve a function identifier to its `namespace:path` resource location."),
    returns = "The string value produced to resolve a function identifier to its `namespace:path` resource location.",
    example = "let loc = cmd::function_id(ate_golden_apple);\nassert_eq!(loc, \"powers:ate_golden_apple\");",
)]
pub fn function_id(id: impl crate::function::IntoFunctionRef) -> String {
    id.into_function_id()
}

/// Validated counterpart to [`function_id`].
///
/// [`IntoFunctionRef`]'s registered-pointer, [`FunctionId`](crate::resource_ref::FunctionId),
/// and [`ResourceLocation`](crate::ResourceLocation) implementors are always
/// well-formed by construction, but the `&str`/`String` raw-path escape hatch
/// is not — this validates the resolved `namespace:path` resource location
/// (or the `__sand_local:path` sentinel used for not-yet-namespaced local
/// function pointers) before returning it. This closes the raw-string gap in
/// `function_id` noted in [#175](https://github.com/ThatOneToast/sand/issues/175)
/// (identified during the #287 review as the same shape of bypass as
/// [`try_call`]/[`try_function`]).
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::try_function_id",
    aliases = ["sand::cmd::try_function_id", "sand::prelude::cmd::try_function_id"],
    module = "sand::command",
    summary = "Validated counterpart to [`function_id`]. [`IntoFunctionRef`]'s registered-pointer, [`FunctionId`](sand::resource_ref::FunctionId), and [`ResourceLocation`](sand::ResourceLocation) implementors are always well-formed by construction, but the `&str`/`String` raw-path escape hatch is not — this validates the resolved `namespace:path` resource location (or the `__sand_local:path` sentinel used for not-yet-namespaced local function pointers) before returning it. This closes the raw-string gap in `function_id` noted in [#175](https://github.com/ThatOneToast/sand/issues/175) (identified during the #287 review as the same shape of bypass as [`try_call`]/[`try_function`]).",
    context = "Validated counterpart to [`function_id`]. [`IntoFunctionRef`]'s registered-pointer, [`FunctionId`](sand::resource_ref::FunctionId), and [`ResourceLocation`](sand::ResourceLocation) implementors are always well-formed by construction, but the `&str`/`String` raw-path escape hatch is not — this validates the resolved `namespace:path` resource location (or the `__sand_local:path` sentinel used for not-yet-namespaced local function pointers) before returning it. This closes the raw-string gap in `function_id` noted in [#175](https://github.com/ThatOneToast/sand/issues/175) (identified during the #287 review as the same shape of bypass as [`try_call`]/[`try_function`]). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(id = "`id` provides the typed resource identifier or location used to use validated counterpart to [`function_id`]. [`IntoFunctionRef`]'s registered-pointer, [`FunctionId`](sand::resource_ref::FunctionId), and [`ResourceLocation`](sand::ResourceLocation) implementors are always well-formed by construction, but the `&str`/`String` raw-path escape hatch is not — this validates the resolved `namespace:path` resource location (or the `__sand_local:path` sentinel used for not-yet-namespaced local function pointers) before returning it. This closes the raw-string gap in `function_id` noted in [#175](https://github.com/ThatOneToast/sand/issues/175) (identified during the #287 review as the same shape of bypass as [`try_call`]/[`try_function`])."),
    returns = "The `sand :: command :: CommandResult < String >` value produced to use validated counterpart to [`function_id`]. [`IntoFunctionRef`]'s registered-pointer, [`FunctionId`](sand::resource_ref::FunctionId), and [`ResourceLocation`](sand::ResourceLocation) implementors are always well-formed by construction, but the `&str`/`String` raw-path escape hatch is not — this validates the resolved `namespace:path` resource location (or the `__sand_local:path` sentinel used for not-yet-namespaced local function pointers) before returning it. This closes the raw-string gap in `function_id` noted in [#175](https://github.com/ThatOneToast/sand/issues/175) (identified during the #287 review as the same shape of bypass as [`try_call`]/[`try_function`]).",
    example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoFunctionRef)  {\n    let try_function_id = sand::command::try_function_id(id);\n}",
)]
pub fn try_function_id(
    id: impl crate::function::IntoFunctionRef,
) -> sand_commands::CommandResult<String> {
    let function_id = id.into_function_id();
    sand_commands::validate::resource_location_shape(&function_id, "cmd::try_function_id", "id")
        .map_err(|e| e.with_code("SAND-COMMAND-ARG-FUNCTION-ID"))?;
    Ok(function_id)
}

/// Show a typed datapack dialog to one or more players.
///
/// Dialogs are available in Minecraft Java 1.21.6+ / pack format 80+.
/// The command emitted is `dialog show <targets> <dialog>`.
///
/// # Examples
///
/// ```rust,ignore
/// use sand_core::prelude::*;
///
/// cmd::show_dialog(Target::self_(), DialogId::local("welcome"));
/// cmd::show_dialog(
///     Target::players(),
///     DialogId::custom("other_pack:settings".parse().unwrap()),
/// );
/// ```
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::show_dialog",
    aliases = ["sand::cmd::show_dialog", "sand::prelude::cmd::show_dialog"],
    module = "sand::command",
    summary = "Show a typed datapack dialog to one or more players.",
    context = "Show a typed datapack dialog to one or more players. Dialogs are available in Minecraft Java 1.21.6+ / pack format 80+. The command emitted is `dialog show <targets> <dialog>`.",
    minecraft = "Dialogs are available in Minecraft Java 1.21.6+ / pack format 80+. The command emitted is `dialog show <targets> <dialog>`.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(selector = "`selector` provides the Minecraft target selection used to show a typed datapack dialog to one or more players.", dialog = "`dialog` is used to show a typed datapack dialog to one or more players."),
    returns = "The string value produced to show a typed datapack dialog to one or more players.",
    example = "use sand::prelude::*;\ncmd::show_dialog(Target::self_(), DialogId::local(\"welcome\"));\ncmd::show_dialog(\nSelector::all_players(),\nDialogId::custom(\"other_pack:settings\".parse().unwrap()),\n);",
)]
pub fn show_dialog(
    selector: impl TargetArgument,
    dialog: impl sand_components::dialog::IntoDialogRef,
) -> String {
    format!("dialog show {selector} {}", dialog.into_dialog_ref())
}

/// Validated counterpart to [`show_dialog`] — validates `selector` through
/// [`Target`]'s normal validation before returning command text. `dialog`
/// resolution is already typed via [`IntoDialogRef`](sand_components::dialog::IntoDialogRef).
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::try_show_dialog",
    aliases = ["sand::cmd::try_show_dialog", "sand::prelude::cmd::try_show_dialog"],
    module = "sand::command",
    summary = "Validated counterpart to [`show_dialog`] — validates `selector` through [`Target`]'s normal validation before returning command text. `dialog` resolution is already typed via [`IntoDialogRef`](sand::component::IntoDialogRef).",
    context = "Validated counterpart to [`show_dialog`] — validates `selector` through [`Target`]'s normal validation before returning command text. `dialog` resolution is already typed via [`IntoDialogRef`](sand::component::IntoDialogRef). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(selector = "Validated counterpart to [`show_dialog`] — validates `selector` through [`Target`]'s normal validation before returning command text. `dialog` resolution is already typed via [`IntoDialogRef`](sand::component::IntoDialogRef).", dialog = "Validated counterpart to [`show_dialog`] — validates `selector` through [`Target`]'s normal validation before returning command text. `dialog` resolution is already typed via [`IntoDialogRef`](sand::component::IntoDialogRef)."),
    returns = "The `sand :: command :: CommandResult < String >` value produced to use validated counterpart to [`show_dialog`] — validates `selector` through [`Target`]'s normal validation before returning command text. `dialog` resolution is already typed via [`IntoDialogRef`](sand::component::IntoDialogRef).",
    example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Target, dialog: impl sand::component::IntoDialogRef)  {\n    let try_show_dialog = sand::command::try_show_dialog(selector, dialog);\n}",
)]
pub fn try_show_dialog(
    selector: impl TargetArgument,
    dialog: impl sand_components::dialog::IntoDialogRef,
) -> sand_commands::CommandResult<String> {
    selector.validate(&CommandProfile::unprofiled())?;
    Ok(show_dialog(selector, dialog))
}

/// `tellraw <target> <json>` — send a rich JSON text component to a target.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::tellraw",
    aliases = ["sand::cmd::tellraw", "sand::prelude::cmd::tellraw"],
    module = "sand::command",
    summary = "`tellraw <target> <json>` — send a rich JSON text component to a target.",
    context = "`tellraw <target> <json>` — send a rich JSON text component to a target. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(target = "`target` provides the entity, block, or command target used to emit the documented `tellraw <target> <json>` — send a rich JSON text component to a target form.", text = "`text` supplies the documented `tellraw <target> <json>` — send a rich JSON text component to a target form."),
    returns = "The string value produced to emit the documented `tellraw <target> <json>` — send a rich JSON text component to a target form.",
    example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Target, text: sand::text::TextComponent)  {\n    let tellraw = sand::command::tellraw(target, text);\n}",
)]
pub fn tellraw(target: impl TargetArgument, text: TextComponent) -> String {
    TextCommand::tellraw(target.into_target_selector(), text).build()
}

/// `tellraw <target> <raw_json>` — send a raw JSON text component to a target.
///
/// Raw/unchecked: `target` and `json` are interpolated verbatim, with no
/// selector or JSON validation. Prefer [`tellraw`] (validated
/// [`TextComponent`]) or [`try_tellraw_raw`] (validates the target selector
/// and that `json` is at least syntactically valid JSON) in normal code —
/// see [#175](https://github.com/ThatOneToast/sand/issues/175).
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::tellraw_raw",
    aliases = ["sand::cmd::tellraw_raw", "sand::prelude::cmd::tellraw_raw"],
    module = "sand::command",
    summary = "`tellraw <target> <raw_json>` — send a raw JSON text component to a target.",
    context = "`tellraw <target> <raw_json>` — send a raw JSON text component to a target. Raw/unchecked: `target` and `json` are interpolated verbatim, with no selector or JSON validation. Prefer [`tellraw`] (validated [`TextComponent`]) or [`try_tellraw_raw`] (validates the target selector and that `json` is at least syntactically valid JSON) in normal code — see [#175](https://github.com/ThatOneToast/sand/issues/175).",
    minecraft = "Raw/unchecked: `target` and `json` are interpolated verbatim, with no selector or JSON validation. Prefer [`tellraw`] (validated [`TextComponent`]) or [`try_tellraw_raw`] (validates the target selector and that `json` is at least syntactically valid JSON) in normal code — see [#175](https://github.com/ThatOneToast/sand/issues/175).",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(target = "Raw/unchecked: `target` and `json` are interpolated verbatim, with no selector or JSON validation. Prefer [`tellraw`] (validated [`TextComponent`]) or [`try_tellraw_raw`] (validates the target selector and that `json` is at least syntactically valid JSON) in normal code — see [#175](https://github.com/ThatOneToast/sand/issues/175).", json = "Raw/unchecked: `target` and `json` are interpolated verbatim, with no selector or JSON validation. Prefer [`tellraw`] (validated [`TextComponent`]) or [`try_tellraw_raw`] (validates the target selector and that `json` is at least syntactically valid JSON) in normal code — see [#175](https://github.com/ThatOneToast/sand/issues/175)."),
    returns = "The string value produced to emit the documented `tellraw <target> <raw_json>` — send a raw JSON text component to a target form.",
    example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(target: impl std::fmt::Display, json: impl Into < String >)  {\n    let tellraw_raw = sand::command::tellraw_raw(target, json);\n}",
)]
pub fn tellraw_raw(target: impl std::fmt::Display, json: impl Into<String>) -> String {
    format!("tellraw {target} {}", json.into())
}

/// Validated counterpart to [`tellraw_raw`].
///
/// Validates `target` through [`Target`]'s normal validation and parses
/// `json` as JSON syntax (it does not validate it against the text-component
/// schema the way [`TextComponent`] does — that would duplicate the
/// component-level validation `Text`/`TextComponent` already own).
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::try_tellraw_raw",
    aliases = ["sand::cmd::try_tellraw_raw", "sand::prelude::cmd::try_tellraw_raw"],
    module = "sand::command",
    summary = "Validated counterpart to [`tellraw_raw`]. Validates `target` through [`Target`]'s normal validation and parses `json` as JSON syntax (it does not validate it against the text-component schema the way [`TextComponent`] does — that would duplicate the component-level validation `Text`/`TextComponent` already own).",
    context = "Validated counterpart to [`tellraw_raw`]. Validates `target` through [`Target`]'s normal validation and parses `json` as JSON syntax (it does not validate it against the text-component schema the way [`TextComponent`] does — that would duplicate the component-level validation `Text`/`TextComponent` already own). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Validates `target` through [`Target`]'s normal validation and parses `json` as JSON syntax (it does not validate it against the text-component schema the way [`TextComponent`] does — that would duplicate the component-level validation `Text`/`TextComponent` already own).",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(target = "Validates `target` through [`Target`]'s normal validation and parses `json` as JSON syntax (it does not validate it against the text-component schema the way [`TextComponent`] does — that would duplicate the component-level validation `Text`/`TextComponent` already own).", json = "Validates `target` through [`Target`]'s normal validation and parses `json` as JSON syntax (it does not validate it against the text-component schema the way [`TextComponent`] does — that would duplicate the component-level validation `Text`/`TextComponent` already own)."),
    returns = "The `sand :: command :: CommandResult < String >` value produced to use validated counterpart to [`tellraw_raw`]. Validates `target` through [`Target`]'s normal validation and parses `json` as JSON syntax (it does not validate it against the text-component schema the way [`TextComponent`] does — that would duplicate the component-level validation `Text`/`TextComponent` already own).",
    example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Target, json: impl Into < String >)  {\n    let try_tellraw_raw = sand::command::try_tellraw_raw(target, json);\n}",
)]
pub fn try_tellraw_raw(
    target: impl TargetArgument,
    json: impl Into<String>,
) -> sand_commands::CommandResult<String> {
    target.validate(&CommandProfile::unprofiled())?;
    let json = json.into();
    serde_json::from_str::<serde_json::Value>(&json).map_err(|e| {
        sand_commands::CommandError::new("cmd::try_tellraw_raw", "json", e.to_string())
            .with_code("SAND-COMMAND-ARG-TELLRAW-JSON")
    })?;
    Ok(format!("tellraw {target} {json}"))
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::IntoGiveItem",
    aliases = ["sand::cmd::IntoGiveItem", "sand::prelude::cmd::IntoGiveItem"],
    module = "sand::command",
    summary = "Conversion accepted by [`give`]'s `item` parameter.",
    context = "Conversion accepted by [`give`]'s `item` parameter. Implemented for: - `&str`/`String` — the untyped escape hatch; no validation beyond what the `give` command syntax itself enforces. - Sand's profile-generated vanilla item enum, when available (e.g. `vanilla::Item::Diamond`). - [`sand::registry::ItemId`] (and `&ItemId`) — validated custom/modded item identifiers (`ItemId::minecraft`/`::custom`). Prefer the typed forms in normal code.",
    minecraft = "Implemented for: - `&str`/`String` — the untyped escape hatch; no validation beyond what the `give` command syntax itself enforces. - Sand's profile-generated vanilla item enum, when available (e.g. `vanilla::Item::Diamond`). - [`sand::registry::ItemId`] (and `&ItemId`) — validated custom/modded item identifiers (`ItemId::minecraft`/`::custom`).",
    use_when = ["Prefer the typed forms in normal code."],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::IntoGiveItem;",
)]
/// Conversion accepted by [`give`]'s `item` parameter.
///
/// Implemented for:
/// - `&str`/`String` — the untyped escape hatch; no validation beyond what
///   the `give` command syntax itself enforces.
/// - Sand's profile-generated vanilla item enum, when available (e.g.
///   `vanilla::Item::Diamond`).
/// - [`sand_components::registry::ItemId`] (and `&ItemId`) — validated
///   custom/modded item identifiers (`ItemId::minecraft`/`::custom`).
///
/// Prefer the typed forms in normal code.
pub trait IntoGiveItem {
    /// Convert to the item's resource location, e.g. `"minecraft:diamond"`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::IntoGiveItem::into_give_item",
        aliases = ["sand::cmd::IntoGiveItem::into_give_item", "sand::prelude::cmd::IntoGiveItem::into_give_item"],
        module = "sand::command",
        summary = "Convert to the item's resource location, e.g. `\"minecraft:diamond\"`.",
        context = "Convert to the item's resource location, e.g. `\"minecraft:diamond\"`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The string value produced to convert to the item's resource location, e.g. `\"minecraft:diamond\"`.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::command::IntoGiveItem>(into_give_item_value: T)  {\n    let into_give_item = into_give_item_value.into_give_item();\n}",
    )]
    fn into_give_item(self) -> String;
}

impl IntoGiveItem for String {
    fn into_give_item(self) -> String {
        self
    }
}

impl IntoGiveItem for &str {
    fn into_give_item(self) -> String {
        self.to_string()
    }
}

impl IntoGiveItem for &String {
    fn into_give_item(self) -> String {
        self.clone()
    }
}

impl IntoGiveItem for sand_components::registry::ItemId {
    fn into_give_item(self) -> String {
        self.to_string()
    }
}

impl IntoGiveItem for &sand_components::registry::ItemId {
    fn into_give_item(self) -> String {
        self.to_string()
    }
}

impl IntoGiveItem for crate::generated::Item {
    fn into_give_item(self) -> String {
        self.resource_location().to_owned()
    }
}

impl IntoGiveItem for sand_components::CustomItem {
    fn into_give_item(self) -> String {
        self.to_string()
    }
}

impl IntoGiveItem for &sand_components::CustomItem {
    fn into_give_item(self) -> String {
        self.to_string()
    }
}

/// `give <targets> <item>` — give an item stack to one or more players.
///
/// # Examples
/// ```
/// use sand_core::cmd;
/// use sand_components::ItemId;
/// use sand::command::Target;
///
/// cmd::give(
///     Target::players(),
///     ItemId::minecraft("diamond").unwrap(),
/// );
/// cmd::give(Target::self_(), "minecraft:diamond_sword");
/// ```
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::give",
    aliases = ["sand::cmd::give", "sand::prelude::cmd::give"],
    module = "sand::command",
    summary = "`give <targets> <item>` — give an item stack to one or more players.",
    context = "`give <targets> <item>` — give an item stack to one or more players. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(selector = "`selector` provides the Minecraft target selection used to emit the documented `give <targets> <item>` — give an item stack to one or more players form.", item = "`item` provides the item value or item predicate used to emit the documented `give <targets> <item>` — give an item stack to one or more players form."),
    returns = "The string value produced to emit the documented `give <targets> <item>` — give an item stack to one or more players form.",
    example = "use sand::command;\nuse sand::registry::ItemId;\nuse sand::command::Target;\ncmd::give(\nSelector::all_players(),\nItemId::minecraft(\"diamond\").unwrap(),\n);\ncmd::give(Target::self_(), \"minecraft:diamond_sword\");",
)]
pub fn give(selector: impl TargetArgument, item: impl IntoGiveItem) -> String {
    format!("give {selector} {}", item.into_give_item())
}

/// Validated counterpart to [`give`].
///
/// Typed [`IntoGiveItem`] implementors (the profile-generated vanilla item
/// enum, [`sand_components::registry::ItemId`], and [`sand_components::CustomItem`])
/// are already well-formed by construction, but the `&str`/`String` raw
/// escape hatch is not — this validates the leading `namespace:path` item ID
/// (any trailing `[...]`/`{...}` item-component/NBT payload is preserved
/// verbatim, matching `sand::command::Inventory`'s item validation) and the
/// target `selector` before returning command text.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::try_give",
    aliases = ["sand::cmd::try_give", "sand::prelude::cmd::try_give"],
    module = "sand::command",
    summary = "Validated counterpart to [`give`]. Typed [`IntoGiveItem`] implementors (the profile-generated vanilla item enum, [`sand::registry::ItemId`], and [`sand::component::CustomItem`]) are already well-formed by construction, but the `&str`/`String` raw escape hatch is not — this validates the leading `namespace:path` item ID (any trailing `[...]`/`{...}` item-component/NBT payload is preserved verbatim, matching `sand::command::Inventory`'s item validation) and the target `selector` before returning command text.",
    context = "Validated counterpart to [`give`]. Typed [`IntoGiveItem`] implementors (the profile-generated vanilla item enum, [`sand::registry::ItemId`], and [`sand::component::CustomItem`]) are already well-formed by construction, but the `&str`/`String` raw escape hatch is not — this validates the leading `namespace:path` item ID (any trailing `[...]`/`{...}` item-component/NBT payload is preserved verbatim, matching `sand::command::Inventory`'s item validation) and the target `selector` before returning command text. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Typed [`IntoGiveItem`] implementors (the profile-generated vanilla item enum, [`sand::registry::ItemId`], and [`sand::component::CustomItem`]) are already well-formed by construction, but the `&str`/`String` raw escape hatch is not — this validates the leading `namespace:path` item ID (any trailing `[...]`/`{...}` item-component/NBT payload is preserved verbatim, matching `sand::command::Inventory`'s item validation) and the target `selector` before returning command text.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(selector = "Typed [`IntoGiveItem`] implementors (the profile-generated vanilla item enum, [`sand::registry::ItemId`], and [`sand::component::CustomItem`]) are already well-formed by construction, but the `&str`/`String` raw escape hatch is not — this validates the leading `namespace:path` item ID (any trailing `[...]`/`{...}` item-component/NBT payload is preserved verbatim, matching `sand::command::Inventory`'s item validation) and the target `selector` before returning command text.", item = "`item` provides the item value or item predicate used to use validated counterpart to [`give`]. Typed [`IntoGiveItem`] implementors (the profile-generated vanilla item enum, [`sand::registry::ItemId`], and [`sand::component::CustomItem`]) are already well-formed by construction, but the `&str`/`String` raw escape hatch is not — this validates the leading `namespace:path` item ID (any trailing `[...]`/`{...}` item-component/NBT payload is preserved verbatim, matching `sand::command::Inventory`'s item validation) and the target `selector` before returning command text."),
    returns = "The `sand :: command :: CommandResult < String >` value produced to use validated counterpart to [`give`]. Typed [`IntoGiveItem`] implementors (the profile-generated vanilla item enum, [`sand::registry::ItemId`], and [`sand::component::CustomItem`]) are already well-formed by construction, but the `&str`/`String` raw escape hatch is not — this validates the leading `namespace:path` item ID (any trailing `[...]`/`{...}` item-component/NBT payload is preserved verbatim, matching `sand::command::Inventory`'s item validation) and the target `selector` before returning command text.",
    example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Target, item: impl sand::command::IntoGiveItem)  {\n    let try_give = sand::command::try_give(selector, item);\n}",
)]
pub fn try_give(
    selector: impl TargetArgument,
    item: impl IntoGiveItem,
) -> sand_commands::CommandResult<String> {
    selector.validate(&CommandProfile::unprofiled())?;
    let item = item.into_give_item();
    let id_part = item.find(['[', '{']).map_or(item.as_str(), |i| &item[..i]);
    sand_commands::validate::resource_location_shape(id_part, "cmd::try_give", "item")?;
    Ok(format!("give {selector} {item}"))
}

/// `return fail` — stop the current function with a failure return value.
///
/// In Minecraft 1.20.2+, `return fail` terminates the current `.mcfunction`
/// and reports failure (return value −1) to callers using `execute … run function`.
/// Use inside branch or helper functions to halt that branch.
///
/// ```rust,ignore
/// when(HAS_CELLS.of("@s").is_true()).then_all([
///     tellraw(Target::self_(), Text::new("Already granted")),
///     cmd::return_fail(),
/// ]);
/// ```
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::return_fail",
    aliases = ["sand::cmd::return_fail", "sand::prelude::cmd::return_fail"],
    module = "sand::command",
    summary = "`return fail` — stop the current function with a failure return value.",
    context = "`return fail` — stop the current function with a failure return value. In Minecraft 1.20.2+, `return fail` terminates the current `.mcfunction` and reports failure (return value −1) to callers using `execute … run function`. Use inside branch or helper functions to halt that branch.",
    minecraft = "In Minecraft 1.20.2+, `return fail` terminates the current `.mcfunction` and reports failure (return value −1) to callers using `execute … run function`. Use inside branch or helper functions to halt that branch.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    returns = "The string value produced to emit the documented `return fail` — stop the current function with a failure return value form.",
    example = "when(HAS_CELLS.of(\"@s\").is_true()).then_all([\ntellraw(Target::self_(), Text::new(\"Already granted\")),\ncmd::return_fail(),\n]);",
)]
pub fn return_fail() -> String {
    "return fail".to_string()
}

/// `return <value>` — stop the current function with an integer return value.
///
/// `cmd::return_cmd(0)` → `return 0` (success, also readable by `execute store result`).
/// `cmd::return_cmd(1)` → `return 1`.
///
/// In Minecraft 1.20.2+, `return <n>` terminates the current `.mcfunction`
/// with the given result code. Callers using `execute … run function` see this value.
///
/// ```rust,ignore
/// unless(HAS_CELLS.of("@s").is_true()).then_all([
///     HAS_CELLS.enable("@s"),
///     cmd::return_cmd(0),
/// ]);
/// ```
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::return_cmd",
    aliases = ["sand::cmd::return_cmd", "sand::prelude::cmd::return_cmd"],
    module = "sand::command",
    summary = "`return <value>` — stop the current function with an integer return value.",
    context = "`return <value>` — stop the current function with an integer return value. `cmd::return_cmd(0)` → `return 0` (success, also readable by `execute store result`). `cmd::return_cmd(1)` → `return 1`. In Minecraft 1.20.2+, `return <n>` terminates the current `.mcfunction` with the given result code. Callers using `execute … run function` see this value.",
    minecraft = "In Minecraft 1.20.2+, `return <n>` terminates the current `.mcfunction` with the given result code. Callers using `execute … run function` see this value.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(value = "`value` provides the value being applied or compared used to emit the documented `return <value>` — stop the current function with an integer return value form."),
    returns = "The string value produced to emit the documented `return <value>` — stop the current function with an integer return value form.",
    example = "unless(HAS_CELLS.of(\"@s\").is_true()).then_all([\nHAS_CELLS.enable(\"@s\"),\ncmd::return_cmd(0),\n]);",
)]
pub fn return_cmd(value: i32) -> String {
    format!("return {value}")
}

/// Explicit escape hatch for raw Minecraft command syntax.
///
/// Prefer typed builders for normal datapack code. Use this for interop with
/// other datapacks, modded commands, snapshot-only syntax, future features not
/// modeled by Sand yet, or focused debugging.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::raw",
    aliases = ["sand::cmd::raw", "sand::prelude::cmd::raw"],
    module = "sand::command",
    summary = "Explicit escape hatch for raw Minecraft command syntax.",
    context = "Explicit escape hatch for raw Minecraft command syntax. Prefer typed builders for normal datapack code. Use this for interop with other datapacks, modded commands, snapshot-only syntax, future features not modeled by Sand yet, or focused debugging.",
    minecraft = "Prefer typed builders for normal datapack code. Use this for interop with other datapacks, modded commands, snapshot-only syntax, future features not modeled by Sand yet, or focused debugging.",
    use_when = ["Prefer typed builders for normal datapack code. Use this for interop with other datapacks, modded commands, snapshot-only syntax, future features not modeled by Sand yet, or focused debugging."],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(command = "`command` sets the command for explicit escape hatch for raw Minecraft command syntax."),
    returns = "The `sand :: command :: RawCommand` value produced to use explicit escape hatch for raw Minecraft command syntax.",
    example = "use sand::prelude::*;\n\nfn demonstrate(command: impl Into < String >)  {\n    let raw = sand::command::raw(command);\n}",
)]
pub fn raw(command: impl Into<String>) -> sand_commands::RawCommand {
    sand_commands::RawCommand::new(command)
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Command",
    aliases = ["sand::cmd::Command", "sand::prelude::cmd::Command"],
    module = "sand::command",
    summary = "A typed Minecraft command that can be serialized to a command string.",
    context = "A typed Minecraft command that can be serialized to a command string. All command builders generated from the Minecraft command tree implement this compatibility marker. It is distinct from [`RenderCommand`], the fallible profile-aware validation contract implemented by migrated typed command foundations. New handwritten builders should prefer [`RenderCommand`]; generated marker commands are conservatively checked at the function export boundary. Since [`Command`] requires [`std::fmt::Display`], you can use command builders directly in [`sand::mcfunction!`]:",
    minecraft = "All command builders generated from the Minecraft command tree implement this compatibility marker. It is distinct from [`RenderCommand`], the fallible profile-aware validation contract implemented by migrated typed command foundations. New handwritten builders should prefer [`RenderCommand`]; generated marker commands are conservatively checked at the function export boundary.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Command;",
)]
/// A typed Minecraft command that can be serialized to a command string.
///
/// All command builders generated from the Minecraft command tree implement
/// this compatibility marker. It is distinct from [`RenderCommand`], the
/// fallible profile-aware validation contract implemented by migrated typed
/// command foundations. New handwritten builders should prefer
/// [`RenderCommand`]; generated marker commands are conservatively checked at
/// the function export boundary.
///
/// Since [`Command`] requires [`std::fmt::Display`], you can use command
/// builders directly in [`crate::mcfunction!`]:
/// ```rust,ignore
/// mcfunction! {
///     cmd::kill(Target::entities().tag("mob"));
///     "raw fallback command string";
/// }
/// ```
pub trait Command: std::fmt::Display {}

impl Command for sand_commands::DataCommand {}
impl Command for sand_commands::Sound {}

// Include the generated command builders from commands.json.
#[allow(warnings, clippy::all)]
mod _generated {
    use super::*;
    use crate::ResourceLocation;
    include!(concat!(env!("OUT_DIR"), "/commands.rs"));
}
#[allow(unused)]
pub use _generated::*;

#[cfg(test)]
mod tests {
    use crate::resource_ref::DialogId;

    const GENERATED_COMMANDS: &str = include_str!(concat!(env!("OUT_DIR"), "/commands.rs"));
    const GENERATED_REGISTRIES: &str = include_str!(concat!(env!("OUT_DIR"), "/registries.rs"));
    const GENERATED_BLOCK_STATES: &str = include_str!(concat!(env!("OUT_DIR"), "/block_states.rs"));
    const GENERATED_COMMAND_API: &str =
        include_str!(concat!(env!("OUT_DIR"), "/commands.api.json"));
    const GENERATED_REGISTRY_API: &str =
        include_str!(concat!(env!("OUT_DIR"), "/registries.api.json"));

    fn generated_api_health(
        commands: &str,
        registries: &str,
        block_states: &str,
    ) -> Result<(), String> {
        for (name, contents) in [
            ("commands.rs", commands),
            ("registries.rs", registries),
            ("block_states.rs", block_states),
        ] {
            if contents.trim().is_empty() {
                return Err(format!("{name} should contain generated Rust API"));
            }
            if contents.contains("Generation failed") {
                return Err(format!("{name} contains a codegen fallback placeholder"));
            }
        }

        for (contents, symbol, file) in [
            (commands, "pub struct Say", "commands.rs"),
            (commands, "pub fn say(", "commands.rs"),
            (registries, "pub enum Item", "registries.rs"),
            (registries, "pub enum Block", "registries.rs"),
            (
                block_states,
                "pub struct OakDoorProperties",
                "block_states.rs",
            ),
        ] {
            if !contents.contains(symbol) {
                return Err(format!("{file} is missing representative API `{symbol}`"));
            }
        }
        Ok(())
    }

    #[test]
    fn generated_api_health_files_are_not_placeholders() {
        generated_api_health(
            GENERATED_COMMANDS,
            GENERATED_REGISTRIES,
            GENERATED_BLOCK_STATES,
        )
        .unwrap();
    }

    #[test]
    fn generated_api_health_rejects_empty_and_placeholder_files() {
        assert!(generated_api_health("", "registries", "block states").is_err());
        assert!(
            generated_api_health("// Generation failed", "registries", "block states").is_err()
        );
    }

    #[test]
    fn generated_api_health_has_representative_command_builders() {
        for generated_symbol in [
            "pub struct Say",
            "pub fn say(",
            "pub struct Damage",
            "pub fn damage(",
            "pub struct Summon",
            "pub fn summon(",
            "pub struct Teleport",
            "pub fn teleport(",
        ] {
            assert!(
                GENERATED_COMMANDS.contains(generated_symbol),
                "commands.rs is missing representative generated builder `{generated_symbol}`"
            );
        }
    }

    #[test]
    fn generated_provider_metadata_covers_every_public_generated_item() {
        let commands: serde_json::Value = serde_json::from_str(GENERATED_COMMAND_API).unwrap();
        let registries: serde_json::Value = serde_json::from_str(GENERATED_REGISTRY_API).unwrap();
        let command_entries = commands["entries"].as_array().unwrap();
        let registry_entries = registries["entries"].as_array().unwrap();

        let command_rust_items = GENERATED_COMMANDS
            .lines()
            .filter(|line| {
                let line = line.trim_start();
                line.starts_with("pub struct ") || line.starts_with("pub fn ")
            })
            .count();
        assert_eq!(command_entries.len(), command_rust_items);
        assert_eq!(command_entries.len(), 1_233);

        assert_eq!(registry_entries.len(), 4_867);
        assert!(registry_entries.iter().any(|entry| {
            entry["definition_identity"] == "sand_core::generated::Item::Diamond"
                && entry["contract"]["canonical_path"] == "sand::vanilla::Item::Diamond"
        }));
    }

    #[test]
    fn raw_escape_hatch_is_explicit() {
        assert_eq!(
            super::raw("function other_pack:api/do_special_thing"),
            "function other_pack:api/do_special_thing"
        );
    }

    #[test]
    fn try_function_matches_function_for_valid_id() {
        assert_eq!(
            super::try_function("my_pack:api/do_thing").unwrap(),
            super::function("my_pack:api/do_thing")
        );
    }

    #[test]
    fn try_function_rejects_malformed_id() {
        assert!(super::try_function("not a resource location").is_err());
    }

    #[test]
    fn try_tellraw_raw_matches_tellraw_raw_for_valid_json() {
        assert_eq!(
            super::try_tellraw_raw(super::Selector::self_(), r#"{"text":"hi"}"#).unwrap(),
            super::tellraw_raw(super::Selector::self_(), r#"{"text":"hi"}"#)
        );
    }

    #[test]
    fn try_tellraw_raw_rejects_malformed_json() {
        assert!(super::try_tellraw_raw(super::Selector::self_(), "{not json}").is_err());
    }

    #[test]
    fn try_tellraw_raw_rejects_invalid_selector() {
        assert!(
            super::try_tellraw_raw(super::Selector::all_entities().limit(0), r#"{"text":"hi"}"#)
                .is_err()
        );
    }

    #[test]
    fn try_function_id_matches_function_id_for_valid_raw_path() {
        assert_eq!(
            super::try_function_id("my_pack:api/do_thing").unwrap(),
            super::function_id("my_pack:api/do_thing")
        );
    }

    #[test]
    fn try_function_id_rejects_malformed_raw_path() {
        assert!(super::try_function_id("not a resource location").is_err());
        assert!(super::try_function_id("Bad Path").is_err());
    }

    #[test]
    fn try_call_matches_call_for_valid_raw_path() {
        assert_eq!(
            super::try_call("my_pack:api/do_thing").unwrap(),
            super::call("my_pack:api/do_thing")
        );
    }

    #[test]
    fn try_call_rejects_malformed_raw_path() {
        assert!(super::try_call("not a resource location").is_err());
        assert!(super::try_call("Bad Path").is_err());
    }

    #[test]
    fn try_give_matches_give_for_valid_item() {
        assert_eq!(
            super::try_give(super::Selector::self_(), "minecraft:diamond_sword").unwrap(),
            super::give(super::Selector::self_(), "minecraft:diamond_sword")
        );
    }

    #[test]
    fn try_give_rejects_malformed_item_id() {
        assert!(super::try_give(super::Selector::self_(), "Diamond").is_err());
        assert!(super::try_give(super::Selector::self_(), "diamond").is_err());
    }

    #[test]
    fn try_give_accepts_component_syntax_as_escape_hatch() {
        assert_eq!(
            super::try_give(
                super::Selector::self_(),
                "minecraft:diamond_sword[custom_name='\"Foo\"']"
            )
            .unwrap(),
            "give @s minecraft:diamond_sword[custom_name='\"Foo\"']"
        );
    }

    #[test]
    fn try_give_rejects_invalid_selector() {
        assert!(
            super::try_give(
                super::Selector::all_entities().limit(0),
                "minecraft:diamond"
            )
            .is_err()
        );
    }

    #[test]
    fn try_show_dialog_matches_show_dialog_for_valid_selector() {
        assert_eq!(
            super::try_show_dialog(super::Selector::self_(), DialogId::local("welcome")).unwrap(),
            super::show_dialog(super::Selector::self_(), DialogId::local("welcome"))
        );
    }

    #[test]
    fn try_show_dialog_rejects_invalid_selector() {
        assert!(
            super::try_show_dialog(
                super::Selector::all_entities().limit(0),
                DialogId::local("welcome")
            )
            .is_err()
        );
    }

    #[test]
    fn show_dialog_local_ref() {
        assert_eq!(
            super::show_dialog(super::Selector::self_(), DialogId::local("welcome")),
            "dialog show @s __sand_local:welcome"
        );
    }

    #[test]
    fn show_dialog_external_ref() {
        assert_eq!(
            super::show_dialog(
                super::Selector::all_players(),
                DialogId::custom("other_pack:settings".parse().unwrap())
            ),
            "dialog show @a other_pack:settings"
        );
    }
}
