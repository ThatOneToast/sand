//! Deterministic unicode codepoint allocation for HUD components.
//!
//! Sand automatically assigns characters from the Unicode Private Use Area
//! (U+E000–U+F8FF) based on an FNV-1a hash of the component name, so users
//! never need to manage codepoints by hand.
//!
//! # Layout
//!
//! | Range | Purpose | Slots |
//! |---|---|---|
//! | U+E000–U+EFFF | Progress bars (32 codepoints each) | 128 bar slots |
//! | U+F000–U+F8FF | Static elements (1 codepoint each) | 2304 element slots |
//!
//! A "slot" is the block of codepoints reserved for a single named component.
//! The slot index is `FNV-1a(name) % num_slots`. Collision probability is low
//! for typical pack sizes, and [`export_resourcepack_json`] warns when it
//! detects any overlap.
//!
//! [`export_resourcepack_json`]: crate::export_resourcepack_json

/// Base codepoint for the bar region (U+E000).
const BAR_BASE: u32 = 0xE000;
/// Number of codepoints reserved per bar (max steps per bar).
const BAR_BLOCK: u32 = 32;
/// Number of bar slots available = (0xF000 - 0xE000) / 32 = 128.
const BAR_SLOTS: u32 = 128;

/// Base codepoint for the element region (U+F000).
const ELEM_BASE: u32 = 0xF000;
/// Number of element slots available = 0xF7FF - 0xF000 + 1 = 2048.
///
/// U+F800..U+F8FF is reserved for space-advance characters used by
/// [`advance_x`].
const ELEM_SLOTS: u32 = 2048;

// ── Space-advance characters (U+F800..U+F8FF) ─────────────────────────────────
//
// Each character in this region is registered in the Minecraft font's `space`
// provider with a specific advance width (in pixels). Positive advances move
// the cursor right; negative advances move it left.
//
// Encoding: powers-of-two so any offset in [-2047, +2047] can be expressed
// with at most 11 characters.
//
// | Codepoint | Advance |
// |-----------|---------|
// | U+F801    |      +1 |
// | U+F802    |      +2 |
// | U+F803    |      +4 |
// | U+F804    |      +8 |
// | U+F805    |     +16 |
// | U+F806    |     +32 |
// | U+F807    |     +64 |
// | U+F808    |    +128 |
// | U+F809    |    +256 |
// | U+F80A    |    +512 |
// | U+F80B    |   +1024 |
// | U+F811    |      -1 |
// | U+F812    |      -2 |
// | U+F813    |      -4 |
// | U+F814    |      -8 |
// | U+F815    |     -16 |
// | U+F816    |     -32 |
// | U+F817    |     -64 |
// | U+F818    |    -128 |
// | U+F819    |    -256 |
// | U+F81A    |    -512 |
// | U+F81B    |   -1024 |

const SPACE_POS_BASE: u32 = 0xF801; // positive-advance series start
const SPACE_NEG_BASE: u32 = 0xF811; // negative-advance series start
const SPACE_BITS: u32 = 11; // 2^0 .. 2^10 (values 1 .. 1024)

// ── Hash ──────────────────────────────────────────────────────────────────────

/// FNV-1a 32-bit hash.
fn fnv1a(s: &str) -> u32 {
    let mut hash: u32 = 2_166_136_261;
    for b in s.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    hash
}

// ── Bar helpers ───────────────────────────────────────────────────────────────

/// Return the first codepoint assigned to the bar named `name`.
///
/// All `steps` frames of the bar are allocated at
/// `base..base + steps`. With `BAR_BLOCK = 32`, bars with up to 32 frames
/// are collision-free within the bar region.
pub fn bar_base_codepoint(name: &str) -> u32 {
    let slot = fnv1a(name) % BAR_SLOTS;
    BAR_BASE + slot * BAR_BLOCK
}

/// Return the unicode character for frame `frame` of bar `name`.
///
/// # Example
///
/// ```
/// let ch = sand_resourcepack::bar_char("health", 0);
/// assert!(('\u{E000}'..='\u{F8FF}').contains(&ch));
/// ```
pub fn bar_char(name: &str, frame: u32) -> char {
    char::from_u32(bar_base_codepoint(name) + frame).unwrap_or('\u{FFFD}')
}

// ── Element helpers ───────────────────────────────────────────────────────────

/// Return the codepoint assigned to the element named `name`.
pub fn element_codepoint(name: &str) -> u32 {
    let slot = fnv1a(name) % ELEM_SLOTS;
    ELEM_BASE + slot
}

/// Return the unicode character assigned to the element named `name`.
///
/// # Example
///
/// ```
/// let ch = sand_resourcepack::element_char("hotbar_bg");
/// assert!(('\u{E000}'..='\u{F8FF}').contains(&ch));
/// ```
pub fn element_char(name: &str) -> char {
    char::from_u32(element_codepoint(name)).unwrap_or('\u{FFFD}')
}

// ── Space advance helpers ─────────────────────────────────────────────────────

