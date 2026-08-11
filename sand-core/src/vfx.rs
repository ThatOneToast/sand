//! Reusable typed visual/audio effects.
//!
//! A [`Vfx`] groups particle, sound, and raw command steps so datapack authors
//! can define an effect once and emit deterministic command lists wherever it
//! is needed.
//!
//! ```rust,ignore
//! use sand::prelude::*;
//!
//! fn level_up_vfx() -> Vfx {
//!     Vfx::new("level_up")
//!         .particle(
//!             VfxParticle::named("minecraft:happy_villager")
//!                 .count(20)
//!                 .spread(0.6, 1.0, 0.6),
//!         )
//!         .sound(
//!             VfxSound::new("minecraft:entity.player.levelup")
//!                 .source(SoundSource::Player)
//!                 .volume(1.0)
//!                 .pitch(1.2),
//!         )
//! }
//!
//! #[function]
//! pub fn level_up() {
//!     for cmd in level_up_vfx().play_at(Selector::self_()) {
//!         cmd;
//!     }
//! }
//! ```

use crate::cmd::{
    Build, CommandProfile, Execute, Particle, ParticleBuilder, ParticleSpread, RawCommand,
    RenderCommand, Selector, Sound, SoundSource, Validate, Vec3,
};
use sand_commands::{CommandResult, IntoParticleId, IntoSoundEvent};
use sand_macros::api;

/// A reusable group of visual/audio commands.
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::vfx::Vfx",
    aliases = ["sand::cmd::Vfx", "sand::command::Vfx", "sand::prelude::Vfx", "sand::prelude::cmd::Vfx"],
    module = "sand::vfx",
    summary = "Sequences reusable particle, sound, and explicit raw-command presentation steps.",
    context = "An effect carries an ordered author-level recipe that can be played at a target, for an audience, or at a position.",
    minecraft = "Each playback helper emits particle, playsound, and execute commands in the declared order.",
    use_when = ["Reusing a presentation sequence across functions", "Keeping visual and audio commands ordered together"],
    avoid_when = ["Defining a datapack JSON resource", "Encoding gameplay state or conditions"],
    example = "let effect = Vfx::new(\"level_up\").particle(VfxParticle::happy_villager());"
)]
pub struct Vfx {
    name: String,
    steps: Vec<VfxStep>,
}

