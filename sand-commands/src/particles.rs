//! Particle effect builders for Minecraft datapacks.
//!
//! # Quick-start
//!
//! ```rust,ignore
//! use sand_commands::{ParticleBuilder, Particle, ParticleSpread};
//!
//! // Colored dust ring
//! let cmds = ParticleBuilder::new(Particle::dust_hex(0xFF4400, 1.5))
//!     .circle(2.0, 1.0, 32);
//!
//! // Arbitrary point list
//! let cmds = ParticleBuilder::new(Particle::named("minecraft:end_rod"))
//!     .points_at(&[[0.0,0.0,0.0],[1.0,1.0,0.0],[2.0,0.0,0.0]]);
//! ```

use std::collections::BTreeMap;

use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};

// ── Particle ──────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::IntoParticleId",
    aliases = ["sand::cmd::IntoParticleId", "sand::prelude::cmd::IntoParticleId"],
    module = "sand::command",
    summary = "Conversion into a particle resource-location token.",
    context = "Conversion into a particle resource-location token. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::IntoParticleId;",
)]
/// Conversion into a particle resource-location token.
pub trait IntoParticleId {
    /// Converts a typed or validated value into a Minecraft particle identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::IntoParticleId::into_particle_id",
        aliases = ["sand::cmd::IntoParticleId::into_particle_id", "sand::prelude::cmd::IntoParticleId::into_particle_id"],
        module = "sand::command",
        summary = "Converts a typed or validated value into a Minecraft particle identifier.",
        context = "Converts a typed or validated value into a Minecraft particle identifier. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The string value produced to convert a typed or validated value into a Minecraft particle identifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::command::IntoParticleId>(into_particle_id_value: T)  {\n    let into_particle_id = into_particle_id_value.into_particle_id();\n}",
    )]
    fn into_particle_id(self) -> String;
}

impl IntoParticleId for String {
    fn into_particle_id(self) -> String {
        self
    }
}

impl IntoParticleId for &str {
    fn into_particle_id(self) -> String {
        self.to_string()
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Particle",
    aliases = ["sand::cmd::Particle", "sand::prelude::Particle", "sand::prelude::cmd::Particle"],
    module = "sand::command",
    summary = "A Minecraft particle type with its parameters.",
    context = "A Minecraft particle type with its parameters. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Particle;",
    variants(Block = "`minecraft:block` particle showing a block's break texture.", Dust = "Colored `minecraft:dust` particle. RGB values in `0.0–1.0`.", DustColorTransition = "`minecraft:dust_color_transition` — animates from one color to another.", Item = "`minecraft:item` particle showing an item's texture.", Named = "A named particle with no extra parameters, e.g. `\"minecraft:flame\"`.", Raw = "Explicit opaque particle token.", SculkCharge = "`minecraft:sculk_charge` with a rotation in radians.", Shriek = "`minecraft:shriek` with a delay in ticks before appearing."),
    variant_fields(Block = ["`minecraft:block` particle showing a block's break texture."], Dust(b = "`b` provides the blue channel when colored `minecraft:dust` particle. RGB values in `0.0–1.0`.", g = "`g` provides the green channel when colored `minecraft:dust` particle. RGB values in `0.0–1.0`.", r = "`r` provides the red channel when colored `minecraft:dust` particle. RGB values in `0.0–1.0`.", scale = "`scale` provides the particle scale when colored `minecraft:dust` particle. RGB values in `0.0–1.0`."), DustColorTransition(from_b = "`from_b` provides the from b when `minecraft:dust_color_transition` — animates from one color to another.", from_g = "`from_g` provides the from g when `minecraft:dust_color_transition` — animates from one color to another.", from_r = "`from_r` provides the from r when `minecraft:dust_color_transition` — animates from one color to another.", scale = "`scale` provides the particle scale when `minecraft:dust_color_transition` — animates from one color to another.", to_b = "`to_b` provides the to b when `minecraft:dust_color_transition` — animates from one color to another.", to_g = "`to_g` provides the to g when `minecraft:dust_color_transition` — animates from one color to another.", to_r = "`to_r` provides the to r when `minecraft:dust_color_transition` — animates from one color to another."), Item = ["`minecraft:item` particle showing an item's texture."], Named = ["A named particle with no extra parameters, e.g. `\"minecraft:flame\"`."], Raw = ["Explicit opaque particle token."], SculkCharge(roll = "`roll` provides the roll when `minecraft:sculk_charge` with a rotation in radians."), Shriek(delay = "`delay` provides the delay when `minecraft:shriek` with a delay in ticks before appearing.")),
)]
/// A Minecraft particle type with its parameters.
#[derive(Debug, Clone)]
pub enum Particle {
    /// A named particle with no extra parameters, e.g. `"minecraft:flame"`.
    Named(#[doc = "A named particle with no extra parameters, e.g. `\"minecraft:flame\"`."] String),
    /// Colored `minecraft:dust` particle. RGB values in `0.0–1.0`.
    Dust {
        #[doc = "`r` provides the red channel when colored `minecraft:dust` particle. RGB values in `0.0–1.0`."]
        r: f32,
        #[doc = "`g` provides the green channel when colored `minecraft:dust` particle. RGB values in `0.0–1.0`."]
        g: f32,
        #[doc = "`b` provides the blue channel when colored `minecraft:dust` particle. RGB values in `0.0–1.0`."]
        b: f32,
        #[doc = "`scale` provides the particle scale when colored `minecraft:dust` particle. RGB values in `0.0–1.0`."]
        scale: f32,
    },
    /// `minecraft:dust_color_transition` — animates from one color to another.
    DustColorTransition {
        /// `from_r` provides the from r when `minecraft:dust_color_transition` — animates from one color to another.
        from_r: f32,
        /// `from_g` provides the from g when `minecraft:dust_color_transition` — animates from one color to another.
        from_g: f32,
        /// `from_b` provides the from b when `minecraft:dust_color_transition` — animates from one color to another.
        from_b: f32,
        /// `to_r` provides the to r when `minecraft:dust_color_transition` — animates from one color to another.
        to_r: f32,
        /// `to_g` provides the to g when `minecraft:dust_color_transition` — animates from one color to another.
        to_g: f32,
        /// `to_b` provides the to b when `minecraft:dust_color_transition` — animates from one color to another.
        to_b: f32,
        /// `scale` provides the particle scale when `minecraft:dust_color_transition` — animates from one color to another.
        scale: f32,
    },
    /// `minecraft:block` particle showing a block's break texture.
    Block(#[doc = "`minecraft:block` particle showing a block's break texture."] String),
    /// `minecraft:item` particle showing an item's texture.
    Item(#[doc = "`minecraft:item` particle showing an item's texture."] String),
    /// `minecraft:sculk_charge` with a rotation in radians.
    SculkCharge {
        #[doc = "`roll` provides the roll when `minecraft:sculk_charge` with a rotation in radians."]
        roll: f32,
    },
    /// `minecraft:shriek` with a delay in ticks before appearing.
    Shriek {
        #[doc = "`delay` provides the delay when `minecraft:shriek` with a delay in ticks before appearing."]
        delay: u32,
    },
    /// Explicit opaque particle token.
    Raw(#[doc = "Explicit opaque particle token."] String),
}

impl Particle {
    /// A named particle with no extra parameters (e.g. `"minecraft:flame"`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Particle::named",
        aliases = ["sand::cmd::Particle::named", "sand::prelude::Particle::named", "sand::prelude::cmd::Particle::named"],
        module = "sand::command",
        kind = "method",
        summary = "A named particle with no extra parameters (e.g. `\"minecraft:flame\"`).",
        context = "A named particle with no extra parameters (e.g. `\"minecraft:flame\"`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` provides the author-visible text value used to use a named particle with no extra parameters (e.g. `\"minecraft:flame\"`)."),
        returns = "A newly constructed `Particle` configured to use a named particle with no extra parameters (e.g. `\"minecraft:flame\"`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: impl sand::command::IntoParticleId)  {\n    let particle = sand::command::Particle::named(name);\n}",
    )]
    pub fn named(name: impl IntoParticleId) -> Self {
        Particle::Named(name.into_particle_id())
    }

    /// Create an intentionally opaque particle token.
    ///
    /// Sand renders this unchanged and does not apply particle-specific
    /// compatibility checks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Particle::raw_token",
        aliases = ["sand::cmd::Particle::raw_token", "sand::prelude::Particle::raw_token", "sand::prelude::cmd::Particle::raw_token"],
        module = "sand::command",
        kind = "method",
        summary = "Create an intentionally opaque particle token. Sand renders this unchanged and does not apply particle-specific compatibility checks.",
        context = "Create an intentionally opaque particle token. Sand renders this unchanged and does not apply particle-specific compatibility checks. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(token = "`token` supplies the token value used to create an intentionally opaque particle token. Sand renders this unchanged and does not apply particle-specific compatibility checks."),
        returns = "A newly constructed `Particle` configured to create an intentionally opaque particle token. Sand renders this unchanged and does not apply particle-specific compatibility checks.",
        example = "use sand::prelude::*;\n\nfn demonstrate(token: impl Into < String >)  {\n    let particle = sand::command::Particle::raw_token(token);\n}",
    )]
    pub fn raw_token(token: impl Into<String>) -> Self {
        Self::Raw(token.into())
    }

    /// Colored dust particle. RGB values in `0.0–1.0`, scale is size (1.0 = default).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Particle::dust",
        aliases = ["sand::cmd::Particle::dust", "sand::prelude::Particle::dust", "sand::prelude::cmd::Particle::dust"],
        module = "sand::command",
        kind = "method",
        summary = "Colored dust particle. RGB values in `0.0–1.0`, scale is size (1.0 = default).",
        context = "Colored dust particle. RGB values in `0.0–1.0`, scale is size (1.0 = default). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(r = "`r` supplies the r value used to use colored dust particle. RGB values in `0.0–1.0`, scale is size (1.0 = default).", g = "`g` supplies the g value used to use colored dust particle. RGB values in `0.0–1.0`, scale is size (1.0 = default).", b = "`b` supplies the b value used to use colored dust particle. RGB values in `0.0–1.0`, scale is size (1.0 = default).", scale = "`scale` supplies the scale value used to use colored dust particle. RGB values in `0.0–1.0`, scale is size (1.0 = default)."),
        returns = "A newly constructed `Particle` configured to use colored dust particle. RGB values in `0.0–1.0`, scale is size (1.0 = default).",
        example = "use sand::prelude::*;\n\nfn demonstrate(r: f32, g: f32, b: f32, scale: f32)  {\n    let particle = sand::command::Particle::dust(r, g, b, scale);\n}",
    )]
    pub fn dust(r: f32, g: f32, b: f32, scale: f32) -> Self {
        Particle::Dust { r, g, b, scale }
    }

    /// Colored dust from 8-bit RGB (0–255).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Particle::dust_u8",
        aliases = ["sand::cmd::Particle::dust_u8", "sand::prelude::Particle::dust_u8", "sand::prelude::cmd::Particle::dust_u8"],
        module = "sand::command",
        kind = "method",
        summary = "Colored dust from 8-bit RGB (0–255).",
        context = "Colored dust from 8-bit RGB (0–255). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(r = "`r` supplies the r value used to use colored dust from 8-bit RGB (0–255).", g = "`g` supplies the g value used to use colored dust from 8-bit RGB (0–255).", b = "`b` supplies the b value used to use colored dust from 8-bit RGB (0–255).", scale = "`scale` supplies the scale value used to use colored dust from 8-bit RGB (0–255)."),
        returns = "A newly constructed `Particle` configured to use colored dust from 8-bit RGB (0–255).",
        example = "use sand::prelude::*;\n\nfn demonstrate(r: u8, g: u8, b: u8, scale: f32)  {\n    let particle = sand::command::Particle::dust_u8(r, g, b, scale);\n}",
    )]
    pub fn dust_u8(r: u8, g: u8, b: u8, scale: f32) -> Self {
        Particle::Dust {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            scale,
        }
    }

    /// Colored dust from a hex RGB value, e.g. `0xFF4400` for orange.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Particle::dust_hex",
        aliases = ["sand::cmd::Particle::dust_hex", "sand::prelude::Particle::dust_hex", "sand::prelude::cmd::Particle::dust_hex"],
        module = "sand::command",
        kind = "method",
        summary = "Colored dust from a hex RGB value, e.g. `0xFF4400` for orange.",
        context = "Colored dust from a hex RGB value, e.g. `0xFF4400` for orange. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(hex = "`hex` supplies the hex value used to use colored dust from a hex RGB value, e.g. `0xFF4400` for orange.", scale = "`scale` supplies the scale value used to use colored dust from a hex RGB value, e.g. `0xFF4400` for orange."),
        returns = "A newly constructed `Particle` configured to use colored dust from a hex RGB value, e.g. `0xFF4400` for orange.",
        example = "use sand::prelude::*;\n\nfn demonstrate(hex: u32, scale: f32)  {\n    let particle = sand::command::Particle::dust_hex(hex, scale);\n}",
    )]
    pub fn dust_hex(hex: u32, scale: f32) -> Self {
        Particle::dust_u8(
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
            scale,
        )
    }

    /// Color-transitioning dust. RGB values in `0.0–1.0`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Particle::dust_transition",
        aliases = ["sand::cmd::Particle::dust_transition", "sand::prelude::Particle::dust_transition", "sand::prelude::cmd::Particle::dust_transition"],
        module = "sand::command",
        kind = "method",
        summary = "Color-transitioning dust. RGB values in `0.0–1.0`.",
        context = "Color-transitioning dust. RGB values in `0.0–1.0`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(from_r = "`from_r` supplies the from r value used to use color-transitioning dust. RGB values in `0.0–1.0`.", from_g = "`from_g` supplies the from g value used to use color-transitioning dust. RGB values in `0.0–1.0`.", from_b = "`from_b` supplies the from b value used to use color-transitioning dust. RGB values in `0.0–1.0`.", to_r = "`to_r` supplies the to r value used to use color-transitioning dust. RGB values in `0.0–1.0`.", to_g = "`to_g` supplies the to g value used to use color-transitioning dust. RGB values in `0.0–1.0`.", to_b = "`to_b` supplies the to b value used to use color-transitioning dust. RGB values in `0.0–1.0`.", scale = "`scale` supplies the scale value used to use color-transitioning dust. RGB values in `0.0–1.0`."),
        returns = "A newly constructed `Particle` configured to use color-transitioning dust. RGB values in `0.0–1.0`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(from_r: f32, from_g: f32, from_b: f32, to_r: f32, to_g: f32, to_b: f32, scale: f32)  {\n    let particle = sand::command::Particle::dust_transition(from_r, from_g, from_b, to_r, to_g, to_b, scale);\n}",
    )]
    pub fn dust_transition(
        from_r: f32,
        from_g: f32,
        from_b: f32,
        to_r: f32,
        to_g: f32,
        to_b: f32,
        scale: f32,
    ) -> Self {
        Particle::DustColorTransition {
            from_r,
            from_g,
            from_b,
            to_r,
            to_g,
            to_b,
            scale,
        }
    }

    /// Color-transitioning dust from two hex RGB values.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Particle::dust_transition_hex",
        aliases = ["sand::cmd::Particle::dust_transition_hex", "sand::prelude::Particle::dust_transition_hex", "sand::prelude::cmd::Particle::dust_transition_hex"],
        module = "sand::command",
        kind = "method",
        summary = "Color-transitioning dust from two hex RGB values.",
        context = "Color-transitioning dust from two hex RGB values. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(from_hex = "`from_hex` supplies the from hex value used to use color-transitioning dust from two hex RGB values.", to_hex = "`to_hex` supplies the to hex value used to use color-transitioning dust from two hex RGB values.", scale = "`scale` supplies the scale value used to use color-transitioning dust from two hex RGB values."),
        returns = "A newly constructed `Particle` configured to use color-transitioning dust from two hex RGB values.",
        example = "use sand::prelude::*;\n\nfn demonstrate(from_hex: u32, to_hex: u32, scale: f32)  {\n    let particle = sand::command::Particle::dust_transition_hex(from_hex, to_hex, scale);\n}",
    )]
    pub fn dust_transition_hex(from_hex: u32, to_hex: u32, scale: f32) -> Self {
        let [fr, fg, fb] = hex_to_f32(from_hex);
        let [tr, tg, tb] = hex_to_f32(to_hex);
        Particle::DustColorTransition {
            from_r: fr,
            from_g: fg,
            from_b: fb,
            to_r: tr,
            to_g: tg,
            to_b: tb,
            scale,
        }
    }

    /// Block break texture particle, e.g. `"minecraft:stone"`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Particle::block",
        aliases = ["sand::cmd::Particle::block", "sand::prelude::Particle::block", "sand::prelude::cmd::Particle::block"],
        module = "sand::command",
        kind = "method",
        summary = "Block break texture particle, e.g. `\"minecraft:stone\"`.",
        context = "Block break texture particle, e.g. `\"minecraft:stone\"`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(state = "`state` supplies the state value used to block break texture particle, e.g. `\"minecraft:stone\"`."),
        returns = "A newly constructed `Particle` configured to block break texture particle, e.g. `\"minecraft:stone\"`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(state: impl Into < String >)  {\n    let particle = sand::command::Particle::block(state);\n}",
    )]
    pub fn block(state: impl Into<String>) -> Self {
        Particle::Block(state.into())
    }

    /// Item texture particle, e.g. `"minecraft:diamond_sword"`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Particle::item",
        aliases = ["sand::cmd::Particle::item", "sand::prelude::Particle::item", "sand::prelude::cmd::Particle::item"],
        module = "sand::command",
        kind = "method",
        summary = "Item texture particle, e.g. `\"minecraft:diamond_sword\"`.",
        context = "Item texture particle, e.g. `\"minecraft:diamond_sword\"`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(item = "`item` provides the item value or item predicate used to item texture particle, e.g. `\"minecraft:diamond_sword\"`."),
        returns = "A newly constructed `Particle` configured to item texture particle, e.g. `\"minecraft:diamond_sword\"`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(item: impl Into < String >)  {\n    let particle = sand::command::Particle::item(item);\n}",
    )]
    pub fn item(item: impl Into<String>) -> Self {
        Particle::Item(item.into())
    }

    /// `minecraft:sculk_charge` with a roll angle in radians.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Particle::sculk_charge",
        aliases = ["sand::cmd::Particle::sculk_charge", "sand::prelude::Particle::sculk_charge", "sand::prelude::cmd::Particle::sculk_charge"],
        module = "sand::command",
        kind = "method",
        summary = "`minecraft:sculk_charge` with a roll angle in radians.",
        context = "`minecraft:sculk_charge` with a roll angle in radians. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(roll = "`roll` supplies the roll value used to emit the documented `minecraft:sculk_charge` with a roll angle in radians form."),
        returns = "A newly constructed `Particle` configured to emit the documented `minecraft:sculk_charge` with a roll angle in radians form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(roll: f32)  {\n    let particle = sand::command::Particle::sculk_charge(roll);\n}",
    )]
    pub fn sculk_charge(roll: f32) -> Self {
        Particle::SculkCharge { roll }
    }

    /// `minecraft:shriek` with a delay in ticks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Particle::shriek",
        aliases = ["sand::cmd::Particle::shriek", "sand::prelude::Particle::shriek", "sand::prelude::cmd::Particle::shriek"],
        module = "sand::command",
        kind = "method",
        summary = "`minecraft:shriek` with a delay in ticks.",
        context = "`minecraft:shriek` with a delay in ticks. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(delay = "`delay` supplies the delay value used to emit the documented `minecraft:shriek` with a delay in ticks form."),
        returns = "A newly constructed `Particle` configured to emit the documented `minecraft:shriek` with a delay in ticks form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(delay: u32)  {\n    let particle = sand::command::Particle::shriek(delay);\n}",
    )]
    pub fn shriek(delay: u32) -> Self {
        Particle::Shriek { delay }
    }

    fn command_token(&self) -> String {
        match self {
            Particle::Named(n) => n.clone(),
            Particle::Dust { r, g, b, scale } => {
                format!(
                    "minecraft:dust{{color:[{},{},{}],scale:{}}}",
                    fmt_c(*r),
                    fmt_c(*g),
                    fmt_c(*b),
                    fmt_c(*scale)
                )
            }
            Particle::DustColorTransition {
                from_r,
                from_g,
                from_b,
                to_r,
                to_g,
                to_b,
                scale,
            } => {
                format!(
                    "minecraft:dust_color_transition{{from_color:[{},{},{}],to_color:[{},{},{}],scale:{}}}",
                    fmt_c(*from_r),
                    fmt_c(*from_g),
                    fmt_c(*from_b),
                    fmt_c(*to_r),
                    fmt_c(*to_g),
                    fmt_c(*to_b),
                    fmt_c(*scale),
                )
            }
            Particle::Block(s) => format!("minecraft:block {s}"),
            Particle::Item(s) => format!("minecraft:item {s}"),
            Particle::SculkCharge { roll } => format!("minecraft:sculk_charge {}", fmt_c(*roll)),
            Particle::Shriek { delay } => format!("minecraft:shriek {delay}"),
            Particle::Raw(token) => token.clone(),
        }
    }
}

