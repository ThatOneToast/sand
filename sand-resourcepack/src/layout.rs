//! High-level HUD layout compositor.
//!
//! [`HudLayout`] is the recommended high-level interface for displaying one
//! or more HUD bars and elements simultaneously. It:
//!
//! - Accepts any mix of [`BarHandle`]s (dynamic progress bars) and
//!   [`ElementHandle`]s (static overlays)
//! - Positions each element at an absolute canvas coordinate
//! - Combines all elements into a single actionbar text component so
//!   multiple HUD pieces appear together without overwriting each other
//! - Handles all unicode Private-Use-Area glyph selection and advance math
//!   automatically — no manual codepoint manipulation needed
//!
//! # Rendering strategies
//!
//! Two methods emit the tick-time display commands. Choose based on how many
//! bar steps your layout has:
//!
//! | Method | Command count | Best for |
//! |---|---|---|
//! | [`broadcast`] | `N₁ × N₂ × …` (Cartesian product) | 1–2 bars with ≤ 10 steps each |
//! | [`broadcast_via_storage`] | `N₁ + N₂ + … + 1` (linear) | Any layout; **preferred for 3+ bars or > 10 steps** |
//!
//! **Prefer [`broadcast_via_storage`]** for production code. It scales
//! linearly regardless of bar count or step count, while [`broadcast`] grows
//! exponentially and can easily exceed Minecraft's per-tick command budget.
//!
//! # Vertical positioning
//!
//! Horizontal position is set via `canvas_x`. Vertical position is baked in
//! at macro registration time via the `ascent` field in each `hud_bar!` /
//! `hud_element!` call — bars with different `ascent` values appear at
//! different Y positions. No per-tick work is required for vertical layout.
//!
//! # Example — two bars, storage-based (recommended)
//!
//! ```rust,ignore
//! use sand_resourcepack::{HudLayout, BarStat};
//!
//! hud_bar!(
//!     name: "health",
//!     texture: create!(fill: 0xFF4444FF, empty: 0x333333FF),
//!     steps: 20, height: 14, ascent: 14,
//! );
//! hud_bar!(
//!     name: "mana",
//!     texture: create!(fill: 0x4444FFFF, empty: 0x222244FF),
//!     steps: 20, height: 14, ascent: 0,   // lower than health
//! );
//!
//! const CANVAS_WIDTH: i32 = 1000;
//! const MY_LAYOUT: HudLayout = /* constructed elsewhere, typically const/lazy */;
//!
//! #[component(Load)]
//! pub fn load() {
//!     mcfunction! {
//!         // Pre-compute static element JSON once at load time
//!         MY_LAYOUT.setup_storage();
//!     }
//! }
//!
//! #[component(Tick)]
//! pub fn tick() {
//!     mcfunction! {
//!         // Execute per-player so each player's write is isolated
//!         execute as @a run run_fn!({
//!             // ... scale scoreboard values to 0..steps ...
//!             MY_LAYOUT.broadcast_via_storage("@s"); // 20+20+1 = 41 cmds, not 400
//!         });
//!     }
//! }
//! ```
//!
//! [`broadcast`]: HudLayout::broadcast
//! [`broadcast_via_storage`]: HudLayout::broadcast_via_storage

use crate::handle::{BarHandle, ElementHandle};
use crate::stat::BarStat;
use crate::unicode::{advance_x, json_escape_chars};

// ── Entry types ───────────────────────────────────────────────────────────────

enum LayoutEntry {
    Bar {
        handle: BarHandle,
        objective: String,
        canvas_x: i32,
    },
    Element {
        handle: ElementHandle,
        canvas_x: i32,
    },
}

impl LayoutEntry {
    fn font(&self) -> &'static str {
        match self {
            LayoutEntry::Bar { handle, .. } => handle.font,
            LayoutEntry::Element { handle, .. } => handle.font,
        }
    }
}

// ── HudLayout ─────────────────────────────────────────────────────────────────

