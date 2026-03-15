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
/// | The frame changes at runtime based on a scoreboard value | [`display_commands`] |
///
/// [`display_commands`] returns one command **per frame** — Minecraft selects
/// the right one based on an `execute if score` check. You are responsible
/// for computing the score first (e.g., scaling health ÷ max health × steps
/// using scoreboard operations), then calling this to emit the conditional
/// display commands.
///
/// # Full worked example — health bar
///
/// ```rust,ignore
/// hud_bar!(
///     name: "health",
///     texture: create!(fill: 0xFF4444FF, empty: 0x333333FF),
///     steps: 10,   // 10 frames: empty, 10%, 20%, …, 100%
///     height: 14,  // pixel height — increase to make the bar larger
///     ascent: 14,  // keep == height to align bar top at baseline
/// );
/// // The macro generates: pub const HEALTH: BarHandle = ...;
///
/// #[component(Load)]
/// pub fn load() {
///     // Create the objectives used by the scaling math.
///     "scoreboard objectives add hp_raw dummy";
///     "scoreboard objectives add hp_max dummy";
///     "scoreboard objectives add hp_frame dummy";
///     "scoreboard players set #steps hp_frame 10";
/// }
///
/// #[component(Tick)]
/// pub fn tick() {
///     // 1. Read current health (×100 for integer precision).
///     "execute as @a store result score @s hp_raw run data get entity @s Health 100";
///     // 2. Read max health (×100 for same scale).
///     "execute as @a store result score @s hp_max run attribute @s minecraft:generic.max_health get 100";
///     // 3. Copy raw into frame slot, then scale: frame = hp_raw * steps / hp_max.
///     "execute as @a run scoreboard players operation @s hp_frame = @s hp_raw";
///     "execute as @a run scoreboard players operation @s hp_frame *= #steps hp_frame";
///     "execute as @a run scoreboard players operation @s hp_frame /= @s hp_max";
///     // 4. Clamp to [0, steps-1] to guard against absorption / edge cases.
///     "execute as @a if score @s hp_frame matches 10.. run scoreboard players set @s hp_frame 9";
///     "execute as @a if score @s hp_frame matches ..-1 run scoreboard players set @s hp_frame 0";
///     // 5. Emit one title command per frame — Minecraft picks the right one.
///     HEALTH.display_commands("@s", "hp_frame", "my_pack");
/// }
/// ```
///
/// # Showing a fixed frame (non-dynamic)
///
/// If you just want to always show a specific fill level — for example,
/// a decorative bar that is always displayed full — use [`show`] directly:
///
/// ```rust,ignore
/// // Always show the fully-filled frame (frame = steps - 1).
/// HEALTH.show("@a", HEALTH.steps - 1, "my_pack");
/// ```
pub struct BarHandle {
    /// Name passed to `hud_bar!` — used for auto-unicode derivation.
    pub name: &'static str,
    /// Number of frames in the sprite strip (and number of unicode characters
    /// assigned to this bar). Frame indices run from `0` (empty) to
    /// `steps - 1` (full).
    pub steps: u32,
    /// Font file name (without extension) this bar is registered under.
    pub font: &'static str,
}

impl BarHandle {
    /// The unicode character assigned to `frame`.
    ///
    /// Frame `0` is the empty state; frame `self.steps - 1` is fully filled.
    /// These characters live in the Unicode Private Use Area and are mapped
    /// to their sprite-strip slice by the font JSON Sand generates.
    pub fn char(&self, frame: u32) -> char {
        crate::unicode::bar_char(self.name, frame)
    }

    /// Minecraft JSON text component string for `frame`.
    ///
    /// The returned string can be passed directly to `title`, `actionbar`, or
    /// `tellraw` commands. It encodes the Private Use Area character together
    /// with the font identifier so Minecraft knows which font file to look up.
    pub fn text_json(&self, frame: u32, namespace: &str) -> String {
        crate::unicode::bar_text_json(self.name, frame, namespace, self.font)
    }

