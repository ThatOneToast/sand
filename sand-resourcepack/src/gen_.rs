//! Programmatic PNG generation for HUD components.
//!
//! Used by [`GenHudBar`] and [`GenHudElement`] to produce sprite-strip and
//! solid-color textures at build time from simple color parameters, so users
//! do not need to create PNG assets by hand.
//!
//! [`GenHudBar`]: crate::components::GenHudBar
//! [`GenHudElement`]: crate::components::GenHudElement

use png::Encoder;

// ── Color ─────────────────────────────────────────────────────────────────────

/// An RGBA color with 8-bit components.
///
/// Colors in the `gen!()` macro are specified as `0xRRGGBBAA` packed `u32`
/// literals (e.g. `0xFF4444FF` for opaque red, `0x00000080` for 50%
/// transparent black).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create from individual RGBA components.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    /// Create from a packed `0xRRGGBBAA` u32.
    ///
    /// # Example
    ///
    /// ```
    /// # use sand_resourcepack::Color;
    /// let red = Color::from_u32(0xFF0000FF);
    /// assert_eq!(red.r, 0xFF);
    /// assert_eq!(red.g, 0x00);
    /// assert_eq!(red.b, 0x00);
    /// assert_eq!(red.a, 0xFF);
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

/// Encode raw RGBA pixel data (row-major, top-to-bottom) into a PNG byte vector.
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
/// tall. Each frame of width `frame_width` represents one fill level:
///
/// - Frame 0 → fully empty (all `empty` colored pixels)
/// - Frame `steps − 1` → fully filled (all `fill` colored pixels)
/// - Intermediate frames are linearly interpolated left-to-right
///
/// Frames are rendered as a **pill / capsule shape**: the left and right ends
/// are rounded using a circle of radius `height / 2`. A 1-pixel dark border
/// rings the capsule, and a vertical brightness gradient is applied inside
/// (slightly lighter at the top, darker at the bottom). The fill/empty
/// boundary is a straight vertical cut across the capsule interior.
///
/// # Parameters
///
/// | Parameter | Description |
/// |---|---|
/// | `fill` | Color of the filled portion |
/// | `empty` | Color of the empty / background portion |
/// | `steps` | Number of frames (≥ 1) |
/// | `frame_width` | Pixel width of each individual frame |
/// | `height` | Pixel height of the entire sprite strip |
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
    // Axis of the capsule runs horizontally at y = radius.
    let axis_y = radius;
    let left_cap_x = radius;
    let right_cap_x = fw - radius;

    for frame in 0..steps {
        // Columns filled in this frame (left-to-right, straight cut).
        let filled_cols = if steps == 1 {
            frame_width
        } else {
            ((frame as u64 * frame_width as u64) / (steps as u64 - 1)) as u32
        };

        for py in 0..height {
            for px in 0..frame_width {
                // Sample at pixel center.
                let cx = px as f32 + 0.5;
                let cy = py as f32 + 0.5;

                // Distance from pixel center to the capsule axis segment.
                let nearest_x = cx.max(left_cap_x).min(right_cap_x);
                let dx = cx - nearest_x;
                let dy = cy - axis_y;
                let dist = (dx * dx + dy * dy).sqrt();

                // Outside the capsule — leave transparent.
                if dist > radius {
                    continue;
                }

                // Choose the base color from fill vs empty region.
                let base = if px < filled_cols { fill } else { empty };

                let pixel = if dist > radius - 1.5 {
                    // Border ring: darken to ~25 % of base.
                    Color::rgba(
                        (base.r as f32 * 0.25) as u8,
                        (base.g as f32 * 0.25) as u8,
                        (base.b as f32 * 0.25) as u8,
                        base.a,
                    )
                } else {
                    // Interior: apply top-bright / bottom-dark gradient.
                    // y_t = 0 at top, 1 at bottom.
                    let y_t = cy / h;
                    // Gradient: 1.25 at top → 0.65 at bottom.
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
/// Used by [`GenHudElement`] when the user does not supply a source PNG and
/// instead requests a programmatic fill via `gen!(color: 0xRRGGBBAA)`.
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
    fn gen_bar_png_produces_valid_png() {
        let bytes = gen_bar_png(
            Color::from_u32(0xFF0000FF),
            Color::from_u32(0x333333FF),
            10,
            9,
            9,
        );
        // PNG magic number
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn gen_element_png_produces_valid_png() {
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
        // Verify PNG dimensions in the IHDR chunk (bytes 16..20 = width, 20..24 = height).
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        assert_eq!(width, 50);
        assert_eq!(height, 8);
    }
}