impl Validate for Particle {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        match self {
            Self::Named(id) => {
                crate::validate::resource_location_shape(id, "ParticleCommand", "particle.id")
                    .map(|_| ())
                    .map_err(|error| particle_error("SAND-PARTICLE-ID", error.field, error.message))
            }
            Self::Dust { r, g, b, scale } => {
                validate_color(*r, "particle.color.r")?;
                validate_color(*g, "particle.color.g")?;
                validate_color(*b, "particle.color.b")?;
                validate_scale(*scale)
            }
            Self::DustColorTransition {
                from_r,
                from_g,
                from_b,
                to_r,
                to_g,
                to_b,
                scale,
            } => {
                for (field, value) in [
                    ("particle.from_color.r", *from_r),
                    ("particle.from_color.g", *from_g),
                    ("particle.from_color.b", *from_b),
                    ("particle.to_color.r", *to_r),
                    ("particle.to_color.g", *to_g),
                    ("particle.to_color.b", *to_b),
                ] {
                    validate_color(value, field)?;
                }
                validate_scale(*scale)
            }
            Self::Block(state) => validate_particle_payload_id(state, "particle.block"),
            Self::Item(item) => validate_particle_payload_id(item, "particle.item"),
            Self::SculkCharge { roll } => validate_finite(*roll as f64, "particle.roll"),
            Self::Shriek { .. } | Self::Raw(_) => Ok(()),
        }
    }
}

// ── ParticleSpread ─────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ParticleSpread",
    aliases = ["sand::cmd::ParticleSpread", "sand::prelude::ParticleSpread", "sand::prelude::cmd::ParticleSpread"],
    module = "sand::command",
    summary = "Spread/dispersion of a particle from its spawn position.",
    context = "Spread/dispersion of a particle from its spawn position. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ParticleSpread;",
    fields(dx = "`dx` provides the x-axis spread when spread/dispersion of a particle from its spawn position.", dy = "`dy` provides the y-axis spread when spread/dispersion of a particle from its spawn position.", dz = "`dz` provides the z-axis spread when spread/dispersion of a particle from its spawn position."),
)]
/// Spread/dispersion of a particle from its spawn position.
#[derive(Debug, Clone)]
pub struct ParticleSpread {
    /// `dx` provides the x-axis spread when spread/dispersion of a particle from its spawn position.
    pub dx: f64,
    /// `dy` provides the y-axis spread when spread/dispersion of a particle from its spawn position.
    pub dy: f64,
    /// `dz` provides the z-axis spread when spread/dispersion of a particle from its spawn position.
    pub dz: f64,
}

