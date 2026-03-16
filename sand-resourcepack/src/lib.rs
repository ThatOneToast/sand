//! # sand-resourcepack
//!
//! Resource pack support for the [Sand](https://github.com/ThatOneToast/sand)
//! Minecraft datapack toolkit.
//!
//! This crate provides everything needed to define a Minecraft **resource pack**
//! alongside your datapack:
//!
//! - [`ResourcePackComponent`] — trait implemented by all resource pack element types
//! - [`HudBar`] — bitmap-font progress bars for HUD overlays
//! - [`HudElement`] — static single-character HUD textures
//! - [`RawTexture`] — copies arbitrary PNG assets into the pack
//! - [`export_resourcepack_json`] — collects all registered components and
//!   serializes them for `sand build --resourcepack`
//! - [`resource_pack_format_for`] — maps a Minecraft version string to the
//!   correct resource pack format number
//!
//! # Usage with macros
//!
//! Enable the `resourcepack` feature on `sand-macros` to get the
//! [`hud_bar!`], [`hud_element!`], and [`texture!`] declarative macros:
//!
//! ```toml
//! # Cargo.toml
//! [dependencies]
//! sand-resourcepack = { path = "..." }
//! sand-macros = { path = "...", features = ["resourcepack"] }
//! ```
//!
//! ```rust,ignore
//! use sand_macros::{hud_bar, hud_element, texture};
//!
//! // Progress bar from a user PNG — unicode is auto-assigned
//! hud_bar!(
//!     name: "health",
//!     texture: "src/assets/health_bar.png",
//!     steps: 10,
//!     height: 14,   // pixel height of the rendered glyph
//!     ascent: 14,   // keep ascent == height to align the top of the bar at the baseline
//! );
//!
//! // Progress bar with programmatic pill-shaped sprite strip
//! // `create!` generates the PNG at build time — no external image needed
//! hud_bar!(
//!     name: "mana",
//!     texture: create!(fill: 0x4444FFFF, empty: 0x222244FF),
//!     steps: 10,
//!     height: 14,
//!     ascent: 14,
//! );
//!
//! // Static overlay from a user PNG — unicode is auto-assigned
//! hud_element!(
//!     name: "hotbar_bg",
//!     texture: "src/assets/hotbar.png",
//!     height: 22,
//!     ascent: -10,
//! );
//!
//! // Static overlay with programmatic solid-color texture
//! hud_element!(
//!     name: "dark_overlay",
//!     texture: gen!(color: 0x00000080),
//!     height: 22,
//!     ascent: -10,
//! );
//!
//! // Raw texture copy — no font JSON
//! texture!(
//!     id: "my_pack:item/custom_sword",
//!     path: "src/assets/custom_sword.png",
//! );
//! ```
//!
//! # Sizing and positioning
//!
//! ## Size
//!
//! Two fields control the rendered size of a bar:
//!
//! | Field | Effect |
//! |---|---|
//! | `height` | Pixel height of the glyph as Minecraft renders it |
//! | `frame_width` (inside `create!`) | Pixel width per frame; defaults to `2 × height` for pill proportions |
//!
//! Increase `height` (e.g. 9 → 14 → 20) to make the bar larger on screen.
//! Always set `ascent` to match the new `height` so the bar aligns correctly.
//! The in-game apparent size also depends on the player's GUI Scale setting.
//!
//! ## Vertical position
//!
//! `ascent` is the pixel offset from the Minecraft text baseline to the **top**
//! of the glyph. Use it to push bars up or down on screen:
//!
//! | `ascent` value | Effect |
//! |---|---|
//! | `== height` | Top of bar sits on the baseline (typical for actionbar HUD) |
//! | `< 0` | Bar moves below the baseline — useful for sub-hotbar elements |
//! | Large positive (e.g. `70`) | Pushes bar far above the actionbar area |
//!
//! ## Horizontal position
//!
//! The actionbar and title are center-aligned. Use [`BarHandle::show_at`] or
//! [`BarHandle::display_commands_at`] to shift a bar left or right from center:
//!
//! ```rust,ignore
//! // Show the health bar 40 px to the left of center.
//! HEALTH.show_at("@a", frame, "my_pack", -40);
//!
//! // Dynamic bar shifted 40 px right.
//! HEALTH.display_commands_at("@s", "hp_frame", "my_pack", 40);
//! ```
//!
//! [`advance_x`] returns the raw space-advance characters if you need to
//! compose the offset into your own text component.
//!
//! # Displaying HUD elements in commands
//!
//! Use the generated [`BarHandle`] / [`ElementHandle`] constants that `hud_bar!`
//! and `hud_element!` produce — no manual unicode handling needed:
//!
//! ```rust,ignore
//! // Show health bar at a fixed frame
//! HEALTH.show("@a", frame, "my_pack");
//!
//! // Dynamic bar driven by a scoreboard value
//! HEALTH.display_commands("@s", "hp_frame", "my_pack");
//!
//! // Show a static element
//! HOTBAR_BG.show("@a", "my_pack");
//! ```
//!
//! # Export hook
//!
//! Add a `sand_resource_export` binary to your project (see the template in
//! `sand`'s scaffold) and call [`export_resourcepack_json`] from it. Then run
//! `sand build --resourcepack` to produce the resource pack output.
//!
//! # Version support
//!
//! Currently targets Minecraft 1.21.11 (resource pack format 61). The
//! [`resource_pack_format_for`] function maps any supported version string to
//! the correct format number, with 61 used as the fallback for unknown or
//! future versions.

pub mod component;
pub mod components;
pub mod descriptor;
pub mod export;
pub mod gen_;
pub mod handle;
pub mod layout;
pub mod pack_format;
pub mod stat;
pub mod unicode;

pub use component::{AssetContent, AssetOutput, ResourcePackComponent, ResourcePackRecord};
pub use components::{
    BitmapFont, BitmapProvider, FontProvider, GenHudBar, GenHudElement, HudBar, HudElement,
    RawTexture,
};
pub use descriptor::ResourcePackDescriptor;
pub use export::export_resourcepack_json;
pub use gen_::Color;
pub use handle::{BarHandle, ElementHandle};
pub use layout::HudLayout;
pub use pack_format::resource_pack_format_for;
pub use stat::BarStat;
pub use unicode::{advance_x, bar_char, bar_text_json, element_char, element_text_json};

/// Re-exported so proc macros can write `::sand_resourcepack::inventory::submit!`
/// without requiring users to add `inventory` as a direct dependency.
#[doc(hidden)]
pub use inventory;
