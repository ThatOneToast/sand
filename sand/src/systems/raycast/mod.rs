//! Synchronous block sampling composed from predicate resources and functions.
//!
//! A cast samples forward along the current command rotation, preserving its
//! executor and dimension. Register the definition with `#[datapack_component]`
//! and call its `cast()` action from a function. This is discrete sampling, not
//! voxel traversal: thin blocks between samples can be skipped.

use crate::component::{IntoCommands, McFunction};
use crate::prelude::{
    BlockPredicate, FunctionId, FunctionRef, LocationPredicate, Predicate, PredicateId,
    PredicateRoot, ResourceLocation,
};
use crate::registration::{
    ComponentContent, DatapackComponent, DatapackRegistration, IntoDatapack, LifecycleContribution,
};
use sand_commands::{
    Anchor, BlockPos, CommandError, CommandResult, Coord, ObjectiveName, ScoreHolder, Selector,
    Vec3,
};
use sand_core::ir::{Actions, Cmd, ConditionIr, ExecuteOp, ScorePlayersOp};
use sand_core::serde_json::Value;

const SAMPLE_LIMIT: usize = 256;

#[derive(Clone)]
enum Search {
    Scan(BlockPredicate),
    Collision(Option<BlockPredicate>),
}

/// A reusable, registered block raycast definition.
///
/// Samples exclude the origin and include the exact maximum-distance endpoint:
/// `step, 2*step, …, range`, with the last interval shortened when needed.
/// At most 256 samples are allowed. This bounds traversal work, not arbitrary
/// recursion in user callbacks. No entity executor means a no-op.
///
/// Hit/miss callbacks keep the caster, rotation and dimension and run at the
/// terminating sample. Player-context callbacks are invoked only for players;
/// a non-player still terminates there, without an additional miss callback.
/// Unloaded or out-of-world samples terminate as a miss. No chunks are loaded by a cast.
/// The miss callback is optional: without it, a miss clears state and returns.
///
/// ```no_run
/// use sand::command::Anchor;
/// use sand::prelude::*;
/// use sand::systems::raycast::BlockRaycast;
///
/// #[datapack_component]
/// fn mining_ray() -> BlockRaycast {
///     BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap())
///         .origin(Anchor::Eyes)
///         .scan_for(BlockPredicate::new().tag(TagId::minecraft("logs").unwrap()))
///         .max_distance(5.0)
///         .step(0.25)
///         .on_hit(found_log)
/// }
/// #[function]
/// fn found_log() {
///     WorldContext::current().block(BlockPos::here()).replace(vanilla::Block::Air);
/// }
/// #[function]
/// fn use_tool() {
///     mining_ray().cast();
/// }
/// ```
#[sand_macros::api(
    path = "sand::systems::raycast::BlockRaycast", module = "sand::systems::raycast",
    summary = "Defines a bounded synchronous block raycast with registered callbacks.",
    context = "Register the definition once and reuse cast actions without naming generated functions or counters.",
    minecraft = "Samples along current rotation with one entity-owned counter per definition; terminal returns preserve nested callback completion.",
    use_when = ["Scanning through nonmatching blocks", "Stopping at the first sampled nonpassable block"],
    avoid_when = ["Exact voxel traversal or entity targeting is needed"],
    example = r#"use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
#[function]
fn found_log() { cmd::say("Found a matching block"); }
#[datapack_component]
fn mining_ray() -> BlockRaycast {
    BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap())
        .scan_for(BlockPredicate::new().tag(TagId::minecraft("logs").unwrap()))
        .on_hit(found_log)
}
"#
)]
#[derive(Clone)]
pub struct BlockRaycast {
    id: ResourceLocation,
    origin: Anchor,
    distance: f64,
    step: f64,
    search: Option<Search>,
    hit: Option<FunctionId>,
    miss: Option<FunctionId>,
}