impl ParticleSpread {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleSpread::POINT",
        aliases = ["sand::cmd::ParticleSpread::POINT", "sand::prelude::ParticleSpread::POINT", "sand::prelude::cmd::ParticleSpread::POINT"],
        module = "sand::command",
        kind = "associated_const",
        summary = "No spread — particles appear exactly at the specified position.",
        context = "No spread — particles appear exactly at the specified position. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        example = "use sand::command::ParticleSpread;",
    )]
    /// No spread — particles appear exactly at the specified position.
    pub const POINT: Self = Self {
        dx: 0.0,
        dy: 0.0,
        dz: 0.0,
    };

    /// Uniform spread in all three directions.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleSpread::uniform",
        aliases = ["sand::cmd::ParticleSpread::uniform", "sand::prelude::ParticleSpread::uniform", "sand::prelude::cmd::ParticleSpread::uniform"],
        module = "sand::command",
        kind = "method",
        summary = "Uniform spread in all three directions.",
        context = "Uniform spread in all three directions. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(v = "`v` supplies the v value used to uniform spread in all three directions."),
        returns = "A newly constructed `ParticleSpread` configured to uniform spread in all three directions.",
        example = "use sand::prelude::*;\n\nfn demonstrate(v: f64)  {\n    let particle_spread = sand::command::ParticleSpread::uniform(v);\n}",
    )]
    pub fn uniform(v: f64) -> Self {
        Self {
            dx: v,
            dy: v,
            dz: v,
        }
    }

    /// Custom per-axis spread.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleSpread::new",
        aliases = ["sand::cmd::ParticleSpread::new", "sand::prelude::ParticleSpread::new", "sand::prelude::cmd::ParticleSpread::new"],
        module = "sand::command",
        kind = "method",
        summary = "Custom per-axis spread.",
        context = "Custom per-axis spread. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(dx = "`dx` provides the x-axis offset or spread used to use custom per-axis spread.", dy = "`dy` provides the y-axis offset or spread used to use custom per-axis spread.", dz = "`dz` provides the z-axis offset or spread used to use custom per-axis spread."),
        returns = "A newly constructed `ParticleSpread` configured to use custom per-axis spread.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dx: f64, dy: f64, dz: f64)  {\n    let particle_spread = sand::command::ParticleSpread::new(dx, dy, dz);\n}",
    )]
    pub fn new(dx: f64, dy: f64, dz: f64) -> Self {
        Self { dx, dy, dz }
    }
}

impl Validate for ParticleSpread {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        for (field, value) in [
            ("spread.dx", self.dx),
            ("spread.dy", self.dy),
            ("spread.dz", self.dz),
        ] {
            validate_non_negative(value, field)?;
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ParticleCommand",
    aliases = ["sand::cmd::ParticleCommand", "sand::prelude::cmd::ParticleCommand"],
    module = "sand::command",
    summary = "One structured `particle` command retained until validation and rendering.",
    context = "One structured `particle` command retained until validation and rendering. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ParticleCommand;",
)]
/// One structured `particle` command retained until validation and rendering.
#[derive(Debug, Clone)]
pub struct ParticleCommand {
    particle: Particle,
    position: [f64; 3],
    spread: ParticleSpread,
    speed: f64,
    count: u32,
    force: bool,
}

impl Validate for ParticleCommand {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.particle.validate(profile)?;
        self.spread.validate(profile)?;
        validate_non_negative(self.speed, "speed")?;
        if self.count == 0 {
            return Err(particle_error(
                "SAND-PARTICLE-COUNT",
                "count",
                "particles_per_point must be greater than zero",
            ));
        }
        for (field, value) in [
            ("position.x", self.position[0]),
            ("position.y", self.position[1]),
            ("position.z", self.position[2]),
        ] {
            validate_finite(value, field)?;
        }
        Ok(())
    }
}

impl RenderCommand for ParticleCommand {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        let mode = if self.force { "force" } else { "normal" };
        format!(
            "particle {} ~{} ~{} ~{} {} {} {} {} {} {mode}",
            self.particle.command_token(),
            fmt_f(self.position[0]),
            fmt_f(self.position[1]),
            fmt_f(self.position[2]),
            fmt_f(self.spread.dx),
            fmt_f(self.spread.dy),
            fmt_f(self.spread.dz),
            fmt_f(self.speed),
            self.count,
        )
    }
}

// ── ParticleBuilder ────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ParticleBuilder",
    aliases = ["sand::cmd::ParticleBuilder", "sand::prelude::ParticleBuilder", "sand::prelude::cmd::ParticleBuilder"],
    module = "sand::command",
    summary = "Fluent builder for generating `particle` commands.",
    context = "Fluent builder for generating `particle` commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ParticleBuilder;",
)]
/// Fluent builder for generating `particle` commands.
#[derive(Debug, Clone)]
pub struct ParticleBuilder {
    particle: Particle,
    spread: ParticleSpread,
    speed: f64,
    particles_per_point: u32,
    force: bool,
}

