/// A lightweight handle to a registered [`hud_bar!`](sand_macros::hud_bar)
/// component, generated automatically by the macro.
///
/// # What is a frame?
///
/// A progress bar is represented in Minecraft as a **sprite strip** — a single
/// PNG image that contains every possible fill state laid out horizontally,
/// left to right. Each column of pixels in that strip is called a **frame**.
///
/// ```text
/// steps = 5, each frame is 9 × 9 px:
///
///  frame 0      frame 1      frame 2      frame 3      frame 4
/// ┌─────────┬─────────┬─────────┬─────────┬─────────┐
/// │░░░░░░░░░│▓░░░░░░░░│▓▓░░░░░░░│▓▓▓░░░░░░│▓▓▓▓▓▓▓▓▓│
/// └─────────┴─────────┴─────────┴─────────┴─────────┘
///  0 % full  25 % full  50 % full  75 % full  100 % full
/// ```
///
/// **Frame 0** is the completely empty state. **Frame `steps - 1`** is
/// completely full. Everything in between is a partial fill.
///
/// Each frame is assigned its own unicode character in the Private Use Area.
/// Sand handles the character assignments automatically — you never choose
/// codepoints by hand.
///
/// # Choosing the right method
///
/// | Situation | Method |
/// |---|---|
/// | You know the frame at compile time (e.g., always show frame 0 as a default) | [`show`] |
/// | The frame changes at runtime based on a scoreboard value, in a per-player context | [`display_commands`] |
/// | The frame changes at runtime, called from a `#[component(Tick)]` function | [`broadcast_commands`] |
///
/// # Positioning
///
/// All `_at` methods accept an `x_offset` in **font pixels** from screen
/// center (positive = right, negative = left). Alternatively, use
/// `_at_canvas` to work in a virtual coordinate space:
///
/// ```rust,ignore
/// // Place the bar 200 px left of center (font pixels).
/// HEALTH.show_at("@a", 5, "my_pack", -200);
///
/// // Same using a 1000-unit virtual canvas (center = 500).
/// HEALTH.show_at_canvas("@a", 5, "my_pack", 300, 1000);
/// ```
///
/// For simultaneous display of multiple bars and elements, use
/// [`HudLayout`](crate::HudLayout) instead of calling individual methods.
///
/// # Full worked example — health bar
///
/// ```rust,ignore
/// hud_bar!(
///     name = "health",
///     texture = create!(fill = 0xFF4444FF, empty = 0x333333FF),
///     steps = 20,  // one step per HP point (Minecraft health 0–20)
///     height = 14,
///     ascent = 14,
/// );
/// // The macro generates: pub const HEALTH: BarHandle = ...;
///
/// #[component(Load)]
/// pub fn load() {
///     mcfunction! {
///         cmd::scoreboard_objectives_add("hp_frame", "dummy");
///     }
/// }
///
/// #[component(Tick)]
/// pub fn tick() {
///     mcfunction! {
///         // 1. Read health into hp_frame (Health × 0.95 maps 0–20 → 0–19).
///         "execute as @a store result score @s hp_frame run data get entity @s Health 0.95";
///         // 2. Clamp to [0, steps-1].
///         "execute as @a if score @s hp_frame matches 20.. run scoreboard players set @s hp_frame 19";
///         "execute as @a if score @s hp_frame matches ..-1 run scoreboard players set @s hp_frame 0";
///         // 3. Broadcast — one command per frame.
///         HEALTH.broadcast_commands("@a", "hp_frame", "my_pack");
///     }
/// }
/// ```
///
/// [`show`]: BarHandle::show
/// [`display_commands`]: BarHandle::display_commands
/// [`broadcast_commands`]: BarHandle::broadcast_commands
#[derive(Copy, Clone)]
pub struct BarHandle {
    /// Name passed to `hud_bar!` — used for auto-unicode derivation.
    pub name: &'static str,
    /// Number of frames in the sprite strip (and number of unicode characters
    /// assigned to this bar). Frame indices run from `0` (empty) to
    /// `steps - 1` (full).
    pub steps: u32,
    /// Font file name (without extension) this bar is registered under.
    pub font: &'static str,
    /// Pixel width of one frame in the sprite strip texture.
    ///
    /// Minecraft renders each glyph with an advance of `frame_width + 1`
    /// (one pixel of inter-character spacing is added). This value is used by
    /// the `_at` positioning methods and [`HudLayout`](crate::HudLayout) to
    /// correctly place the bar relative to screen center.
    ///
    /// Set automatically by `hud_bar!` when `texture = create!(...)` is used.
    /// For user-supplied PNGs the macro sets this to `0` (unknown); positioning
    /// will be approximate in that case.
    pub frame_width: u32,
}