impl BlockRaycast {
    /// Define a raycast identity; defaults to eyes, range 6, step 0.25.
    /// Set a search mode and an on_hit callback before registering it.
    #[sand_macros::api(
        path = "sand::systems::raycast::BlockRaycast::new", module = "sand::systems::raycast", kind = "method",
        summary = "Creates a reusable block raycast definition.",
        context = "Definitions retain validated identities until registration and command export; casts run only in Minecraft.",
        minecraft = "Traversal preserves executor, rotation and dimension; callbacks run at the terminating sample with canonical context guards.",
        use_when = ["Composing a reusable synchronous block cast"],
        avoid_when = ["Exact voxel traversal or unbounded callback recursion is required"],
        params(id = "The typed logical resource identity; local namespace resolves per export."),
        returns = "An inert definition with eyes, range six, and quarter-block steps.",
        example = r#"use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
let ray = BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap());"#
    )]
    pub fn new(id: ResourceLocation) -> Self {
        Self {
            id,
            origin: Anchor::Eyes,
            distance: 6.0,
            step: 0.25,
            search: None,
            hit: None,
            miss: None,
        }
    }

    /// Choose eyes or feet relative to the current execution position.
    /// Eyes are applied once at entry; local steps preserve the sampled height.
    /// Rotation always follows the current command rotation (normally caster view).
    #[sand_macros::api(
        path = "sand::systems::raycast::BlockRaycast::origin", module = "sand::systems::raycast", kind = "method",
        summary = "Chooses the anchor applied once at cast entry.",
        context = "Definitions retain validated identities until registration and command export; casts run only in Minecraft.",
        minecraft = "Traversal preserves executor, rotation and dimension; callbacks run at the terminating sample with canonical context guards.",
        use_when = ["Composing a reusable synchronous block cast"],
        avoid_when = ["Exact voxel traversal or unbounded callback recursion is required"],
        params(origin = "Eyes or feet relative to the current execution position."),
        returns = "The updated raycast definition.",
        example = r#"use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
let ray = BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap());
let ray = ray.origin(sand::command::Anchor::Eyes);"#
    )]
    pub fn origin(mut self, origin: Anchor) -> Self {
        self.origin = origin;
        self
    }

    /// Find the first matching sampled block, passing through every nonmatch.
    /// This intentionally permits finding targets behind walls.
    #[sand_macros::api(
        path = "sand::systems::raycast::BlockRaycast::scan_for", module = "sand::systems::raycast", kind = "method",
        summary = "Selects scanning through nonmatching sampled blocks.",
        context = "Definitions retain validated identities until registration and command export; casts run only in Minecraft.",
        minecraft = "Traversal preserves executor, rotation and dimension; callbacks run at the terminating sample with canonical context guards.",
        use_when = ["Composing a reusable synchronous block cast"],
        avoid_when = ["Exact voxel traversal or unbounded callback recursion is required"],
        params(block = "Canonical block predicate for a hit, including typed tags and states."),
        returns = "The raycast configured to pass through every nonmatch.",
        example = r#"use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
let ray = BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap());
let ray = ray.scan_for(BlockPredicate::new().tag(TagId::minecraft("logs").unwrap()));"#
    )]
    pub fn scan_for(mut self, block: BlockPredicate) -> Self {
        self.search = Some(Search::Scan(block));
        self
    }

    /// Stop at the first sampled block that does not match `pass_through`.
    /// Set the passable criterion explicitly; this method clears a prior mode.
    #[sand_macros::api(
        path = "sand::systems::raycast::BlockRaycast::first_collision", module = "sand::systems::raycast", kind = "method",
        summary = "Selects stopping at the first nonpassable loaded sample.",
        context = "Definitions retain validated identities until registration and command export; casts run only in Minecraft.",
        minecraft = "Traversal preserves executor, rotation and dimension; callbacks run at the terminating sample with canonical context guards.",
        use_when = ["Composing a reusable synchronous block cast"],
        avoid_when = ["Exact voxel traversal or unbounded callback recursion is required"],
        returns = "The raycast awaiting an explicit pass_through criterion.",
        example = r#"use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
