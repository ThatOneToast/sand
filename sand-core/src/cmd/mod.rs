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
//! use sand_core::cmd::{self, Execute, Selector};
//!
//! mcfunction! {
//!     cmd::give(Selector::all_players(), "diamond_sword").count(1);
//!     cmd::kill(Selector::all_entities().tag("enemy"));
//!     Execute::new()
//!         .as_(Selector::all_players())
//!         .if_score_matches("@s", "playtime", "100..")
//!         .run(cmd::say("100 ticks!"));
//! }
//! ```

// ── Internal modules (sand-core-specific) ─────────────────────────────────────

mod cooldown;
mod data;
mod effect;
mod fn_macros;
mod typed_execute;

// ── Re-exports from sand-commands ─────────────────────────────────────────────

/// Command construction and the shared profile-aware validation boundary.
pub use sand_commands::{
    Build, CommandProfile, EffectCommand, EffectDuration, RawCommand, RenderCommand, Validate,
};

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
    Damage as DamageBuilder, DamageAmount, DamageKind, EntityTarget, EntityTargets, GameMode, Many,
    One, PlayerTarget, PlayerTargets, Selector, SingleEntity, SinglePlayer, SortOrder, TargetBase,
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
pub use crate::vfx::{
    IntoParticleStep, IntoSoundStep, IntoVfxSelector, Vfx, VfxParticle, VfxSound, VfxStep,
};
pub use data::{Storage, StorageKind};
pub use effect::{EffectGive, effect_clear, effect_clear_effect, effect_give, effect_give_raw};
pub use fn_macros::{
    FunctionMacroArg, FunctionMacroArgs, call_with, function_with, macro_line, macro_var,
    try_call_with, try_macro_var,
};
pub use typed_execute::{ConditionedExecute, ExecuteExt, TypedExecute};

/// Call a function by resolved reference.
///
/// Accepts registered `#[function]` pointers, [`FunctionRef`](crate::resource_ref::FunctionRef),
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
/// cmd::call(FunctionRef::external("other_pack:api/do_thing").unwrap());
///
/// // Resource location
/// cmd::call(ResourceLocation::new("my_pack", "my_func").unwrap());
/// ```
pub fn call(id: impl crate::function::IntoFunctionRef) -> String {
    id.into_function_command()
}

/// Validated counterpart to [`call`].
///
/// [`crate::function::IntoFunctionRef`]'s registered-pointer, [`FunctionRef`](crate::resource_ref::FunctionRef),
/// and [`ResourceLocation`](crate::ResourceLocation) implementors are always
/// well-formed by construction, but the `&str`/`String` raw-path escape hatch
/// is not — this validates the resolved `namespace:path` resource location
/// (or the `__sand_local:path` sentinel used for not-yet-namespaced local
/// function pointers) before returning command text.
pub fn try_call(id: impl crate::function::IntoFunctionRef) -> sand_commands::CommandResult<String> {
    let line = id.into_function_command();
    let function_id = line.strip_prefix("function ").unwrap_or(line.as_str());
    sand_commands::validate::resource_location_shape(function_id, "cmd::try_call", "id")
        .map_err(|e| e.with_code("SAND-COMMAND-ARG-FUNCTION-ID"))?;
    Ok(line)
}

/// `function <namespace:path>` — run a datapack function by resource location.
///
/// Raw/unchecked: `id` is interpolated verbatim, with no resource-location
/// validation. This explicit fallback keeps the common function command
/// available even when generated vanilla command builders cannot be produced
/// in a local/CI build. Prefer [`call`] (registered typed function
/// references) or [`try_function`] (validated resource-location string) in
/// normal code — see [#175](https://github.com/ThatOneToast/sand/issues/175).
pub fn function(id: impl std::fmt::Display) -> String {
    format!("function {id}")
}