impl Vfx {
    /// Create a new named VFX asset.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::Vfx::new",
        summary = "Starts a reusable VFX sequence with a diagnostic label.",
        context = "The label identifies validation failures without becoming a Minecraft resource identifier.",
        minecraft = "The label is not emitted; later steps determine the Minecraft commands.",
        use_when = ["Beginning a named presentation sequence"],
        avoid_when = ["Declaring a function or other namespaced datapack resource"],
        params(name = "A readable label included in VFX validation diagnostics."),
        returns = "An empty effect ready to receive ordered steps.",
        example = "let effect = Vfx::new(\"level_up\");"
    )]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
        }
    }

    /// Stable author-facing name for this effect.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::Vfx::name",
        summary = "Returns the effect's diagnostic label.",
        context = "The label makes an invalid VFX sequence identifiable in an export error.",
        minecraft = "This label does not appear in generated Minecraft commands.",
        use_when = ["Reporting or inspecting a VFX validation error"],
        avoid_when = ["Choosing a Minecraft function or resource path"],
        returns = "The label supplied when the effect was created.",
        example = "assert_eq!(Vfx::new(\"level_up\").name(), \"level_up\");"
    )]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add a particle step.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::Vfx::particle",
        summary = "Appends a typed particle step to this effect.",
        context = "A concrete VfxParticle keeps particle geometry and visibility configuration explicit at the effect boundary.",
        minecraft = "Emits one or more particle commands when the effect is played.",
        use_when = ["Adding a visual particle cue"],
        avoid_when = ["Playing a sound or inserting an advanced raw command"],
        params(particle = "The fully configured particle presentation step."),
        returns = "This effect with the particle step appended after existing steps.",
        example = "let effect = Vfx::new(\"spark\").particle(VfxParticle::happy_villager());"
    )]
    pub fn particle(mut self, particle: VfxParticle) -> Self {
        self.steps.push(VfxStep::Particle(particle));
        self
    }

    /// Add a sound step.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::Vfx::sound",
        summary = "Appends a typed playsound step to this effect.",
        context = "A concrete VfxSound keeps audience, source, position, and numeric sound settings together.",
        minecraft = "Emits a playsound command when the effect is played.",
        use_when = ["Adding an audible cue to an effect"],
        avoid_when = ["Adding particle geometry or an advanced raw command"],
        params(sound = "The fully configured playsound presentation step."),
        returns = "This effect with the sound step appended after existing steps.",
        example = "let effect = Vfx::new(\"ding\").sound(VfxSound::new(\"minecraft:block.note_block.bell\"));"
    )]
    pub fn sound(mut self, sound: VfxSound) -> Self {
        self.steps.push(VfxStep::Sound(sound));
        self
    }

    /// Add an explicit raw command escape hatch.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::Vfx::command",
        summary = "Appends one explicitly raw Minecraft command to this effect.",
        context = "RawCommand makes an advanced escape hatch visible in the type signature rather than accepting an arbitrary string.",
        minecraft = "Emits the supplied one-line command in sequence with the other VFX commands.",
        use_when = ["Integrating a validated command syntax Sand does not model yet"],
        avoid_when = ["A typed particle, sound, state, or command API is available"],
        params(command = "The explicit raw command to validate and emit."),
        returns = "This effect with the raw command appended after existing steps.",
        example = "let effect = Vfx::new(\"notice\").command(RawCommand::new(\"say ready\"));"
    )]
    pub fn command(mut self, command: RawCommand) -> Self {
        self.steps.push(VfxStep::Command(command));
        self
    }

    /// Number of steps in the effect.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::Vfx::len",
        summary = "Counts the ordered presentation steps in this effect.",
        context = "Step count helps inspect a composed effect without rendering command strings.",
        minecraft = "Each step may emit one or more Minecraft commands during playback.",
        use_when = ["Inspecting a composed effect"],
        avoid_when = ["Determining the number of commands after particle geometry expands"],
        returns = "The number of declared VFX steps.",
        example = "assert_eq!(Vfx::new(\"empty\").len(), 0);"
    )]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Whether this effect has no steps.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::Vfx::is_empty",
        summary = "Checks whether this effect has no presentation steps.",
        context = "An empty effect is valid but emits no commands.",
        minecraft = "No particle, playsound, or raw command is emitted for an empty effect.",
        use_when = ["Skipping optional presentation work"],
        avoid_when = ["Checking whether a nonempty effect will pass step validation"],
        returns = "True when no steps have been appended.",
        example = "assert!(Vfx::new(\"empty\").is_empty());"
    )]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Render commands at the current execution position.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::Vfx::play",
        summary = "Renders this effect at the current execution context.",
        context = "This convenience path preserves step order without adding an execute wrapper.",
        minecraft = "Particles use the current position and sounds use their configured audience or Minecraft's default self target.",
        use_when = ["Emitting an effect from the current function context"],
        avoid_when = ["Input must be validated before rendering; use try_play"],
        returns = "The ordered Minecraft command lines emitted by the effect.",
        example = "let commands = Vfx::new(\"spark\").particle(VfxParticle::happy_villager()).play();"
    )]
    pub fn play(&self) -> Vec<String> {
        self.steps.iter().flat_map(VfxStep::render).collect()
    }

    /// Validate every typed particle/sound step, then render deterministically.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::Vfx::try_play",
        summary = "Validates every step before rendering the effect at the current context.",
        context = "Validation reports the effect label so a bad particle, sound, or raw command can be located before export.",
        minecraft = "Rejects invalid command arguments rather than emitting malformed particle or playsound lines.",
        use_when = ["Validating author input before export or tests"],
        avoid_when = ["Rendering a previously validated trusted effect on a hot path"],
        returns = "Ordered command lines or a contextual command validation error.",
        example = "let commands = Vfx::new(\"spark\").try_play()?;"
    )]
    pub fn try_play(&self) -> CommandResult<Vec<String>> {
        self.steps
            .iter()
            .map(VfxStep::try_render)
            .collect::<CommandResult<Vec<_>>>()
            .map(|groups| groups.into_iter().flatten().collect())
            .map_err(|error| error.with_context(format!("VFX `{}`", self.name)))
    }

    /// Render commands at an entity/player selector.
    ///
    /// Particle and raw command steps are wrapped with `execute at <target>`.
    /// Sound steps are also targeted to the same selector.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::Vfx::play_at",
        summary = "Renders this effect at each selected entity's position.",
        context = "The explicit Selector prevents implicit target conversion and keeps positional execution distinct from sound audience selection.",
        minecraft = "Wraps particle and raw steps in execute at; sounds preserve their configured audience or use the executing entity.",
        use_when = ["Playing an effect where matching entities are located"],
        avoid_when = ["Broadcasting a sound to an audience without changing position; use play_for"],
        params(target = "The selector whose execution positions receive the effect."),
        returns = "Ordered command lines wrapped for the selected positions.",
        example = "let commands = Vfx::new(\"spark\").play_at(Selector::self_());"
    )]
    pub fn play_at(&self, target: Selector) -> Vec<String> {
        self.steps
            .iter()
            .flat_map(|step| step.render_at(&target))
            .collect()
    }

    /// Render commands for a player audience at the current execution position.
    ///
    /// Particle and raw command steps are emitted unchanged. Sound steps target
    /// the supplied audience.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::Vfx::play_for",
        summary = "Renders this effect for a selected sound audience at the current position.",
        context = "Audience selection affects sound recipients without implicitly changing where particle or raw steps execute.",
        minecraft = "Sound steps target the selector; particle and raw steps remain at the current execution context.",
        use_when = ["Broadcasting the audio portion of an effect to selected players"],
        avoid_when = ["Moving the effect to each selected entity; use play_at"],
        params(audience = "The selector that receives sound steps."),
        returns = "Ordered command lines for the requested audience.",
        example = "let commands = Vfx::new(\"ding\").play_for(Selector::all_players());"
    )]
    pub fn play_for(&self, audience: Selector) -> Vec<String> {
        self.steps
            .iter()
            .flat_map(|step| step.render_for(&audience))
            .collect()
    }

    /// Render commands at a specific execution position.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::Vfx::play_positioned",
        summary = "Renders this effect at one explicit Minecraft position.",
        context = "An explicit Vec3 makes fixed-coordinate presentation independent of the current executor.",
        minecraft = "Wraps particle and raw steps with execute positioned and supplies the position to sound steps.",
        use_when = ["Playing an effect at a calculated or fixed location"],
        avoid_when = ["Following every entity selected by a selector; use play_at"],
        params(position = "The Minecraft position at which to render the effect."),
        returns = "Ordered command lines positioned at the requested location.",
        example = "let commands = Vfx::new(\"spark\").play_positioned(Vec3::absolute(0.0, 64.0, 0.0));"
    )]
    pub fn play_positioned(&self, position: Vec3) -> Vec<String> {
        self.steps
            .iter()
            .flat_map(|step| step.render_positioned(&position))
            .collect()
    }
}

