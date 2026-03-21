//! Programmatic PNG sprite generation for HUD components.
//!
//! This module generates PNG textures at build time from simple color
//! parameters, so you do not need external image editing tools to create
//! standard HUD assets. The generated PNGs are fed directly into the resource
//! pack's font provider system.
//!
//! # Sprite types
//!
//! | Function | Output | Used by |
//! |---|---|---|
//! | [`gen_bar_png`] | Horizontal sprite-strip with `N` frames | [`GenHudBar`] |
//! | [`gen_element_png`] | Solid-color rectangle | [`GenHudElement`] |
//!
//! # Coordinate convention
//!
//! All PNG data is **row-major, top-to-bottom** with 4 bytes per pixel (RGBA).
//! The resulting `Vec<u8>` is valid PNG-encoded data ready to write to disk or
//! embed directly into the resource pack export.
//!
//! [`GenHudBar`]: crate::components::GenHudBar
//! [`GenHudElement`]: crate::components::GenHudElement

use png::Encoder;

// ── Color ─────────────────────────────────────────────────────────────────────

/// An RGBA color with 8-bit components, used as sprite fill parameters.
///
/// Colors in the `create!()` and `gen!()` macros are specified as packed
/// `0xRRGGBBAA` `u32` literals:
/// - `0xFF4444FF` — opaque red
/// - `0x222244FF` — opaque dark navy
/// - `0x00000080` — 50% transparent black
///
/// # Examples
/// ```
/// use sand_resourcepack::Color;
///
/// let red = Color::from_u32(0xFF0000FF);
/// assert_eq!(red.r, 0xFF);
/// assert_eq!(red.g, 0x00);
/// assert_eq!(red.a, 0xFF);
///
/// let semi_black = Color::rgba(0, 0, 0, 128);
/// assert_eq!(semi_black.a, 128);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    /// Red channel (0–255).
    pub r: u8,
    /// Green channel (0–255).
    pub g: u8,
    /// Blue channel (0–255).
    pub b: u8,
    /// Alpha channel (0 = fully transparent, 255 = fully opaque).
    pub a: u8,
}

impl Color {
    /// Create a color from individual RGBA components.
    ///
    /// All channels are `0–255`. Alpha `255` is fully opaque; `0` is transparent.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    /// Create a color from a packed `0xRRGGBBAA` `u32` literal.
    ///
    /// This is the format used by the `create!` and `gen!` macros.
    ///
    /// # Example
    /// ```
    /// use sand_resourcepack::Color;
    ///
    /// let red = Color::from_u32(0xFF0000FF);
    /// assert_eq!((red.r, red.g, red.b, red.a), (255, 0, 0, 255));
    /// ```
    pub const fn from_u32(packed: u32) -> Self {
        Color {
            r: ((packed >> 24) & 0xFF) as u8,
            g: ((packed >> 16) & 0xFF) as u8,
            b: ((packed >> 8) & 0xFF) as u8,
            a: (packed & 0xFF) as u8,
        }
    }
}

// ── PNG encoding ──────────────────────────────────────────────────────────────

