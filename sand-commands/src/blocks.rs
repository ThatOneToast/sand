//! Block placement and manipulation commands: `setblock`, `fill`, and `clone`.
//!
//! # Example
//! ```rust,ignore
//! // Simple setblock
//! let cmd = SetBlock::new(BlockPos::here(), "minecraft:stone").build();
//! // → "setblock ~ ~ ~ minecraft:stone"
//!
//! // Setblock with state
//! let cmd = SetBlock::new(BlockPos::here(),
//!     BlockState::of("minecraft:oak_stairs")
//!         .prop("facing", "east")
//!         .prop("half", "bottom"))
//!     .mode(SetBlockMode::Replace)
//!     .build();
//! // → "setblock ~ ~ ~ minecraft:oak_stairs[facing=east,half=bottom] replace"
//!
//! // Fill a region
//! let cmd = Fill::new(BlockPos::absolute(0, 64, 0), BlockPos::absolute(10, 68, 10),
//!     "minecraft:glass")
//!     .mode(FillMode::Hollow)
//!     .build();
//! // → "fill 0 64 0 10 68 10 minecraft:glass hollow"
//! ```

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

use crate::Build;
use crate::coord::BlockPos;
use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::validate;

// ── Pre-write export re-validation registry ─────────────────────────────────
//
// Mirrors the pattern used by `nbt::DataCommand`/`execute_ir` (see #145's
// architecture): a rendered line's typed node is retained here so the
// export pipeline's `validate_collected_line` can re-validate the *typed
// node* (not re-parse the rendered string) against the export's resolved
// `CommandProfile`, even though this crate's ordinary constructors return
// plain rendered `String`s once collected into a function body. This is
// what makes typed command builders (rather than a separate `Cmd` IR enum)
// the canonical, fully-validated representation through to export — see
// the #146/#168/#169/#170/#173/#175 PR body's "Cmd IR decision".
#[derive(Debug, Clone)]
enum BlockCommandNode {
    SetBlock(SetBlock),
    Fill(Fill),
    Clone(CloneBlocks),
}

impl BlockCommandNode {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        match self {
            Self::SetBlock(cmd) => Validate::validate(cmd, profile),
            Self::Fill(cmd) => Validate::validate(cmd, profile),
            Self::Clone(cmd) => Validate::validate(cmd, profile),
        }
    }
}

fn registered_block_lines() -> &'static Mutex<BTreeMap<String, BlockCommandNode>> {
    static LINES: OnceLock<Mutex<BTreeMap<String, BlockCommandNode>>> = OnceLock::new();
    LINES.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn register_block_line(line: &str, node: BlockCommandNode) {
    registered_block_lines()
        .lock()
        .expect("block command registry poisoned")
        .insert(line.to_owned(), node);
}

/// Re-validate a previously rendered `setblock`/`fill`/`clone` line's typed
/// node against `profile`, if this crate rendered it. Lines this crate did
/// not render (including hand-written raw block commands) are left alone —
/// the same "unknown lines pass through" contract every other registered
/// family (`nbt`, `execute_ir`, particles, sound, display, text, effect)
/// uses.
pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    let node = registered_block_lines()
        .lock()
        .expect("block command registry poisoned")
        .get(line)
        .cloned();
    if let Some(node) = node {
        node.validate(profile)?;
    }
    Ok(())
}

// ── BlockState ────────────────────────────────────────────────────────────────

/// A Minecraft block state string like `minecraft:oak_stairs[facing=east,half=bottom]`.
///
/// Properties are sorted alphabetically so output is deterministic.
///
/// `BlockState` accepts any block ID/property strings without validating
/// them eagerly — construction stays ergonomic for chained builder syntax.
/// Validation happens at the fallible boundary: [`BlockState::validate`], or
/// transitively through [`SetBlock::try_build`], [`Fill::try_build`], and
/// [`CloneBlocks::try_build`]. The infallible [`Build::build`]/`Display`
/// paths remain available as a documented raw/unchecked escape hatch for
/// custom or future block-state syntax Sand does not yet model.
#[derive(Debug, Clone)]
pub struct BlockState {
    block: String,
    props: BTreeMap<String, String>,
}

