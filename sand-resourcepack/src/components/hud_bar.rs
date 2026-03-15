use crate::component::{AssetOutput, ResourcePackComponent};
use crate::components::font::{BitmapFont, BitmapProvider};

/// A bitmap-font progress bar for HUD overlays.
///
/// `HudBar` maps a horizontal sprite strip (a PNG containing `steps` frames
/// side by side) to a sequence of unicode characters. Each character
/// represents one fill level of the bar.
///
/// Use [`bar_text_json`](crate::bar_text_json) to get the JSON text component
/// string for displaying a specific frame in `title`, `actionbar`, or `tellraw`
/// commands — no manual unicode handling needed.
///
/// # Output files
///
/// | File | Purpose |
/// |---|---|
/// | `assets/<ns>/font/<font_name>.json` | Registers the bitmap provider |
/// | `assets/<ns>/textures/<texture_dest>.png` | The copied sprite strip |
///
/// # Macro
///
/// Prefer the [`hud_bar!`](sand_macros::hud_bar) macro over constructing this
/// struct directly. Unicode codepoints are assigned automatically — you do not
/// need to specify them:
///
/// ```rust,ignore
/// use sand_macros::hud_bar;
///
/// hud_bar!(
///     name: "health",
///     texture: "src/assets/health_bar.png",
///     steps: 10,
///     height: 9,
///     ascent: 9,
/// );
/// ```
///
/// To display the bar in a command, use [`bar_text_json`](crate::bar_text_json):
///
/// ```rust,ignore
/// let json = sand_resourcepack::bar_text_json("health", frame, "my_pack", "default");
/// mcfunction! { format!("title @a actionbar {json}"); }
/// ```
pub struct HudBar {
    /// Identifier used in diagnostics and auto-unicode derivation.
    pub name: &'static str,

    /// Project-root-relative path to the source sprite strip PNG.
    ///
    /// The PNG should contain exactly `steps` frames arranged horizontally,
    /// all the same width, with height matching the `height` field.
    ///
    /// Example: `"src/assets/health_bar.png"` (10 frames × 9 px tall).
    pub texture_src: &'static str,

    /// Destination sub-path inside `assets/<namespace>/textures/` (without
    /// extension), e.g. `"font/health_bar"`.
    ///
    /// Defaults to `"font/<name>"` when built via the macro.
    pub texture_dest: &'static str,

    /// Override the first unicode codepoint for the bar frames.
    ///
    /// When `None` (the default when using the macro), the codepoint is
    /// derived automatically from the component name so you never need to
    /// manage unicode values by hand. Use `Some(c)` only when you need to
    /// control the exact codepoint assignment.
    pub unicode_start: Option<char>,

    /// Number of frames in the sprite strip (including the empty frame).
    ///
    /// A 10-step bar typically has 11 frames (0 % through 100 % in 10 %
    /// increments).
    pub steps: u32,

    /// Rendered height of each character in pixels.
    pub height: i32,

    /// Vertical offset from the screen baseline to the top of the glyph.
    pub ascent: i32,

    /// Name of the font file (without extension) this bar belongs to.
    ///
    /// Multiple bars and elements that share the same `font` are merged into
    /// one `assets/<namespace>/font/<font>.json` file.
    ///
    /// Defaults to `"default"` when built via the macro.
    pub font: &'static str,
}

impl HudBar {
    fn chars_row(&self) -> String {
        let start = self
            .unicode_start
            .map(|c| c as u32)
            .unwrap_or_else(|| crate::unicode::bar_base_codepoint(self.name));
        (start..start + self.steps)
            .filter_map(char::from_u32)
            .collect()
    }
}

impl ResourcePackComponent for HudBar {
    fn assets(&self, namespace: &str) -> Vec<AssetOutput> {
        let chars_row = self.chars_row();
        let file_ref = format!("{}:{}.png", namespace, self.texture_dest);

        let font = BitmapFont {
            font_name: self.font,
            provider: BitmapProvider {
                file: file_ref,
                height: self.height,
                ascent: self.ascent,
                chars: vec![chars_row],
            },
            texture_src: Some(self.texture_src),
            texture_dest: self.texture_dest,
        };

        font.assets(namespace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::AssetContent;

    fn test_bar() -> HudBar {
        HudBar {
            name: "health",
            texture_src: "src/assets/health_bar.png",
            texture_dest: "font/health_bar",
            unicode_start: Some('\u{E000}'),
            steps: 3,
            height: 9,
            ascent: 9,
            font: "default",
        }
    }

    #[test]
    fn chars_row_explicit() {
        let bar = test_bar();
        let row: Vec<char> = bar.chars_row().chars().collect();
        assert_eq!(row.len(), 3);
        assert_eq!(row[0], '\u{E000}');
        assert_eq!(row[1], '\u{E001}');
        assert_eq!(row[2], '\u{E002}');
    }

    #[test]
    fn chars_row_auto_assigned() {
        let bar = HudBar {
            unicode_start: None,
            ..test_bar()
        };
        let row: Vec<char> = bar.chars_row().chars().collect();
        assert_eq!(row.len(), 3);
        // Auto-assigned chars should be in the bar region U+E000..U+EFFF.
        for ch in &row {
            assert!((*ch as u32) >= 0xE000 && (*ch as u32) < 0xF000);
        }
    }

    #[test]
    fn assets_produces_font_and_texture() {
        let bar = test_bar();
        let outputs = bar.assets("my_pack");
        assert_eq!(outputs.len(), 2);

        assert_eq!(outputs[0].path, "assets/my_pack/font/default.json");
        match &outputs[0].content {
            AssetContent::Json(v) => {
                let providers = v["providers"].as_array().unwrap();
                assert_eq!(providers.len(), 1);
                assert_eq!(providers[0]["type"], "bitmap");
                assert_eq!(providers[0]["height"], 9);
                assert_eq!(providers[0]["ascent"], 9);
            }
            _ => panic!("expected Json"),
        }

        assert_eq!(
            outputs[1].path,
            "assets/my_pack/textures/font/health_bar.png"
        );
        match &outputs[1].content {
            AssetContent::CopyFrom(src) => assert_eq!(src, "src/assets/health_bar.png"),
            _ => panic!("expected CopyFrom"),
        }
    }
}