/// Encode raw RGBA pixel data (row-major, top-to-bottom) into a valid PNG byte vector.
///
/// `pixels` must have exactly `width * height * 4` bytes. Returns a self-contained
/// PNG binary blob that can be written directly to a `.png` file or embedded into
/// the resource pack.
pub(crate) fn encode_png_rgba(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut encoder = Encoder::new(&mut buf, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("png: write header");
    writer.write_image_data(pixels).expect("png: write data");
    drop(writer);
    buf
}

// ── Bar sprite strip ──────────────────────────────────────────────────────────

/// Generate a horizontal sprite-strip PNG for a HUD progress bar.
///
/// The output image is `(steps × frame_width)` pixels wide and `height` pixels
/// tall. Frames are laid out left-to-right, each `frame_width` pixels wide:
///
/// - **Frame 0** → fully empty (all `empty` colored pixels)
/// - **Frame `steps − 1`** → fully filled (all `fill` colored pixels)
/// - **Intermediate frames** → linearly interpolated fill level (left-to-right cut)
///
/// Each frame is rendered as a **pill / capsule shape**: the left and right
/// ends are rounded with a circle of radius `height / 2`. A dark border ring
/// (~1.5 px) outlines the capsule, and a subtle vertical brightness gradient
/// is applied to the interior (slightly lighter at top, slightly darker at
/// bottom).
///
/// # Parameters
///
/// | Parameter | Type | Description |
/// |---|---|---|
/// | `fill` | [`Color`] | Color of the filled (progress) portion |
/// | `empty` | [`Color`] | Color of the unfilled (background) portion |
/// | `steps` | `u32` | Number of frames — must be ≥ 1 |
/// | `frame_width` | `u32` | Pixel width of each individual frame |
/// | `height` | `u32` | Pixel height of the entire strip |
///
/// For pill proportions, use `frame_width ≈ 2 × height`. The resulting PNG is
/// passed to the `hud_bar!` `texture: create!(...)` argument.
///
/// # Panics
///
/// Panics if `steps == 0`.
pub fn gen_bar_png(
    fill: Color,
    empty: Color,
    steps: u32,
    frame_width: u32,
    height: u32,
) -> Vec<u8> {
    assert!(steps > 0, "gen_bar_png: steps must be > 0");

    let total_width = steps * frame_width;
    // Initialize to fully transparent.
    let mut pixels = vec![0u8; (total_width * height * 4) as usize];

    let fw = frame_width as f32;
    let h = height as f32;
    let radius = h * 0.5;
    // Capsule axis runs horizontally at y = radius (vertical center).
    let axis_y = radius;
    let left_cap_x = radius;
    let right_cap_x = fw - radius;

    for frame in 0..steps {
        // Number of columns filled in this frame.
        let filled_cols = if steps == 1 {
            frame_width
        } else {
            ((frame as u64 * frame_width as u64) / (steps as u64 - 1)) as u32
        };

        for py in 0..height {
            for px in 0..frame_width {
                // Sample at pixel center for sub-pixel accuracy.
                let cx = px as f32 + 0.5;
                let cy = py as f32 + 0.5;

                // Distance from pixel center to the nearest point on the capsule axis segment.
                let nearest_x = cx.max(left_cap_x).min(right_cap_x);
                let dx = cx - nearest_x;
                let dy = cy - axis_y;
                let dist = (dx * dx + dy * dy).sqrt();

                // Outside the capsule — leave transparent.
                if dist > radius {
                    continue;
                }

                // Base color: fill region or empty region.
                let base = if px < filled_cols { fill } else { empty };

                let pixel = if dist > radius - 1.5 {
                    // Border ring: darken to ~25 % of base color.
                    Color::rgba(
                        (base.r as f32 * 0.25) as u8,
                        (base.g as f32 * 0.25) as u8,
                        (base.b as f32 * 0.25) as u8,
                        base.a,
                    )
                } else {
                    // Interior: apply top-bright / bottom-dark gradient.
                    // y_t goes 0.0 (top) → 1.0 (bottom).
                    let y_t = cy / h;
                    // Gradient multiplier: 1.25 at top, 0.65 at bottom.
                    let g = 1.25 - y_t * 0.60;
                    Color::rgba(
                        ((base.r as f32 * g) as u32).min(255) as u8,
                        ((base.g as f32 * g) as u32).min(255) as u8,
                        ((base.b as f32 * g) as u32).min(255) as u8,
                        base.a,
                    )
                };

                let idx = ((py * total_width + frame * frame_width + px) * 4) as usize;
                pixels[idx] = pixel.r;
                pixels[idx + 1] = pixel.g;
                pixels[idx + 2] = pixel.b;
                pixels[idx + 3] = pixel.a;
            }
        }
    }

    encode_png_rgba(total_width, height, &pixels)
}

// ── Solid element ─────────────────────────────────────────────────────────────

/// Generate a solid-color RGBA PNG of the given dimensions.
///
/// All pixels are set to `color`. Used by [`GenHudElement`] when the user does
/// not supply a source PNG and instead requests a solid fill via
/// `gen!(color: 0xRRGGBBAA)`.
///
/// Set `color.a < 255` for semi-transparent overlays (e.g. dark vignettes).
///
/// [`GenHudElement`]: crate::components::GenHudElement
pub fn gen_element_png(color: Color, width: u32, height: u32) -> Vec<u8> {
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    for chunk in pixels.chunks_exact_mut(4) {
        chunk[0] = color.r;
        chunk[1] = color.g;
        chunk[2] = color.b;
        chunk[3] = color.a;
    }
    encode_png_rgba(width, height, &pixels)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_from_u32() {
        let c = Color::from_u32(0xDEADBEEF);
        assert_eq!(c.r, 0xDE);
        assert_eq!(c.g, 0xAD);
        assert_eq!(c.b, 0xBE);
        assert_eq!(c.a, 0xEF);
    }

    #[test]
    fn color_rgba_constructor() {
        let c = Color::rgba(10, 20, 30, 40);
        assert_eq!((c.r, c.g, c.b, c.a), (10, 20, 30, 40));
    }

    #[test]
    fn gen_bar_png_produces_valid_png_magic() {
        let bytes = gen_bar_png(
            Color::from_u32(0xFF0000FF),
            Color::from_u32(0x333333FF),
            10,
            9,
            9,
        );
        // PNG magic number must be the first 8 bytes
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn gen_element_png_produces_valid_png_magic() {
        let bytes = gen_element_png(Color::from_u32(0x00000080), 32, 32);
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn gen_bar_png_dimensions() {
        // steps=5, frame_width=10, height=8 → total 50×8
        let bytes = gen_bar_png(
            Color::rgba(255, 0, 0, 255),
            Color::rgba(50, 50, 50, 255),
            5,
            10,
            8,
        );
        // IHDR chunk: bytes 16..20 = width, 20..24 = height
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        assert_eq!(width, 50);
        assert_eq!(height, 8);
    }

    #[test]
    fn gen_element_png_dimensions() {
        let bytes = gen_element_png(Color::rgba(0, 255, 0, 200), 64, 16);
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        assert_eq!(width, 64);
        assert_eq!(height, 16);
    }
}