let ray = BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap());
let passable = BlockPredicate::new().blocks(vec![BlockId::minecraft("air").unwrap()]);
let ray = ray.first_collision().pass_through(passable);"#
    )]
    pub fn first_collision(mut self) -> Self {
        self.search = Some(Search::Collision(None));
        self
    }

    /// Set which blocks first-collision mode may pass through.
    /// Calling this selects first-collision mode; block tags use canonical BlockPredicate.
    #[sand_macros::api(
        path = "sand::systems::raycast::BlockRaycast::pass_through", module = "sand::systems::raycast", kind = "method",
        summary = "Defines the passable blocks in first-collision mode.",
        context = "Definitions retain validated identities until registration and command export; casts run only in Minecraft.",
        minecraft = "Traversal preserves executor, rotation and dimension; callbacks run at the terminating sample with canonical context guards.",
        use_when = ["Composing a reusable synchronous block cast"],
        avoid_when = ["Exact voxel traversal or unbounded callback recursion is required"],
        params(block = "Canonical predicate describing blocks the cast may pass through."),
        returns = "The raycast configured for first collision.",
        example = r#"use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
let ray = BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap());
let ray = ray.pass_through(BlockPredicate::new().blocks(vec![BlockId::minecraft("air").unwrap()]));"#
    )]
    pub fn pass_through(mut self, block: BlockPredicate) -> Self {
        self.search = Some(Search::Collision(Some(block)));
        self
    }

    /// Set the finite, positive maximum distance in blocks.
    #[sand_macros::api(
        path = "sand::systems::raycast::BlockRaycast::max_distance", module = "sand::systems::raycast", kind = "method",
        summary = "Sets the exact endpoint distance in blocks.",
        context = "Definitions retain validated identities until registration and command export; casts run only in Minecraft.",
        minecraft = "Traversal preserves executor, rotation and dimension; callbacks run at the terminating sample with canonical context guards.",
        use_when = ["Composing a reusable synchronous block cast"],
        avoid_when = ["Exact voxel traversal or unbounded callback recursion is required"],
        params(distance = "A finite positive range; export rejects invalid values."),
        returns = "The updated definition.",
        example = r#"use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
let ray = BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap());
let ray = ray.max_distance(5.0);"#
    )]
    pub fn max_distance(mut self, distance: f64) -> Self {
        self.distance = distance;
        self
    }

    /// Set the finite, positive sampling interval in blocks.
    #[sand_macros::api(
        path = "sand::systems::raycast::BlockRaycast::step", module = "sand::systems::raycast", kind = "method",
        summary = "Sets spacing between samples before the clamped endpoint.",
        context = "Definitions retain validated identities until registration and command export; casts run only in Minecraft.",
        minecraft = "Traversal preserves executor, rotation and dimension; callbacks run at the terminating sample with canonical context guards.",
        use_when = ["Composing a reusable synchronous block cast"],
        avoid_when = ["Exact voxel traversal or unbounded callback recursion is required"],
        params(step = "A finite positive interval; at most 256 samples may be needed."),
        returns = "The updated definition.",
        example = r#"use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
let ray = BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap());
let ray = ray.step(0.25);"#
    )]
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// Choose the callback for a matching sample (or first collision).
    /// Only registered functions are accepted at export, including explicit IDs.
    /// Player-context callbacks are skipped for nonplayers; the hit still terminates.
    #[sand_macros::api(
        path = "sand::systems::raycast::BlockRaycast::on_hit", module = "sand::systems::raycast", kind = "method",
        summary = "Sets the registered callback invoked at a hit sample.",
        context = "Definitions retain validated identities until registration and command export; casts run only in Minecraft.",
        minecraft = "Traversal preserves executor, rotation and dimension; callbacks run at the terminating sample with canonical context guards.",
        use_when = ["Composing a reusable synchronous block cast"],
        avoid_when = ["Exact voxel traversal or unbounded callback recursion is required"],
        params(callback = "A FunctionRef resolving to a registered function. Player callbacks are skipped for nonplayers without causing a miss."),
        returns = "The definition with its hit callback.",
        example = r#"use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