impl ParticleBuilder {
    /// Create a new builder for the given particle type.
    ///
    /// Defaults: `spread = POINT`, `speed = 0`, `particles_per_point = 1`, `force = true`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::new",
        aliases = ["sand::cmd::ParticleBuilder::new", "sand::prelude::ParticleBuilder::new", "sand::prelude::cmd::ParticleBuilder::new"],
        module = "sand::command",
        kind = "method",
        summary = "Create a new builder for the given particle type.",
        context = "Create a new builder for the given particle type. Defaults: `spread = POINT`, `speed = 0`, `particles_per_point = 1`, `force = true`.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(particle = "`particle` supplies the particle value used to create a new builder for the given particle type."),
        returns = "A newly constructed `ParticleBuilder` configured to create a new builder for the given particle type.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle: sand::command::Particle)  {\n    let particle_builder = sand::command::ParticleBuilder::new(particle);\n}",
    )]
    pub fn new(particle: Particle) -> Self {
        Self {
            particle,
            spread: ParticleSpread::POINT,
            speed: 0.0,
            particles_per_point: 1,
            force: true,
        }
    }

    /// Set the random spread box around each particle's spawn position.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::spread",
        aliases = ["sand::cmd::ParticleBuilder::spread", "sand::prelude::ParticleBuilder::spread", "sand::prelude::cmd::ParticleBuilder::spread"],
        module = "sand::command",
        kind = "method",
        summary = "Set the random spread box around each particle's spawn position.",
        context = "Set the random spread box around each particle's spawn position. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(spread = "`spread` supplies the spread value used to set the random spread box around each particle's spawn position."),
        returns = "The `ParticleBuilder` value with the documented change applied to set the random spread box around each particle's spawn position.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: sand::command::ParticleBuilder, spread: sand::command::ParticleSpread)  {\n    let updated_particle_builder = particle_builder_value.spread(spread);\n}",
    )]
    pub fn spread(mut self, spread: ParticleSpread) -> Self {
        self.spread = spread;
        self
    }

    /// Set the initial speed of each particle after spawning.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::speed",
        aliases = ["sand::cmd::ParticleBuilder::speed", "sand::prelude::ParticleBuilder::speed", "sand::prelude::cmd::ParticleBuilder::speed"],
        module = "sand::command",
        kind = "method",
        summary = "Set the initial speed of each particle after spawning.",
        context = "Set the initial speed of each particle after spawning. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(speed = "`speed` supplies the speed value used to set the initial speed of each particle after spawning."),
        returns = "The `ParticleBuilder` value with the documented change applied to set the initial speed of each particle after spawning.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: sand::command::ParticleBuilder, speed: f64)  {\n    let updated_particle_builder = particle_builder_value.speed(speed);\n}",
    )]
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    /// Number of particles Minecraft spawns per command call (default: `1`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::particles_per_point",
        aliases = ["sand::cmd::ParticleBuilder::particles_per_point", "sand::prelude::ParticleBuilder::particles_per_point", "sand::prelude::cmd::ParticleBuilder::particles_per_point"],
        module = "sand::command",
        kind = "method",
        summary = "Number of particles Minecraft spawns per command call (default: `1`).",
        context = "Number of particles Minecraft spawns per command call (default: `1`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(n = "`n` supplies the n value used to number of particles Minecraft spawns per command call (default: `1`)."),
        returns = "The `ParticleBuilder` value with the documented change applied to number of particles Minecraft spawns per command call (default: `1`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: sand::command::ParticleBuilder, n: u32)  {\n    let updated_particle_builder = particle_builder_value.particles_per_point(n);\n}",
    )]
    pub fn particles_per_point(mut self, n: u32) -> Self {
        self.particles_per_point = n;
        self
    }

    /// Whether to use `force` visibility mode (default: `true`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::force",
        aliases = ["sand::cmd::ParticleBuilder::force", "sand::prelude::ParticleBuilder::force", "sand::prelude::cmd::ParticleBuilder::force"],
        module = "sand::command",
        kind = "method",
        summary = "Whether to use `force` visibility mode (default: `true`).",
        context = "Whether to use `force` visibility mode (default: `true`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(force = "Whether to use `force` visibility mode (default: `true`)."),
        returns = "The `ParticleBuilder` value with the documented change applied to determine whether to use `force` visibility mode (default: `true`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: sand::command::ParticleBuilder, force: bool)  {\n    let updated_particle_builder = particle_builder_value.force(force);\n}",
    )]
    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Validate the particle and numeric command settings without rendering.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::validate",
        aliases = ["sand::cmd::ParticleBuilder::validate", "sand::prelude::ParticleBuilder::validate", "sand::prelude::cmd::ParticleBuilder::validate"],
        module = "sand::command",
        kind = "method",
        summary = "Validate the particle and numeric command settings without rendering.",
        context = "Validate the particle and numeric command settings without rendering. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "On success, the value produced to validate the particle and numeric command settings without rendering; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder)  {\n    let validate = particle_builder_value.validate();\n}",
    )]
    pub fn validate(&self) -> CommandResult<()> {
        self.command_at([0.0, 0.0, 0.0])
            .validate(&CommandProfile::unprofiled())
    }

    /// Fallible point renderer used by VFX and strict callers.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_points_at",
        aliases = ["sand::cmd::ParticleBuilder::try_points_at", "sand::prelude::ParticleBuilder::try_points_at", "sand::prelude::cmd::ParticleBuilder::try_points_at"],
        module = "sand::command",
        kind = "method",
        summary = "Fallible point renderer used by VFX and strict callers.",
        context = "Fallible point renderer used by VFX and strict callers. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pts = "`pts` supplies the pts value used to use fallible point renderer used by VFX and strict callers."),
        returns = "On success, the value produced to use fallible point renderer used by VFX and strict callers; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, pts: & [[f64 ; 3]])  {\n    let try_points_at = particle_builder_value.try_points_at(pts);\n}",
    )]
    pub fn try_points_at(&self, pts: &[[f64; 3]]) -> CommandResult<Vec<String>> {
        if pts.is_empty() {
            return Err(particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "points",
                "geometry must contain at least one point",
            ));
        }
        pts.iter()
            .map(|point| self.command_at(*point).try_build())
            .collect()
    }

    /// Fallible circle generator with explicit geometry diagnostics.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_circle",
        aliases = ["sand::cmd::ParticleBuilder::try_circle", "sand::prelude::ParticleBuilder::try_circle", "sand::prelude::cmd::ParticleBuilder::try_circle"],
        module = "sand::command",
        kind = "method",
        summary = "Fallible circle generator with explicit geometry diagnostics.",
        context = "Fallible circle generator with explicit geometry diagnostics. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to use fallible circle generator with explicit geometry diagnostics.", y_offset = "`y_offset` supplies the y offset value used to use fallible circle generator with explicit geometry diagnostics.", points = "`points` supplies the points value used to use fallible circle generator with explicit geometry diagnostics."),
        returns = "On success, the value produced to use fallible circle generator with explicit geometry diagnostics; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, y_offset: f64, points: usize)  {\n    let try_circle = particle_builder_value.try_circle(radius, y_offset, points);\n}",
    )]
    pub fn try_circle(
        &self,
        radius: f64,
        y_offset: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(radius, "geometry.radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(points)?;
        Ok(self.circle(radius, y_offset, points))
    }

    /// Validates an arc geometry and returns its ordered particle commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_arc",
        aliases = ["sand::cmd::ParticleBuilder::try_arc", "sand::prelude::ParticleBuilder::try_arc", "sand::prelude::cmd::ParticleBuilder::try_arc"],
        module = "sand::command",
        kind = "method",
        summary = "Validates an arc geometry and returns its ordered particle commands.",
        context = "Validates an arc geometry and returns its ordered particle commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to validate an arc geometry and returns its ordered particle commands.", y_offset = "`y_offset` supplies the y offset value used to validate an arc geometry and returns its ordered particle commands.", start_deg = "`start_deg` supplies the start deg value used to validate an arc geometry and returns its ordered particle commands.", end_deg = "`end_deg` supplies the end deg value used to validate an arc geometry and returns its ordered particle commands.", points = "`points` supplies the points value used to validate an arc geometry and returns its ordered particle commands."),
        returns = "Validates an arc geometry and returns its ordered particle commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, y_offset: f64, start_deg: f64, end_deg: f64, points: usize)  {\n    let try_arc = particle_builder_value.try_arc(radius, y_offset, start_deg, end_deg, points);\n}",
    )]
    pub fn try_arc(
        &self,
        radius: f64,
        y_offset: f64,
        start_deg: f64,
        end_deg: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(radius, "geometry.radius")?;
        for (field, value) in [
            ("geometry.y_offset", y_offset),
            ("geometry.start_degrees", start_deg),
            ("geometry.end_degrees", end_deg),
        ] {
            validate_finite(value, field)?;
        }
        validate_points(points)?;
        Ok(self.arc(radius, y_offset, start_deg, end_deg, points))
    }

    /// Validates a regular polygon and returns its ordered particle commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_polygon",
        aliases = ["sand::cmd::ParticleBuilder::try_polygon", "sand::prelude::ParticleBuilder::try_polygon", "sand::prelude::cmd::ParticleBuilder::try_polygon"],
        module = "sand::command",
        kind = "method",
        summary = "Validates a regular polygon and returns its ordered particle commands.",
        context = "Validates a regular polygon and returns its ordered particle commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(sides = "`sides` supplies the sides value used to validate a regular polygon and returns its ordered particle commands.", radius = "`radius` supplies the radius value used to validate a regular polygon and returns its ordered particle commands.", y_offset = "`y_offset` supplies the y offset value used to validate a regular polygon and returns its ordered particle commands.", points_per_side = "`points_per_side` supplies the points per side value used to validate a regular polygon and returns its ordered particle commands."),
        returns = "Validates a regular polygon and returns its ordered particle commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, sides: usize, radius: f64, y_offset: f64, points_per_side: usize)  {\n    let try_polygon = particle_builder_value.try_polygon(sides, radius, y_offset, points_per_side);\n}",
    )]
    pub fn try_polygon(
        &self,
        sides: usize,
        radius: f64,
        y_offset: f64,
        points_per_side: usize,
    ) -> CommandResult<Vec<String>> {
        if sides < 3 {
            return Err(particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "geometry.sides",
                "polygon sides must be at least 3",
            ));
        }
        validate_points(points_per_side)?;
        sides.checked_mul(points_per_side).ok_or_else(|| {
            particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "geometry.points",
                "polygon output count overflows `usize`",
            )
        })?;
        validate_geometry_non_negative(radius, "geometry.radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        Ok(self.polygon(sides, radius, y_offset, points_per_side))
    }

    /// Validates a star geometry and returns its ordered particle commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_star",
        aliases = ["sand::cmd::ParticleBuilder::try_star", "sand::prelude::ParticleBuilder::try_star", "sand::prelude::cmd::ParticleBuilder::try_star"],
        module = "sand::command",
        kind = "method",
        summary = "Validates a star geometry and returns its ordered particle commands.",
        context = "Validates a star geometry and returns its ordered particle commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(arms = "`arms` supplies the arms value used to validate a star geometry and returns its ordered particle commands.", outer_radius = "`outer_radius` supplies the outer radius value used to validate a star geometry and returns its ordered particle commands.", inner_radius = "`inner_radius` supplies the inner radius value used to validate a star geometry and returns its ordered particle commands.", y_offset = "`y_offset` supplies the y offset value used to validate a star geometry and returns its ordered particle commands."),
        returns = "Validates a star geometry and returns its ordered particle commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, arms: usize, outer_radius: f64, inner_radius: f64, y_offset: f64)  {\n    let try_star = particle_builder_value.try_star(arms, outer_radius, inner_radius, y_offset);\n}",
    )]
    pub fn try_star(
        &self,
        arms: usize,
        outer_radius: f64,
        inner_radius: f64,
        y_offset: f64,
    ) -> CommandResult<Vec<String>> {
        if arms < 2 || arms.checked_mul(2).is_none() {
            return Err(particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "geometry.arms",
                "star arms must be at least 2 and small enough to double safely",
            ));
        }
        validate_geometry_non_negative(outer_radius, "geometry.outer_radius")?;
        validate_geometry_non_negative(inner_radius, "geometry.inner_radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        Ok(self.star(arms, outer_radius, inner_radius, y_offset))
    }

    /// Fallible sphere generator with explicit geometry diagnostics.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_sphere",
        aliases = ["sand::cmd::ParticleBuilder::try_sphere", "sand::prelude::ParticleBuilder::try_sphere", "sand::prelude::cmd::ParticleBuilder::try_sphere"],
        module = "sand::command",
        kind = "method",
        summary = "Fallible sphere generator with explicit geometry diagnostics.",
        context = "Fallible sphere generator with explicit geometry diagnostics. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to use fallible sphere generator with explicit geometry diagnostics.", y_offset = "`y_offset` supplies the y offset value used to use fallible sphere generator with explicit geometry diagnostics.", points = "`points` supplies the points value used to use fallible sphere generator with explicit geometry diagnostics."),
        returns = "On success, the value produced to use fallible sphere generator with explicit geometry diagnostics; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, y_offset: f64, points: usize)  {\n    let try_sphere = particle_builder_value.try_sphere(radius, y_offset, points);\n}",
    )]
    pub fn try_sphere(
        &self,
        radius: f64,
        y_offset: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(radius, "geometry.radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(points)?;
        Ok(self.sphere(radius, y_offset, points))
    }

    /// Fallible helix generator with explicit geometry diagnostics.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_helix",
        aliases = ["sand::cmd::ParticleBuilder::try_helix", "sand::prelude::ParticleBuilder::try_helix", "sand::prelude::cmd::ParticleBuilder::try_helix"],
        module = "sand::command",
        kind = "method",
        summary = "Fallible helix generator with explicit geometry diagnostics.",
        context = "Fallible helix generator with explicit geometry diagnostics. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to use fallible helix generator with explicit geometry diagnostics.", height = "`height` supplies the height value used to use fallible helix generator with explicit geometry diagnostics.", turns = "`turns` supplies the turns value used to use fallible helix generator with explicit geometry diagnostics.", points = "`points` supplies the points value used to use fallible helix generator with explicit geometry diagnostics."),
        returns = "On success, the value produced to use fallible helix generator with explicit geometry diagnostics; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, height: f64, turns: f64, points: usize)  {\n    let try_helix = particle_builder_value.try_helix(radius, height, turns, points);\n}",
    )]
    pub fn try_helix(
        &self,
        radius: f64,
        height: f64,
        turns: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(radius, "geometry.radius")?;
        validate_geometry_non_negative(height, "geometry.height")?;
        validate_non_negative(turns, "geometry.turns")?;
        if turns == 0.0 {
            return Err(particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "geometry.turns",
                "helix turns must be greater than zero",
            ));
        }
        validate_points(points)?;
        Ok(self.helix(radius, height, turns, points))
    }

    /// Validates a double helix and returns its ordered particle commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_double_helix",
        aliases = ["sand::cmd::ParticleBuilder::try_double_helix", "sand::prelude::ParticleBuilder::try_double_helix", "sand::prelude::cmd::ParticleBuilder::try_double_helix"],
        module = "sand::command",
        kind = "method",
        summary = "Validates a double helix and returns its ordered particle commands.",
        context = "Validates a double helix and returns its ordered particle commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to validate a double helix and returns its ordered particle commands.", height = "`height` supplies the height value used to validate a double helix and returns its ordered particle commands.", turns = "`turns` supplies the turns value used to validate a double helix and returns its ordered particle commands.", points = "`points` supplies the points value used to validate a double helix and returns its ordered particle commands."),
        returns = "Validates a double helix and returns its ordered particle commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, height: f64, turns: f64, points: usize)  {\n    let try_double_helix = particle_builder_value.try_double_helix(radius, height, turns, points);\n}",
    )]
    pub fn try_double_helix(
        &self,
        radius: f64,
        height: f64,
        turns: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        self.try_helix(radius, height, turns, points)?;
        Ok(self.double_helix(radius, height, turns, points))
    }

    /// Validates a line segment and returns its ordered particle commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_line",
        aliases = ["sand::cmd::ParticleBuilder::try_line", "sand::prelude::ParticleBuilder::try_line", "sand::prelude::cmd::ParticleBuilder::try_line"],
        module = "sand::command",
        kind = "method",
        summary = "Validates a line segment and returns its ordered particle commands.",
        context = "Validates a line segment and returns its ordered particle commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(from = "`from` supplies the from value used to validate a line segment and returns its ordered particle commands.", to = "`to` supplies the to value used to validate a line segment and returns its ordered particle commands.", points = "`points` supplies the points value used to validate a line segment and returns its ordered particle commands."),
        returns = "Validates a line segment and returns its ordered particle commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, from: [f64 ; 3], to: [f64 ; 3], points: usize)  {\n    let try_line = particle_builder_value.try_line(from, to, points);\n}",
    )]
    pub fn try_line(
        &self,
        from: [f64; 3],
        to: [f64; 3],
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_points(points)?;
        for (index, value) in from.into_iter().chain(to).enumerate() {
            validate_finite(value, format!("geometry.coordinate[{index}]"))?;
        }
        Ok(self.line(from, to, points))
    }

    /// Validates a filled disc and returns its ordered particle commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_disc",
        aliases = ["sand::cmd::ParticleBuilder::try_disc", "sand::prelude::ParticleBuilder::try_disc", "sand::prelude::cmd::ParticleBuilder::try_disc"],
        module = "sand::command",
        kind = "method",
        summary = "Validates a filled disc and returns its ordered particle commands.",
        context = "Validates a filled disc and returns its ordered particle commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to validate a filled disc and returns its ordered particle commands.", y_offset = "`y_offset` supplies the y offset value used to validate a filled disc and returns its ordered particle commands.", density = "`density` supplies the density value used to validate a filled disc and returns its ordered particle commands."),
        returns = "Validates a filled disc and returns its ordered particle commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, y_offset: f64, density: usize)  {\n    let try_disc = particle_builder_value.try_disc(radius, y_offset, density);\n}",
    )]
    pub fn try_disc(
        &self,
        radius: f64,
        y_offset: f64,
        density: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(radius, "geometry.radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(density)?;
        Ok(self.disc(radius, y_offset, density))
    }

    /// Validates a torus geometry and returns its ordered particle commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_torus",
        aliases = ["sand::cmd::ParticleBuilder::try_torus", "sand::prelude::ParticleBuilder::try_torus", "sand::prelude::cmd::ParticleBuilder::try_torus"],
        module = "sand::command",
        kind = "method",
        summary = "Validates a torus geometry and returns its ordered particle commands.",
        context = "Validates a torus geometry and returns its ordered particle commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(major_radius = "`major_radius` supplies the major radius value used to validate a torus geometry and returns its ordered particle commands.", minor_radius = "`minor_radius` supplies the minor radius value used to validate a torus geometry and returns its ordered particle commands.", y_offset = "`y_offset` supplies the y offset value used to validate a torus geometry and returns its ordered particle commands.", rings = "`rings` supplies the rings value used to validate a torus geometry and returns its ordered particle commands.", segments_per_ring = "`segments_per_ring` supplies the segments per ring value used to validate a torus geometry and returns its ordered particle commands."),
        returns = "Validates a torus geometry and returns its ordered particle commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, major_radius: f64, minor_radius: f64, y_offset: f64, rings: usize, segments_per_ring: usize)  {\n    let try_torus = particle_builder_value.try_torus(major_radius, minor_radius, y_offset, rings, segments_per_ring);\n}",
    )]
    pub fn try_torus(
        &self,
        major_radius: f64,
        minor_radius: f64,
        y_offset: f64,
        rings: usize,
        segments_per_ring: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(major_radius, "geometry.major_radius")?;
        validate_geometry_non_negative(minor_radius, "geometry.minor_radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(rings)?;
        validate_points(segments_per_ring)?;
        rings.checked_mul(segments_per_ring).ok_or_else(|| {
            particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "geometry.points",
                "torus output count overflows `usize`",
            )
        })?;
        Ok(self.torus(
            major_radius,
            minor_radius,
            y_offset,
            rings,
            segments_per_ring,
        ))
    }

    /// Validates a cone geometry and returns its ordered particle commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_cone",
        aliases = ["sand::cmd::ParticleBuilder::try_cone", "sand::prelude::ParticleBuilder::try_cone", "sand::prelude::cmd::ParticleBuilder::try_cone"],
        module = "sand::command",
        kind = "method",
        summary = "Validates a cone geometry and returns its ordered particle commands.",
        context = "Validates a cone geometry and returns its ordered particle commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(base_radius = "`base_radius` supplies the base radius value used to validate a cone geometry and returns its ordered particle commands.", height = "`height` supplies the height value used to validate a cone geometry and returns its ordered particle commands.", y_offset = "`y_offset` supplies the y offset value used to validate a cone geometry and returns its ordered particle commands.", rings = "`rings` supplies the rings value used to validate a cone geometry and returns its ordered particle commands."),
        returns = "Validates a cone geometry and returns its ordered particle commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, base_radius: f64, height: f64, y_offset: f64, rings: usize)  {\n    let try_cone = particle_builder_value.try_cone(base_radius, height, y_offset, rings);\n}",
    )]
    pub fn try_cone(
        &self,
        base_radius: f64,
        height: f64,
        y_offset: f64,
        rings: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(base_radius, "geometry.base_radius")?;
        validate_geometry_non_negative(height, "geometry.height")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(rings)?;
        Ok(self.cone(base_radius, height, y_offset, rings))
    }

    /// Validates a radial burst and returns its ordered particle commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_burst",
        aliases = ["sand::cmd::ParticleBuilder::try_burst", "sand::prelude::ParticleBuilder::try_burst", "sand::prelude::cmd::ParticleBuilder::try_burst"],
        module = "sand::command",
        kind = "method",
        summary = "Validates a radial burst and returns its ordered particle commands.",
        context = "Validates a radial burst and returns its ordered particle commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to validate a radial burst and returns its ordered particle commands.", y_offset = "`y_offset` supplies the y offset value used to validate a radial burst and returns its ordered particle commands.", points = "`points` supplies the points value used to validate a radial burst and returns its ordered particle commands."),
        returns = "Validates a radial burst and returns its ordered particle commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, y_offset: f64, points: usize)  {\n    let try_burst = particle_builder_value.try_burst(radius, y_offset, points);\n}",
    )]
    pub fn try_burst(
        &self,
        radius: f64,
        y_offset: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(radius, "geometry.radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(points)?;
        Ok(self.burst(radius, y_offset, points))
    }

    /// Validates a wave geometry and returns its ordered particle commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_wave",
        aliases = ["sand::cmd::ParticleBuilder::try_wave", "sand::prelude::ParticleBuilder::try_wave", "sand::prelude::cmd::ParticleBuilder::try_wave"],
        module = "sand::command",
        kind = "method",
        summary = "Validates a wave geometry and returns its ordered particle commands.",
        context = "Validates a wave geometry and returns its ordered particle commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(length = "`length` supplies the length value used to validate a wave geometry and returns its ordered particle commands.", amplitude = "`amplitude` supplies the amplitude value used to validate a wave geometry and returns its ordered particle commands.", cycles = "`cycles` supplies the cycles value used to validate a wave geometry and returns its ordered particle commands.", y_offset = "`y_offset` supplies the y offset value used to validate a wave geometry and returns its ordered particle commands.", points = "`points` supplies the points value used to validate a wave geometry and returns its ordered particle commands."),
        returns = "Validates a wave geometry and returns its ordered particle commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, length: f64, amplitude: f64, cycles: f64, y_offset: f64, points: usize)  {\n    let try_wave = particle_builder_value.try_wave(length, amplitude, cycles, y_offset, points);\n}",
    )]
    pub fn try_wave(
        &self,
        length: f64,
        amplitude: f64,
        cycles: f64,
        y_offset: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(length, "geometry.length")?;
        validate_geometry_non_negative(amplitude, "geometry.amplitude")?;
        validate_non_negative(cycles, "geometry.cycles")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(points)?;
        Ok(self.wave(length, amplitude, cycles, y_offset, points))
    }

    /// Validates a rectangular grid and returns its ordered particle commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::try_grid",
        aliases = ["sand::cmd::ParticleBuilder::try_grid", "sand::prelude::ParticleBuilder::try_grid", "sand::prelude::cmd::ParticleBuilder::try_grid"],
        module = "sand::command",
        kind = "method",
        summary = "Validates a rectangular grid and returns its ordered particle commands.",
        context = "Validates a rectangular grid and returns its ordered particle commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(width = "`width` supplies the width value used to validate a rectangular grid and returns its ordered particle commands.", depth = "`depth` supplies the depth value used to validate a rectangular grid and returns its ordered particle commands.", cols = "`cols` supplies the cols value used to validate a rectangular grid and returns its ordered particle commands.", rows = "`rows` supplies the rows value used to validate a rectangular grid and returns its ordered particle commands.", y_offset = "`y_offset` supplies the y offset value used to validate a rectangular grid and returns its ordered particle commands."),
        returns = "Validates a rectangular grid and returns its ordered particle commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, width: f64, depth: f64, cols: usize, rows: usize, y_offset: f64)  {\n    let try_grid = particle_builder_value.try_grid(width, depth, cols, rows, y_offset);\n}",
    )]
    pub fn try_grid(
        &self,
        width: f64,
        depth: f64,
        cols: usize,
        rows: usize,
        y_offset: f64,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(width, "geometry.width")?;
        validate_geometry_non_negative(depth, "geometry.depth")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(cols)?;
        validate_points(rows)?;
        cols.checked_mul(rows).ok_or_else(|| {
            particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "geometry.points",
                "grid output count overflows `usize`",
            )
        })?;
        Ok(self.grid(width, depth, cols, rows, y_offset))
    }

    // ── Shape generators ──────────────────────────────────────────────────────

    /// A horizontal ring of particles at `y_offset` above the executor.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::circle",
        aliases = ["sand::cmd::ParticleBuilder::circle", "sand::prelude::ParticleBuilder::circle", "sand::prelude::cmd::ParticleBuilder::circle"],
        module = "sand::command",
        kind = "method",
        summary = "A horizontal ring of particles at `y_offset` above the executor.",
        context = "A horizontal ring of particles at `y_offset` above the executor. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to use a horizontal ring of particles at `y_offset` above the executor.", y_offset = "A horizontal ring of particles at `y_offset` above the executor.", points = "`points` supplies the points value used to use a horizontal ring of particles at `y_offset` above the executor."),
        returns = "The ordered values produced to use a horizontal ring of particles at `y_offset` above the executor.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, y_offset: f64, points: usize)  {\n    let values = particle_builder_value.circle(radius, y_offset, points);\n}",
    )]
    pub fn circle(&self, radius: f64, y_offset: f64, points: usize) -> Vec<String> {
        (0..points)
            .map(|i| {
                let a = TAU * i as f64 / points as f64;
                self.cmd(radius * a.cos(), y_offset, radius * a.sin())
            })
            .collect()
    }

    /// A partial arc of a circle, from `start_deg` to `end_deg` (degrees).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::arc",
        aliases = ["sand::cmd::ParticleBuilder::arc", "sand::prelude::ParticleBuilder::arc", "sand::prelude::cmd::ParticleBuilder::arc"],
        module = "sand::command",
        kind = "method",
        summary = "A partial arc of a circle, from `start_deg` to `end_deg` (degrees).",
        context = "A partial arc of a circle, from `start_deg` to `end_deg` (degrees). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to use a partial arc of a circle, from `start_deg` to `end_deg` (degrees).", y_offset = "`y_offset` supplies the y offset value used to use a partial arc of a circle, from `start_deg` to `end_deg` (degrees).", start_deg = "A partial arc of a circle, from `start_deg` to `end_deg` (degrees).", end_deg = "A partial arc of a circle, from `start_deg` to `end_deg` (degrees).", points = "`points` supplies the points value used to use a partial arc of a circle, from `start_deg` to `end_deg` (degrees)."),
        returns = "The ordered values produced to use a partial arc of a circle, from `start_deg` to `end_deg` (degrees).",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, y_offset: f64, start_deg: f64, end_deg: f64, points: usize)  {\n    let values = particle_builder_value.arc(radius, y_offset, start_deg, end_deg, points);\n}",
    )]
    pub fn arc(
        &self,
        radius: f64,
        y_offset: f64,
        start_deg: f64,
        end_deg: f64,
        points: usize,
    ) -> Vec<String> {
        if points == 0 {
            return vec![];
        }
        let start = start_deg.to_radians();
        let end = end_deg.to_radians();
        let steps = if points == 1 { 1 } else { points - 1 };
        (0..points)
            .map(|i| {
                let a = start + (end - start) * i as f64 / steps as f64;
                self.cmd(radius * a.cos(), y_offset, radius * a.sin())
            })
            .collect()
    }

    /// A regular polygon at `y_offset`. `points_per_side` particles per edge.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::polygon",
        aliases = ["sand::cmd::ParticleBuilder::polygon", "sand::prelude::ParticleBuilder::polygon", "sand::prelude::cmd::ParticleBuilder::polygon"],
        module = "sand::command",
        kind = "method",
        summary = "A regular polygon at `y_offset`. `points_per_side` particles per edge.",
        context = "A regular polygon at `y_offset`. `points_per_side` particles per edge. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(sides = "`sides` supplies the sides value used to use a regular polygon at `y_offset`. `points_per_side` particles per edge.", radius = "`radius` supplies the radius value used to use a regular polygon at `y_offset`. `points_per_side` particles per edge.", y_offset = "A regular polygon at `y_offset`. `points_per_side` particles per edge.", points_per_side = "A regular polygon at `y_offset`. `points_per_side` particles per edge."),
        returns = "The ordered values produced to use a regular polygon at `y_offset`. `points_per_side` particles per edge.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, sides: usize, radius: f64, y_offset: f64, points_per_side: usize)  {\n    let values = particle_builder_value.polygon(sides, radius, y_offset, points_per_side);\n}",
    )]
    pub fn polygon(
        &self,
        sides: usize,
        radius: f64,
        y_offset: f64,
        points_per_side: usize,
    ) -> Vec<String> {
        if sides < 3 || points_per_side == 0 {
            return vec![];
        }
        let mut cmds = Vec::new();
        for side in 0..sides {
            let a1 = TAU * side as f64 / sides as f64;
            let a2 = TAU * (side + 1) as f64 / sides as f64;
            let (x1, z1) = (radius * a1.cos(), radius * a1.sin());
            let (x2, z2) = (radius * a2.cos(), radius * a2.sin());
            let steps = points_per_side.max(1);
            for p in 0..steps {
                let t = p as f64 / steps as f64;
                cmds.push(self.cmd(x1 + (x2 - x1) * t, y_offset, z1 + (z2 - z1) * t));
            }
        }
        cmds
    }

    /// A star shape with `arms` points, alternating outer and inner radii.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::star",
        aliases = ["sand::cmd::ParticleBuilder::star", "sand::prelude::ParticleBuilder::star", "sand::prelude::cmd::ParticleBuilder::star"],
        module = "sand::command",
        kind = "method",
        summary = "A star shape with `arms` points, alternating outer and inner radii.",
        context = "A star shape with `arms` points, alternating outer and inner radii. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(arms = "A star shape with `arms` points, alternating outer and inner radii.", outer_radius = "`outer_radius` supplies the outer radius value used to use a star shape with `arms` points, alternating outer and inner radii.", inner_radius = "`inner_radius` supplies the inner radius value used to use a star shape with `arms` points, alternating outer and inner radii.", y_offset = "`y_offset` supplies the y offset value used to use a star shape with `arms` points, alternating outer and inner radii."),
        returns = "The ordered values produced to use a star shape with `arms` points, alternating outer and inner radii.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, arms: usize, outer_radius: f64, inner_radius: f64, y_offset: f64)  {\n    let values = particle_builder_value.star(arms, outer_radius, inner_radius, y_offset);\n}",
    )]
    pub fn star(
        &self,
        arms: usize,
        outer_radius: f64,
        inner_radius: f64,
        y_offset: f64,
    ) -> Vec<String> {
        if arms < 2 {
            return vec![];
        }
        let total = arms * 2;
        (0..total)
            .map(|i| {
                let a = TAU * i as f64 / total as f64;
                let r = if i % 2 == 0 {
                    outer_radius
                } else {
                    inner_radius
                };
                self.cmd(r * a.cos(), y_offset, r * a.sin())
            })
            .collect()
    }

    /// A sphere surface using the Fibonacci lattice for even distribution.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::sphere",
        aliases = ["sand::cmd::ParticleBuilder::sphere", "sand::prelude::ParticleBuilder::sphere", "sand::prelude::cmd::ParticleBuilder::sphere"],
        module = "sand::command",
        kind = "method",
        summary = "A sphere surface using the Fibonacci lattice for even distribution.",
        context = "A sphere surface using the Fibonacci lattice for even distribution. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to use a sphere surface using the Fibonacci lattice for even distribution.", y_offset = "`y_offset` supplies the y offset value used to use a sphere surface using the Fibonacci lattice for even distribution.", points = "`points` supplies the points value used to use a sphere surface using the Fibonacci lattice for even distribution."),
        returns = "The ordered values produced to use a sphere surface using the Fibonacci lattice for even distribution.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, y_offset: f64, points: usize)  {\n    let values = particle_builder_value.sphere(radius, y_offset, points);\n}",
    )]
    pub fn sphere(&self, radius: f64, y_offset: f64, points: usize) -> Vec<String> {
        let gr = (1.0 + 5.0_f64.sqrt()) / 2.0;
        (0..points)
            .map(|i| {
                let theta = TAU * i as f64 / gr;
                let phi = ((1.0 - 2.0 * (i as f64 + 0.5) / points as f64).clamp(-1.0, 1.0)).acos();
                self.cmd(
                    radius * phi.sin() * theta.cos(),
                    radius * phi.cos() + y_offset,
                    radius * phi.sin() * theta.sin(),
                )
            })
            .collect()
    }

    /// The upper hemisphere only (y ≥ y_offset).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::hemisphere",
        aliases = ["sand::cmd::ParticleBuilder::hemisphere", "sand::prelude::ParticleBuilder::hemisphere", "sand::prelude::cmd::ParticleBuilder::hemisphere"],
        module = "sand::command",
        kind = "method",
        summary = "The upper hemisphere only (y ≥ y_offset).",
        context = "The upper hemisphere only (y ≥ y_offset). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to use the upper hemisphere only (y ≥ y_offset).", y_offset = "`y_offset` supplies the y offset value used to use the upper hemisphere only (y ≥ y_offset).", points = "`points` supplies the points value used to use the upper hemisphere only (y ≥ y_offset)."),
        returns = "The ordered values produced to use the upper hemisphere only (y ≥ y_offset).",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, y_offset: f64, points: usize)  {\n    let values = particle_builder_value.hemisphere(radius, y_offset, points);\n}",
    )]
    pub fn hemisphere(&self, radius: f64, y_offset: f64, points: usize) -> Vec<String> {
        self.sphere(radius, y_offset, points * 2)
            .into_iter()
            .filter(|cmd| {
                let y_val = extract_relative_y(cmd);
                y_val >= y_offset - 1e-9
            })
            .take(points)
            .collect()
    }

    /// A rising spiral helix.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::helix",
        aliases = ["sand::cmd::ParticleBuilder::helix", "sand::prelude::ParticleBuilder::helix", "sand::prelude::cmd::ParticleBuilder::helix"],
        module = "sand::command",
        kind = "method",
        summary = "A rising spiral helix.",
        context = "A rising spiral helix. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to use a rising spiral helix.", height = "`height` supplies the height value used to use a rising spiral helix.", turns = "`turns` supplies the turns value used to use a rising spiral helix.", points = "`points` supplies the points value used to use a rising spiral helix."),
        returns = "The ordered values produced to use a rising spiral helix.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, height: f64, turns: f64, points: usize)  {\n    let values = particle_builder_value.helix(radius, height, turns, points);\n}",
    )]
    pub fn helix(&self, radius: f64, height: f64, turns: f64, points: usize) -> Vec<String> {
        (0..points)
            .map(|i| {
                let t = i as f64 / (points as f64 - 1.0).max(1.0);
                let a = TAU * turns * t;
                self.cmd(radius * a.cos(), height * t, radius * a.sin())
            })
            .collect()
    }

    /// Two interleaved helices (double-helix / DNA shape).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::double_helix",
        aliases = ["sand::cmd::ParticleBuilder::double_helix", "sand::prelude::ParticleBuilder::double_helix", "sand::prelude::cmd::ParticleBuilder::double_helix"],
        module = "sand::command",
        kind = "method",
        summary = "Two interleaved helices (double-helix / DNA shape).",
        context = "Two interleaved helices (double-helix / DNA shape). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to use two interleaved helices (double-helix / DNA shape).", height = "`height` supplies the height value used to use two interleaved helices (double-helix / DNA shape).", turns = "`turns` supplies the turns value used to use two interleaved helices (double-helix / DNA shape).", points = "`points` supplies the points value used to use two interleaved helices (double-helix / DNA shape)."),
        returns = "The ordered values produced to use two interleaved helices (double-helix / DNA shape).",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, height: f64, turns: f64, points: usize)  {\n    let values = particle_builder_value.double_helix(radius, height, turns, points);\n}",
    )]
    pub fn double_helix(&self, radius: f64, height: f64, turns: f64, points: usize) -> Vec<String> {
        let half = points / 2;
        let mut cmds = self.helix(radius, height, turns, half);
        let rest = points - half;
        cmds.extend((0..rest).map(|i| {
            let t = i as f64 / (rest as f64 - 1.0).max(1.0);
            let a = TAU * turns * t + std::f64::consts::PI;
            self.cmd(radius * a.cos(), height * t, radius * a.sin())
        }));
        cmds
    }

    /// A straight line from `from` to `to` (both relative to executor).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::line",
        aliases = ["sand::cmd::ParticleBuilder::line", "sand::prelude::ParticleBuilder::line", "sand::prelude::cmd::ParticleBuilder::line"],
        module = "sand::command",
        kind = "method",
        summary = "A straight line from `from` to `to` (both relative to executor).",
        context = "A straight line from `from` to `to` (both relative to executor). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(from = "A straight line from `from` to `to` (both relative to executor).", to = "A straight line from `from` to `to` (both relative to executor).", points = "`points` supplies the points value used to use a straight line from `from` to `to` (both relative to executor)."),
        returns = "The ordered values produced to use a straight line from `from` to `to` (both relative to executor).",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, from: [f64 ; 3], to: [f64 ; 3], points: usize)  {\n    let values = particle_builder_value.line(from, to, points);\n}",
    )]
    pub fn line(&self, from: [f64; 3], to: [f64; 3], points: usize) -> Vec<String> {
        match points {
            0 => vec![],
            1 => vec![self.cmd(from[0], from[1], from[2])],
            _ => (0..points)
                .map(|i| {
                    let t = i as f64 / (points - 1) as f64;
                    self.cmd(
                        from[0] + (to[0] - from[0]) * t,
                        from[1] + (to[1] - from[1]) * t,
                        from[2] + (to[2] - from[2]) * t,
                    )
                })
                .collect(),
        }
    }

    /// A filled disc of concentric rings. `density` controls ring count per unit radius.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::disc",
        aliases = ["sand::cmd::ParticleBuilder::disc", "sand::prelude::ParticleBuilder::disc", "sand::prelude::cmd::ParticleBuilder::disc"],
        module = "sand::command",
        kind = "method",
        summary = "A filled disc of concentric rings. `density` controls ring count per unit radius.",
        context = "A filled disc of concentric rings. `density` controls ring count per unit radius. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to use a filled disc of concentric rings. `density` controls ring count per unit radius.", y_offset = "`y_offset` supplies the y offset value used to use a filled disc of concentric rings. `density` controls ring count per unit radius.", density = "A filled disc of concentric rings. `density` controls ring count per unit radius."),
        returns = "The ordered values produced to use a filled disc of concentric rings. `density` controls ring count per unit radius.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, y_offset: f64, density: usize)  {\n    let values = particle_builder_value.disc(radius, y_offset, density);\n}",
    )]
    pub fn disc(&self, radius: f64, y_offset: f64, density: usize) -> Vec<String> {
        let rings = (radius * density as f64).ceil() as usize;
        let mut cmds = vec![self.cmd(0.0, y_offset, 0.0)];
        for ring in 1..=rings {
            let r = radius * ring as f64 / rings as f64;
            let pts = ((TAU * r * density as f64).ceil() as usize).max(4);
            cmds.extend(self.circle(r, y_offset, pts));
        }
        cmds
    }

    /// A 3D torus (donut shape).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::torus",
        aliases = ["sand::cmd::ParticleBuilder::torus", "sand::prelude::ParticleBuilder::torus", "sand::prelude::cmd::ParticleBuilder::torus"],
        module = "sand::command",
        kind = "method",
        summary = "A 3D torus (donut shape).",
        context = "A 3D torus (donut shape). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(major_radius = "`major_radius` supplies the major radius value used to use a 3D torus (donut shape).", minor_radius = "`minor_radius` supplies the minor radius value used to use a 3D torus (donut shape).", y_offset = "`y_offset` supplies the y offset value used to use a 3D torus (donut shape).", rings = "`rings` supplies the rings value used to use a 3D torus (donut shape).", segments_per_ring = "`segments_per_ring` supplies the segments per ring value used to use a 3D torus (donut shape)."),
        returns = "The ordered values produced to use a 3D torus (donut shape).",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, major_radius: f64, minor_radius: f64, y_offset: f64, rings: usize, segments_per_ring: usize)  {\n    let values = particle_builder_value.torus(major_radius, minor_radius, y_offset, rings, segments_per_ring);\n}",
    )]
    pub fn torus(
        &self,
        major_radius: f64,
        minor_radius: f64,
        y_offset: f64,
        rings: usize,
        segments_per_ring: usize,
    ) -> Vec<String> {
        let mut cmds = Vec::new();
        for ring in 0..rings {
            let phi = TAU * ring as f64 / rings as f64;
            let cx = major_radius * phi.cos();
            let cz = major_radius * phi.sin();
            for seg in 0..segments_per_ring {
                let theta = TAU * seg as f64 / segments_per_ring as f64;
                let x = cx + minor_radius * theta.cos() * phi.cos();
                let y = minor_radius * theta.sin() + y_offset;
                let z = cz + minor_radius * theta.cos() * phi.sin();
                cmds.push(self.cmd(x, y, z));
            }
        }
        cmds
    }

    /// A cone rising from the base ring up to an apex.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::cone",
        aliases = ["sand::cmd::ParticleBuilder::cone", "sand::prelude::ParticleBuilder::cone", "sand::prelude::cmd::ParticleBuilder::cone"],
        module = "sand::command",
        kind = "method",
        summary = "A cone rising from the base ring up to an apex.",
        context = "A cone rising from the base ring up to an apex. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(base_radius = "`base_radius` supplies the base radius value used to use a cone rising from the base ring up to an apex.", height = "`height` supplies the height value used to use a cone rising from the base ring up to an apex.", y_offset = "`y_offset` supplies the y offset value used to use a cone rising from the base ring up to an apex.", rings = "`rings` supplies the rings value used to use a cone rising from the base ring up to an apex."),
        returns = "The ordered values produced to use a cone rising from the base ring up to an apex.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, base_radius: f64, height: f64, y_offset: f64, rings: usize)  {\n    let values = particle_builder_value.cone(base_radius, height, y_offset, rings);\n}",
    )]
    pub fn cone(&self, base_radius: f64, height: f64, y_offset: f64, rings: usize) -> Vec<String> {
        let mut cmds = Vec::new();
        let ring_count = rings.max(1);
        for ring in 0..=ring_count {
            let t = ring as f64 / ring_count as f64;
            let r = base_radius * (1.0 - t);
            let y = height * t + y_offset;
            let pts = ((TAU * r * 8.0).ceil() as usize).max(4);
            for i in 0..pts {
                let a = TAU * i as f64 / pts as f64;
                cmds.push(self.cmd(r * a.cos(), y, r * a.sin()));
            }
        }
        cmds
    }

    /// An outward burst (sphere with extra spread, simulates an explosion).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::burst",
        aliases = ["sand::cmd::ParticleBuilder::burst", "sand::prelude::ParticleBuilder::burst", "sand::prelude::cmd::ParticleBuilder::burst"],
        module = "sand::command",
        kind = "method",
        summary = "An outward burst (sphere with extra spread, simulates an explosion).",
        context = "An outward burst (sphere with extra spread, simulates an explosion). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the radius value used to use an outward burst (sphere with extra spread, simulates an explosion).", y_offset = "`y_offset` supplies the y offset value used to use an outward burst (sphere with extra spread, simulates an explosion).", points = "`points` supplies the points value used to use an outward burst (sphere with extra spread, simulates an explosion)."),
        returns = "The ordered values produced to use an outward burst (sphere with extra spread, simulates an explosion).",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, radius: f64, y_offset: f64, points: usize)  {\n    let values = particle_builder_value.burst(radius, y_offset, points);\n}",
    )]
    pub fn burst(&self, radius: f64, y_offset: f64, points: usize) -> Vec<String> {
        let boosted = Self {
            spread: ParticleSpread::new(
                self.spread.dx + radius * 0.15,
                self.spread.dy + radius * 0.15,
                self.spread.dz + radius * 0.15,
            ),
            ..self.clone()
        };
        boosted.sphere(radius, y_offset, points)
    }

    /// A horizontal sine wave along the X axis.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::wave",
        aliases = ["sand::cmd::ParticleBuilder::wave", "sand::prelude::ParticleBuilder::wave", "sand::prelude::cmd::ParticleBuilder::wave"],
        module = "sand::command",
        kind = "method",
        summary = "A horizontal sine wave along the X axis.",
        context = "A horizontal sine wave along the X axis. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(length = "`length` supplies the length value used to use a horizontal sine wave along the X axis.", amplitude = "`amplitude` supplies the amplitude value used to use a horizontal sine wave along the X axis.", cycles = "`cycles` supplies the cycles value used to use a horizontal sine wave along the X axis.", y_offset = "`y_offset` supplies the y offset value used to use a horizontal sine wave along the X axis.", points = "`points` supplies the points value used to use a horizontal sine wave along the X axis."),
        returns = "The ordered values produced to use a horizontal sine wave along the X axis.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, length: f64, amplitude: f64, cycles: f64, y_offset: f64, points: usize)  {\n    let values = particle_builder_value.wave(length, amplitude, cycles, y_offset, points);\n}",
    )]
    pub fn wave(
        &self,
        length: f64,
        amplitude: f64,
        cycles: f64,
        y_offset: f64,
        points: usize,
    ) -> Vec<String> {
        if points == 0 {
            return vec![];
        }
        (0..points)
            .map(|i| {
                let t = i as f64 / (points as f64 - 1.0).max(1.0);
                let x = length * t - length / 2.0;
                let y = amplitude * (TAU * cycles * t).sin() + y_offset;
                self.cmd(x, y, 0.0)
            })
            .collect()
    }

    /// A flat rectangular grid of particles.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::grid",
        aliases = ["sand::cmd::ParticleBuilder::grid", "sand::prelude::ParticleBuilder::grid", "sand::prelude::cmd::ParticleBuilder::grid"],
        module = "sand::command",
        kind = "method",
        summary = "A flat rectangular grid of particles.",
        context = "A flat rectangular grid of particles. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(width = "`width` supplies the width value used to use a flat rectangular grid of particles.", depth = "`depth` supplies the depth value used to use a flat rectangular grid of particles.", cols = "`cols` supplies the cols value used to use a flat rectangular grid of particles.", rows = "`rows` supplies the rows value used to use a flat rectangular grid of particles.", y_offset = "`y_offset` supplies the y offset value used to use a flat rectangular grid of particles."),
        returns = "The ordered values produced to use a flat rectangular grid of particles.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, width: f64, depth: f64, cols: usize, rows: usize, y_offset: f64)  {\n    let values = particle_builder_value.grid(width, depth, cols, rows, y_offset);\n}",
    )]
    pub fn grid(
        &self,
        width: f64,
        depth: f64,
        cols: usize,
        rows: usize,
        y_offset: f64,
    ) -> Vec<String> {
        let mut cmds = Vec::new();
        let cols = cols.max(1);
        let rows = rows.max(1);
        for row in 0..rows {
            for col in 0..cols {
                let x = if cols > 1 {
                    -width / 2.0 + width * col as f64 / (cols - 1) as f64
                } else {
                    0.0
                };
                let z = if rows > 1 {
                    -depth / 2.0 + depth * row as f64 / (rows - 1) as f64
                } else {
                    0.0
                };
                cmds.push(self.cmd(x, y_offset, z));
            }
        }
        cmds
    }

    /// Spawn a particle at each point in the given list (relative offsets from executor).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleBuilder::points_at",
        aliases = ["sand::cmd::ParticleBuilder::points_at", "sand::prelude::ParticleBuilder::points_at", "sand::prelude::cmd::ParticleBuilder::points_at"],
        module = "sand::command",
        kind = "method",
        summary = "Spawn a particle at each point in the given list (relative offsets from executor).",
        context = "Spawn a particle at each point in the given list (relative offsets from executor). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pts = "`pts` supplies the pts value used to spawn a particle at each point in the given list (relative offsets from executor)."),
        returns = "The ordered values produced to spawn a particle at each point in the given list (relative offsets from executor).",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle_builder_value: &sand::command::ParticleBuilder, pts: & [[f64 ; 3]])  {\n    let values = particle_builder_value.points_at(pts);\n}",
    )]
    pub fn points_at(&self, pts: &[[f64; 3]]) -> Vec<String> {
        pts.iter().map(|[x, y, z]| self.cmd(*x, *y, *z)).collect()
    }

    fn cmd(&self, x: f64, y: f64, z: f64) -> String {
        let command = self.command_at([x, y, z]);
        let line = command.render_unchecked(&CommandProfile::unprofiled());
        register_line(&line, command);
        line
    }

    fn command_at(&self, position: [f64; 3]) -> ParticleCommand {
        ParticleCommand {
            particle: self.particle.clone(),
            position,
            spread: self.spread.clone(),
            speed: self.speed,
            count: self.particles_per_point,
            force: self.force,
        }
    }
}