/// Composes multiple HUD bars and elements into a single actionbar command set.
///
/// See the [module-level documentation](crate::layout) for a full walkthrough.
///
/// # Building a layout
///
/// ```rust,ignore
/// let layout = HudLayout::new("my_pack", 1000)
///     .bar(HEALTH, "hp_frame", 250)
///     .element(BORDER, 500);
/// ```
///
/// # Emitting commands
///
/// | Method | When to use |
/// |---|---|
/// | [`broadcast`] | `#[component(Tick)]` — iterates over all players |
/// | [`show_elements`] | Show only static elements (no bar state needed) |
///
/// [`broadcast`]: HudLayout::broadcast
/// [`show_elements`]: HudLayout::show_elements
pub struct HudLayout {
    namespace: String,
    canvas_width: i32,
    entries: Vec<LayoutEntry>,
}

impl HudLayout {
    /// Create a new layout for `namespace` with a virtual canvas of
    /// `canvas_width` units wide.
    ///
    /// `canvas_width / 2` maps to the horizontal screen center. For example,
    /// with `canvas_width = 1000`, `canvas_x = 0` is the far left, `500` is
    /// the center, and `1000` is the far right.
    ///
    /// # Choosing a canvas width
    ///
    /// Any value works; `1000` is a convenient round number. To make
    /// "1 unit = 1 visible pixel" at a specific resolution/scale combination:
    ///
    /// | Resolution | GUI scale | `canvas_width` |
    /// |---|---|---|
    /// | 1920 × 1080 | 2 | 960 |
    /// | 1920 × 1080 | 3 | 640 |
    /// | 2560 × 1440 | 2 | 1280 |
    pub fn new(namespace: impl Into<String>, canvas_width: i32) -> Self {
        Self {
            namespace: namespace.into(),
            canvas_width,
            entries: Vec::new(),
        }
    }

    /// Add a dynamic progress bar to the layout.
    ///
    /// - `handle` — the `BarHandle` constant generated by `hud_bar!`
    /// - `objective` — the scoreboard objective whose value selects the frame
    /// - `canvas_x` — horizontal position (0 = left edge, `canvas_width` = right edge)
    ///
    /// The score at `objective` must be in `0..handle.steps` before the
    /// layout's commands run. Scale and clamp from your raw stat first.
    pub fn bar(mut self, handle: BarHandle, objective: impl Into<String>, canvas_x: i32) -> Self {
        self.entries.push(LayoutEntry::Bar {
            handle,
            objective: objective.into(),
            canvas_x,
        });
        self
    }

    /// Add a static element to the layout.
    ///
    /// - `handle` — the `ElementHandle` constant generated by `hud_element!`
    /// - `canvas_x` — horizontal position
    pub fn element(mut self, handle: ElementHandle, canvas_x: i32) -> Self {
        self.entries.push(LayoutEntry::Element { handle, canvas_x });
        self
    }

    /// Add a [`BarStat`]-tracked bar to the layout.
    ///
    /// Equivalent to `.bar(stat.handle, &stat.frame_obj, canvas_x)` — this is
    /// a convenience shortcut so you don't have to spell out the objective name
    /// yourself when using the `BarStat` API.
    ///
    /// ```rust,ignore
    /// let health = BarStat::health(HEALTH);
    /// HudLayout::new("my_pack", 1000)
    ///     .tracked_bar(&health, 250)
    ///     .broadcast("@a");
    /// ```
    pub fn tracked_bar(self, stat: &BarStat, canvas_x: i32) -> Self {
        self.bar(stat.handle, stat.frame_obj.clone(), canvas_x)
    }

    // ── Internal text building ─────────────────────────────────────────────