#[function]
fn found_log() { cmd::say("Found a matching block"); }
let ray = BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap());
let ray = ray.on_hit(found_log);"#
    )]
    pub fn on_hit(mut self, callback: impl FunctionRef) -> Self {
        self.hit = Some(callback.function_id());
        self
    }

    /// Choose the optional callback for range exhaustion or an unloaded sample.
    /// Without this callback, a miss clears the counter and returns silently.
    /// Player-context callbacks are skipped for nonplayers; traversal still terminates.
    #[sand_macros::api(
        path = "sand::systems::raycast::BlockRaycast::on_miss", module = "sand::systems::raycast", kind = "method",
        summary = "Sets the registered callback for exhaustion or unloaded samples.",
        context = "Definitions retain validated identities until registration and command export; casts run only in Minecraft.",
        minecraft = "Traversal preserves executor, rotation and dimension; callbacks run at the terminating sample with canonical context guards.",
        use_when = ["Composing a reusable synchronous block cast"],
        avoid_when = ["Exact voxel traversal or unbounded callback recursion is required"],
        params(callback = "A FunctionRef resolving to a registered function. Player callbacks are skipped for nonplayers."),
        returns = "The definition with its miss callback.",
        example = r#"use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
#[function]
fn no_target() { cmd::say("No matching block"); }
let ray = BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap());
let ray = ray.on_miss(no_target);"#
    )]
    pub fn on_miss(mut self, callback: impl FunctionRef) -> Self {
        self.miss = Some(callback.function_id());
        self
    }

    /// Calculate `ceil(max_distance / step)`, rejecting invalid values or more
    /// than 256 samples. The final interval ends exactly at max_distance, even
    /// when the range is smaller than one step. Quotients within two floating-point
    /// representable values of a positive integer are rounded to that integer
    /// before taking the ceiling, avoiding duplicate endpoint samples from
    /// decimal input rounding (for example, `0.9 / 0.03`).
    #[sand_macros::api(
        path = "sand::systems::raycast::BlockRaycast::sample_count", module = "sand::systems::raycast", kind = "method",
        summary = "Calculates and validates the bounded sample count.",
        context = "Definitions retain validated identities until registration and command export; casts run only in Minecraft.",
        minecraft = "Traversal preserves executor, rotation and dimension; callbacks run at the terminating sample with canonical context guards.",
        use_when = ["Composing a reusable synchronous block cast"],
        avoid_when = ["Exact voxel traversal or unbounded callback recursion is required"],
        returns = "Ceiling of range divided by step, or an error for invalid or excessive traversal.",
        example = r#"use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
let ray = BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap());
assert_eq!(ray.max_distance(2.5).step(1.0).sample_count().unwrap(), 3);"#
    )]
    pub fn sample_count(&self) -> CommandResult<usize> {
        if !self.distance.is_finite()
            || self.distance <= 0.0
            || !self.step.is_finite()
            || self.step <= 0.0
        {
            return Err(self.error("distance and step must be finite and positive"));
        }
        let quotient = self.distance / self.step;
        let nearest = quotient.round();
        let normalized = if nearest >= 1.0
            && quotient >= nearest.next_down().next_down()
            && quotient <= nearest.next_up().next_up()
        {
            nearest
        } else {
            quotient
        };
        let count = normalized.ceil().max(1.0);
        if !count.is_finite() || count < 1.0 || count > SAMPLE_LIMIT as f64 {
            return Err(self.error("sample count must be between 1 and 256"));
        }
        Ok(count as usize)
    }

    /// Cast from the current execution position and rotation as the current entity.
    /// Registration remains separate. The action retains its resource reference
    /// across cached bodies and exports; it does not run or register in Rust.
    #[sand_macros::api(
        path = "sand::systems::raycast::BlockRaycast::cast", module = "sand::systems::raycast", kind = "method",
        summary = "Creates an action that casts as the current entity.",
        context = "Definitions retain validated identities until registration and command export; casts run only in Minecraft.",
        minecraft = "Traversal preserves executor, rotation and dimension; callbacks run at the terminating sample with canonical context guards.",
        use_when = ["Composing a reusable synchronous block cast"],
        avoid_when = ["Exact voxel traversal or unbounded callback recursion is required"],
        returns = "Owned Actions referencing this separately registered definition.",
        example = r#"use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