/// A single step in a [`Vfx`] asset.
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::vfx::VfxStep",
    aliases = ["sand::cmd::VfxStep", "sand::command::VfxStep", "sand::prelude::VfxStep", "sand::prelude::cmd::VfxStep"],
    module = "sand::vfx",
    summary = "Represents one ordered particle, sound, or explicit raw-command effect step.",
    context = "Vfx owns the sequence, while this enum keeps each presentation medium explicit for composition and inspection.",
    minecraft = "Each variant contributes particle, playsound, or a validated raw command during playback.",
    use_when = ["Inspecting or constructing an explicit VFX sequence"],
    avoid_when = ["Representing gameplay state, conditions, or a datapack JSON component"],
    example = "let step = VfxStep::Particle(VfxParticle::happy_villager());",
    variants(
        Particle = "Emits the configured particle geometry.",
        Sound = "Emits the configured playsound command.",
        Command = "Emits one explicit RawCommand escape hatch."
    ),
    variant_fields(
        Particle = ["The configured typed particle presentation."],
        Sound = ["The configured typed playsound presentation."],
        Command = ["The explicit validated raw command to emit."]
    )
)]
pub enum VfxStep {
    /// Particle commands rendered via [`ParticleBuilder`].
    Particle(VfxParticle),
    /// Sound command rendered via [`Sound`].
    Sound(VfxSound),
    /// Explicit raw command escape hatch emitted in sequence.
    Command(RawCommand),
}

impl VfxStep {
    fn render(&self) -> Vec<String> {
        match self {
            Self::Particle(step) => step.render(),
            Self::Sound(step) => vec![step.render(None, None)],
            Self::Command(command) => vec![command.to_string()],
        }
    }

    fn try_render(&self) -> CommandResult<Vec<String>> {
        match self {
            Self::Particle(step) => step.try_render(),
            Self::Sound(step) => step.try_render(None, None).map(|line| vec![line]),
            Self::Command(command) => command.try_build().map(|line| vec![line]),
        }
    }

    fn render_at(&self, target: &Selector) -> Vec<String> {
        match self {
            Self::Particle(step) => step
                .render()
                .into_iter()
                .map(|cmd| Execute::new().at(target.clone()).run(cmd))
                .collect(),
            Self::Sound(step) => {
                // Sound audience must NOT be the positional `target`.
                // Using `target` as the audience inside `execute at <target>`
                // would fork the sound once per matched entity and replay it
                // to the entire audience on every fork — e.g.
                //   execute at @a run playsound … @a
                // Instead, use the sound's own configured audience or `@s`
                // (the entity currently executing), which is always safe.
                let cmd = step.render_with_own_audience(Some(Vec3::here()));
                vec![Execute::new().at(target.clone()).run(cmd)]
            }
            Self::Command(command) => {
                vec![Execute::new().at(target.clone()).run(command.to_string())]
            }
        }
    }

    fn render_for(&self, audience: &Selector) -> Vec<String> {
        match self {
            Self::Particle(step) => step.render(),
            Self::Sound(step) => vec![step.render(Some(audience), None)],
            Self::Command(command) => vec![command.to_string()],
        }
    }

    fn render_positioned(&self, position: &Vec3) -> Vec<String> {
        match self {
            Self::Particle(step) => step
                .render()
                .into_iter()
                .map(|cmd| Execute::new().positioned(position.clone()).run(cmd))
                .collect(),
            Self::Sound(step) => vec![step.render(None, Some(position.clone()))],
            Self::Command(command) => {
                vec![
                    Execute::new()
                        .positioned(position.clone())
                        .run(command.to_string()),
                ]
            }
        }
    }
}

/// A reusable particle step rendered by [`ParticleBuilder`].
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::vfx::VfxParticle",
    aliases = ["sand::cmd::VfxParticle", "sand::command::VfxParticle", "sand::prelude::VfxParticle", "sand::prelude::cmd::VfxParticle"],
    module = "sand::vfx",
    summary = "Configures a reusable Minecraft particle presentation step.",
    context = "The builder keeps a typed particle, geometry, speed, count, and visibility together before an effect renders it.",
    minecraft = "Builds one or more particle commands with exact offsets and Minecraft visibility mode.",
    use_when = ["Adding a configurable particle cue to Vfx", "Reusing particle geometry across effects"],
    avoid_when = ["Playing a sound or passing an untyped raw command"],
    example = "let step = VfxParticle::happy_villager().count(20).spread_uniform(0.5);"
)]
pub struct VfxParticle {
    particle: Particle,
    spread: ParticleSpread,
    speed: f64,
    count: u32,
    visibility: VfxParticleVisibility,
    points: Vec<[f64; 3]>,
}