    /// Returns a single `title <target> actionbar …` command that shows
    /// frame `frame` of this bar to `target`.
    ///
    /// Use this when you know the frame at Rust compile/build time (e.g., to
    /// hard-code a "full bar" decoration or a default empty state). For
    /// bars that change at runtime, use [`display_commands`] instead.
    ///
    /// [`display_commands`]: BarHandle::display_commands
    pub fn show(&self, target: &str, frame: u32, namespace: &str) -> String {
        let json = self.text_json(frame, namespace);
        format!("title {target} actionbar {json}")
    }

    /// Like [`show`], but shifts the bar horizontally by `x_offset` **font pixels**
    /// relative to its natural centered position.
    ///
    /// Positive values move right; negative values move left.
    ///
    /// # Understanding font pixels
    ///
    /// `x_offset` is measured in the same units as the font texture — the same
    /// scale as `height` and `ascent`. One font pixel equals one rendered pixel
    /// at **GUI scale 1**. At higher GUI scales the visual shift is proportionally
    /// larger:
    ///
    /// | GUI scale | `x_offset: 100` shifts visually |
    /// |---|---|
    /// | 1 | 100 screen pixels |
    /// | 2 | 200 screen pixels |
    /// | 3 | 300 screen pixels |
    ///
    /// For most setups (1920 × 1080 @ GUI scale 2) the visible canvas is
    /// roughly **960 font pixels** wide, so `x_offset = ±480` reaches the
    /// screen edges from center. Use [`show_at_canvas`] to express positions
    /// in a virtual coordinate space instead.
    ///
    /// [`show_at_canvas`]: BarHandle::show_at_canvas
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Shift 80 font pixels right of center.
    /// HEALTH.show_at("@a", 5, "my_pack", 80);
    /// ```
    ///
    /// [`show`]: BarHandle::show
    pub fn show_at(&self, target: &str, frame: u32, namespace: &str, x_offset: i32) -> String {
        let advance = crate::unicode::advance_x(x_offset);
        let bar_ch = self.char(frame);
        let text = crate::unicode::json_escape_chars(&format!("{advance}{bar_ch}"));
        let font_id = format!("{namespace}:{}", self.font);
        let json = format!(r#"{{"text":"{text}","font":"{font_id}","color":"white"}}"#);
        format!("title {target} actionbar {json}")
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
                let advance = crate::unicode::advance_x(x_offset);
                let bar_ch = self.char(frame);
                let text = crate::unicode::json_escape_chars(&format!("{advance}{bar_ch}"));
                let font_id = format!("{namespace}:{}", self.font);
                let json =
                    format!(r#"{{"text":"{text}","font":"{font_id}","color":"white"}}"#);
                format!(
                    "execute if score {holder} {objective} matches {frame}..{frame} run title {holder} actionbar {json}"
                )
            })
            .collect()
    }

    /// Like [`show`], but positions the bar using a virtual canvas coordinate.
    ///
    /// This is the easiest way to position bars — define your own coordinate
    /// space and let Sand handle the conversion.
    ///
    /// - `canvas_x` — horizontal position in your virtual canvas (0 = left edge).
    /// - `canvas_width` — total width of the virtual canvas.
    ///
    /// `canvas_width / 2` maps to the screen center. Example: with
    /// `canvas_width = 200`, `canvas_x = 0` is far left, `canvas_x = 100`
    /// is center, `canvas_x = 200` is far right.
    ///
    /// # Choosing `canvas_width`
    ///
    /// Set `canvas_width` to your screen width divided by GUI scale — this
    /// makes your canvas "1 unit = 1 visible pixel":
    ///
    /// | Resolution | GUI scale | `canvas_width` |
    /// |---|---|---|
    /// | 1920 × 1080 | 1 | 1920 |
    /// | 1920 × 1080 | 2 | 960 |
    /// | 1920 × 1080 | 3 | 640 |
    /// | 1920 × 1080 | 4 | 480 |
    ///
    /// You can also use any arbitrary scale — e.g., `canvas_width = 1000` for
    /// a 0–1000 coordinate system where 500 = center.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // 0..960 canvas (1920px @ GUI scale 2). Bar at 50px from left.
    /// HEALTH.show_at_canvas("@a", 5, "my_pack", 50, 960);
    ///
    /// // 0..1000 virtual canvas. Bar at 25% from left.
    /// HEALTH.show_at_canvas("@a", 5, "my_pack", 250, 1000);
    /// ```
    ///
    /// [`show`]: BarHandle::show
    pub fn show_at_canvas(
        &self,
        target: &str,
        frame: u32,
        namespace: &str,
        canvas_x: i32,
        canvas_width: i32,
    ) -> String {
        let offset = canvas_x - canvas_width / 2;
        self.show_at(target, frame, namespace, offset)
    }