#[function]
fn found_log() { cmd::say("Found a matching block"); }
#[datapack_component]
fn mining_ray() -> BlockRaycast {
    BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap())
        .scan_for(BlockPredicate::new().tag(TagId::minecraft("logs").unwrap()))
        .on_hit(found_log)
}
#[function]
fn use_tool() { mining_ray().cast(); }"#
    )]
    pub fn cast(&self) -> Actions {
        Cmd::Function(self.resource("start").to_string()).into_commands()
    }

    fn resource(&self, suffix: &str) -> ResourceLocation {
        ResourceLocation::new(
            self.id.namespace(),
            format!("__sand_system/raycast/{}/{suffix}", self.id.path()),
        )
        .expect("validated resource path")
    }

    fn error(&self, message: &str) -> CommandError {
        CommandError::new(
            "block raycast",
            "definition",
            format!("{}: {message}", self.id),
        )
    }

    fn validate(&self) -> CommandResult<usize> {
        let count = self.sample_count()?;
        if self.search.is_none() || matches!(self.search, Some(Search::Collision(None))) {
            return Err(self.error("choose scan_for or first_collision with pass_through"));
        }
        if self.hit.is_none() {
            return Err(self.error("an on_hit callback is required"));
        }
        Ok(count)
    }
}

impl IntoDatapack for BlockRaycast {
    fn into_datapack(self) -> DatapackRegistration {
        let block = match &self.search {
            Some(Search::Scan(block) | Search::Collision(Some(block))) => block.clone(),
            _ => BlockPredicate::new(), // Every generated function validates the definition.
        };
        let predicate = Predicate::new(
            PredicateId::custom(self.resource("criterion")),
            PredicateRoot::location(LocationPredicate::new().block(block)),
        );
        // An unconstrained block predicate still requires a real, loaded block
        // position. `execute if loaded` alone checks chunks, not vertical bounds.
        let valid_sample = Predicate::new(
            PredicateId::custom(self.resource("valid_sample")),
            PredicateRoot::location(LocationPredicate::new().block(BlockPredicate::new())),
        );
        let load = self.resource("load");
        let mut registration = DatapackRegistration::new()
            .component(predicate)
            .component(valid_sample)
            .lifecycle(LifecycleContribution::load(
                Cmd::Function(load.to_string())
                    .try_render()
                    .expect("validated load reference"),
            ));
        for part in [
            Part::Start,
            Part::Advance,
            Part::Sample,
            Part::Hit,
            Part::Miss,
            Part::Load,
        ] {
            registration = registration.component(RaycastFunction {
                location: self.resource(part.name()),
                definition: self.clone(),
                part,
            });
        }
        registration
    }
}

#[derive(Clone, Copy)]
enum Part {
    Start,
    Advance,
    Sample,
    Hit,
    Miss,
    Load,
}
impl Part {
    fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Advance => "advance",
            Self::Sample => "sample",
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::Load => "load",
        }
    }
}
struct RaycastFunction {
    location: ResourceLocation,
    definition: BlockRaycast,
    part: Part,
}

fn execute(operations: Vec<ExecuteOp>, run: Cmd) -> Cmd {
    Cmd::Execute {
        operations,
        run: Box::new(run),
    }
}
fn terminal(run: Cmd) -> Cmd {
    Cmd::ReturnRun(Box::new(run))
}
fn position(distance: f64) -> ExecuteOp {
    ExecuteOp::Positioned(Vec3::new(
        Coord::local(),
        Coord::local(),
        Coord::local_n(distance),
    ))
}

