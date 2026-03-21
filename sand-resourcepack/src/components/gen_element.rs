use crate::component::{AssetContent, AssetOutput, ResourcePackComponent};
use crate::components::font::{BitmapFont, BitmapProvider};
use crate::sprite::{Color, gen_element_png};

/// A static HUD overlay element whose texture is **generated at build time**
/// from a solid color rather than copied from a user-supplied PNG.
///
/// Constructed via the [`hud_element!`] macro with a `gen!(...)` expression
/// in the `texture:` field:
///
/// ```rust,ignore
/// use sand_macros::hud_element;
///
/// hud_element!(
///     name: "hotbar_bg",
///     texture: gen!(color: 0x00000080),
///     height: 22,
///     ascent: -10,
/// );
/// ```
///
/// [`hud_element!`]: sand_macros::hud_element
pub struct GenHudElement {
    /// Unique identifier used in diagnostics and codepoint derivation.
    pub name: &'static str,

    /// Destination sub-path inside `assets/<namespace>/textures/` (without
    /// extension), e.g. `"font/hotbar_bg"`.
    pub texture_dest: &'static str,

    /// Override the unicode codepoint for this element.
    ///
    /// When `None` (the default), the codepoint is derived automatically from
    /// the component name via [`element_char`](crate::element_char).
    pub unicode: Option<char>,

    /// Rendered glyph height in pixels. Also used as `width` when `width`
    /// is `0`.
    pub height: i32,

    /// Vertical offset from the baseline to the top of the glyph.
    pub ascent: i32,

    /// Name of the font file (without extension).
    pub font: &'static str,

    /// Packed `0xRRGGBBAA` fill color for the generated texture.
    pub color: u32,

    /// Pixel width of the generated texture.
    ///
    /// A value of `0` means "use `height`" (produces a square texture).
    pub width: u32,
}

impl GenHudElement {
    fn effective_unicode(&self) -> char {
        self.unicode
            .unwrap_or_else(|| crate::unicode::element_char(self.name))
    }

    fn effective_width(&self) -> u32 {
        if self.width == 0 {
            self.height as u32
        } else {
            self.width
        }
    }
}

impl ResourcePackComponent for GenHudElement {
    fn assets(&self, namespace: &str) -> Vec<AssetOutput> {
        let unicode = self.effective_unicode();
        let file_ref = format!("{}:{}.png", namespace, self.texture_dest);
        let width = self.effective_width();

        let font = BitmapFont {
            font_name: self.font,
            provider: BitmapProvider {
                file: file_ref,
                height: self.height,
                ascent: self.ascent,
                chars: vec![unicode.to_string()],
            },
            texture_src: None,
            texture_dest: self.texture_dest,
        };

        let mut outputs = font.assets(namespace);

        // Generated PNG bytes.
        let color = Color::from_u32(self.color);
        let png_bytes = gen_element_png(color, width, self.height as u32);
        let tex_path = format!("assets/{}/textures/{}.png", namespace, self.texture_dest);
        outputs.push(AssetOutput {
            path: tex_path,
            content: AssetContent::Bytes(png_bytes),
        });

        outputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::AssetContent;

    fn test_gen_elem() -> GenHudElement {
        GenHudElement {
            name: "hotbar_bg",
            texture_dest: "font/hotbar_bg",
            unicode: None,
            height: 22,
            ascent: -10,
            font: "hud",
            color: 0x00000080,
            width: 0,
        }
    }

    #[test]
    fn assets_produces_font_and_bytes() {
        let elem = test_gen_elem();
        let outputs = elem.assets("my_pack");
        assert_eq!(outputs.len(), 2);

        assert_eq!(outputs[0].path, "assets/my_pack/font/hud.json");
        match &outputs[0].content {
            AssetContent::Json(v) => {
                assert_eq!(v["providers"][0]["type"], "bitmap");
                assert_eq!(v["providers"][0]["height"], 22);
            }
            _ => panic!("expected Json"),
        }

        assert_eq!(
            outputs[1].path,
            "assets/my_pack/textures/font/hotbar_bg.png"
        );
        match &outputs[1].content {
            AssetContent::Bytes(b) => {
                assert_eq!(&b[..8], b"\x89PNG\r\n\x1a\n");
            }
            _ => panic!("expected Bytes"),
        }
    }

    #[test]
    fn explicit_unicode_is_respected() {
        let elem = GenHudElement {
            unicode: Some('\u{F100}'),
            ..test_gen_elem()
        };
        let outputs = elem.assets("ns");
        match &outputs[0].content {
            AssetContent::Json(v) => {
                let chars = v["providers"][0]["chars"][0].as_str().unwrap();
                assert_eq!(chars.chars().next().unwrap(), '\u{F100}');
            }
            _ => panic!(),
        }
    }
}