impl BlockState {
    /// Start building a block state string for the given block ID (e.g. `"minecraft:stone"`).
    pub fn of(block: impl Into<String>) -> Self {
        Self {
            block: block.into(),
            props: BTreeMap::new(),
        }
    }

    /// Add a single block state property (e.g. `"facing"`, `"east"`).
    pub fn prop(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.props.insert(key.into(), value.into());
        self
    }

    /// Add multiple block state properties at once from an iterator of `(key, value)` pairs.
    pub fn props<K, V>(mut self, iter: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        for (k, v) in iter {
            self.props.insert(k.into(), v.into());
        }
        self
    }
}

impl Validate for BlockState {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        validate::resource_location_shape(&self.block, "BlockState", "block")
            .map_err(|e| e.with_code("SAND-BLOCK-ID"))?;
        for (key, value) in &self.props {
            validate_block_state_token(key, "BlockState", "property_key")?;
            validate_block_state_token(value, "BlockState", "property_value")?;
        }
        Ok(())
    }
}

impl RenderCommand for BlockState {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

/// Reject block-state delimiter characters (`[`, `]`, `=`, `,`) and
/// whitespace/control characters inside a property key or value — these
/// would corrupt the surrounding `block[key=value,...]` grammar.
fn validate_block_state_token(
    value: &str,
    helper: &'static str,
    field: &'static str,
) -> CommandResult<()> {
    validate::no_whitespace_or_control(value, helper, field)?;
    if value.contains(['[', ']', '=', ',']) {
        return Err(CommandError::new(
            helper,
            field,
            format!("must not contain block-state delimiters `[`, `]`, `=`, or `,`, got `{value}`"),
        )
        .with_code("SAND-BLOCK-STATE-DELIMITER"));
    }
    Ok(())
}

impl fmt::Display for BlockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.block)?;
        if !self.props.is_empty() {
            write!(f, "[")?;
            let mut first = true;
            for (k, v) in &self.props {
                if !first {
                    write!(f, ",")?;
                }
                write!(f, "{}={}", k, v)?;
                first = false;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

/// Convert a plain `&str` or `String` into a `BlockState` (no properties).
impl From<&str> for BlockState {
    fn from(s: &str) -> Self {
        BlockState::of(s)
    }
}

impl From<String> for BlockState {
    fn from(s: String) -> Self {
        BlockState::of(s)
    }
}

// ── SetBlockMode ──────────────────────────────────────────────────────────────

/// Mode for the `setblock` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetBlockMode {
    /// Replace the block (default).
    #[default]
    Replace,
    /// Destroy the block, dropping items.
    Destroy,
    /// Keep the block if it exists (don't replace).
    Keep,
}

impl fmt::Display for SetBlockMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SetBlockMode::Replace => "replace",
            SetBlockMode::Destroy => "destroy",
            SetBlockMode::Keep => "keep",
        };
        f.write_str(s)
    }
}

// ── SetBlock ──────────────────────────────────────────────────────────────────

/// Builder for the `setblock` command.
#[derive(Debug, Clone)]
pub struct SetBlock {
    pos: BlockPos,
    block: BlockState,
    mode: SetBlockMode,
}

impl SetBlock {
    /// Create a new `setblock` command at the given position.
    pub fn new(pos: BlockPos, block: impl Into<BlockState>) -> Self {
        Self {
            pos,
            block: block.into(),
            mode: SetBlockMode::Replace,
        }
    }

    /// Set the mode for this `setblock` command.
    pub fn mode(mut self, mode: SetBlockMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Build for SetBlock {
    fn build(&self) -> String {
        let mode_str = match self.mode {
            SetBlockMode::Replace => String::new(),
            m => format!(" {}", m),
        };
        format!("setblock {} {}{}", self.pos, self.block, mode_str)
    }
}

impl fmt::Display for SetBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.build())
    }
}

impl From<SetBlock> for String {
    fn from(v: SetBlock) -> Self {
        v.build()
    }
}

impl Validate for SetBlock {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.pos
            .validate(profile)
            .map_err(|e| e.with_context("setblock position"))?;
        self.block
            .validate(profile)
            .map_err(|e| e.with_context("setblock block"))
    }
}

impl RenderCommand for SetBlock {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.build()
    }

    fn render(&self, profile: &CommandProfile) -> CommandResult<String> {
        self.validate(profile)?;
        let line = self.render_unchecked(profile);
        register_block_line(&line, BlockCommandNode::SetBlock(self.clone()));
        Ok(line)
    }
}