impl RaycastFunction {
    fn build(&self) -> CommandResult<McFunction> {
        let definition = &self.definition;
        let count = definition.validate()?;
        let id = sand_core::function::resolve_export_location(&definition.id)?;
        let initializer =
            sand_core::function::resolve_export_location(&definition.resource("load"))?;
        let objective = ObjectiveName::logical(format!("sand raycast {id} remaining"));
        sand_core::function::reserve_generated_objectives([objective.clone()], Some(initializer))?;
        let objective = objective.to_string();
        let score = |range: &str| ConditionIr::ScoreMatches {
            holder: ScoreHolder::self_(),
            objective: objective.clone(),
            range: range.into(),
        };
        let call = |part: &str| Cmd::Function(definition.resource(part).to_string());
        let set = |value| {
            Cmd::ScorePlayers(ScorePlayersOp::Set {
                selector: "@s".into(),
                objective: objective.clone(),
                value,
            })
        };
        let commands = match self.part {
            Part::Start => vec![
                execute(
                    vec![ExecuteOp::Unless(ConditionIr::Entity(Selector::self_()))],
                    Cmd::Return(0),
                ),
                set(count as i32),
                terminal(execute(
                    vec![
                        ExecuteOp::Anchored(definition.origin),
                        position(0.0),
                        ExecuteOp::Anchored(Anchor::Feet),
                    ],
                    call("advance"),
                )),
            ],
            Part::Advance => vec![
                execute(
                    vec![ExecuteOp::Unless(score(&format!("1..{SAMPLE_LIMIT}")))],
                    Cmd::Return(0),
                ),
                execute(
                    vec![ExecuteOp::If(score("1"))],
                    terminal(execute(
                        vec![position(
                            definition.distance - definition.step * (count - 1) as f64,
                        )],
                        call("sample"),
                    )),
                ),
                terminal(execute(vec![position(definition.step)], call("sample"))),
            ],
            Part::Sample => {
                let criterion =
                    ConditionIr::Predicate(definition.resource("criterion").to_string());
                let hit = match definition.search {
                    Some(Search::Scan(_)) => ExecuteOp::If(criterion),
                    _ => ExecuteOp::Unless(criterion),
                };
                vec![
                    Cmd::ScorePlayers(ScorePlayersOp::Remove {
                        selector: "@s".into(),
                        objective: objective.clone(),
                        amount: 1,
                    }),
                    execute(
                        vec![ExecuteOp::Unless(ConditionIr::Loaded(BlockPos::here()))],
                        terminal(call("miss")),
                    ),
                    execute(
                        vec![ExecuteOp::Unless(ConditionIr::Predicate(
                            definition.resource("valid_sample").to_string(),
                        ))],
                        terminal(call("miss")),
                    ),
                    execute(vec![hit], terminal(call("hit"))),
                    execute(vec![ExecuteOp::If(score("0"))], terminal(call("miss"))),
                    terminal(call("advance")),
                ]
            }
            Part::Hit | Part::Miss => vec![
                Cmd::ScorePlayers(ScorePlayersOp::Reset {
                    selector: "@s".into(),
                    objective: objective.clone(),
                }),
                match self.part {
                    Part::Hit => definition.hit.clone(),
                    _ => definition.miss.clone(),
                }
                .map(|callback| terminal(Cmd::RegisteredFunction(callback)))
                .unwrap_or(Cmd::Return(0)),
            ],
            Part::Load => vec![Cmd::ScoreDefine {
                objective,
                criterion: "dummy".into(),
            }],
        };
        Ok(McFunction::new(self.location.clone()).commands(commands))
    }
}

impl DatapackComponent for RaycastFunction {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }
    fn component_dir(&self) -> &'static str {
        "function"
    }
    fn file_extension(&self) -> &'static str {
        "mcfunction"
    }
    fn to_json(&self) -> Value {
        match self.try_content().expect("raycast must validate") {
            ComponentContent::Text(text) => Value::String(text),
            ComponentContent::Json(json) => json,
        }
    }
    fn try_content(&self) -> sand_components::error::Result<ComponentContent> {
        self.build()
            .map_err(|error| sand_components::SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "raycast".into(),
                field: error.field.clone(),
                message: error.to_string(),
            })?
            .try_content()
    }
}
