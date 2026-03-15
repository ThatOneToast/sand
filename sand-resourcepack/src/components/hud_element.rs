use crate::component::{AssetOutput, ResourcePackComponent};
use crate::components::font::{BitmapFont, BitmapProvider};

/// A static single-character HUD overlay element.
///
/// `HudElement` maps one unicode character to a single PNG texture,
/// useful for fixed HUD graphics such as background frames, icons, or
/// decorative overlays that do not change dynamically.
///
/// Use [`element_text_json`](crate::element_text_json) to get the JSON text
/// component string for displaying the element in commands — no manual unicode
/// handling needed.
///
/// # Output files
///
/// | File | Purpose |
/// |---|---|
/// | `assets/<ns>/font/<font_name>.json` | Registers the bitmap provider |
/// | `assets/<ns>/textures/<texture_dest>.png` | The copied texture |
///
/// # Negative-space positioning technique
///
/// Minecraft renders font characters at the cursor position. By combining a
/// character with a negative advance width (defined in a `space` font
/// provider), you can overlay multiple HUD layers at the same screen
/// position. Use a negative `ascent` to push the character below the
/// default baseline.
///
/// # Macro
///
/// Prefer the [`hud_element!`](sand_macros::hud_element) macro over
/// constructing this struct directly. Unicode codepoints are assigned
/// automatically:
///
/// ```rust,ignore
/// use sand_macros::hud_element;
///
/// hud_element!(
///     name: "hotbar_bg",
///     texture: "src/assets/hotbar.png",
///     height: 22,
///     ascent: -10,
/// );
/// ```
///
/// To display the element in a command, use
/// [`element_text_json`](crate::element_text_json):
///
/// ```rust,ignore
/// let json = sand_resourcepack::element_text_json("hotbar_bg", "my_pack", "default");
/// mcfunction! { format!("title @a actionbar {json}"); }
/// ```
pub struct HudElement {
    /// Identifier used in diagnostics and auto-unicode derivation.
    pub name: &'static str,

    /// Project-root-relative path to the source PNG.
    ///
    /// Example: `"src/assets/hotbar.png"`.
    pub texture_src: &'static str,

    /// Destination sub-path inside `assets/<namespace>/textures/` (without
    /// extension), e.g. `"font/hotbar"`.
    pub texture_dest: &'static str,

    /// Override the unicode codepoint for this element.
    ///
    /// When `None` (the default when using the macro), the codepoint is
    /// derived automatically from the component name. Use `Some(c)` only
    /// when you need exact control over the codepoint assignment.
    pub unicode: Option<char>,

    /// Rendered height of the character in pixels.
    pub height: i32,

    /// Vertical offset from the screen baseline to the top of the glyph.
    ///
    /// Positive → above baseline. Negative → below baseline (useful for
    /// overlays beneath the default GUI layer).
    pub ascent: i32,

    /// Name of the font file (without extension) this element belongs to.
    ///
    /// Defaults to `"default"` when built via the macro.
    pub font: &'static str,
}

impl HudElement {
    fn effective_unicode(&self) -> char {
        self.unicode
            .unwrap_or_else(|| crate::unicode::element_char(self.name))
    }
}

impl ResourcePackComponent for HudElement {
    fn assets(&self, namespace: &str) -> Vec<AssetOutput> {
        let unicode = self.effective_unicode();
        let file_ref = format!("{}:{}.png", namespace, self.texture_dest);

        let font = BitmapFont {
            font_name: self.font,
            provider: BitmapProvider {
                file: file_ref,
                height: self.height,
                ascent: self.ascent,
                chars: vec![unicode.to_string()],
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

    fn test_element() -> HudElement {
        HudElement {
            name: "hotbar",
            texture_src: "src/assets/hotbar.png",
            texture_dest: "font/hotbar",
            unicode: Some('\u{E100}'),
            height: 22,
            ascent: -10,
            font: "hud",
        }
    }

    #[test]
    fn explicit_unicode_used() {
        let elem = test_element();
        let outputs = elem.assets("my_pack");
        match &outputs[0].content {
            AssetContent::Json(v) => {
                let chars = v["providers"][0]["chars"][0].as_str().unwrap();
                assert_eq!(chars.chars().next().unwrap(), '\u{E100}');
            }
            _ => panic!("expected Json"),
        }
    }

    #[test]
    fn auto_unicode_is_in_element_region() {
        let elem = HudElement {
            unicode: None,
            ..test_element()
        };
        let outputs = elem.assets("my_pack");
        match &outputs[0].content {
            AssetContent::Json(v) => {
                let chars = v["providers"][0]["chars"][0].as_str().unwrap();
                let ch = chars.chars().next().unwrap() as u32;
                assert!(ch >= 0xF000 && ch <= 0xF8FF);
            }
            _ => panic!("expected Json"),
        }
    }

    #[test]
    fn assets_produces_font_and_texture() {
        let elem = test_element();
        let outputs = elem.assets("my_pack");
        assert_eq!(outputs.len(), 2);

        assert_eq!(outputs[0].path, "assets/my_pack/font/hud.json");
        match &outputs[0].content {
            AssetContent::Json(v) => {
                let p = &v["providers"][0];
                assert_eq!(p["type"], "bitmap");
                assert_eq!(p["height"], 22);
                assert_eq!(p["ascent"], -10);
            }
            _ => panic!("expected Json"),
        }

        assert_eq!(outputs[1].path, "assets/my_pack/textures/font/hotbar.png");
        match &outputs[1].content {
            AssetContent::CopyFrom(src) => assert_eq!(src, "src/assets/hotbar.png"),
            _ => panic!("expected CopyFrom"),
        }
    }
}