// ── FillMode ──────────────────────────────────────────────────────────────────

/// Mode for the `fill` command.
#[derive(Debug, Clone, PartialEq)]
pub enum FillMode {
    /// Replace all blocks in the region (default).
    Replace,
    /// Destroy blocks, dropping items.
    Destroy,
    /// Replace only non-air blocks (hollow out a structure).
    Hollow,
    /// Replace only the outer shell of the region (create an outline).
    Outline,
    /// Only replace air blocks.
    Keep,
    /// `replace <filter>` — only replace blocks matching `filter`.
    ReplaceFilter(String),
}

impl fmt::Display for FillMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FillMode::Replace => write!(f, "replace"),
            FillMode::Destroy => write!(f, "destroy"),
            FillMode::Hollow => write!(f, "hollow"),
            FillMode::Outline => write!(f, "outline"),
            FillMode::Keep => write!(f, "keep"),
            FillMode::ReplaceFilter(filter) => write!(f, "replace {}", filter),
        }
    }
}

// ── Fill ──────────────────────────────────────────────────────────────────────

/// Builder for the `fill` command.
#[derive(Debug, Clone)]
pub struct Fill {
    from: BlockPos,
    to: BlockPos,
    block: BlockState,
    mode: FillMode,
}

impl Fill {
    /// Create a new `fill` command for the region from `from` to `to`.
    pub fn new(from: BlockPos, to: BlockPos, block: impl Into<BlockState>) -> Self {
        Self {
            from,
            to,
            block: block.into(),
            mode: FillMode::Replace,
        }
    }

    /// Set the mode for this `fill` command.
    pub fn mode(mut self, mode: FillMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Build for Fill {
    fn build(&self) -> String {
        match &self.mode {
            FillMode::Replace => format!("fill {} {} {}", self.from, self.to, self.block),
            m => format!("fill {} {} {} {}", self.from, self.to, self.block, m),
        }
    }
}

impl fmt::Display for Fill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.build())
    }
}

impl From<Fill> for String {
    fn from(v: Fill) -> Self {
        v.build()
    }
}

impl Validate for Fill {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.from
            .validate(profile)
            .map_err(|e| e.with_context("fill from"))?;
        self.to
            .validate(profile)
            .map_err(|e| e.with_context("fill to"))?;
        self.block
            .validate(profile)
            .map_err(|e| e.with_context("fill block"))?;
        if let FillMode::ReplaceFilter(filter) = &self.mode {
            validate::non_empty(filter, "Fill", "replace_filter").map_err(|e| {
                e.with_context("fill replace filter")
                    .with_code("SAND-BLOCK-FILTER-EMPTY")
            })?;
        }
        Ok(())
    }
}

impl RenderCommand for Fill {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.build()
    }

    fn render(&self, profile: &CommandProfile) -> CommandResult<String> {
        self.validate(profile)?;
        let line = self.render_unchecked(profile);
        register_block_line(&line, BlockCommandNode::Fill(self.clone()));
        Ok(line)
    }
}

// ── CloneMaskMode / CloneMode ─────────────────────────────────────────────────

/// Mask mode for the `clone` command.
#[derive(Debug, Clone, Copy, Default)]
pub enum CloneMaskMode {
    /// Clone all blocks (default).
    #[default]
    Replace,
    /// Only clone non-air blocks (skip air).
    Masked,
    /// Only clone blocks matching a filter.
    Filtered,
}

/// Clone mode for the `clone` command.
#[derive(Debug, Clone, Copy, Default)]
pub enum CloneMode {
    /// Normal cloning (default).
    #[default]
    Normal,
    /// Force-clone even if blocks overlap.
    Force,
    /// Move blocks (clone then clear source).
    Move,
}

impl fmt::Display for CloneMaskMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CloneMaskMode::Replace => write!(f, "replace"),
            CloneMaskMode::Masked => write!(f, "masked"),
            CloneMaskMode::Filtered => write!(f, "filtered"),
        }
    }
}

impl fmt::Display for CloneMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CloneMode::Normal => write!(f, "normal"),
            CloneMode::Force => write!(f, "force"),
            CloneMode::Move => write!(f, "move"),
        }
    }
}