impl VfxParticle {
    /// Create a particle step for a concrete particle value.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxParticle::new",
        summary = "Starts a particle step from Sand's typed Particle value.",
        context = "Accepting Particle preserves its configured Minecraft particle syntax instead of parsing another string at the VFX boundary.",
        minecraft = "Uses the supplied particle as the base of emitted particle commands.",
        use_when = ["A Particle value is already available or needs advanced particle data"],
        avoid_when = ["Selecting a named particle identifier directly; use named"],
        params(particle = "The typed particle value to emit."),
        returns = "A particle step with one forced particle at the current position by default.",
        example = "let step = VfxParticle::new(Particle::named(\"minecraft:crit\"));"
    )]
    pub fn new(particle: Particle) -> Self {
        Self {
            particle,
            spread: ParticleSpread::POINT,
            speed: 0.0,
            count: 1,
            visibility: VfxParticleVisibility::Force,
            points: vec![[0.0, 0.0, 0.0]],
        }
    }

    /// Create a named particle step.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxParticle::named",
        summary = "Starts a particle step from a Minecraft particle identifier.",
        context = "This concise boundary delegates identifier conversion to Sand's shared particle-ID support.",
        minecraft = "Emits the requested particle identifier in the particle command.",
        use_when = ["Using a named vanilla or supported custom particle"],
        avoid_when = ["Constructing a particle with structured particle data; use new"],
        params(name = "The particle identifier accepted by Sand's particle command API."),
        returns = "A default particle step for that identifier.",
        example = "let step = VfxParticle::named(\"minecraft:crit\");"
    )]
    pub fn named(name: impl IntoParticleId) -> Self {
        Self::new(Particle::named(name))
    }

    /// Convenience constructor for `minecraft:happy_villager`.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxParticle::happy_villager",
        summary = "Starts a particle step for Minecraft's happy-villager particle.",
        context = "The common positive-feedback particle deserves a readable authoring shorthand.",
        minecraft = "Emits minecraft:happy_villager particles.",
        use_when = ["Showing a positive reward, success, or villager-style cue"],
        avoid_when = ["Selecting another particle effect; use named or new"],
        returns = "A default happy-villager particle step.",
        example = "let step = VfxParticle::happy_villager().count(12);"
    )]
    pub fn happy_villager() -> Self {
        Self::named("minecraft:happy_villager")
    }

    /// Set the random spread box around each point.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxParticle::spread",
        summary = "Sets independent random particle spread distances on each axis.",
        context = "A nonuniform spread matches effects such as tall bursts or flat clouds without a raw particle command.",
        minecraft = "Writes the particle command's delta X, Y, and Z values.",
        use_when = ["Shaping a rectangular particle distribution"],
        avoid_when = ["The same spread is wanted on all axes; use spread_uniform"],
        params(dx = "Horizontal X spread distance.", dy = "Vertical Y spread distance.", dz = "Horizontal Z spread distance."),
        returns = "This particle step with the requested spread box.",
        example = "let step = VfxParticle::happy_villager().spread(0.5, 1.0, 0.5);"
    )]
    pub fn spread(mut self, dx: f64, dy: f64, dz: f64) -> Self {
        self.spread = ParticleSpread::new(dx, dy, dz);
        self
    }

    /// Set a uniform random spread around each point.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxParticle::spread_uniform",
        summary = "Sets the same random particle spread on all axes.",
        context = "Uniform spread is the compact form for a symmetric cloud around every point.",
        minecraft = "Writes the same delta value for X, Y, and Z in the particle command.",
        use_when = ["Creating a symmetric particle cloud"],
        avoid_when = ["Vertical and horizontal spread need different values; use spread"],
        params(value = "The shared spread distance for all three axes."),
        returns = "This particle step with uniform spread.",
        example = "let step = VfxParticle::happy_villager().spread_uniform(0.5);"
    )]
    pub fn spread_uniform(mut self, value: f64) -> Self {
        self.spread = ParticleSpread::uniform(value);
        self
    }

    /// Set initial particle speed.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxParticle::speed",
        summary = "Sets the initial speed used by the particle command.",
        context = "Speed is explicit because Minecraft interprets it together with the selected particle and spread.",
        minecraft = "Writes the particle command's speed argument.",
        use_when = ["A particle effect needs initial motion"],
        avoid_when = ["Keeping Minecraft's stationary default speed"],
        params(speed = "The initial particle speed."),
        returns = "This particle step with the requested speed.",
        example = "let step = VfxParticle::named(\"minecraft:crit\").speed(0.1);"
    )]
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    /// Set the number of particles spawned per point.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxParticle::count",
        summary = "Sets how many particles Minecraft spawns at each configured point.",
        context = "Count belongs to the step so every point in a multi-offset effect is configured consistently.",
        minecraft = "Writes the particle command's count argument for each emitted point.",
        use_when = ["Controlling the density of a visual cue"],
        avoid_when = ["Changing the number of distinct offsets; use offsets"],
        params(count = "The particle count to spawn at each point."),
        returns = "This particle step with the requested count.",
        example = "let step = VfxParticle::happy_villager().count(20);"
    )]
    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    /// Set whether Minecraft sends this particle step with normal or forced visibility.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxParticle::visibility",
        summary = "Selects Minecraft's normal or forced particle visibility mode.",
        context = "A named mode replaces the ambiguous force boolean and makes the rendering tradeoff explicit in author code.",
        minecraft = "Selects the normal or force mode token in the particle command.",
        use_when = ["Choosing whether particles should bypass normal visibility distance limits"],
        avoid_when = ["Changing the particle count, speed, or geometry"],
        params(visibility = "The named Minecraft particle visibility mode."),
        returns = "This particle step with the requested visibility mode.",
        example = "let step = VfxParticle::happy_villager().visibility(VfxParticleVisibility::Normal);"
    )]
    pub fn visibility(mut self, visibility: VfxParticleVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Spawn at one relative offset.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxParticle::offset",
        summary = "Sets one relative particle offset from the playback position.",
        context = "A single offset is the compact geometry form for a focused particle cue.",
        minecraft = "Writes the offset as the particle command's position relative to the current context.",
        use_when = ["Placing one particle point away from the effect origin"],
        avoid_when = ["Emitting several ordered points; use offsets"],
        params(x = "Relative X offset.", y = "Relative Y offset.", z = "Relative Z offset."),
        returns = "This particle step with one configured offset.",
        example = "let step = VfxParticle::named(\"minecraft:crit\").offset(0.0, 1.0, 0.0);"
    )]
    pub fn offset(mut self, x: f64, y: f64, z: f64) -> Self {
        self.points = vec![[x, y, z]];
        self
    }

    /// Spawn at multiple relative offsets in deterministic order.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxParticle::offsets",
        summary = "Sets several relative particle offsets in emission order.",
        context = "A finite iterator provides reusable geometry while retaining deterministic command ordering.",
        minecraft = "Emits one particle command per offset in iterator order.",
        use_when = ["Drawing a small authored particle pattern"],
        avoid_when = ["A single point is sufficient; use offset"],
        params(points = "Relative X, Y, Z offsets to emit in order."),
        returns = "This particle step with the supplied point sequence.",
        example = "let step = VfxParticle::named(\"minecraft:end_rod\").offsets([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);"
    )]
    pub fn offsets(mut self, points: impl IntoIterator<Item = [f64; 3]>) -> Self {
        self.points = points.into_iter().collect();
        self
    }

    /// Render this particle step to command strings.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxParticle::render",
        summary = "Renders this particle step into ordered Minecraft command lines.",
        context = "Rendering delegates to ParticleBuilder so VFX stays aligned with Sand's lower-level particle grammar.",
        minecraft = "Produces one particle command for every configured offset.",
        use_when = ["Embedding a particle step outside a full Vfx sequence"],
        avoid_when = ["Input must be checked first; use try_render"],
        returns = "The particle command lines in configured point order.",
        example = "let commands = VfxParticle::happy_villager().render();"
    )]
    pub fn render(&self) -> Vec<String> {
        ParticleBuilder::new(self.particle.clone())
            .spread(self.spread.clone())
            .speed(self.speed)
            .particles_per_point(self.count)
            .force(self.visibility.is_force())
            .points_at(&self.points)
    }

    /// Validate the underlying particle command and every point before rendering.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxParticle::try_render",
        summary = "Validates this particle step before rendering command lines.",
        context = "The fallible path catches malformed particle identifiers, numeric values, and empty geometry at the authoring boundary.",
        minecraft = "Prevents invalid particle command arguments from reaching datapack export.",
        use_when = ["Checking a particle step built from configuration or dynamic values"],
        avoid_when = ["Rendering a previously validated trusted step on a hot path"],
        returns = "The particle command lines or a command validation error.",
        example = "let commands = VfxParticle::happy_villager().try_render()?;"
    )]
    pub fn try_render(&self) -> CommandResult<Vec<String>> {
        ParticleBuilder::new(self.particle.clone())
            .spread(self.spread.clone())
            .speed(self.speed)
            .particles_per_point(self.count)
            .force(self.visibility.is_force())
            .try_points_at(&self.points)
    }
}