    /// Like [`display_commands`], but positions each frame using a virtual
    /// canvas coordinate.
    ///
    /// See [`show_at_canvas`] for a full explanation of `canvas_x` and
    /// `canvas_width`.
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
        let offset = canvas_x - canvas_width / 2;
        self.display_commands_at(holder, objective, namespace, offset)
    }

    /// Returns one `execute if score … run title …` command **per frame**.
    ///
    /// Use this when the MCFunction is **already running as a specific player**
    /// (i.e. `@s` is defined). For a standard `#[component(Tick)]` function
    /// use [`broadcast_commands`] instead — tick functions run without an
    /// entity context so `@s` is undefined there.
    ///
    /// [`broadcast_commands`]: BarHandle::broadcast_commands
    ///
    /// # Score contract
    ///
    /// The score at `holder`/`objective` **must already be in `0..self.steps`**
    /// before these commands run. Scale and clamp your raw value first.
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

    /// Like [`display_commands`], but designed for **tick functions** where
    /// there is no player context.
    ///
    /// Each generated command is:
    /// ```text
    /// execute as <executor> if score @s <objective> matches N..N run title @s actionbar {...}
    /// ```
    ///
    /// `executor` is a selector that expands to all target players, typically
    /// `"@a"`. Minecraft runs the command once *per matched player*, so each
    /// player sees the frame that corresponds to their own score.
    ///
    /// # Score contract
    ///
    /// Each player's score at `objective` must be in `0..self.steps` before
    /// these commands run. Scale from your raw stat (health, mana, etc.) into
    /// this range first — see the full example below.
    ///
    /// # Example — health bar in a tick function
    ///
    /// ```rust,ignore
    /// hud_bar!(
    ///     name: "health",
    ///     texture: create!(fill: 0xFF4444FF, empty: 0x333333FF),
    ///     steps: 20,   // one step per HP point (Minecraft health 0–20)
    ///     height: 14,
    ///     ascent: 14,
    /// );
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
    ///         // 1. Read health into hp_frame (Health is a float; × 0.95 keeps
    ///         //    full health ≤ 19 so the result stays within 0..19).
    ///         "execute as @a store result score @s hp_frame run data get entity @s Health 0.95";
    ///         // 2. Clamp to [0, steps-1].
    ///         "execute as @a if score @s hp_frame matches 20.. run scoreboard players set @s hp_frame 19";
    ///         "execute as @a if score @s hp_frame matches ..-1 run scoreboard players set @s hp_frame 0";
    ///         // 3. Display — one command per frame, each prefixed with execute as @a.
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
                let advance = crate::unicode::advance_x(x_offset);
                let bar_ch = self.char(frame);
                let text = crate::unicode::json_escape_chars(&format!("{advance}{bar_ch}"));
                let font_id = format!("{namespace}:{}", self.font);
                let json = format!(r#"{{"text":"{text}","font":"{font_id}","color":"white"}}"#);
                format!(
                    "execute as {executor} if score @s {objective} matches {frame}..{frame} run title @s actionbar {json}"
                )
            })
            .collect()
    }

    /// Like [`broadcast_commands`], but positions each frame using a virtual
    /// canvas coordinate. See [`show_at_canvas`] for how canvas coordinates work.
    ///
    /// [`broadcast_commands`]: BarHandle::broadcast_commands
    /// [`show_at_canvas`]: BarHandle::show_at_canvas
    pub fn broadcast_commands_at_canvas(
        &self,
        executor: &str,
        objective: &str,
        namespace: &str,
        canvas_x: i32,
        canvas_width: i32,
    ) -> Vec<String> {
        let offset = canvas_x - canvas_width / 2;
        self.broadcast_commands_at(executor, objective, namespace, offset)
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
///     name: "hotbar_bg",
///     texture: "src/assets/hotbar.png",
///     height: 22,
///     ascent: -10,  // negative ascent pushes the graphic below the baseline
/// );
/// // The macro generates: pub const HOTBAR_BG: ElementHandle = ...;
///
/// #[component(Tick)]
/// pub fn tick() {
///     // Show the element to every player every tick.
///     HOTBAR_BG.show("@a", "my_pack");
/// }
/// ```
pub struct ElementHandle {
    /// Name passed to `hud_element!` — used for auto-unicode derivation.
    pub name: &'static str,
    /// Font file name (without extension) this element is registered under.
    pub font: &'static str,
}

impl ElementHandle {
    /// The unicode character assigned to this element.
    ///
    /// Lives in the Unicode Private Use Area (U+F000–U+F8FF). Sand assigns
    /// it deterministically from the element name — you never need to manage
    /// codepoints by hand.
    pub fn char(&self) -> char {
        crate::unicode::element_char(self.name)
    }

    /// Minecraft JSON text component string for this element.
    ///
    /// Can be passed directly to `title`, `actionbar`, or `tellraw`.
    pub fn text_json(&self, namespace: &str) -> String {
        crate::unicode::element_text_json(self.name, namespace, self.font)
    }

    /// Returns a single `title <target> actionbar …` command that shows
    /// this element to `target`.
    pub fn show(&self, target: &str, namespace: &str) -> String {
        let json = self.text_json(namespace);
        format!("title {target} actionbar {json}")
    }

    /// Like [`show`], but shifts the element horizontally by `x_offset` font pixels
    /// relative to center. See [`BarHandle::show_at`] for unit details.
    ///
    /// Use [`show_at_canvas`] for a simpler coordinate model.
    ///
    /// [`show_at_canvas`]: ElementHandle::show_at_canvas
    pub fn show_at(&self, target: &str, namespace: &str, x_offset: i32) -> String {
        let advance = crate::unicode::advance_x(x_offset);
        let elem_ch = self.char();
        let text = crate::unicode::json_escape_chars(&format!("{advance}{elem_ch}"));
        let font_id = format!("{namespace}:{}", self.font);
        let json = format!(r#"{{"text":"{text}","font":"{font_id}","color":"white"}}"#);
        format!("title {target} actionbar {json}")
    }

    /// Like [`show`], but positions the element using a virtual canvas coordinate.
    ///
    /// See [`BarHandle::show_at_canvas`] for a full explanation.
    pub fn show_at_canvas(
        &self,
        target: &str,
        namespace: &str,
        canvas_x: i32,
        canvas_width: i32,
    ) -> String {
        let offset = canvas_x - canvas_width / 2;
        self.show_at(target, namespace, offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEALTH: BarHandle = BarHandle {
        name: "health",
        steps: 10,
        font: "hud",
    };
    const HOTBAR_BG: ElementHandle = ElementHandle {
        name: "hotbar_bg",
        font: "hud",
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
    fn canvas_center_equals_show() {
        // canvas_x == canvas_width/2 → offset 0 → same as show()
        let canvas = HEALTH.show_at_canvas("@a", 3, "my_pack", 480, 960);
        let normal = HEALTH.show("@a", 3, "my_pack");
        assert_eq!(canvas, normal);
    }

    #[test]
    fn canvas_left_is_negative_offset() {
        // canvas_x = 0 on a 960-wide canvas → offset = -480
        let canvas = HEALTH.show_at_canvas("@a", 3, "my_pack", 0, 960);
        let shifted = HEALTH.show_at("@a", 3, "my_pack", -480);
        assert_eq!(canvas, shifted);
    }

    #[test]
    fn canvas_display_commands_count() {
        let cmds = HEALTH.display_commands_at_canvas("@s", "hp_frame", "my_pack", 200, 960);
        assert_eq!(cmds.len(), 10);
    }
}