// ── ParticleEffect ────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ParticleEffect",
    aliases = ["sand::cmd::ParticleEffect", "sand::prelude::cmd::ParticleEffect"],
    module = "sand::command",
    summary = "Static particle geometry generators (thin wrappers around [`ParticleBuilder`]).",
    context = "Static particle geometry generators (thin wrappers around [`ParticleBuilder`]). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ParticleEffect;",
)]
/// Static particle geometry generators (thin wrappers around [`ParticleBuilder`]).
pub struct ParticleEffect;

impl ParticleEffect {
    /// Horizontal ring of particles.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleEffect::circle",
        aliases = ["sand::cmd::ParticleEffect::circle", "sand::prelude::cmd::ParticleEffect::circle"],
        module = "sand::command",
        kind = "method",
        summary = "Horizontal ring of particles.",
        context = "Horizontal ring of particles. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(particle = "`particle` supplies the particle value used to use horizontal ring of particles.", radius = "`radius` supplies the radius value used to use horizontal ring of particles.", y_offset = "`y_offset` supplies the y offset value used to use horizontal ring of particles.", count = "`count` provides the requested numeric amount used to use horizontal ring of particles.", spread = "`spread` supplies the spread value used to use horizontal ring of particles."),
        returns = "The ordered values produced to use horizontal ring of particles.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle: & str, radius: f64, y_offset: f64, count: usize, spread: & sand::command::ParticleSpread)  {\n    let values = sand::command::ParticleEffect::circle(particle, radius, y_offset, count, spread);\n}",
    )]
    pub fn circle(
        particle: &str,
        radius: f64,
        y_offset: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .circle(radius, y_offset, count)
    }

    /// Sphere surface (Fibonacci distribution).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleEffect::sphere",
        aliases = ["sand::cmd::ParticleEffect::sphere", "sand::prelude::cmd::ParticleEffect::sphere"],
        module = "sand::command",
        kind = "method",
        summary = "Sphere surface (Fibonacci distribution).",
        context = "Sphere surface (Fibonacci distribution). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(particle = "`particle` supplies the particle value used to sphere surface (Fibonacci distribution).", radius = "`radius` supplies the radius value used to sphere surface (Fibonacci distribution).", y_offset = "`y_offset` supplies the y offset value used to sphere surface (Fibonacci distribution).", count = "`count` provides the requested numeric amount used to sphere surface (Fibonacci distribution).", spread = "`spread` supplies the spread value used to sphere surface (Fibonacci distribution)."),
        returns = "The ordered values produced to sphere surface (Fibonacci distribution).",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle: & str, radius: f64, y_offset: f64, count: usize, spread: & sand::command::ParticleSpread)  {\n    let values = sand::command::ParticleEffect::sphere(particle, radius, y_offset, count, spread);\n}",
    )]
    pub fn sphere(
        particle: &str,
        radius: f64,
        y_offset: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .sphere(radius, y_offset, count)
    }

    /// Rising spiral helix.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleEffect::helix",
        aliases = ["sand::cmd::ParticleEffect::helix", "sand::prelude::cmd::ParticleEffect::helix"],
        module = "sand::command",
        kind = "method",
        summary = "Rising spiral helix.",
        context = "Rising spiral helix. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(particle = "`particle` supplies the particle value used to use rising spiral helix.", radius = "`radius` supplies the radius value used to use rising spiral helix.", height = "`height` supplies the height value used to use rising spiral helix.", turns = "`turns` supplies the turns value used to use rising spiral helix.", count = "`count` provides the requested numeric amount used to use rising spiral helix.", spread = "`spread` supplies the spread value used to use rising spiral helix."),
        returns = "The ordered values produced to use rising spiral helix.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle: & str, radius: f64, height: f64, turns: f64, count: usize, spread: & sand::command::ParticleSpread)  {\n    let values = sand::command::ParticleEffect::helix(particle, radius, height, turns, count, spread);\n}",
    )]
    pub fn helix(
        particle: &str,
        radius: f64,
        height: f64,
        turns: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .helix(radius, height, turns, count)
    }

    /// Straight line between two relative points.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleEffect::line",
        aliases = ["sand::cmd::ParticleEffect::line", "sand::prelude::cmd::ParticleEffect::line"],
        module = "sand::command",
        kind = "method",
        summary = "Straight line between two relative points.",
        context = "Straight line between two relative points. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(particle = "`particle` supplies the particle value used to use straight line between two relative points.", x1 = "`x1` supplies the x1 value used to use straight line between two relative points.", y1 = "`y1` supplies the y1 value used to use straight line between two relative points.", z1 = "`z1` supplies the z1 value used to use straight line between two relative points.", x2 = "`x2` supplies the x2 value used to use straight line between two relative points.", y2 = "`y2` supplies the y2 value used to use straight line between two relative points.", z2 = "`z2` supplies the z2 value used to use straight line between two relative points.", count = "`count` provides the requested numeric amount used to use straight line between two relative points.", spread = "`spread` supplies the spread value used to use straight line between two relative points."),
        returns = "The ordered values produced to use straight line between two relative points.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle: & str, x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64, count: usize, spread: & sand::command::ParticleSpread)  {\n    let values = sand::command::ParticleEffect::line(particle, x1, y1, z1, x2, y2, z2, count, spread);\n}",
    )]
    #[allow(clippy::too_many_arguments)]
    pub fn line(
        particle: &str,
        x1: f64,
        y1: f64,
        z1: f64,
        x2: f64,
        y2: f64,
        z2: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .line([x1, y1, z1], [x2, y2, z2], count)
    }

    /// Outward burst (sphere with boosted spread).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleEffect::burst",
        aliases = ["sand::cmd::ParticleEffect::burst", "sand::prelude::cmd::ParticleEffect::burst"],
        module = "sand::command",
        kind = "method",
        summary = "Outward burst (sphere with boosted spread).",
        context = "Outward burst (sphere with boosted spread). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(particle = "`particle` supplies the particle value used to use outward burst (sphere with boosted spread).", radius = "`radius` supplies the radius value used to use outward burst (sphere with boosted spread).", y_offset = "`y_offset` supplies the y offset value used to use outward burst (sphere with boosted spread).", count = "`count` provides the requested numeric amount used to use outward burst (sphere with boosted spread).", spread = "`spread` supplies the spread value used to use outward burst (sphere with boosted spread)."),
        returns = "The ordered values produced to use outward burst (sphere with boosted spread).",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle: & str, radius: f64, y_offset: f64, count: usize, spread: & sand::command::ParticleSpread)  {\n    let values = sand::command::ParticleEffect::burst(particle, radius, y_offset, count, spread);\n}",
    )]
    pub fn burst(
        particle: &str,
        radius: f64,
        y_offset: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .burst(radius, y_offset, count)
    }

    /// Two interleaved helices.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleEffect::double_helix",
        aliases = ["sand::cmd::ParticleEffect::double_helix", "sand::prelude::cmd::ParticleEffect::double_helix"],
        module = "sand::command",
        kind = "method",
        summary = "Two interleaved helices.",
        context = "Two interleaved helices. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(particle = "`particle` supplies the particle value used to use two interleaved helices.", radius = "`radius` supplies the radius value used to use two interleaved helices.", height = "`height` supplies the height value used to use two interleaved helices.", turns = "`turns` supplies the turns value used to use two interleaved helices.", count = "`count` provides the requested numeric amount used to use two interleaved helices.", spread = "`spread` supplies the spread value used to use two interleaved helices."),
        returns = "The ordered values produced to use two interleaved helices.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle: & str, radius: f64, height: f64, turns: f64, count: usize, spread: & sand::command::ParticleSpread)  {\n    let values = sand::command::ParticleEffect::double_helix(particle, radius, height, turns, count, spread);\n}",
    )]
    pub fn double_helix(
        particle: &str,
        radius: f64,
        height: f64,
        turns: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .double_helix(radius, height, turns, count)
    }

    /// Filled disc of concentric rings.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ParticleEffect::disc",
        aliases = ["sand::cmd::ParticleEffect::disc", "sand::prelude::cmd::ParticleEffect::disc"],
        module = "sand::command",
        kind = "method",
        summary = "Filled disc of concentric rings.",
        context = "Filled disc of concentric rings. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(particle = "`particle` supplies the particle value used to use filled disc of concentric rings.", radius = "`radius` supplies the radius value used to use filled disc of concentric rings.", y_offset = "`y_offset` supplies the y offset value used to use filled disc of concentric rings.", density = "`density` supplies the density value used to use filled disc of concentric rings.", spread = "`spread` supplies the spread value used to use filled disc of concentric rings."),
        returns = "The ordered values produced to use filled disc of concentric rings.",
        example = "use sand::prelude::*;\n\nfn demonstrate(particle: & str, radius: f64, y_offset: f64, density: usize, spread: & sand::command::ParticleSpread)  {\n    let values = sand::command::ParticleEffect::disc(particle, radius, y_offset, density, spread);\n}",
    )]
    pub fn disc(
        particle: &str,
        radius: f64,
        y_offset: f64,
        density: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .disc(radius, y_offset, density)
    }
}