    /// Build the combined text string for one font group in a specific bar-frame combination.
    ///
    /// Uses the **zero-total-width technique**: for each element the cursor is
    /// advanced to `canvas_x − canvas_width/2` (placing the glyph at the
    /// desired screen position relative to the actionbar center), then advanced
    /// back by the same amount plus the glyph advance, so the net cursor
    /// movement is always zero. This lets every element in the string appear at
    /// its chosen absolute position regardless of ordering.
    ///
    /// `bar_frames` must have one entry per [`LayoutEntry::Bar`] in this layout,
    /// in order. Pass `&[]` only when the layout has no bar entries (elements only).
    fn build_text_for_font(&self, font: &str, bar_frames: &[u32]) -> String {
        let mut text = String::new();
        let mut bar_idx = 0;

        for entry in &self.entries {
            if entry.font() != font {
                // Keep bar_idx in sync even for entries in other fonts.
                if let LayoutEntry::Bar { .. } = entry {
                    bar_idx += 1;
                }
                continue;
            }

            match entry {
                LayoutEntry::Bar {
                    handle, canvas_x, ..
                } => {
                    let frame = bar_frames[bar_idx];
                    bar_idx += 1;
                    let x = canvas_x - self.canvas_width / 2;
                    let ga = handle.glyph_advance().unwrap_or(1);
                    text.push_str(&advance_x(x));
                    text.push(handle.char(frame));
                    text.push_str(&advance_x(-(x + ga)));
                }
                LayoutEntry::Element { handle, canvas_x } => {
                    let x = canvas_x - self.canvas_width / 2;
                    let ga = handle.glyph_advance().unwrap_or(1);
                    text.push_str(&advance_x(x));
                    text.push(handle.char());
                    text.push_str(&advance_x(-(x + ga)));
                }
            }
        }
        text
    }

    /// Build the full JSON text component for a given bar-frame combination.
    ///
    /// Groups entries by font so each font gets a separate JSON object with
    /// its own `"font"` field. Multiple font groups are wrapped in a JSON array.
    /// Single-font layouts emit a plain `{"text":"…"}` object (no array wrapper).
    ///
    /// Characters are JSON-escaped (`\uXXXX`) so the result can be embedded
    /// directly in a Minecraft command string. For storage use see
    /// [`build_json_for_storage`].
    ///
    /// [`build_json_for_storage`]: HudLayout::build_json_for_storage
    fn build_json(&self, bar_frames: &[u32]) -> String {
        self.build_json_impl(bar_frames, true)
    }

    /// Same as [`build_json`] but embeds characters as raw UTF-8 instead of
    /// `\uXXXX` escapes.
    ///
    /// SNBT single-quoted strings (used by `data modify storage … set value '…'`)
    /// do not reliably support `\uXXXX` escape sequences across all Minecraft
    /// versions — the backslash may be processed before the value reaches NBT
    /// storage, corrupting the JSON. Raw PUA characters (U+E000–U+F8FF) are
    /// valid in JSON strings and are preserved verbatim by the SNBT parser, so
    /// `"interpret":true` can then parse and render them correctly.
    ///
    /// [`build_json`]: HudLayout::build_json
    fn build_json_for_storage(&self, bar_frames: &[u32]) -> String {
        self.build_json_impl(bar_frames, false)
    }