impl Validate for VfxParticle {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        self.try_render().map(|_| ())
    }
}

/// Controls the visibility mode of particles emitted by a [`VfxParticle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::vfx::VfxParticleVisibility",
    aliases = ["sand::cmd::VfxParticleVisibility", "sand::command::VfxParticleVisibility", "sand::prelude::VfxParticleVisibility", "sand::prelude::cmd::VfxParticleVisibility"],
    module = "sand::vfx",
    summary = "Selects Minecraft's normal or forced particle visibility mode.",
    context = "A named enum makes the distance-visibility choice self-explanatory where a boolean would be ambiguous.",
    minecraft = "Selects the normal or force token in Minecraft's particle command.",
    use_when = ["Configuring a VfxParticle visibility policy"],
    avoid_when = ["Choosing sound audience or particle geometry"],
    example = "let visibility = VfxParticleVisibility::Normal;",
    variants(
        Force = "Forces particles to use Minecraft's force visibility mode.",
        Normal = "Uses Minecraft's normal distance-limited particle visibility mode."
    )
)]
pub enum VfxParticleVisibility {
    /// Send particles with Minecraft's forced visibility mode.
    Force,
    /// Use Minecraft's normal distance-limited visibility mode.
    Normal,
}

impl VfxParticleVisibility {
    fn is_force(self) -> bool {
        matches!(self, Self::Force)
    }
}

/// A reusable sound step rendered by [`Sound`].
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::vfx::VfxSound",
    aliases = ["sand::cmd::VfxSound", "sand::command::VfxSound", "sand::prelude::VfxSound", "sand::prelude::cmd::VfxSound"],
    module = "sand::vfx",
    summary = "Configures a reusable Minecraft playsound presentation step.",
    context = "The builder keeps event, source, audience, position, and numeric sound controls together before an effect renders it.",
    minecraft = "Builds a playsound command using Minecraft's sound-event, source, audience, position, volume, pitch, and minimum-volume arguments.",
    use_when = ["Adding a configurable sound cue to Vfx", "Reusing a playsound setup across effects"],
    avoid_when = ["Creating particle geometry or an untyped raw command"],
    example = "let step = VfxSound::new(\"minecraft:entity.player.levelup\").source(SoundSource::Player);"
)]
pub struct VfxSound {
    event: String,
    source: SoundSource,
    audience: Option<Selector>,
    position: Option<Vec3>,
    volume: f64,
    pitch: f64,
    min_volume: Option<f64>,
}