// ── Private helpers ────────────────────────────────────────────────────────────

const TAU: f64 = std::f64::consts::TAU;

fn hex_to_f32(hex: u32) -> [f32; 3] {
    [
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
    ]
}

fn fmt_f(v: f64) -> String {
    let rounded = (v * 10000.0).round() / 10000.0;
    if rounded == rounded.trunc() && rounded.abs() < 1e15 {
        format!("{}", rounded as i64)
    } else {
        let s = format!("{:.4}", rounded);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

fn fmt_c(v: f32) -> String {
    let rounded = (v * 100.0).round() / 100.0;
    format!("{:.2}", rounded)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn extract_relative_y(cmd: &str) -> f64 {
    let mut tilde_count = 0;
    for token in cmd.split_whitespace() {
        if let Some(rest) = token.strip_prefix('~') {
            tilde_count += 1;
            if tilde_count == 2 {
                return rest.parse::<f64>().unwrap_or(0.0);
            }
        }
    }
    0.0
}

fn particle_error(
    code: &'static str,
    field: impl Into<String>,
    message: impl Into<String>,
) -> CommandError {
    CommandError::new("ParticleCommand", field, message).with_code(code)
}

fn validate_finite(value: f64, field: impl Into<String>) -> CommandResult<()> {
    let field = field.into();
    if value.is_finite() {
        Ok(())
    } else {
        Err(particle_error(
            "SAND-PARTICLE-NUMERIC",
            field,
            format!("must be finite, got `{value}`"),
        ))
    }
}

fn validate_non_negative(value: f64, field: impl Into<String>) -> CommandResult<()> {
    let field = field.into();
    validate_finite(value, field.clone())?;
    if value < 0.0 {
        Err(particle_error(
            "SAND-PARTICLE-NUMERIC",
            field,
            format!("must be non-negative, got `{value}`"),
        ))
    } else {
        Ok(())
    }
}

fn validate_geometry_non_negative(value: f64, field: &'static str) -> CommandResult<()> {
    validate_non_negative(value, field)
        .map_err(|error| particle_error("SAND-PARTICLE-GEOMETRY", error.field, error.message))
}

fn validate_points(points: usize) -> CommandResult<()> {
    if points == 0 {
        Err(particle_error(
            "SAND-PARTICLE-GEOMETRY",
            "geometry.points",
            "point count must be greater than zero",
        ))
    } else {
        Ok(())
    }
}

fn validate_color(value: f32, field: &'static str) -> CommandResult<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        Err(particle_error(
            "SAND-PARTICLE-COLOR",
            field,
            format!("RGB channels must be finite and within 0.0..=1.0, got `{value}`"),
        ))
    } else {
        Ok(())
    }
}