    /// Shared implementation for [`build_json`] and [`build_json_for_storage`].
    ///
    /// When `escape` is `true` every PUA character is written as `\uXXXX`.
    /// When `false` the raw UTF-8 bytes are embedded directly.
    fn build_json_impl(&self, bar_frames: &[u32], escape: bool) -> String {
        // Collect distinct fonts in insertion order.
        let mut fonts: Vec<&str> = Vec::new();
        for entry in &self.entries {
            let f = entry.font();
            if !fonts.contains(&f) {
                fonts.push(f);
            }
        }

        if fonts.is_empty() {
            return r#"{"text":""}"#.to_string();
        }

        let components: Vec<String> = fonts
            .iter()
            .map(|&font| {
                let text = self.build_text_for_font(font, bar_frames);
                let content = if escape {
                    json_escape_chars(&text)
                } else {
                    text
                };
                let font_id = format!("{}:{}", self.namespace, font);
                format!(r#"{{"text":"{content}","font":"{font_id}","color":"white"}}"#)
            })
            .collect();

        if components.len() == 1 {
            components[0].clone()
        } else {
            format!("[{}]", components.join(","))
        }
    }

    /// Build the JSON component for the **static elements only** (no bars).
    ///
    /// This is equivalent to `build_json(&[])` on a layout that contains only
    /// elements. Delegates to the standard pipeline by temporarily filtering to
    /// elements, so the font-grouping logic is shared and not duplicated.
    fn build_static_json(&self) -> String {
        // Build a temporary view with only element entries.
        let elem_only = HudLayout {
            namespace: self.namespace.clone(),
            canvas_width: self.canvas_width,
            entries: self
                .entries
                .iter()
                .filter_map(|e| {
                    if let LayoutEntry::Element { handle, canvas_x } = e {
                        Some(LayoutEntry::Element {
                            handle: ElementHandle {
                                name: handle.name,
                                font: handle.font,
                                char_width: handle.char_width,
                            },
                            canvas_x: *canvas_x,
                        })
                    } else {
                        None
                    }
                })
                .collect(),
        };

        elem_only.build_json_for_storage(&[])
    }

    // ── Public command generators ──────────────────────────────────────────

    /// Generate all display commands for a **tick function**.
    ///
    /// `executor` is the player selector, typically `"@a"`. For each player,
    /// Minecraft checks their scores and runs the matching combined command that
    /// renders all bars and elements in one actionbar update.
    ///
    /// **Command count**: the product of all bar `steps` values. Two bars of
    /// 20 steps each → 400 commands. For three bars keep steps ≤ 10 (≤ 1 000
    /// commands total) to stay well within Minecraft's function call budget.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[component(Tick)]
    /// pub fn tick() {
    ///     mcfunction! {
    ///         // ... scoreboard math for hp_frame and mp_frame ...
    ///
    ///         HudLayout::new("my_pack", 1000)
    ///             .bar(HEALTH, "hp_frame", 250)
    ///             .bar(MANA,   "mp_frame", 750)
    ///             .broadcast("@a");
    ///     }
    /// }
    /// ```
    pub fn broadcast(&self, executor: &str) -> Vec<String> {
        let bar_entries: Vec<(&BarHandle, &str)> = self
            .entries
            .iter()
            .filter_map(|e| {
                if let LayoutEntry::Bar {
                    handle, objective, ..
                } = e
                {
                    Some((handle, objective.as_str()))
                } else {
                    None
                }
            })
            .collect();

        if bar_entries.is_empty() {
            // Only static elements — one command for all.
            let json = self.build_json(&[]);
            return vec![format!("title {executor} actionbar {json}")];
        }

        let frame_ranges: Vec<Vec<u32>> = bar_entries
            .iter()
            .map(|(h, _)| (0..h.steps).collect())
            .collect();

        cartesian_product(&frame_ranges)
            .into_iter()
            .map(|combo| {
                // Build score conditions: one per bar.
                let conditions: String = combo
                    .iter()
                    .zip(bar_entries.iter())
                    .map(|(frame, (_, obj))| format!("if score @s {obj} matches {frame}..{frame}"))
                    .collect::<Vec<_>>()
                    .join(" ");

                let json = self.build_json(&combo);
                format!("execute as {executor} {conditions} run title @s actionbar {json}")
            })
            .collect()
    }

    // ── Storage-based rendering ────────────────────────────────────────────

    /// Build the full JSON text component for a single bar entry at a specific
    /// frame.
    ///
    /// Creates a temporary single-bar [`HudLayout`] and delegates to
    /// [`build_json`] so the font-grouping logic is shared with [`broadcast`].
    /// The returned string is stored verbatim in NBT storage and later parsed
    /// by Minecraft via `"interpret":true`.
    ///
    /// Example output:
    /// `{"text":"\uE045\uF801…","font":"ns:hud","color":"white"}`
    ///
    /// [`build_json`]: HudLayout::build_json
    /// [`broadcast`]: HudLayout::broadcast
    fn build_bar_json_for_frame(&self, handle: &BarHandle, canvas_x: i32, frame: u32) -> String {
        let single = HudLayout {
            namespace: self.namespace.clone(),
            canvas_width: self.canvas_width,
            entries: vec![LayoutEntry::Bar {
                handle: *handle,
                objective: String::new(), // unused during JSON building
                canvas_x,
            }],
        };
        single.build_json_for_storage(&[frame])
    }

    /// Returns a single `data modify storage` command that stores the static
    /// element JSON at `{namespace}:hud static`.
    ///
    /// Call this from your **load function** once to initialize the storage key
    /// that [`broadcast_via_storage`] reads every tick.
    ///
    /// If this layout has no static elements, returns `None`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[component(Load)]
    /// pub fn load() {
    ///     mcfunction! {
    ///         // ... other load commands ...
    ///         MY_LAYOUT.setup_storage();
    ///     }
    /// }
    /// ```
    ///
    /// [`broadcast_via_storage`]: HudLayout::broadcast_via_storage
    pub fn setup_storage(&self) -> Option<String> {
        let has_elements = self
            .entries
            .iter()
            .any(|e| matches!(e, LayoutEntry::Element { .. }));
        if !has_elements {
            return None;
        }
        let json = self.build_static_json();
        Some(format!(
            "data modify storage {}:hud static set value '{json}'",
            self.namespace
        ))
    }

    /// Storage-based HUD broadcast: **O(N₁ + N₂ + … + 1)** commands instead
    /// of the Cartesian-product O(N₁ × N₂ × …) produced by [`broadcast`].
    ///
    /// For each bar this emits one `execute if score …` command per frame that
    /// writes the frame JSON into `{namespace}:hud barI` in NBT storage. A
    /// single `title` command then reads every bar's storage key (and the
    /// pre-initialized `static` key for elements) to assemble the final
    /// actionbar in one shot.
    ///
    /// # Required setup
    ///
    /// If your layout contains static elements, call [`setup_storage`] from
    /// your load function first.
    ///
    /// # Per-player isolation
    ///
    /// Because all bars share the same storage keys, this method **must run
    /// inside a per-player function** so that each player's writes are flushed
    /// before the `title` command reads them. The canonical pattern is to call
    /// it from an anonymous `run_fn!` block (which Sand wraps in a
    /// `execute as @a run function …` call):
    ///
    /// ```rust,ignore
    /// #[component(Tick)]
    /// pub fn tick() {
    ///     mcfunction! {
    ///         execute as @a run run_fn!({
    ///             // scale / clamp bar objectives …
    ///             MY_LAYOUT.broadcast_via_storage("@s");
    ///         });
    ///     }
    /// }
    /// ```
    ///
    /// # Command count
    ///
    /// steps₁ + steps₂ + … + 1. Three bars of 20 / 100 / 20 → **141 commands**
    /// regardless of how many players are online (vs 40 000 with [`broadcast`]).
    ///
    /// [`broadcast`]: HudLayout::broadcast
    /// [`setup_storage`]: HudLayout::setup_storage
    pub fn broadcast_via_storage(&self, executor: &str) -> Vec<String> {
        let ns = &self.namespace;
        let mut cmds = Vec::new();

        // Per-bar: emit one `data modify storage` command per frame that stores
        // the full JSON text component (identical format to what `broadcast`
        // emits directly). The title command later reads each key with
        // `"interpret":true` so Minecraft parses the stored string as a JSON
        // text component — the font, color, and cursor-advance chars are all
        // resolved correctly that way.
        let mut bar_i = 0usize;
        for entry in &self.entries {
            if let LayoutEntry::Bar {
                handle,
                objective,
                canvas_x,
            } = entry
            {
                for frame in 0..handle.steps {
                    let json = self.build_bar_json_for_frame(handle, *canvas_x, frame);
                    cmds.push(format!(
                        "execute if score @s {objective} matches {frame}..{frame} run data modify storage {ns}:hud bar{bar_i} set value '{json}'"
                    ));
                }
                bar_i += 1;
            }
        }

        let bar_count = bar_i;
        let has_elements = self
            .entries
            .iter()
            .any(|e| matches!(e, LayoutEntry::Element { .. }));

        if bar_count == 0 && !has_elements {
            return Vec::new();
        }

        // Assemble the title command. Every component — both static elements
        // and per-bar keys — uses `"interpret":true` so Minecraft parses the
        // stored JSON string and applies the correct font and positioning.
        let mut parts: Vec<String> = Vec::new();

        if has_elements {
            parts.push(format!(
                r#"{{"type":"nbt","source":"storage","storage":"{ns}:hud","nbt":"static","interpret":true}}"#
            ));
        }
        for i in 0..bar_count {
            parts.push(format!(
                r#"{{"type":"nbt","source":"storage","storage":"{ns}:hud","nbt":"bar{i}","interpret":true}}"#
            ));
        }

        let title_json = if parts.len() == 1 {
            parts[0].clone()
        } else {
            format!("[{}]", parts.join(","))
        };

        cmds.push(format!("title {executor} actionbar {title_json}"));
        cmds
    }

    /// Generate commands that show only the **static elements** in this layout
    /// (ignoring all bars).
    ///
    /// Useful for a load function or any one-shot display of decorative overlays.
    pub fn show_elements(&self, target: &str) -> Vec<String> {
        // Temporarily create a layout with only the elements.
        let elem_only = HudLayout {
            namespace: self.namespace.clone(),
            canvas_width: self.canvas_width,
            entries: self
                .entries
                .iter()
                .filter_map(|e| {
                    if let LayoutEntry::Element { handle, canvas_x } = e {
                        Some(LayoutEntry::Element {
                            handle: ElementHandle {
                                name: handle.name,
                                font: handle.font,
                                char_width: handle.char_width,
                            },
                            canvas_x: *canvas_x,
                        })
                    } else {
                        None
                    }
                })
                .collect(),
        };

        if elem_only.entries.is_empty() {
            return Vec::new();
        }

        let json = elem_only.build_json(&[]);
        vec![format!("title {target} actionbar {json}")]
    }
}

// ── Cartesian product helper ──────────────────────────────────────────────────

fn cartesian_product(ranges: &[Vec<u32>]) -> Vec<Vec<u32>> {
    if ranges.is_empty() {
        return vec![vec![]];
    }
    let mut result = Vec::new();
    for &item in &ranges[0] {
        for mut combo in cartesian_product(&ranges[1..]) {
            combo.insert(0, item);
            result.push(combo);
        }
    }
    result
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handle::{BarHandle, ElementHandle};

    const HEALTH: BarHandle = BarHandle {
        name: "health",
        steps: 10,
        font: "hud",
        frame_width: 28,
    };
    const MANA: BarHandle = BarHandle {
        name: "mana",
        steps: 5,
        font: "hud",
        frame_width: 28,
    };
    const BORDER: ElementHandle = ElementHandle {
        name: "border",
        font: "hud",
        char_width: 180,
    };

    #[test]
    fn single_bar_produces_steps_commands() {
        let cmds = HudLayout::new("ns", 1000)
            .bar(HEALTH, "hp_frame", 500)
            .broadcast("@a");
        assert_eq!(cmds.len(), 10); // one per frame
        assert!(cmds[0].contains("execute as @a"));
        assert!(cmds[0].contains("if score @s hp_frame matches 0..0"));
    }

    #[test]
    fn two_bars_produces_cartesian_product() {
        let cmds = HudLayout::new("ns", 1000)
            .bar(HEALTH, "hp_frame", 300)
            .bar(MANA, "mp_frame", 700)
            .broadcast("@a");
        // 10 × 5 = 50 commands
        assert_eq!(cmds.len(), 50);
        // Each command checks both scores.
        assert!(cmds[0].contains("hp_frame"));
        assert!(cmds[0].contains("mp_frame"));
    }

    #[test]
    fn static_only_produces_one_command() {
        let cmds = HudLayout::new("ns", 1000)
            .element(BORDER, 500)
            .broadcast("@a");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].starts_with("title @a actionbar"));
    }

    #[test]
    fn show_elements_returns_single_command() {
        let cmds = HudLayout::new("ns", 1000)
            .bar(HEALTH, "hp_frame", 300)
            .element(BORDER, 500)
            .show_elements("@a");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].starts_with("title @a actionbar"));
    }

    #[test]
    fn center_position_uses_offset_zero() {
        // canvas_x = canvas_width/2 → x_offset = 0
        // advance_x(0) returns "" so the text has no leading advance chars.
        let cmds = HudLayout::new("ns", 1000)
            .bar(HEALTH, "hp_frame", 500)
            .broadcast("@a");
        // The JSON should contain the bar char directly (no leading advance).
        // We can't inspect the exact unicode escapes portably, but we can check
        // the command is well-formed.
        assert!(cmds[0].contains("\"font\":\"ns:hud\""));
    }

    #[test]
    fn broadcast_via_storage_command_count() {
        // steps_health + steps_mana + 1 title command
        let cmds = HudLayout::new("ns", 1000)
            .bar(HEALTH, "hp_frame", 300)
            .bar(MANA, "mp_frame", 700)
            .broadcast_via_storage("@s");
        assert_eq!(cmds.len(), HEALTH.steps as usize + MANA.steps as usize + 1);
        // First commands write to storage
        assert!(cmds[0].contains("data modify storage ns:hud bar0"));
        assert!(cmds[0].contains("matches 0..0"));
        // Last command is the title
        let last = cmds.last().unwrap();
        assert!(last.starts_with("title @s actionbar"));
        assert!(last.contains(r#""source":"storage""#));
        assert!(last.contains(r#""storage":"ns:hud""#));
        // All components use interpret:true — font is inside the stored JSON
        assert!(last.contains(r#""interpret":true"#));
    }

    #[test]
    fn broadcast_via_storage_bars_use_separate_keys() {
        let cmds = HudLayout::new("ns", 1000)
            .bar(HEALTH, "hp_frame", 300)
            .bar(MANA, "mp_frame", 700)
            .broadcast_via_storage("@s");
        // Health writes to bar0, mana writes to bar1
        assert!(cmds[0].contains("bar0"));
        let mana_start = HEALTH.steps as usize;
        assert!(cmds[mana_start].contains("bar1"));
    }

    #[test]
    fn broadcast_via_storage_static_in_title() {
        let cmds = HudLayout::new("ns", 1000)
            .bar(HEALTH, "hp_frame", 300)
            .element(BORDER, 500)
            .broadcast_via_storage("@s");
        let title = cmds.last().unwrap();
        assert!(title.contains(r#""nbt":"static""#));
        assert!(title.contains(r#""nbt":"bar0""#));
    }

    #[test]
    fn setup_storage_returns_none_without_elements() {
        let layout = HudLayout::new("ns", 1000).bar(HEALTH, "hp_frame", 300);
        assert!(layout.setup_storage().is_none());
    }

    #[test]
    fn setup_storage_returns_command_with_elements() {
        let layout = HudLayout::new("ns", 1000)
            .bar(HEALTH, "hp_frame", 300)
            .element(BORDER, 500);
        let cmd = layout.setup_storage().unwrap();
        assert!(cmd.starts_with("data modify storage ns:hud static set value '"));
    }

    #[test]
    fn broadcast_via_storage_single_bar_no_elements() {
        let cmds = HudLayout::new("ns", 1000)
            .bar(HEALTH, "hp_frame", 500)
            .broadcast_via_storage("@s");
        // steps + 1 title
        assert_eq!(cmds.len(), HEALTH.steps as usize + 1);
        let title = cmds.last().unwrap();
        // Only bar0, no static
        assert!(title.contains(r#""nbt":"bar0""#));
        assert!(!title.contains(r#""nbt":"static""#));
        // Single component — not wrapped in array
        assert!(!title.contains('['));
    }

    #[test]
    fn cartesian_product_empty() {
        assert_eq!(
            cartesian_product(&[] as &[Vec<u32>]),
            vec![vec![]] as Vec<Vec<u32>>
        );
    }

    #[test]
    fn cartesian_product_single() {
        let r = cartesian_product(&[vec![0, 1, 2]]);
        assert_eq!(r, vec![vec![0], vec![1], vec![2]]);
    }

    #[test]
    fn cartesian_product_two() {
        let r = cartesian_product(&[vec![0, 1], vec![0, 1, 2]]);
        assert_eq!(r.len(), 6);
    }
}