impl BarHandle {
    /// The unicode character assigned to `frame`.
    ///
    /// Frame `0` is the empty state; frame `self.steps - 1` is fully filled.
    pub fn char(&self, frame: u32) -> char {
        crate::unicode::bar_char(self.name, frame)
    }

    /// Minecraft JSON text component string for `frame`.
    ///
    /// Encodes the Private Use Area character together with the font identifier.
    /// Can be passed directly to `title`, `actionbar`, or `tellraw`.
    pub fn text_json(&self, frame: u32, namespace: &str) -> String {
        crate::unicode::bar_text_json(self.name, frame, namespace, self.font)
    }

    /// Returns the glyph advance Minecraft uses for one frame of this bar.
    ///
    /// `None` when `frame_width == 0` (unknown — user-supplied PNG without an
    /// explicit frame width). In that case `_at` methods fall back to an
    /// approximate positioning mode.
    pub(crate) fn glyph_advance(&self) -> Option<i32> {
        if self.frame_width > 0 {
            Some((self.frame_width + 1) as i32)
        } else {
            None
        }
    }

    /// Build a positioned JSON text component for `frame` at `x_offset` from
    /// screen center.
    ///
    /// **Zero-total-width technique** (when `frame_width` is known): the string
    /// is `advance_to + char + advance_back` where `advance_back` cancels the
    /// glyph's own advance. The total string width is 0, so Minecraft's
    /// centering places the text origin exactly at screen center and the bar
    /// appears at precisely `screen_center + x_offset`.
    ///
    /// **Approximate** (when `frame_width == 0`): emits only `advance + char`.
    /// Centering shifts the result slightly — the bar ends up at roughly
    /// `screen_center + x_offset / 2`.
    pub(crate) fn positioned_json(&self, frame: u32, namespace: &str, x_offset: i32) -> String {
        let bar_ch = self.char(frame);
        let font_id = format!("{namespace}:{}", self.font);
        match self.glyph_advance() {
            Some(ga) => {
                let fwd = crate::unicode::advance_x(x_offset);
                let bwd = crate::unicode::advance_x(-(x_offset + ga));
                let text = crate::unicode::json_escape_chars(&format!("{fwd}{bar_ch}{bwd}"));
                format!(r#"{{"text":"{text}","font":"{font_id}","color":"white"}}"#)
            }
            None => {
                let fwd = crate::unicode::advance_x(x_offset);
                let text = crate::unicode::json_escape_chars(&format!("{fwd}{bar_ch}"));
                format!(r#"{{"text":"{text}","font":"{font_id}","color":"white"}}"#)
            }
        }
    }

    /// Returns a single `title <target> actionbar …` command that shows
    /// frame `frame` of this bar to `target`.
    ///
    /// Use this when you know the frame at Rust compile/build time. For
    /// bars that change at runtime, use [`broadcast_commands`] instead.
    ///
    /// [`broadcast_commands`]: BarHandle::broadcast_commands
    pub fn show(&self, target: &str, frame: u32, namespace: &str) -> String {
        let json = self.text_json(frame, namespace);
        format!("title {target} actionbar {json}")
    }

    /// Like [`show`], but shifts the bar by `x_offset` **font pixels** from
    /// screen center (positive = right, negative = left).
    ///
    /// Uses the zero-total-width technique for accurate positioning when
    /// `frame_width` is known. See [`positioned_json`] for details.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Show 80 px left of center.
    /// HEALTH.show_at("@a", 5, "my_pack", -80);
    /// ```
    ///
    /// [`show`]: BarHandle::show
    /// [`positioned_json`]: BarHandle::positioned_json
    pub fn show_at(&self, target: &str, frame: u32, namespace: &str, x_offset: i32) -> String {
        let json = self.positioned_json(frame, namespace, x_offset);
        format!("title {target} actionbar {json}")
    }

    /// Like [`show_at`], but takes a virtual canvas position instead of a raw
    /// font-pixel offset.
    ///
    /// `canvas_x = 0` is the left edge; `canvas_x = canvas_width / 2` is the
    /// center; `canvas_x = canvas_width` is the right edge.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // 1000-unit virtual canvas. Bar at 25% from left (left quarter).
    /// HEALTH.show_at_canvas("@a", 5, "my_pack", 250, 1000);
    /// ```
    ///
    /// [`show_at`]: BarHandle::show_at
    pub fn show_at_canvas(
        &self,
        target: &str,
        frame: u32,
        namespace: &str,
        canvas_x: i32,
        canvas_width: i32,
    ) -> String {
        self.show_at(target, frame, namespace, canvas_x - canvas_width / 2)
    }

    /// Returns one `execute if score … run title …` command **per frame**.
    ///
    /// Use this when the MCFunction is **already running as a specific player**
    /// (`@s` is defined). For a standard `#[component(Tick)]` function use
    /// [`broadcast_commands`] instead — tick functions run without an entity
    /// context so `@s` is undefined there.
    ///
    /// [`broadcast_commands`]: BarHandle::broadcast_commands
    pub fn display_commands(&self, holder: &str, objective: &str, namespace: &str) -> Vec<String> {
        (0..self.steps)
            .map(|frame| {
                let json = self.text_json(frame, namespace);
                format!(
                    "execute if score {holder} {objective} matches {frame}..{frame} run title {holder} actionbar {json}"
                )
            })
            .collect()
    }

    /// Like [`display_commands`], but shifts each frame by `x_offset` pixels.
    ///
    /// [`display_commands`]: BarHandle::display_commands
    pub fn display_commands_at(
        &self,
        holder: &str,
        objective: &str,
        namespace: &str,
        x_offset: i32,
    ) -> Vec<String> {
        (0..self.steps)
            .map(|frame| {
                let json = self.positioned_json(frame, namespace, x_offset);
                format!(
                    "execute if score {holder} {objective} matches {frame}..{frame} run title {holder} actionbar {json}"
                )
            })
            .collect()
    }

    /// Like [`display_commands`], but positions each frame using a virtual
    /// canvas coordinate. See [`show_at_canvas`] for how canvas coordinates work.
    ///
    /// [`display_commands`]: BarHandle::display_commands
    /// [`show_at_canvas`]: BarHandle::show_at_canvas
    pub fn display_commands_at_canvas(
        &self,
        holder: &str,
        objective: &str,
        namespace: &str,
        canvas_x: i32,
        canvas_width: i32,
    ) -> Vec<String> {
        self.display_commands_at(holder, objective, namespace, canvas_x - canvas_width / 2)
    }

    /// Returns one `execute as … if score @s … run title @s actionbar …`
    /// command **per frame**, for use in **tick functions**.
    ///
    /// Each generated command:
    /// ```text
    /// execute as <executor> if score @s <objective> matches N..N run title @s actionbar {...}
    /// ```
    ///
    /// `executor` is typically `"@a"`. Minecraft runs once per matched player,
    /// so each player sees the frame matching their own score.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[component(Tick)]
    /// pub fn tick() {
    ///     mcfunction! {
    ///         "execute as @a store result score @s hp_frame run data get entity @s Health 0.95";
    ///         "execute as @a if score @s hp_frame matches 20.. run scoreboard players set @s hp_frame 19";
    ///         HEALTH.broadcast_commands("@a", "hp_frame", "my_pack");
    ///     }
    /// }
    /// ```
    pub fn broadcast_commands(
        &self,
        executor: &str,
        objective: &str,
        namespace: &str,
    ) -> Vec<String> {
        (0..self.steps)
            .map(|frame| {
                let json = self.text_json(frame, namespace);
                format!(
                    "execute as {executor} if score @s {objective} matches {frame}..{frame} run title @s actionbar {json}"
                )
            })
            .collect()
    }

    /// Like [`broadcast_commands`], but shifts each frame by `x_offset` font pixels.
    ///
    /// [`broadcast_commands`]: BarHandle::broadcast_commands
    pub fn broadcast_commands_at(
        &self,
        executor: &str,
        objective: &str,
        namespace: &str,
        x_offset: i32,
    ) -> Vec<String> {
        (0..self.steps)
            .map(|frame| {
                let json = self.positioned_json(frame, namespace, x_offset);
                format!(
                    "execute as {executor} if score @s {objective} matches {frame}..{frame} run title @s actionbar {json}"
                )
            })
            .collect()
    }

    /// Like [`broadcast_commands`], but positions using a virtual canvas coordinate.
    ///
    /// [`broadcast_commands`]: BarHandle::broadcast_commands
    pub fn broadcast_commands_at_canvas(
        &self,
        executor: &str,
        objective: &str,
        namespace: &str,
        canvas_x: i32,
        canvas_width: i32,
    ) -> Vec<String> {
        self.broadcast_commands_at(executor, objective, namespace, canvas_x - canvas_width / 2)
    }
}

/// A lightweight handle to a registered [`hud_element!`](sand_macros::hud_element)
/// component, generated automatically by the macro.
///
/// Unlike a [`BarHandle`], an element has only **one state** — it is a static
/// graphic that is either shown or not. There are no frames, no fill levels,
/// and no scoreboard math required.
///
/// Typical uses: background frames, icon decorations, overlay borders, or any
/// HUD graphic that does not change dynamically.
///
/// # Example
///
/// ```rust,ignore
/// hud_element!(
///     name = "hotbar_bg",
///     texture = "src/assets/hotbar.png",
///     height = 22,
///     ascent = -10,  // negative ascent pushes the graphic below the baseline
/// );
/// // The macro generates: pub const HOTBAR_BG: ElementHandle = ...;
///
/// #[component(Tick)]
/// pub fn tick() {
///     HOTBAR_BG.show("@a", "my_pack");
/// }
/// ```
#[derive(Copy, Clone)]
pub struct ElementHandle {
    /// Name passed to `hud_element!` — used for auto-unicode derivation.
    pub name: &'static str,
    /// Font file name (without extension) this element is registered under.
    pub font: &'static str,
    /// Pixel width of the element texture.
    ///
    /// Minecraft renders the glyph with an advance of `char_width + 1`.
    /// Used by `_at` positioning methods and [`HudLayout`](crate::HudLayout).
    ///
    /// Set automatically by `hud_element!` when `texture = create!(...)` is
    /// used. `0` for user-supplied PNGs (positioning will be approximate).
    pub char_width: u32,
}

impl ElementHandle {
    /// The unicode character assigned to this element.
    pub fn char(&self) -> char {
        crate::unicode::element_char(self.name)
    }

    /// Minecraft JSON text component string for this element.
    pub fn text_json(&self, namespace: &str) -> String {
        crate::unicode::element_text_json(self.name, namespace, self.font)
    }

    /// Returns the glyph advance Minecraft uses for this element.
    pub(crate) fn glyph_advance(&self) -> Option<i32> {
        if self.char_width > 0 {
            Some((self.char_width + 1) as i32)
        } else {
            None
        }
    }

    pub(crate) fn positioned_json(&self, namespace: &str, x_offset: i32) -> String {
        let ch = self.char();
        let font_id = format!("{namespace}:{}", self.font);
        match self.glyph_advance() {
            Some(ga) => {
                let fwd = crate::unicode::advance_x(x_offset);
                let bwd = crate::unicode::advance_x(-(x_offset + ga));
                let text = crate::unicode::json_escape_chars(&format!("{fwd}{ch}{bwd}"));
                format!(r#"{{"text":"{text}","font":"{font_id}","color":"white"}}"#)
            }
            None => {
                let fwd = crate::unicode::advance_x(x_offset);
                let text = crate::unicode::json_escape_chars(&format!("{fwd}{ch}"));
                format!(r#"{{"text":"{text}","font":"{font_id}","color":"white"}}"#)
            }
        }
    }

    /// Returns a single `title <target> actionbar …` command.
    pub fn show(&self, target: &str, namespace: &str) -> String {
        let json = self.text_json(namespace);
        format!("title {target} actionbar {json}")
    }

    /// Like [`show`], but shifts the element by `x_offset` font pixels from center.
    ///
    /// [`show`]: ElementHandle::show
    pub fn show_at(&self, target: &str, namespace: &str, x_offset: i32) -> String {
        let json = self.positioned_json(namespace, x_offset);
        format!("title {target} actionbar {json}")
    }

    /// Like [`show`], but positions the element using a virtual canvas coordinate.
    ///
    /// [`show`]: ElementHandle::show
    pub fn show_at_canvas(
        &self,
        target: &str,
        namespace: &str,
        canvas_x: i32,
        canvas_width: i32,
    ) -> String {
        self.show_at(target, namespace, canvas_x - canvas_width / 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEALTH: BarHandle = BarHandle {
        name: "health",
        steps: 10,
        font: "hud",
        frame_width: 28, // 2 × height=14 default
    };
    const HOTBAR_BG: ElementHandle = ElementHandle {
        name: "hotbar_bg",
        font: "hud",
        char_width: 22,
    };

    #[test]
    fn bar_handle_char_in_pua() {
        let ch = HEALTH.char(0);
        assert!(ch as u32 >= 0xE000 && ch as u32 <= 0xF8FF);
    }

    #[test]
    fn bar_handle_display_commands_count() {
        let cmds = HEALTH.display_commands("@s", "hp_frame", "my_pack");
        assert_eq!(cmds.len(), 10);
        assert!(cmds[0].contains("matches 0..0"));
        assert!(cmds[9].contains("matches 9..9"));
    }

    #[test]
    fn bar_handle_show_contains_json() {
        let cmd = HEALTH.show("@a", 3, "my_pack");
        assert!(cmd.starts_with("title @a actionbar"));
        assert!(cmd.contains("my_pack:hud"));
    }

    #[test]
    fn element_handle_show_contains_json() {
        let cmd = HOTBAR_BG.show("@a", "my_pack");
        assert!(cmd.starts_with("title @a actionbar"));
        assert!(cmd.contains("my_pack:hud"));
    }

    #[test]
    fn show_at_zero_offset_has_advance_back() {
        // With frame_width=28: advance(0)="" but advance_back=advance(-(0+29))
        // The command should contain escape chars for the back-advance.
        let cmd = HEALTH.show_at("@a", 3, "my_pack", 0);
        assert!(cmd.starts_with("title @a actionbar"));
        // The JSON text should contain the back-advance unicode escape.
        assert!(cmd.contains("\\u"));
    }

    #[test]
    fn show_at_canvas_center_uses_zero_offset() {
        // canvas_x == canvas_width/2 → x_offset = 0 → same as show_at(0)
        let canvas = HEALTH.show_at_canvas("@a", 3, "my_pack", 480, 960);
        let at_zero = HEALTH.show_at("@a", 3, "my_pack", 0);
        assert_eq!(canvas, at_zero);
    }

    #[test]
    fn canvas_left_is_negative_offset() {
        // canvas_x = 0 on a 960-wide canvas → x_offset = -480
        let canvas = HEALTH.show_at_canvas("@a", 3, "my_pack", 0, 960);
        let shifted = HEALTH.show_at("@a", 3, "my_pack", -480);
        assert_eq!(canvas, shifted);
    }

    #[test]
    fn canvas_display_commands_count() {
        let cmds = HEALTH.display_commands_at_canvas("@s", "hp_frame", "my_pack", 200, 960);
        assert_eq!(cmds.len(), 10);
    }

    #[test]
    fn broadcast_commands_count() {
        let cmds = HEALTH.broadcast_commands("@a", "hp_frame", "my_pack");
        assert_eq!(cmds.len(), 10);
        assert!(cmds[0].contains("execute as @a"));
        assert!(cmds[0].contains("if score @s hp_frame matches 0..0"));
    }

    #[test]
    fn approximate_mode_when_frame_width_zero() {
        let handle = BarHandle {
            name: "unknown",
            steps: 5,
            font: "hud",
            frame_width: 0,
        };
        // Should not panic; produces approximate JSON.
        let cmd = handle.show_at("@a", 0, "my_pack", -100);
        assert!(cmd.starts_with("title @a actionbar"));
    }
}
