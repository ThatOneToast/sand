/// Block placement and manipulation commands: `setblock`, `fill`, and a
/// `BlockState` builder for composing block state strings.
///
/// # Example
/// ```rust,ignore
/// // Simple setblock
/// let cmd = SetBlock::new(BlockPos::here(), "minecraft:stone").build();
/// // → "setblock ~ ~ ~ minecraft:stone"
///
/// // Setblock with state
/// let cmd = SetBlock::new(BlockPos::here(),
///     BlockState::of("minecraft:oak_stairs")
///         .prop("facing", "east")
///         .prop("half", "bottom"))
///     .mode(SetBlockMode::Replace)
///     .build();
/// // → "setblock ~ ~ ~ minecraft:oak_stairs[facing=east,half=bottom] replace"
///
/// // Fill a region
/// let cmd = Fill::new(BlockPos::absolute(0, 64, 0), BlockPos::absolute(10, 68, 10),
///     "minecraft:glass")
///     .mode(FillMode::Hollow)
///     .build();
/// // → "fill 0 64 0 10 68 10 minecraft:glass hollow"
/// ```
use std::collections::BTreeMap;
use std::fmt;

use super::coord::BlockPos;

// ── BlockState ────────────────────────────────────────────────────────────────

/// A Minecraft block state string like `minecraft:oak_stairs[facing=east,half=bottom]`.
///
/// Properties are sorted alphabetically so output is deterministic.
#[derive(Debug, Clone)]
pub struct BlockState {
    block: String,
    props: BTreeMap<String, String>,
}

impl BlockState {
    /// Start building a block state for `block` (e.g. `"minecraft:stone"`).
    pub fn of(block: impl Into<String>) -> Self {
        Self {
            block: block.into(),
            props: BTreeMap::new(),
        }
    }

    /// Add a block state property.
    pub fn prop(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.props.insert(key.into(), value.into());
        self
    }

    /// Add multiple properties at once from an iterator of `(key, value)` pairs.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetBlockMode {
    #[default]
    Replace,
    Destroy,
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

pub struct SetBlock {
    pos: BlockPos,
    block: BlockState,
    mode: SetBlockMode,
}

impl SetBlock {
    pub fn new(pos: BlockPos, block: impl Into<BlockState>) -> Self {
        Self {
            pos,
            block: block.into(),
            mode: SetBlockMode::Replace,
        }
    }

    pub fn mode(mut self, mode: SetBlockMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn build(self) -> String {
        let mode_str = match self.mode {
            SetBlockMode::Replace => String::new(), // default, can omit
            m => format!(" {}", m),
        };
        format!("setblock {} {}{}", self.pos, self.block, mode_str)
    }
}

// ── FillMode ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FillMode {
    Replace,
    Destroy,
    Hollow,
    Outline,
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

pub struct Fill {
    from: BlockPos,
    to: BlockPos,
    block: BlockState,
    mode: FillMode,
}

impl Fill {
    pub fn new(from: BlockPos, to: BlockPos, block: impl Into<BlockState>) -> Self {
        Self {
            from,
            to,
            block: block.into(),
            mode: FillMode::Replace,
        }
    }

    pub fn mode(mut self, mode: FillMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn build(self) -> String {
        match &self.mode {
            FillMode::Replace => format!("fill {} {} {}", self.from, self.to, self.block),
            m => format!("fill {} {} {} {}", self.from, self.to, self.block, m),
        }
    }
}

// ── Clone ─────────────────────────────────────────────────────────────────────

/// Builder for the `clone` command.
pub struct CloneBlocks {
    from: BlockPos,
    to: BlockPos,
    dest: BlockPos,
    mask_mode: CloneMaskMode,
    clone_mode: CloneMode,
    filter: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CloneMaskMode {
    #[default]
    Replace,
    Masked,
    Filtered,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CloneMode {
    #[default]
    Normal,
    Force,
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

impl CloneBlocks {
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

    pub fn masked(mut self) -> Self {
        self.mask_mode = CloneMaskMode::Masked;
        self
    }

    /// `filtered <block>` — only clone blocks matching `block`.
    pub fn filtered(mut self, block: impl Into<String>) -> Self {
        self.mask_mode = CloneMaskMode::Filtered;
        self.filter = Some(block.into());
        self
    }

    pub fn clone_mode(mut self, mode: CloneMode) -> Self {
        self.clone_mode = mode;
        self
    }

    pub fn build(self) -> String {
        match self.mask_mode {
            CloneMaskMode::Filtered => {
                let filter = self.filter.unwrap_or_default();
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::coord::BlockPos;

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
}
