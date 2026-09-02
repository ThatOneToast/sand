use serde_json::{Value, json};

use crate::component::{AssetContent, AssetOutput, ResourcePackComponent};

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::resourcepack::FontProvider",
    module = "sand::resourcepack",
    summary = "A single provider entry inside a Minecraft font definition file.",
    context = "A single provider entry inside a Minecraft font definition file. Currently only the `bitmap` provider type is supported. Additional provider types (`ttf`, `space`, `unihex`, `reference`) are planned for future versions.",
    minecraft = "The resourcepack exporter writes version-appropriate assets, bitmap-font providers, and pack metadata for the selected Minecraft profile.",
    use_when = ["Building HUD bars, HUD elements, textures, or resource-pack output alongside a Sand datapack"],
    avoid_when = ["The project is datapack-only or needs unrelated resource-pack functionality not modeled by Sand"],
    example = "use sand::resourcepack::FontProvider;",
    availability = ["Cargo feature: resourcepack"],
    variants(Bitmap = "A bitmap font provider that maps unicode characters to regions of a PNG texture."),
    variant_fields(Bitmap = ["A bitmap font provider that maps unicode characters to regions of a PNG texture."]),
)]
/// A single provider entry inside a Minecraft font definition file.
///
/// Currently only the `bitmap` provider type is supported. Additional
/// provider types (`ttf`, `space`, `unihex`, `reference`) are planned
/// for future versions.
pub enum FontProvider {
    /// A bitmap font provider that maps unicode characters to regions of a
    /// PNG texture.
    Bitmap(
        #[doc = "A bitmap font provider that maps unicode characters to regions of a PNG texture."]
        BitmapProvider,
    ),
}

impl FontProvider {
    /// Serialize this provider to the JSON object Minecraft expects inside
    /// the `"providers"` array of a font file.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::resourcepack::FontProvider::to_json",
        module = "sand::resourcepack",
        kind = "method",
        summary = "Serialize this provider to the JSON object Minecraft expects inside the `\"providers\"` array of a font file.",
        context = "Serialize this provider to the JSON object Minecraft expects inside the `\"providers\"` array of a font file. This API defines client-side HUD, font, texture, or resource-pack output while keeping asset registration and exporter inventory wiring private.",
        minecraft = "The resourcepack exporter writes version-appropriate assets, bitmap-font providers, and pack metadata for the selected Minecraft profile.",
        use_when = ["Building HUD bars, HUD elements, textures, or resource-pack output alongside a Sand datapack"],
        avoid_when = ["The project is datapack-only or needs unrelated resource-pack functionality not modeled by Sand"],
        returns = "The `Value` value produced to serialize this provider to the JSON object Minecraft expects inside the `\"providers\"` array of a font file.",
        example = "use sand::prelude::*;\n\nfn demonstrate(font_provider_value: &sand::resourcepack::FontProvider)  {\n    let to_json = font_provider_value.to_json();\n}",
        availability = ["Cargo feature: resourcepack"],
    )]
    pub fn to_json(&self) -> Value {
        match self {
            FontProvider::Bitmap(b) => b.to_json(),
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::resourcepack::BitmapProvider",
    module = "sand::resourcepack",
    summary = "Parameters for a `\"bitmap\"` font provider. A bitmap provider maps a rectangular grid of characters from a PNG image. The image is divided into a grid where each cell corresponds to one unicode character in the `chars` array.",
    context = "Parameters for a `\"bitmap\"` font provider. A bitmap provider maps a rectangular grid of characters from a PNG image. The image is divided into a grid where each cell corresponds to one unicode character in the `chars` array. `assets/<namespace>/font/<name>.json` → `\"providers\"` array entry with `\"type\": \"bitmap\"`.",
    minecraft = "`assets/<namespace>/font/<name>.json` → `\"providers\"` array entry with `\"type\": \"bitmap\"`.",
    use_when = ["Building HUD bars, HUD elements, textures, or resource-pack output alongside a Sand datapack"],
    avoid_when = ["The project is datapack-only or needs unrelated resource-pack functionality not modeled by Sand"],
    example = "use sand::resourcepack::BitmapProvider;",
    availability = ["Cargo feature: resourcepack"],
    fields(ascent = "Vertical offset (in pixels) from the baseline to the top of the glyph.", chars = "Character grid. Each `String` in the outer `Vec` represents one row of the source image; each `char` within that string maps to one column (cell) in that row.", file = "Resource location of the source PNG, e.g. `\"my_pack:font/health_bar.png\"`.", height = "Vertical height (in pixels) to render each character at."),
)]
/// Parameters for a `"bitmap"` font provider.
///
/// A bitmap provider maps a rectangular grid of characters from a PNG image.
/// The image is divided into a grid where each cell corresponds to one
/// unicode character in the `chars` array.
///
/// # Minecraft reference
/// `assets/<namespace>/font/<name>.json` → `"providers"` array entry with
/// `"type": "bitmap"`.
pub struct BitmapProvider {
    /// Resource location of the source PNG, e.g. `"my_pack:font/health_bar.png"`.
    ///
    /// Minecraft resolves font texture resource locations as
    /// `assets/<namespace>/textures/<path>`, so **do not** include `textures/`
    /// in this field — it will result in a doubled path at runtime.
    pub file: String,