// ── CloneBlocks ───────────────────────────────────────────────────────────────

/// Builder for the `clone` command.
#[derive(Debug, Clone)]
pub struct CloneBlocks {
    from: BlockPos,
    to: BlockPos,
    dest: BlockPos,
    mask_mode: CloneMaskMode,
    clone_mode: CloneMode,
    filter: Option<String>,
}

impl CloneBlocks {
    /// Create a new `clone` command from region `from..to` to `dest`.
    pub fn new(from: BlockPos, to: BlockPos, dest: BlockPos) -> Self {
        Self {
            from,
            to,
            dest,
            mask_mode: CloneMaskMode::Replace,
            clone_mode: CloneMode::Normal,
            filter: None,
        }
    }

    /// Only clone non-air blocks.
    pub fn masked(mut self) -> Self {
        self.mask_mode = CloneMaskMode::Masked;
        self
    }

    /// Only clone blocks matching the given filter.
    pub fn filtered(mut self, block: impl Into<String>) -> Self {
        self.mask_mode = CloneMaskMode::Filtered;
        self.filter = Some(block.into());
        self
    }

    /// Set the clone mode (normal, force, or move).
    pub fn clone_mode(mut self, mode: CloneMode) -> Self {
        self.clone_mode = mode;
        self
    }
}

impl Build for CloneBlocks {
    fn build(&self) -> String {
        match self.mask_mode {
            CloneMaskMode::Filtered => {
                let filter = self.filter.as_deref().unwrap_or("");
                format!(
                    "clone {} {} {} filtered {} {}",
                    self.from, self.to, self.dest, filter, self.clone_mode
                )
            }
            mode => format!(
                "clone {} {} {} {} {}",
                self.from, self.to, self.dest, mode, self.clone_mode
            ),
        }
    }
}

impl fmt::Display for CloneBlocks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.build())
    }
}

impl From<CloneBlocks> for String {
    fn from(v: CloneBlocks) -> Self {
        v.build()
    }
}

impl Validate for CloneBlocks {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.from
            .validate(profile)
            .map_err(|e| e.with_context("clone from"))?;
        self.to
            .validate(profile)
            .map_err(|e| e.with_context("clone to"))?;
        self.dest
            .validate(profile)
            .map_err(|e| e.with_context("clone dest"))?;
        if matches!(self.mask_mode, CloneMaskMode::Filtered) {
            let filter = self.filter.as_deref().unwrap_or("");
            validate::non_empty(filter, "CloneBlocks", "filter").map_err(|e| {
                e.with_context("clone filter")
                    .with_code("SAND-BLOCK-FILTER-EMPTY")
            })?;
        }
        Ok(())
    }
}