impl VfxSound {
    /// Begin building a reusable `playsound` step.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxSound::new",
        summary = "Starts a reusable playsound step from a sound event.",
        context = "The event is converted through Sand's shared sound-event input support before the VFX builder adds playback settings.",
        minecraft = "Uses the event as the first argument of the emitted playsound command.",
        use_when = ["Adding a sound event to an effect"],
        avoid_when = ["Adding particles or raw commands to an effect"],
        params(event = "The sound event accepted by Sand's playsound command API."),
        returns = "A default master-channel sound step with volume and pitch of one.",
        example = "let step = VfxSound::new(\"minecraft:block.note_block.bell\");"
    )]
    pub fn new(event: impl IntoSoundEvent) -> Self {
        Self {
            event: event.into_sound_event(),
            source: SoundSource::Master,
            audience: None,
            position: None,
            volume: 1.0,
            pitch: 1.0,
            min_volume: None,
        }
    }

    /// Set a default audience for this sound step.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxSound::to",
        summary = "Sets the selector that receives this sound by default.",
        context = "The explicit Selector separates sound recipients from the positional selector used by Vfx::play_at.",
        minecraft = "Writes the playsound audience argument.",
        use_when = ["A sound should be heard by a known selector"],
        avoid_when = ["Moving the particle or raw-command execution position"],
        params(audience = "The selector that should receive the sound."),
        returns = "This sound step with the requested default audience.",
        example = "let step = VfxSound::new(\"minecraft:block.note_block.bell\").to(Selector::all_players());"
    )]
    pub fn to(mut self, audience: Selector) -> Self {
        self.audience = Some(audience);
        self
    }

    /// Set the sound source/channel category.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxSound::source",
        summary = "Sets Minecraft's sound source category for this step.",
        context = "The source category controls the player's client-side volume channel for an otherwise identical event.",
        minecraft = "Writes the source argument of the playsound command.",
        use_when = ["Selecting a player, ambient, music, or other Minecraft sound channel"],
        avoid_when = ["Selecting the sound event itself"],
        params(source = "The Minecraft sound source category."),
        returns = "This sound step with the requested source category.",
        example = "let step = VfxSound::new(\"minecraft:entity.player.levelup\").source(SoundSource::Player);"
    )]
    pub fn source(mut self, source: SoundSource) -> Self {
        self.source = source;
        self
    }

    /// Set a default sound origin.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxSound::at",
        summary = "Sets the default Minecraft position from which this sound originates.",
        context = "A stored Vec3 lets the same sound step retain a deliberate origin across effect playback sites.",
        minecraft = "Writes the optional position arguments of the playsound command.",
        use_when = ["The sound should originate at a fixed or calculated position"],
        avoid_when = ["Vfx::play_positioned should supply the effect-wide position instead"],
        params(position = "The Minecraft position used as the sound origin."),
        returns = "This sound step with the requested default origin.",
        example = "let step = VfxSound::new(\"minecraft:block.note_block.bell\").at(Vec3::absolute(0.0, 64.0, 0.0));"
    )]
    pub fn at(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }

    /// Set the volume multiplier.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxSound::volume",
        summary = "Sets the playsound volume multiplier.",
        context = "Volume remains on the sound step so an effect can reuse an event with a deliberate loudness.",
        minecraft = "Writes the playsound volume argument.",
        use_when = ["Adjusting the loudness of a sound cue"],
        avoid_when = ["Changing how far a quiet sound can still be heard; use min_volume"],
        params(volume = "The Minecraft playsound volume multiplier."),
        returns = "This sound step with the requested volume.",
        example = "let step = VfxSound::new(\"minecraft:block.note_block.bell\").volume(0.5);"
    )]
    pub fn volume(mut self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    /// Set the pitch multiplier.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxSound::pitch",
        summary = "Sets the playsound pitch multiplier.",
        context = "Pitch is configured with the sound step to preserve a recognizable cue across effects.",
        minecraft = "Writes the playsound pitch argument.",
        use_when = ["Raising or lowering a sound's playback pitch"],
        avoid_when = ["Changing the selected sound event"],
        params(pitch = "The Minecraft playsound pitch multiplier."),
        returns = "This sound step with the requested pitch.",
        example = "let step = VfxSound::new(\"minecraft:block.note_block.bell\").pitch(1.2);"
    )]
    pub fn pitch(mut self, pitch: f64) -> Self {
        self.pitch = pitch;
        self
    }

    /// Set minimum volume for players far from the sound origin.
    #[api(
        registry = sand_api_contract,
        path = "sand::vfx::VfxSound::min_volume",
        summary = "Sets the minimum volume heard far from this sound's origin.",
        context = "This is distinct from ordinary volume: it controls Minecraft's distant-audience floor.",
        minecraft = "Writes the optional minimum-volume argument of the playsound command.",
        use_when = ["Keeping a distant cue audible at a controlled minimum"],
        avoid_when = ["Changing ordinary loudness; use volume"],
        params(min = "The minimum volume for distant listeners."),
        returns = "This sound step with the requested distant-volume floor.",
        example = "let step = VfxSound::new(\"minecraft:block.note_block.bell\").min_volume(0.2);"
    )]
    pub fn min_volume(mut self, min: f64) -> Self {
        self.min_volume = Some(min);
        self
    }

    fn render(&self, audience: Option<&Selector>, position: Option<Vec3>) -> String {
        self.sound(audience, position).build()
    }

    fn try_render(
        &self,
        audience: Option<&Selector>,
        position: Option<Vec3>,
    ) -> CommandResult<String> {
        let sound = self.sound(audience, position);
        sound.try_build()
    }

    fn sound(&self, audience: Option<&Selector>, position: Option<Vec3>) -> Sound {
        let mut sound = Sound::play(self.event.clone())
            .source(self.source)
            .volume(self.volume)
            .pitch(self.pitch);

        if let Some(audience) = audience.cloned().or_else(|| self.audience.clone()) {
            sound = sound.to(audience);
        }

        if let Some(position) = position.or_else(|| self.position.clone()) {
            sound = sound.at(position);
        }

        if let Some(min_volume) = self.min_volume {
            sound = sound.min_volume(min_volume);
        }

        sound
    }

    /// Render using the sound's own configured audience (falling back to `@s`),
    /// never the positional selector passed to `play_at`.
    ///
    /// This prevents `play_at(@a)` from producing
    /// `execute at @a run playsound ... @a`, which would multiply the sound
    /// once per entity fork.
    fn render_with_own_audience(&self, position: Option<Vec3>) -> String {
        let audience = self.audience.clone().unwrap_or_else(Selector::self_);
        self.render(Some(&audience), position)
    }
}