fn validate_scale(scale: f32) -> CommandResult<()> {
    if !scale.is_finite() || scale <= 0.0 {
        Err(particle_error(
            "SAND-PARTICLE-SCALE",
            "particle.scale",
            format!("dust scale must be finite and greater than zero, got `{scale}`"),
        ))
    } else {
        Ok(())
    }
}

fn validate_particle_payload_id(value: &str, field: &'static str) -> CommandResult<()> {
    let id = value
        .split_once(['[', '{', ' '])
        .map_or(value, |(id, _)| id);
    crate::validate::resource_location_shape(id, "ParticleCommand", field)
        .map(|_| ())
        .map_err(|error| particle_error("SAND-PARTICLE-ID", field, error.message))
}

/// Export-scoped registry family holding this module's rendered
/// `particle` command lines and their originating typed nodes.
///
/// State lives in [`crate::export_registry`]'s active layer, so it is
/// per-thread, scoped to whichever [`crate::export_registry::ExportRegistryGuard`]
/// is open, and discarded when that guard drops — including on an early
/// `Err` return or an unwind. There is no process-global map and no
/// per-family reset to remember to call.
pub(crate) struct ParticleLines;

impl crate::export_registry::RegistryFamily for ParticleLines {
    type State = BTreeMap<String, ParticleCommand>;
}