    /// Vertical height (in pixels) to render each character at.
    ///
    /// Can differ from the actual image height to scale the glyph. Use the
    /// same value as the image height for 1:1 rendering.
    pub height: i32,

    /// Vertical offset (in pixels) from the baseline to the top of the glyph.
    ///
    /// Positive values move the character up; negative values move it down.
    /// For HUD elements rendered above the hotbar, use a large positive value
    /// (e.g. the screen height). For elements rendered below (negative-space
    /// trick), use a negative value.
    pub ascent: i32,

    /// Character grid. Each `String` in the outer `Vec` represents one
    /// **row** of the source image; each `char` within that string maps to
    /// one **column** (cell) in that row.
    ///
    /// # Example — single-row 10-frame strip
    /// ```text
    /// chars: vec!["\u{E000}\u{E001}...\u{E009}".to_string()]
    /// ```
    pub chars: Vec<String>,
}

impl BitmapProvider {
    /// Serializes this bitmap font provider to its Minecraft JSON object.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::resourcepack::BitmapProvider::to_json",
        module = "sand::resourcepack",
        kind = "method",
        summary = "Serializes this bitmap font provider to its Minecraft JSON object.",
        context = "Serializes this bitmap font provider to its Minecraft JSON object. This API defines client-side HUD, font, texture, or resource-pack output while keeping asset registration and exporter inventory wiring private.",
        minecraft = "The resourcepack exporter writes version-appropriate assets, bitmap-font providers, and pack metadata for the selected Minecraft profile.",
        use_when = ["Building HUD bars, HUD elements, textures, or resource-pack output alongside a Sand datapack"],
        avoid_when = ["The project is datapack-only or needs unrelated resource-pack functionality not modeled by Sand"],
        returns = "The `Value` value produced to serialize this bitmap font provider to its Minecraft JSON object.",
        example = "use sand::prelude::*;\n\nfn demonstrate(bitmap_provider_value: &sand::resourcepack::BitmapProvider)  {\n    let to_json = bitmap_provider_value.to_json();\n}",
        availability = ["Cargo feature: resourcepack"],
    )]
    pub fn to_json(&self) -> Value {
        json!({
            "type": "bitmap",
            "file": self.file,
            "height": self.height,
            "ascent": self.ascent,
            "chars": self.chars,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::resourcepack::BitmapFont",
    module = "sand::resourcepack",
    summary = "A complete font definition targeting a single font file.",
    context = "A complete font definition targeting a single font file. Wraps one [`BitmapProvider`] and knows which font file to write to. Multiple `BitmapFont` instances that share the same `font_name` are merged by [`export_resourcepack_json`](sand::resourcepack::export_resourcepack_json) into a single `assets/<namespace>/font/<font_name>.json` file. Prefer using the `hud_bar!` or `hud_element!` macros (from `sand-macros`, with the `resourcepack` feature enabled) over constructing this type directly.",
    minecraft = "Wraps one [`BitmapProvider`] and knows which font file to write to. Multiple `BitmapFont` instances that share the same `font_name` are merged by [`export_resourcepack_json`](sand::resourcepack::export_resourcepack_json) into a single `assets/<namespace>/font/<font_name>.json` file.",
    use_when = ["Prefer using the `hud_bar!` or `hud_element!` macros (from `sand-macros`, with the `resourcepack` feature enabled) over constructing this type directly."],
    avoid_when = ["The project is datapack-only or needs unrelated resource-pack functionality not modeled by Sand"],
    example = "use sand::resourcepack::BitmapFont;",
    availability = ["Cargo feature: resourcepack"],
    fields(font_name = "Name of the font file (without extension), e.g. `\"default\"` or `\"hud\"`. Determines the output path: `assets/<namespace>/font/<font_name>.json`.", provider = "The bitmap provider this font contributes.", texture_dest = "Destination sub-path inside `assets/<namespace>/textures/` for the copied texture (without extension), e.g. `\"font/health_bar\"`.", texture_src = "Project-root-relative path to the source PNG to copy into the pack, e.g. `\"src/assets/health_bar.png\"`."),
)]
/// A complete font definition targeting a single font file.
///
/// Wraps one [`BitmapProvider`] and knows which font file to write to.
/// Multiple `BitmapFont` instances that share the same `font_name` are
/// merged by [`export_resourcepack_json`](crate::export_resourcepack_json)
/// into a single `assets/<namespace>/font/<font_name>.json` file.
///
/// Prefer using the `hud_bar!` or `hud_element!` macros (from `sand-macros`,
/// with the `resourcepack` feature enabled) over constructing this
/// type directly.
pub struct BitmapFont {
    /// Name of the font file (without extension), e.g. `"default"` or
    /// `"hud"`. Determines the output path:
    /// `assets/<namespace>/font/<font_name>.json`.
    ///
    /// All providers that share the same `font_name` within a namespace are
    /// merged into one file.
    pub font_name: &'static str,

    /// The bitmap provider this font contributes.
    pub provider: BitmapProvider,

    /// Project-root-relative path to the source PNG to copy into the pack,
    /// e.g. `"src/assets/health_bar.png"`.
    ///
    /// Set to `None` for programmatically generated assets or when the
    /// texture is already present in the pack from another component.
    pub texture_src: Option<&'static str>,

    /// Destination sub-path inside `assets/<namespace>/textures/` for the
    /// copied texture (without extension), e.g. `"font/health_bar"`.
    ///
    /// Ignored when `texture_src` is `None`.
    pub texture_dest: &'static str,
}

impl ResourcePackComponent for BitmapFont {
    fn assets(&self, namespace: &str) -> Vec<AssetOutput> {
        let mut outputs = Vec::new();

        // 1. Font JSON entry — will be merged with peers at export time.
        let font_path = format!("assets/{}/font/{}.json", namespace, self.font_name);
        outputs.push(AssetOutput {
            path: font_path,
            content: AssetContent::Json(json!({
                "providers": [self.provider.to_json()]
            })),
        });

        // 2. Texture copy (if a source file was supplied).
        if let Some(src) = self.texture_src {
            let tex_path = format!("assets/{}/textures/{}.png", namespace, self.texture_dest);
            outputs.push(AssetOutput {
                path: tex_path,
                content: AssetContent::CopyFrom(src.to_string()),
            });
        }

        outputs
    }
}