impl Validate for VfxSound {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.sound(None, None)
            .validate(profile)
            .map_err(|error| error.with_context("VfxSound"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_vfx_produces_no_commands() {
        let vfx = Vfx::new("empty");
        assert!(vfx.is_empty());
        assert_eq!(vfx.play(), Vec::<String>::new());
    }

    #[test]
    fn vfx_uses_particle_and_sound_validation() {
        let particle_error = Vfx::new("bad_particle")
            .particle(VfxParticle::named("Bad Particle"))
            .try_play()
            .unwrap_err();
        assert_eq!(particle_error.code, "SAND-PARTICLE-ID");
        assert!(particle_error.to_string().contains("bad_particle"));

        let sound_error = Vfx::new("bad_sound")
            .sound(VfxSound::new("minecraft:test").pitch(0.0))
            .try_play()
            .unwrap_err();
        assert_eq!(sound_error.code, "SAND-SOUND-NUMERIC");

        assert!(
            Vfx::new("empty_geometry")
                .particle(VfxParticle::happy_villager().offsets([]))
                .try_play()
                .is_err()
        );
    }

    #[test]
    fn typed_raw_command_boundary_validates_before_effect_rendering() {
        let error = Vfx::new("bad_raw")
            .command(RawCommand::new("say first\nsay second"))
            .try_play()
            .unwrap_err();
        assert_eq!(error.code, "SAND-RAW-COMMAND-LINE");
        assert!(error.to_string().contains("VFX `bad_raw`"));
    }

    #[test]
    fn single_particle_uses_particle_builder_output() {
        let commands = Vfx::new("spark")
            .particle(
                VfxParticle::named("minecraft:happy_villager")
                    .count(20)
                    .spread(0.6, 1.0, 0.6),
            )
            .play();

        assert_eq!(
            commands,
            vec!["particle minecraft:happy_villager ~0 ~0 ~0 0.6 1 0.6 0 20 force"]
        );
    }

    #[test]
    fn named_particle_visibility_replaces_the_ambiguous_boolean() {
        let commands = VfxParticle::named("minecraft:crit")
            .visibility(VfxParticleVisibility::Normal)
            .render();
        assert_eq!(
            commands,
            vec!["particle minecraft:crit ~0 ~0 ~0 0 0 0 0 1 normal"]
        );
    }

    #[test]
    fn single_sound_uses_sound_builder_output() {
        let commands = Vfx::new("ding")
            .sound(
                VfxSound::new("minecraft:entity.player.levelup")
                    .source(SoundSource::Player)
                    .volume(1.0)
                    .pitch(1.2),
            )
            .play_for(Selector::self_());

        assert_eq!(
            commands,
            vec!["playsound minecraft:entity.player.levelup player @s ~ ~ ~ 1 1.2"]
        );
    }

    #[test]
    fn combined_steps_preserve_order() {
        let commands = Vfx::new("combo")
            .particle(VfxParticle::named("minecraft:crit"))
            .sound(VfxSound::new("minecraft:block.note_block.bell"))
            .command(RawCommand::new("say done"))
            .play_for(Selector::all_players());

        assert_eq!(
            commands,
            vec![
                "particle minecraft:crit ~0 ~0 ~0 0 0 0 0 1 force",
                "playsound minecraft:block.note_block.bell master @a ~ ~ ~ 1 1",
                "say done",
            ]
        );
    }

    #[test]
    fn play_at_targets_expected_selector() {
        let commands = Vfx::new("self")
            .particle(VfxParticle::named("minecraft:crit"))
            .sound(VfxSound::new("minecraft:block.note_block.bell"))
            .play_at(Selector::self_());

        assert_eq!(
            commands,
            vec![
                "execute at @s run particle minecraft:crit ~0 ~0 ~0 0 0 0 0 1 force",
                "execute at @s run playsound minecraft:block.note_block.bell master @s ~ ~ ~ 1 1",
            ]
        );
    }

    #[test]
    fn positioned_playback_wraps_positioned_commands() {
        let commands = Vfx::new("pos")
            .particle(VfxParticle::named("minecraft:crit"))
            .sound(VfxSound::new("minecraft:block.note_block.bell"))
            .play_positioned(Vec3::absolute(1.0, 2.0, 3.0));

        assert_eq!(
            commands,
            vec![
                "execute positioned 1 2 3 run particle minecraft:crit ~0 ~0 ~0 0 0 0 0 1 force",
                "playsound minecraft:block.note_block.bell master @s 1 2 3 1 1",
            ]
        );
    }

    #[test]
    fn command_output_is_deterministic() {
        let vfx = Vfx::new("deterministic")
            .particle(
                VfxParticle::named("minecraft:end_rod").offsets([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]),
            )
            .sound(VfxSound::new("minecraft:block.note_block.bell"));

        assert_eq!(
            vfx.play_at(Selector::self_()),
            vfx.play_at(Selector::self_())
        );
    }

    // -----------------------------------------------------------------
    // Regression tests for the multi-target sound-duplication bug.
    //
    // `play_at(target)` must NEVER reuse the positional selector as the
    // playsound audience.  With a multi-player selector such as `@a`,
    // Minecraft forks the execute chain once per matched entity; if the
    // playsound audience were also `@a`, every player would hear the
    // sound N times (once per fork).
    // -----------------------------------------------------------------

    #[test]
    fn play_at_all_players_does_not_reuse_selector_as_sound_audience() {
        let commands = Vfx::new("level_up")
            .sound(VfxSound::new("minecraft:entity.player.levelup"))
            .play_at(Selector::all_players());

        // The playsound audience (third argument to playsound) must NOT be @a.
        // playsound syntax: playsound <sound> <source> <audience> [x y z ...]
        // We check that no command has "playsound" with @a as the audience token
        // (the third whitespace-delimited word after "playsound").
        assert!(
            !commands.iter().any(|cmd| {
                // Find the playsound substring and inspect its audience token.
                if let Some(ps_pos) = cmd.find("playsound ") {
                    let after_ps = &cmd[ps_pos + "playsound ".len()..];
                    // tokens: <sound> <source> <audience> ...
                    let mut tokens = after_ps.split_whitespace();
                    tokens.next(); // skip sound event
                    tokens.next(); // skip source
                    tokens.next() == Some("@a")
                } else {
                    false
                }
            }),
            "play_at must not reuse positional selector as sound audience: {commands:?}"
        );
    }

    #[test]
    fn play_at_all_players_sound_audience_is_self() {
        // When no explicit audience is configured on the VfxSound, play_at
        // must fall back to @s (the entity currently executing), not @a.
        let commands = Vfx::new("level_up")
            .sound(VfxSound::new("minecraft:entity.player.levelup"))
            .play_at(Selector::all_players());

        assert_eq!(
            commands,
            vec!["execute at @a run playsound minecraft:entity.player.levelup master @s ~ ~ ~ 1 1"]
        );
    }

    #[test]
    fn play_at_self_particle_behavior_unchanged() {
        // play_at("@s") must still emit the expected positional particle
        // command — the fix must not regress the common single-entity case.
        let commands = Vfx::new("spark")
            .particle(VfxParticle::named("minecraft:happy_villager"))
            .play_at(Selector::self_());

        assert_eq!(
            commands,
            vec!["execute at @s run particle minecraft:happy_villager ~0 ~0 ~0 0 0 0 0 1 force"]
        );
    }

    #[test]
    fn sound_audience_independent_from_positional_selector() {
        // Particle uses the positional target; sound audience is separate.
        let commands = Vfx::new("effect")
            .particle(VfxParticle::named("minecraft:crit"))
            .sound(VfxSound::new("minecraft:block.note_block.bell"))
            .play_at(Selector::all_players());

        assert_eq!(
            commands,
            vec![
                "execute at @a run particle minecraft:crit ~0 ~0 ~0 0 0 0 0 1 force",
                "execute at @a run playsound minecraft:block.note_block.bell master @s ~ ~ ~ 1 1",
            ]
        );
    }

    #[test]
    fn explicit_sound_audience_is_preserved_through_play_at() {
        // If VfxSound has an explicit audience set via `.to(...)`, that
        // audience must survive through `play_at`, ignoring the positional
        // selector entirely.
        let commands = Vfx::new("broadcast")
            .sound(
                VfxSound::new("minecraft:ui.toast.challenge_complete").to(Selector::all_players()),
            )
            .play_at(Selector::self_());

        assert_eq!(
            commands,
            vec![
                "execute at @s run playsound minecraft:ui.toast.challenge_complete master @a ~ ~ ~ 1 1"
            ]
        );
    }

    #[test]
    fn play_at_combined_particle_and_sound_preserves_order() {
        // Combined effect: particle first, then sound, deterministic order.
        let commands = Vfx::new("combo_at")
            .particle(VfxParticle::named("minecraft:end_rod"))
            .sound(VfxSound::new("minecraft:block.note_block.bell"))
            .command(RawCommand::new("say vfx"))
            .play_at(Selector::self_());

        assert_eq!(
            commands,
            vec![
                "execute at @s run particle minecraft:end_rod ~0 ~0 ~0 0 0 0 0 1 force",
                "execute at @s run playsound minecraft:block.note_block.bell master @s ~ ~ ~ 1 1",
                "execute at @s run say vfx",
            ]
        );
    }

    #[test]
    fn play_for_all_players_targets_expected_audience() {
        // play_for must still target the supplied audience — regression guard.
        let commands = Vfx::new("announce")
            .sound(VfxSound::new("minecraft:entity.player.levelup"))
            .play_for(Selector::all_players());

        assert_eq!(
            commands,
            vec!["playsound minecraft:entity.player.levelup master @a ~ ~ ~ 1 1"]
        );
    }
}