/// Returns a string of Private-Use-Area characters whose combined advance
/// width equals `offset` pixels.
///
/// Positive `offset` shifts the cursor **right**; negative shifts it **left**.
/// An offset of `0` returns an empty string.
///
/// The returned characters must be placed in the same font text component as
/// the bar or element you are positioning. Sand automatically registers the
/// required `space` font provider in every exported font file, so no manual
/// font setup is needed.
///
/// # Range
///
/// Supports any offset in `[-2047, +2047]`. Larger absolute values are clamped
/// to 2047 by the binary decomposition.
///
/// # Example
///
/// ```rust,ignore
/// // Shift the health bar 20 pixels to the right of its natural position.
/// let cmd = HEALTH.show_at("@a", 5, "my_pack", 20);
/// ```
pub fn advance_x(offset: i32) -> String {
    if offset == 0 {
        return String::new();
    }
    let (base, magnitude) = if offset > 0 {
        (SPACE_POS_BASE, offset.unsigned_abs())
    } else {
        (SPACE_NEG_BASE, offset.unsigned_abs())
    };
    let mut result = String::new();
    let mut remaining = magnitude;
    // Emit chars from largest bit to smallest so the string is short.
    for bit in (0..SPACE_BITS).rev() {
        let value = 1u32 << bit;
        if remaining >= value {
            remaining -= value;
            if let Some(c) = char::from_u32(base + bit) {
                result.push(c);
            }
        }
    }
    result
}

/// Build a JSON-safe unicode escape string for all chars in `s`.
///
/// Each char is written as `\uXXXX`. This is used internally when composing
/// multi-character text fields (e.g. advance chars + bar char) for JSON text
/// components.
pub(crate) fn json_escape_chars(s: &str) -> String {
    s.chars().map(|c| format!("\\u{:04X}", c as u32)).collect()
}

// ── JSON text component helpers ───────────────────────────────────────────────

/// Return a Minecraft JSON text component string that renders frame `frame`
/// of bar `name` using the given namespace and font file.
///
/// The returned string can be passed directly to `tellraw`, `title`, or
/// `actionbar` commands.
///
/// # Example
///
/// ```rust,ignore
/// let json = sand_resourcepack::bar_text_json("health", 7, "my_pack", "default");
/// // Use in a Minecraft command:
/// // title @a actionbar {"text":"\uE047","font":"my_pack:default","color":"white"}
/// mcfunction! {
///     format!("title @a actionbar {json}");
/// }
/// ```
pub fn bar_text_json(name: &str, frame: u32, namespace: &str, font_name: &str) -> String {
    let cp = bar_base_codepoint(name) + frame;
    let font_id = format!("{namespace}:{font_name}");
    // JSON unicode escape: \uXXXX (4 hex digits, BMP only — PUA fits in BMP).
    format!(r#"{{"text":"\u{cp:04X}","font":"{font_id}","color":"white"}}"#)
}

/// Return a Minecraft JSON text component string that renders the element
/// named `name` using the given namespace and font file.
///
/// # Example
///
/// ```rust,ignore
/// let json = sand_resourcepack::element_text_json("hotbar_bg", "my_pack", "hud");
/// mcfunction! {
///     format!("title @a actionbar {json}");
/// }
/// ```
pub fn element_text_json(name: &str, namespace: &str, font_name: &str) -> String {
    let cp = element_codepoint(name);
    let font_id = format!("{namespace}:{font_name}");
    format!(r#"{{"text":"\u{cp:04X}","font":"{font_id}","color":"white"}}"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_chars_are_in_pua() {
        let ch = bar_char("health", 0);
        assert!(ch as u32 >= 0xE000 && ch as u32 <= 0xF8FF);
    }

    #[test]
    fn element_char_is_in_pua() {
        let ch = element_char("hotbar_bg");
        assert!(ch as u32 >= 0xF000 && ch as u32 <= 0xF8FF);
    }

    #[test]
    fn bar_frames_are_sequential() {
        let base = bar_base_codepoint("health");
        assert_eq!(bar_char("health", 0) as u32, base);
        assert_eq!(bar_char("health", 1) as u32, base + 1);
        assert_eq!(bar_char("health", 9) as u32, base + 9);
    }

    #[test]
    fn different_names_get_different_bases() {
        let a = bar_base_codepoint("health");
        let b = bar_base_codepoint("mana");
        // Both in bar region, but different slots.
        assert!(a >= BAR_BASE && a < ELEM_BASE);
        assert!(b >= BAR_BASE && b < ELEM_BASE);
        assert_ne!(a, b);
    }

    #[test]
    fn advance_x_zero_is_empty() {
        assert_eq!(advance_x(0), "");
    }

    #[test]
    fn advance_x_positive_uses_pos_region() {
        let s = advance_x(1);
        assert_eq!(s.chars().count(), 1);
        assert_eq!(s.chars().next().unwrap() as u32, SPACE_POS_BASE); // U+F801 = +1
    }

    #[test]
    fn advance_x_negative_uses_neg_region() {
        let s = advance_x(-1);
        assert_eq!(s.chars().count(), 1);
        assert_eq!(s.chars().next().unwrap() as u32, SPACE_NEG_BASE); // U+F811 = -1
    }

    #[test]
    fn advance_x_decomposes_correctly() {
        // 3 = 2 + 1 → U+F802 + U+F801 (two chars, from largest bit down)
        let s = advance_x(3);
        let cps: Vec<u32> = s.chars().map(|c| c as u32).collect();
        assert_eq!(cps.len(), 2);
        assert!(cps.contains(&(SPACE_POS_BASE + 1))); // +2
        assert!(cps.contains(&(SPACE_POS_BASE))); // +1
    }

    #[test]
    fn bar_text_json_format() {
        // Just confirm the output is valid-looking JSON with the right unicode escape.
        let json = bar_text_json("health", 0, "my_pack", "default");
        assert!(json.starts_with('{'));
        assert!(json.contains("my_pack:default"));
        assert!(json.contains("\\u"));
    }

    #[test]
    fn element_text_json_format() {
        let json = element_text_json("hotbar_bg", "my_pack", "hud");
        assert!(json.contains("my_pack:hud"));
    }
}