/// Validated counterpart to [`function`]: rejects an `id` that is not a
/// syntactically valid `namespace:path` resource location before returning
/// command text.
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
pub fn function_id(id: impl crate::function::IntoFunctionRef) -> String {
    id.into_function_id()
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
/// cmd::show_dialog(Selector::self_(), DialogRef::local("welcome"));
/// cmd::show_dialog(
///     Selector::all_players(),
///     DialogRef::external("other_pack:settings").unwrap(),
/// );
/// ```
pub fn show_dialog(
    selector: Selector,
    dialog: impl sand_components::dialog::IntoDialogRef,
) -> String {
    format!("dialog show {selector} {}", dialog.into_dialog_ref())
}

/// Validated counterpart to [`show_dialog`] — validates `selector` through
/// [`Selector`]'s normal validation before returning command text. `dialog`
/// resolution is already typed via [`IntoDialogRef`](sand_components::dialog::IntoDialogRef).
pub fn try_show_dialog(
    selector: Selector,
    dialog: impl sand_components::dialog::IntoDialogRef,
) -> sand_commands::CommandResult<String> {
    selector.validate(&CommandProfile::unprofiled())?;
    Ok(show_dialog(selector, dialog))
}

/// `tellraw <target> <json>` — send a rich JSON text component to a target.
pub fn tellraw(target: Selector, text: TextComponent) -> String {
    TextCommand::tellraw(target, text).build()
}

/// `tellraw <target> <raw_json>` — send a raw JSON text component to a target.
///
/// Raw/unchecked: `target` and `json` are interpolated verbatim, with no
/// selector or JSON validation. Prefer [`tellraw`] (validated
/// [`TextComponent`]) or [`try_tellraw_raw`] (validates the target selector
/// and that `json` is at least syntactically valid JSON) in normal code —
/// see [#175](https://github.com/ThatOneToast/sand/issues/175).
pub fn tellraw_raw(target: impl std::fmt::Display, json: impl Into<String>) -> String {
    format!("tellraw {target} {}", json.into())
}

/// Validated counterpart to [`tellraw_raw`].
///
/// Validates `target` through [`Selector`]'s normal validation and parses
/// `json` as JSON syntax (it does not validate it against the text-component
/// schema the way [`TextComponent`] does — that would duplicate the
/// component-level validation `Text`/`TextComponent` already own).
pub fn try_tellraw_raw(
    target: Selector,
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

/// Conversion accepted by [`give`]'s `item` parameter.
///
/// Implemented for:
/// - `&str`/`String` — the untyped escape hatch; no validation beyond what
///   the `give` command syntax itself enforces.
/// - [`sand_core::generated::Item`](crate::generated::Item) — generated
///   vanilla item identifiers (e.g. `vanilla::Item::Diamond`).
/// - [`sand_components::registry::ItemId`] (and `&ItemId`) — validated
///   custom/modded item identifiers (`ItemId::minecraft`/`::custom`).
///
/// Prefer the typed forms in normal code.
pub trait IntoGiveItem {
    /// Convert to the item's resource location, e.g. `"minecraft:diamond"`.
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
/// use sand_core::generated::Item;
/// use sand_commands::Selector;
///
/// cmd::give(Selector::all_players(), Item::Diamond);
/// cmd::give(Selector::self_(), "minecraft:diamond_sword");
/// ```
pub fn give(selector: Selector, item: impl IntoGiveItem) -> String {
    format!("give {selector} {}", item.into_give_item())
}

/// Validated counterpart to [`give`].
///
/// Typed [`IntoGiveItem`] implementors ([`crate::generated::Item`],
/// [`sand_components::registry::ItemId`], [`sand_components::CustomItem`])
/// are already well-formed by construction, but the `&str`/`String` raw
/// escape hatch is not — this validates the leading `namespace:path` item ID
/// (any trailing `[...]`/`{...}` item-component/NBT payload is preserved
/// verbatim, matching `sand_commands::Inventory`'s item validation) and the
/// target `selector` before returning command text.
pub fn try_give(
    selector: Selector,
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
///     tellraw(Selector::self_(), Text::new("Already granted")),
///     cmd::return_fail(),
/// ]);
/// ```
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
pub fn return_cmd(value: i32) -> String {
    format!("return {value}")
}

/// Explicit escape hatch for raw Minecraft command syntax.
///
/// Prefer typed builders for normal datapack code. Use this for interop with
/// other datapacks, modded commands, snapshot-only syntax, future features not
/// modeled by Sand yet, or focused debugging.
pub fn raw(command: impl Into<String>) -> sand_commands::RawCommand {
    sand_commands::RawCommand::new(command)
}

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
///     cmd::kill(Selector::all_entities().tag("mob"));
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
    use crate::resource_ref::DialogRef;

    const GENERATED_COMMANDS: &str = include_str!(concat!(env!("OUT_DIR"), "/commands.rs"));
    const GENERATED_REGISTRIES: &str = include_str!(concat!(env!("OUT_DIR"), "/registries.rs"));
    const GENERATED_BLOCK_STATES: &str = include_str!(concat!(env!("OUT_DIR"), "/block_states.rs"));

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
            "pub struct Tellraw",
            "pub fn tellraw(",
            "pub struct Give",
            "pub fn give(",
            "pub struct Function",
            "pub fn function(",
            "pub struct Damage",
            "pub fn damage(",
        ] {
            assert!(
                GENERATED_COMMANDS.contains(generated_symbol),
                "commands.rs is missing representative generated builder `{generated_symbol}`"
            );
        }
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
            super::try_show_dialog(super::Selector::self_(), DialogRef::local("welcome")).unwrap(),
            super::show_dialog(super::Selector::self_(), DialogRef::local("welcome"))
        );
    }

    #[test]
    fn try_show_dialog_rejects_invalid_selector() {
        assert!(
            super::try_show_dialog(
                super::Selector::all_entities().limit(0),
                DialogRef::local("welcome")
            )
            .is_err()
        );
    }

    #[test]
    fn show_dialog_local_ref() {
        assert_eq!(
            super::show_dialog(super::Selector::self_(), DialogRef::local("welcome")),
            "dialog show @s __sand_local:welcome"
        );
    }

    #[test]
    fn show_dialog_external_ref() {
        assert_eq!(
            super::show_dialog(
                super::Selector::all_players(),
                DialogRef::external("other_pack:settings").unwrap()
            ),
            "dialog show @a other_pack:settings"
        );
    }
}