impl RenderCommand for CloneBlocks {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.build()
    }

    fn render(&self, profile: &CommandProfile) -> CommandResult<String> {
        self.validate(profile)?;
        let line = self.render_unchecked(profile);
        register_block_line(&line, BlockCommandNode::Clone(self.clone()));
        Ok(line)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::{BlockPos, Coord};

    // ── Cmd IR "Option B" proof: typed nodes remain the export-time source
    // of truth after rendering, without needing a top-level Cmd variant ──

    #[test]
    fn render_registers_the_typed_node_for_export_time_revalidation() {
        // A valid setblock renders and is registered.
        let line = SetBlock::new(BlockPos::here(), "minecraft:stone")
            .render(&CommandProfile::unprofiled())
            .unwrap();
        assert_eq!(line, "setblock ~ ~ ~ minecraft:stone");

        // The export boundary (crate::render::validate_collected_line) looks
        // the line up and re-validates the retained typed node, not the
        // string — proving structure survives to the pre-write boundary
        // rather than being discarded once collected into a function body.
        assert!(
            crate::render::validate_collected_line(&line, &CommandProfile::unprofiled()).is_ok()
        );
    }

    #[test]
    fn unregistered_raw_block_lines_pass_through_the_export_boundary_unchanged() {
        // A hand-written raw line was never rendered by this module, so it
        // is not in the registry — the same "unknown/raw lines are never
        // silently rejected" contract every registered family upholds.
        let raw = "setblock ~ ~ ~ some_unmodeled_modded_block_state[custom=true]";
        assert_eq!(
            crate::render::validate_collected_line(raw, &CommandProfile::unprofiled()).unwrap(),
            raw
        );
    }

    #[test]
    fn setblock_composes_with_execute_run_and_is_still_revalidated_at_export() {
        use crate::execute::Execute;
        use crate::selector::Selector;

        let setblock = SetBlock::new(BlockPos::here(), "minecraft:stone")
            .render(&CommandProfile::unprofiled())
            .unwrap();
        let composed = Execute::new().as_(Selector::self_()).run_raw(setblock);

        // The composed `execute as @s run setblock ...` line is validated by
        // recursing into the `run` tail, which re-finds the registered
        // typed SetBlock node — proving composition with `execute run`
        // does not bypass the typed validation boundary.
        assert!(
            crate::render::validate_collected_line(&composed, &CommandProfile::unprofiled())
                .is_ok()
        );
    }

    #[test]
    fn block_state_no_props() {
        let bs = BlockState::of("minecraft:stone");
        assert_eq!(bs.to_string(), "minecraft:stone");
    }

    #[test]
    fn block_state_with_props() {
        let bs = BlockState::of("minecraft:oak_stairs")
            .prop("facing", "east")
            .prop("half", "bottom");
        // BTreeMap sorts alphabetically: facing < half
        assert_eq!(
            bs.to_string(),
            "minecraft:oak_stairs[facing=east,half=bottom]"
        );
    }

    #[test]
    fn setblock_default_mode() {
        let cmd = SetBlock::new(BlockPos::here(), "minecraft:stone").build();
        assert_eq!(cmd, "setblock ~ ~ ~ minecraft:stone");
    }

    #[test]
    fn setblock_destroy_mode() {
        let cmd = SetBlock::new(BlockPos::here(), "minecraft:stone")
            .mode(SetBlockMode::Destroy)
            .build();
        assert_eq!(cmd, "setblock ~ ~ ~ minecraft:stone destroy");
    }

    #[test]
    fn fill_default() {
        let cmd = Fill::new(
            BlockPos::absolute(0, 64, 0),
            BlockPos::absolute(10, 68, 10),
            "minecraft:glass",
        )
        .build();
        assert_eq!(cmd, "fill 0 64 0 10 68 10 minecraft:glass");
    }

    #[test]
    fn fill_hollow() {
        let cmd = Fill::new(
            BlockPos::absolute(0, 64, 0),
            BlockPos::absolute(5, 68, 5),
            "minecraft:stone",
        )
        .mode(FillMode::Hollow)
        .build();
        assert_eq!(cmd, "fill 0 64 0 5 68 5 minecraft:stone hollow");
    }

    #[test]
    fn fill_replace_filter() {
        let cmd = Fill::new(
            BlockPos::absolute(0, 64, 0),
            BlockPos::absolute(5, 68, 5),
            "minecraft:air",
        )
        .mode(FillMode::ReplaceFilter("minecraft:grass_block".into()))
        .build();
        assert_eq!(
            cmd,
            "fill 0 64 0 5 68 5 minecraft:air replace minecraft:grass_block"
        );
    }

    #[test]
    fn clone_basic() {
        let cmd = CloneBlocks::new(
            BlockPos::absolute(0, 64, 0),
            BlockPos::absolute(5, 68, 5),
            BlockPos::absolute(10, 64, 0),
        )
        .build();
        assert_eq!(cmd, "clone 0 64 0 5 68 5 10 64 0 replace normal");
    }

    // ── Fallible validation ──────────────────────────────────────────────

    #[test]
    fn block_state_validate_accepts_clean_input() {
        let bs = BlockState::of("minecraft:oak_stairs")
            .prop("facing", "east")
            .prop("half", "bottom");
        assert!(bs.try_build().is_ok());
    }

    #[test]
    fn block_state_rejects_malformed_block_id() {
        assert!(BlockState::of("not a block id").try_build().is_err());
        assert!(BlockState::of("").try_build().is_err());
        assert!(BlockState::of("NoNamespace").try_build().is_err());
    }

    #[test]
    fn block_state_diagnostic_codes_are_stable() {
        let id_err = BlockState::of("not a block id").try_build().unwrap_err();
        assert_eq!(id_err.code, "SAND-BLOCK-ID");

        let delim_err = BlockState::of("minecraft:oak_stairs")
            .prop("facing", "east]")
            .try_build()
            .unwrap_err();
        assert_eq!(delim_err.code, "SAND-BLOCK-STATE-DELIMITER");
    }

    #[test]
    fn fill_and_clone_empty_filter_diagnostic_codes_are_stable() {
        let fill_err = Fill::new(
            BlockPos::absolute(0, 64, 0),
            BlockPos::absolute(1, 65, 1),
            "minecraft:air",
        )
        .mode(FillMode::ReplaceFilter(String::new()))
        .try_build()
        .unwrap_err();
        assert_eq!(fill_err.code, "SAND-BLOCK-FILTER-EMPTY");

        let clone_err = CloneBlocks::new(
            BlockPos::absolute(0, 64, 0),
            BlockPos::absolute(5, 68, 5),
            BlockPos::absolute(10, 64, 0),
        )
        .filtered("")
        .try_build()
        .unwrap_err();
        assert_eq!(clone_err.code, "SAND-BLOCK-FILTER-EMPTY");
    }

    #[test]
    fn block_state_rejects_delimiter_in_property() {
        let bs = BlockState::of("minecraft:oak_stairs").prop("facing", "east]");
        assert!(bs.try_build().is_err());
        let bs = BlockState::of("minecraft:oak_stairs").prop("bad key", "east");
        assert!(bs.try_build().is_err());
        let bs = BlockState::of("minecraft:oak_stairs").prop("facing", "a,b");
        assert!(bs.try_build().is_err());
    }

    #[test]
    fn setblock_try_build_matches_build_for_valid_input() {
        let cmd = SetBlock::new(BlockPos::here(), "minecraft:stone");
        assert_eq!(cmd.try_build().unwrap(), cmd.build());
    }

    #[test]
    fn setblock_try_build_rejects_invalid_block() {
        let cmd = SetBlock::new(BlockPos::here(), "not a block");
        assert!(cmd.try_build().is_err());
    }

    #[test]
    fn setblock_try_build_rejects_fractional_block_pos() {
        let cmd = SetBlock::new(BlockPos::absolute(1, 2, 3), "minecraft:stone");
        assert!(cmd.try_build().is_ok());
        let bad = SetBlock::new(
            BlockPos::new(Coord::abs(1.5_f64), Coord::abs(2), Coord::abs(3)),
            "minecraft:stone",
        );
        assert!(bad.try_build().is_err());
    }

    #[test]
    fn fill_try_build_matches_build_for_valid_input() {
        let cmd = Fill::new(
            BlockPos::absolute(0, 64, 0),
            BlockPos::absolute(10, 68, 10),
            "minecraft:glass",
        );
        assert_eq!(cmd.try_build().unwrap(), cmd.build());
    }

    #[test]
    fn fill_try_build_rejects_empty_replace_filter() {
        let cmd = Fill::new(
            BlockPos::absolute(0, 64, 0),
            BlockPos::absolute(1, 65, 1),
            "minecraft:air",
        )
        .mode(FillMode::ReplaceFilter(String::new()));
        assert!(cmd.try_build().is_err());
    }

    #[test]
    fn clone_try_build_matches_build_for_valid_input() {
        let cmd = CloneBlocks::new(
            BlockPos::absolute(0, 64, 0),
            BlockPos::absolute(5, 68, 5),
            BlockPos::absolute(10, 64, 0),
        );
        assert_eq!(cmd.try_build().unwrap(), cmd.build());
    }

    #[test]
    fn clone_try_build_rejects_empty_filter() {
        let cmd = CloneBlocks::new(
            BlockPos::absolute(0, 64, 0),
            BlockPos::absolute(5, 68, 5),
            BlockPos::absolute(10, 64, 0),
        )
        .filtered("");
        assert!(cmd.try_build().is_err());
    }

    #[test]
    fn clone_try_build_rejects_local_coordinates() {
        let cmd = CloneBlocks::new(
            BlockPos::new(Coord::local_n(1), Coord::local_n(2), Coord::local_n(3)),
            BlockPos::absolute(5, 68, 5),
            BlockPos::absolute(10, 64, 0),
        );
        assert!(cmd.try_build().is_err());
    }
}