fn register_line(line: &str, command: ParticleCommand) {
    crate::export_registry::register_line::<ParticleLines, _>(line, command);
}

pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    crate::export_registry::validate_registered_line::<ParticleLines, _>(
        line,
        profile,
        |command, profile| command.validate(profile),
    )
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn builder(name: &str) -> ParticleBuilder {
        ParticleBuilder::new(Particle::named(name))
    }

    #[test]
    fn circle_count() {
        let cmds = builder("minecraft:flame").circle(2.0, 0.0, 8);
        assert_eq!(cmds.len(), 8);
        assert!(cmds[0].starts_with("particle minecraft:flame"));
        assert!(
            cmds[0].contains("~2 ~0 ~0"),
            "first point at radius: {}",
            cmds[0]
        );
    }

    #[test]
    fn arc_partial() {
        let cmds = builder("minecraft:flame").arc(2.0, 0.0, 0.0, 90.0, 4);
        assert_eq!(cmds.len(), 4);
        assert!(cmds[0].contains("~2 ~0 ~0"), "arc start: {}", cmds[0]);
        assert!(cmds[3].contains("~0 ~0 ~2"), "arc end: {}", cmds[3]);
    }

    #[test]
    fn polygon_sides() {
        let cmds = builder("minecraft:crit").polygon(4, 2.0, 0.0, 4);
        assert_eq!(cmds.len(), 16);
    }

    #[test]
    fn star_arms() {
        let cmds = builder("minecraft:end_rod").star(5, 2.0, 0.8, 0.0);
        assert_eq!(cmds.len(), 10);
    }

    #[test]
    fn sphere_count() {
        let cmds = builder("minecraft:end_rod").sphere(3.0, 0.0, 20);
        assert_eq!(cmds.len(), 20);
    }

    #[test]
    fn helix_count() {
        let cmds = builder("minecraft:soul_fire_flame").helix(1.0, 5.0, 2.0, 30);
        assert_eq!(cmds.len(), 30);
    }

    #[test]
    fn double_helix_count() {
        let cmds = builder("minecraft:flame").double_helix(1.0, 4.0, 2.0, 40);
        assert_eq!(cmds.len(), 40);
    }

    #[test]
    fn line_endpoints() {
        let cmds = builder("minecraft:crit").line([0.0, 0.0, 0.0], [3.0, 0.0, 0.0], 4);
        assert_eq!(cmds.len(), 4);
        assert!(cmds[0].contains("~0 ~0 ~0"), "start: {}", cmds[0]);
        assert!(cmds[3].contains("~3 ~0 ~0"), "end: {}", cmds[3]);
    }

    #[test]
    fn torus_count() {
        let cmds = builder("minecraft:end_rod").torus(3.0, 0.8, 0.0, 16, 8);
        assert_eq!(cmds.len(), 16 * 8);
    }

    #[test]
    fn cone_apex_at_top() {
        let cmds = builder("minecraft:flame").cone(2.0, 3.0, 0.0, 4);
        let apex = cmds.last().unwrap();
        assert!(apex.contains("~0 ~3 ~0"), "apex: {apex}");
    }

    #[test]
    fn wave_count() {
        let cmds = builder("minecraft:witch").wave(8.0, 1.0, 2.0, 0.0, 32);
        assert_eq!(cmds.len(), 32);
    }

    #[test]
    fn grid_count() {
        let cmds = builder("minecraft:flame").grid(4.0, 4.0, 5, 5, 0.0);
        assert_eq!(cmds.len(), 25);
    }

    #[test]
    fn points_at() {
        let pts = [[0.0_f64, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let cmds = builder("minecraft:flash").points_at(&pts);
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn dust_colored_format() {
        let cmds = ParticleBuilder::new(Particle::dust(1.0, 0.0, 0.0, 1.0)).circle(1.0, 0.0, 4);
        assert!(
            cmds[0].starts_with("particle minecraft:dust{color:[1,0,0],scale:1}"),
            "{}",
            cmds[0]
        );
    }

    #[test]
    fn dust_hex_red() {
        let p = Particle::dust_hex(0xFF0000, 1.0);
        if let Particle::Dust { r, g, b, .. } = p {
            assert!((r - 1.0).abs() < 0.01);
            assert!(g < 0.01);
            assert!(b < 0.01);
        } else {
            panic!("not dust");
        }
    }

    #[test]
    fn dust_transition_format() {
        let cmds = ParticleBuilder::new(Particle::dust_transition_hex(0xFF0000, 0x0000FF, 1.0))
            .points_at(&[[0.0, 0.0, 0.0]]);
        assert!(
            cmds[0].starts_with("particle minecraft:dust_color_transition"),
            "{}",
            cmds[0]
        );
    }

    #[test]
    fn force_normal_mode() {
        let force_cmd = builder("minecraft:flame")
            .force(true)
            .points_at(&[[0.0, 0.0, 0.0]]);
        let normal_cmd = builder("minecraft:flame")
            .force(false)
            .points_at(&[[0.0, 0.0, 0.0]]);
        assert!(force_cmd[0].ends_with("force"), "{}", force_cmd[0]);
        assert!(normal_cmd[0].ends_with("normal"), "{}", normal_cmd[0]);
    }

    #[test]
    fn fmt_f_precision() {
        assert_eq!(fmt_f(0.0), "0");
        assert_eq!(fmt_f(1.0), "1");
        assert_eq!(fmt_f(1.5), "1.5");
        assert_eq!(fmt_f(1.2345678), "1.2346");
    }

    #[test]
    fn particle_validation_covers_ids_numbers_and_geometry() {
        let profile = CommandProfile::unprofiled();
        assert!(
            Particle::named("modded:custom_particle")
                .validate(&profile)
                .is_ok()
        );
        assert_eq!(
            Particle::named("Bad Particle")
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-PARTICLE-ID"
        );
        assert_eq!(
            Particle::dust(f32::NAN, 0.0, 0.0, 1.0)
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-PARTICLE-COLOR"
        );
        assert_eq!(
            Particle::dust(1.0, 0.0, 0.0, 0.0)
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-PARTICLE-SCALE"
        );
        let builder = ParticleBuilder::new(Particle::named("minecraft:flame"));
        assert!(builder.try_circle(-1.0, 0.0, 4).is_err());
        assert!(builder.try_helix(1.0, 2.0, 0.0, 4).is_err());
        assert!(builder.try_grid(1.0, 1.0, 0, 2, 0.0).is_err());
        assert!(builder.try_points_at(&[]).is_err());
    }

    #[test]
    fn particle_raw_token_is_opaque() {
        assert!(
            Particle::raw_token("modded payload")
                .validate(&CommandProfile::unprofiled())
                .is_ok()
        );
    }
}
