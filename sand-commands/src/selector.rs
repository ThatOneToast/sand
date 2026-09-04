//! Entity/player selector (`@a`, `@e`, `@s`, etc.) with a typed builder API.

use std::fmt;
use std::marker::PhantomData;

use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::validate;

// ── Entity type conversion ──────────────────────────────────────────────────────

/// Conversion accepted by entity-type filter/target methods (`entity_type`,
/// `not_type`, `summon`, ...).
///
/// Implemented for `&str`/`String` (the untyped escape hatch — no validation
/// beyond what the selector/command syntax itself enforces) and for Sand's
/// typed vanilla/custom entity-type identifiers: the generated vanilla
/// entity-type enum when the selected Minecraft profile provides one, and
/// `EntityTypeId` (validated custom/modded IDs).
/// Prefer the typed identifiers in normal code; the string forms remain for
/// compatibility and cases with no typed representation yet.
///
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::IntoEntityType",
    aliases = ["sand::cmd::IntoEntityType", "sand::prelude::cmd::IntoEntityType"],
    module = "sand::command",
    summary = "Conversion accepted by entity-type filter/target methods (`entity_type`, `not_type`, `summon`, ...).",
    context = "Conversion accepted by entity-type filter/target methods (`entity_type`, `not_type`, `summon`, ...). Implemented for `&str`/`String` (the untyped escape hatch — no validation beyond what the selector/command syntax itself enforces) and for Sand's typed vanilla/custom entity-type identifiers: the generated vanilla entity-type enum when the selected Minecraft profile provides one, and `EntityTypeId` (validated custom/modded IDs). Prefer the typed identifiers in normal code; the string forms remain for compatibility and cases with no typed representation yet.",
    minecraft = "Implemented for `&str`/`String` (the untyped escape hatch — no validation beyond what the selector/command syntax itself enforces) and for Sand's typed vanilla/custom entity-type identifiers: the generated vanilla entity-type enum when the selected Minecraft profile provides one, and `EntityTypeId` (validated custom/modded IDs). Prefer the typed identifiers in normal code; the string forms remain for compatibility and cases with no typed representation yet.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::IntoEntityType;",
)]
pub trait IntoEntityType {
    /// Convert to the entity type's resource location, e.g. `"minecraft:marker"`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::IntoEntityType::into_entity_type",
        aliases = ["sand::cmd::IntoEntityType::into_entity_type", "sand::prelude::cmd::IntoEntityType::into_entity_type"],
        module = "sand::command",
        summary = "Convert to the entity type's resource location, e.g. `\"minecraft:marker\"`.",
        context = "Convert to the entity type's resource location, e.g. `\"minecraft:marker\"`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The string value produced to convert to the entity type's resource location, e.g. `\"minecraft:marker\"`.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::command::IntoEntityType>(into_entity_type_value: T)  {\n    let into_entity_type = into_entity_type_value.into_entity_type();\n}",
    )]
    fn into_entity_type(self) -> String;
}

impl IntoEntityType for String {
    fn into_entity_type(self) -> String {
        self
    }
}

impl IntoEntityType for &str {
    fn into_entity_type(self) -> String {
        self.to_string()
    }
}

impl IntoEntityType for &String {
    fn into_entity_type(self) -> String {
        self.clone()
    }
}

// ── Public types ──────────────────────────────────────────────────────────────

/// An entity/player selector for use in Minecraft commands.
///
/// Selectors target entities in the world. Construct with a base selector (e.g., `all_players()`)
/// then refine with builder methods to add filters (tags, distance, team, etc.).
///
/// # Examples
/// ```
/// use sand_commands::Target;
///
/// // @a[tag=ready,limit=1]
/// let sel = Target::players().tag("ready").limit(1).unwrap();
/// assert_eq!(sel.to_string(), "@a[tag=ready,limit=1]");
///
/// // @s
/// assert_eq!(Target::self_().to_string(), "@s");
/// ```
#[derive(Debug, Clone)]
#[doc(hidden)]
#[must_use = "selectors do nothing until passed to a command"]
pub struct Selector {
    base: TargetBase,
    args: Vec<SelectorArg>,
}

impl PartialEq for Selector {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for Selector {}

impl From<Selector> for String {
    fn from(s: Selector) -> Self {
        s.to_string()
    }
}

impl From<&Selector> for String {
    fn from(s: &Selector) -> Self {
        s.to_string()
    }
}

/// The base target variant of a selector.
#[derive(Debug, Clone, PartialEq)]
enum TargetBase {
    #[doc = "Selects the all players form of the target base Minecraft command value."]
    AllPlayers,
    #[doc = "Selects the all entities form of the target base Minecraft command value."]
    AllEntities,
    #[doc = "Selects the nearest player form of the target base Minecraft command value."]
    NearestPlayer,
    #[doc = "Selects the self  form of the target base Minecraft command value."]
    Self_,
    #[doc = "Selects the random player form of the target base Minecraft command value."]
    RandomPlayer,
    #[doc = "Selects the player form of the target base Minecraft command value."]
    Player(#[doc = "Selects the player form of the target base Minecraft command value."] String),
    /// Explicit unchecked selector syntax for advanced/modded grammar.
    Raw(#[doc = "Explicit unchecked selector syntax for advanced/modded grammar."] String),
}

/// Marker for selector wrappers that are statically known to select one target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc(hidden)]
pub enum One {}

/// Marker for selector wrappers that may select multiple targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc(hidden)]
pub enum Many {}

// ── Canonical target model ───────────────────────────────────────────────────

/// Hidden category marker for targets that may select any entity.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnyTarget {}

/// Hidden category marker for targets statically restricted to players.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayersOnly {}

/// The canonical entity/player target used by Sand command and query APIs.
///
/// Construct targets through the discoverable associated functions on this
/// type. The category and cardinality parameters are inferred from those
/// constructors and from narrowing methods, so normal author code never needs
/// to name them.
///
/// ```
/// use sand_commands::Target;
///
/// let enemies = Target::entities().tag("enemy").within_blocks(16.0);
/// let nearest = enemies.nearest();
/// let players = Target::players().gamemode(sand_commands::GameMode::Survival);
///
/// assert_eq!(nearest.to_string(), "@e[tag=enemy,distance=..16,sort=nearest,limit=1]");
/// assert_eq!(players.to_string(), "@a[gamemode=survival]");
/// ```
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Target",
    aliases = ["sand::cmd::Target", "sand::prelude::Target", "sand::prelude::cmd::Target"],
    module = "sand::command",
    summary = "The canonical typed Minecraft entity/player target.",
    context = "Target is the normal authoring model for constructing, filtering, narrowing, iterating, and passing Minecraft entity/player selections to commands. Category and cardinality are inferred from constructors and narrowing methods.",
    minecraft = "Renders a validated Minecraft selector or literal player name while preserving player-only capabilities and single-target cardinality in its hidden type state.",
    use_when = ["Selecting entities or players for a command", "Iterating matching entities with TargetExecution"],
    avoid_when = ["Representing the currently bound executor as an EntityContext", "Representing fake or wildcard scoreboard holders"],
    example = "let enemies = sand::command::Target::entities().tag(\"enemy\").nearest();",
)]
#[derive(Debug)]
#[must_use = "targets do nothing until passed to a command or iterated"]
pub struct Target<K = AnyTarget, A = Many> {
    raw: Selector,
    _kind: PhantomData<K>,
    _arity: PhantomData<A>,
}

impl<K, A> Target<K, A> {
    fn from_selector(raw: Selector) -> Self {
        Self {
            raw,
            _kind: PhantomData,
            _arity: PhantomData,
        }
    }

    /// Converts into the low-level selector representation.
    #[doc(hidden)]
    pub(crate) fn into_selector(self) -> Selector {
        self.raw
    }

    /// Restricts the target to entities with `tag`.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::tag", aliases = ["sand::cmd::Target::tag", "sand::prelude::Target::tag", "sand::prelude::cmd::Target::tag"], module = "sand::command", summary = "Restricts a target to entities with a tag.", context = "Adds a validated tag filter while preserving target category and cardinality.", minecraft = "Emits tag=<tag> in the selector argument list.", use_when = ["Filtering a target by entity tag"], avoid_when = ["Injecting an unmodeled selector fragment"], params(tag = "The entity tag to require."), returns = "The same typed target with the tag filter applied.", example = "let target = sand::command::Target::entities().tag(\"enemy\");")]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.raw = self.raw.tag(tag);
        self
    }

    /// Excludes entities with `tag`.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::without_tag", aliases = ["sand::cmd::Target::without_tag", "sand::prelude::Target::without_tag", "sand::prelude::cmd::Target::without_tag"], module = "sand::command", summary = "Excludes entities with a tag.", context = "Adds a validated negated tag filter while preserving target category and cardinality.", minecraft = "Emits tag=!<tag> in the selector argument list.", use_when = ["Excluding a tagged entity"], avoid_when = ["Injecting an unmodeled selector fragment"], params(tag = "The entity tag to exclude."), returns = "The same typed target with the exclusion applied.", example = "let target = sand::command::Target::entities().without_tag(\"friendly\");")]
    pub fn without_tag(mut self, tag: impl Into<String>) -> Self {
        self.raw = self.raw.not_tag(tag);
        self
    }

    /// Alias for [`Target::without_tag`].
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::not_tag", aliases = ["sand::cmd::Target::not_tag", "sand::prelude::Target::not_tag", "sand::prelude::cmd::Target::not_tag"], module = "sand::command", summary = "Alias for Target::without_tag.", context = "Provides the selector-style negation spelling while preserving the canonical Target value.", minecraft = "Emits tag=!<tag> in the selector argument list.", use_when = ["Using symmetric tag/not_tag filter naming"], avoid_when = ["A single canonical spelling is preferred; use without_tag"], params(tag = "The entity tag to exclude."), returns = "The same typed target with the exclusion applied.", example = "let target = sand::command::Target::entities().not_tag(\"friendly\");")]
    pub fn not_tag(self, tag: impl Into<String>) -> Self {
        self.without_tag(tag)
    }

    fn with_distance_range(mut self, range: TargetRange) -> Self {
        self.raw = self.raw.distance_typed(range);
        self
    }

    /// Restricts the target to entities within `max` blocks.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::within_blocks", aliases = ["sand::cmd::Target::within_blocks", "sand::prelude::Target::within_blocks", "sand::prelude::cmd::Target::within_blocks"], module = "sand::command", summary = "Restricts a target to entities within a maximum distance.", context = "Convenience form of a typed distance upper bound.", minecraft = "Emits distance=..<max>.", use_when = ["Selecting nearby entities or players"], avoid_when = ["Selecting a full minimum/maximum range"], params(max = "The inclusive maximum distance in blocks."), returns = "The same typed target with the distance filter applied.", example = "let target = sand::command::Target::entities().within_blocks(16.0);")]
    pub fn within_blocks(self, max: f64) -> Self {
        self.with_distance_range(TargetRange::at_most(max))
    }

    /// Restricts the target to entities at least `min` blocks away.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::distance_min", aliases = ["sand::cmd::Target::distance_min", "sand::prelude::Target::distance_min", "sand::prelude::cmd::Target::distance_min"], module = "sand::command", summary = "Restricts a target to entities beyond a minimum distance.", context = "Convenience form of a typed distance lower bound.", minecraft = "Emits distance=<min>...", use_when = ["Excluding nearby entities by distance"], avoid_when = ["Selecting a full minimum/maximum range"], params(min = "The inclusive minimum distance in blocks."), returns = "The same typed target with the distance filter applied.", example = "let target = sand::command::Target::entities().distance_min(1.0);")]
    pub fn distance_min(self, min: f64) -> Self {
        self.with_distance_range(TargetRange::at_least(min))
    }

    /// Restricts the target to entities between `min` and `max` blocks away.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::distance_range", aliases = ["sand::cmd::Target::distance_range", "sand::prelude::Target::distance_range", "sand::prelude::cmd::Target::distance_range"], module = "sand::command", summary = "Restricts a target to an inclusive distance range.", context = "Builds a typed two-sided selector distance range.", minecraft = "Emits distance=<min>..<max>.", use_when = ["Selecting entities inside a distance band"], avoid_when = ["Only an upper bound is needed; use within_blocks"], params(min = "The inclusive minimum distance.", max = "The inclusive maximum distance."), returns = "The same typed target with the distance range applied.", example = "let target = sand::command::Target::entities().distance_range(2.0, 16.0);")]
    pub fn distance_range(self, min: f64, max: f64) -> Self {
        self.with_distance_range(TargetRange::between(min, max))
    }

    /// Restricts the target to a scoreboard range.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::score", aliases = ["sand::cmd::Target::score", "sand::prelude::Target::score", "sand::prelude::cmd::Target::score"], module = "sand::command", summary = "Adds one typed scoreboard filter to a target.", context = "Validates the objective and score range without requiring a hand-formatted selector score map.", minecraft = "Emits a scores={<objective>=<range>} selector entry.", use_when = ["Filtering by one scoreboard objective"], avoid_when = ["Filtering by several objectives at once; use scores"], params(objective = "The validated scoreboard objective.", range = "The accepted integer score range."), returns = "The filtered target or an objective validation error.", example = "let target = sand::command::Target::entities().score(sand::command::ObjectiveName::new(\"threat\"), sand::command::ScoreRange::at_least(5))?;")]
    pub fn score(
        mut self,
        objective: crate::ObjectiveName,
        range: ScoreRange,
    ) -> CommandResult<Self> {
        self.raw = self.raw.score_typed(objective, range)?;
        Ok(self)
    }

    /// Restricts the target with one or more typed scoreboard filters.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::scores", aliases = ["sand::cmd::Target::scores", "sand::prelude::Target::scores", "sand::prelude::cmd::Target::scores"], module = "sand::command", summary = "Adds typed scoreboard filters directly to a target.", context = "Accepts ordinary objective/range pairs so authors do not need a separate selector-map wrapper. Objectives and ranges are checked at the normal target validation boundary.", minecraft = "Emits scores={<objective>=<range>,...} in insertion order.", use_when = ["Filtering by several scoreboard objectives"], avoid_when = ["Filtering by one objective; use score"], params(scores = "Objective/range pairs to require."), returns = "The same target with the typed score filters applied.", example = "let target = sand::command::Target::entities().scores([(ObjectiveName::new(\"threat\"), ScoreRange::at_least(5))]);")]
    pub fn scores(
        mut self,
        scores: impl IntoIterator<Item = (crate::ObjectiveName, ScoreRange)>,
    ) -> Self {
        self.raw = self.raw.scores_typed(
            scores
                .into_iter()
                .map(|(objective, range)| (objective.to_string(), range)),
        );
        self
    }

    /// Explicit raw escape hatch for a scoreboard selector map.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::scores_raw", aliases = ["sand::cmd::Target::scores_raw", "sand::prelude::Target::scores_raw", "sand::prelude::cmd::Target::scores_raw"], module = "sand::command", summary = "Adds an explicitly raw scoreboard selector map.", context = "Escape hatch for future or modded score syntax not represented by ObjectiveName and ScoreRange pairs.", minecraft = "Emits the supplied fragment inside scores={...} after shape validation.", use_when = ["Using future or modded score selector syntax"], avoid_when = ["Target::score or Target::scores can represent the filter"], params(scores = "The raw score-map fragment."), returns = "The same typed target with the raw filter applied.", example = "let target = sand::command::Target::entities().scores_raw(\"threat=5..\");")]
    pub fn scores_raw(mut self, scores: impl Into<String>) -> Self {
        self.raw = self.raw.scores(scores);
        self
    }

    /// Restricts the target to members of `team`.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::team", aliases = ["sand::cmd::Target::team", "sand::prelude::Target::team", "sand::prelude::cmd::Target::team"], module = "sand::command", summary = "Restricts a target to a team.", context = "Adds a validated team selector filter.", minecraft = "Emits team=<team>.", use_when = ["Selecting members of a scoreboard team"], avoid_when = ["Selecting entities outside a team"], params(team = "The required team name."), returns = "The same typed target with the team filter applied.", example = "let target = sand::command::Target::entities().team(\"red\");")]
    pub fn team(mut self, team: impl Into<String>) -> Self {
        self.raw = self.raw.team(team);
        self
    }

    /// Excludes members of `team`.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::not_team", aliases = ["sand::cmd::Target::not_team", "sand::prelude::Target::not_team", "sand::prelude::cmd::Target::not_team"], module = "sand::command", summary = "Excludes a team from a target.", context = "Adds a validated negated team selector filter.", minecraft = "Emits team=!<team>.", use_when = ["Excluding members of a scoreboard team"], avoid_when = ["Selecting members of one team"], params(team = "The excluded team name."), returns = "The same typed target with the exclusion applied.", example = "let target = sand::command::Target::entities().not_team(\"blue\");")]
    pub fn not_team(mut self, team: impl Into<String>) -> Self {
        self.raw = self.raw.not_team(team);
        self
    }

    /// Restricts the target to entities with the supplied name.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::name", aliases = ["sand::cmd::Target::name", "sand::prelude::Target::name", "sand::prelude::cmd::Target::name"], module = "sand::command", summary = "Restricts a target to an entity name.", context = "Adds a validated name selector filter.", minecraft = "Emits name=<name>.", use_when = ["Selecting a named entity"], avoid_when = ["Selecting a literal player target; use named_player"], params(name = "The required entity name."), returns = "The same typed target with the name filter applied.", example = "let target = sand::command::Target::entities().name(\"Boss\");")]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.raw = self.raw.name(name);
        self
    }

    /// Excludes entities with the supplied name.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::not_name", aliases = ["sand::cmd::Target::not_name", "sand::prelude::Target::not_name", "sand::prelude::cmd::Target::not_name"], module = "sand::command", summary = "Excludes an entity name from a target.", context = "Adds a validated negated name selector filter.", minecraft = "Emits name=!<name>.", use_when = ["Excluding a named entity"], avoid_when = ["Selecting a literal player target"], params(name = "The excluded entity name."), returns = "The same typed target with the exclusion applied.", example = "let target = sand::command::Target::entities().not_name(\"Friendly\");")]
    pub fn not_name(mut self, name: impl Into<String>) -> Self {
        self.raw = self.raw.not_name(name);
        self
    }

    /// Sets the selector origin used by relative spatial filters.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::at_pos", aliases = ["sand::cmd::Target::at_pos", "sand::prelude::Target::at_pos", "sand::prelude::cmd::Target::at_pos"], module = "sand::command", summary = "Sets the origin used by spatial target filters.", context = "Adds validated x, y, and z selector arguments while preserving category and cardinality.", minecraft = "Emits x, y, and z selector arguments.", use_when = ["Centering distance or volume filters at a position"], avoid_when = ["Changing execute position; use Execute::positioned"], params(x = "The x coordinate.", y = "The y coordinate.", z = "The z coordinate."), returns = "The same typed target with an explicit selector origin.", example = "let target = sand::command::Target::entities().at_pos(0.0, 64.0, 0.0);")]
    pub fn at_pos(mut self, x: f64, y: f64, z: f64) -> Self {
        self.raw = self.raw.at_pos(x, y, z);
        self
    }

    /// Adds a selector bounding box.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::volume", aliases = ["sand::cmd::Target::volume", "sand::prelude::Target::volume", "sand::prelude::cmd::Target::volume"], module = "sand::command", summary = "Adds a selector bounding volume.", context = "Adds validated dx, dy, and dz selector arguments.", minecraft = "Emits dx, dy, and dz selector arguments.", use_when = ["Selecting entities in an axis-aligned box"], avoid_when = ["A radial distance filter is intended"], params(dx = "The x extent.", dy = "The y extent.", dz = "The z extent."), returns = "The same typed target with the volume applied.", example = "let target = sand::command::Target::entities().volume(4.0, 2.0, 4.0);")]
    pub fn volume(mut self, dx: f64, dy: f64, dz: f64) -> Self {
        self.raw = self.raw.volume(dx, dy, dz);
        self
    }

    /// Excludes the current executor when the selector is centered at `@s`.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::excluding_self", aliases = ["sand::cmd::Target::excluding_self", "sand::prelude::Target::excluding_self", "sand::prelude::cmd::Target::excluding_self"], module = "sand::command", summary = "Excludes the current executor from a target.", context = "Applies Sand's distance-based self-exclusion filter without changing target type state.", minecraft = "Emits distance=0.1.. relative to the executor.", use_when = ["Selecting nearby entities other than the executor"], avoid_when = ["The selector is not evaluated around the current executor"], returns = "The same typed target with self excluded.", example = "let target = sand::command::Target::entities().excluding_self();")]
    pub fn excluding_self(mut self) -> Self {
        self.raw = self.raw.exclude_self_distance();
        self
    }

    /// Explicit raw escape hatch for an NBT selector filter.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::nbt_raw", aliases = ["sand::cmd::Target::nbt_raw", "sand::prelude::Target::nbt_raw", "sand::prelude::cmd::Target::nbt_raw"], module = "sand::command", summary = "Adds an explicitly raw NBT selector filter.", context = "Escape hatch for selector NBT syntax that has no typed representation.", minecraft = "Emits nbt=<snbt> after structural validation.", use_when = ["Filtering by NBT that Sand cannot model"], avoid_when = ["A typed state or score filter can express the intent"], params(nbt = "The raw SNBT selector fragment."), returns = "The same typed target with the NBT filter applied.", example = "let target = sand::command::Target::entities().nbt_raw(\"{Silent:1b}\");")]
    pub fn nbt_raw(mut self, nbt: impl Into<String>) -> Self {
        self.raw = self.raw.nbt_raw(nbt);
        self
    }

    /// Explicit raw escape hatch for a predicate selector filter.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::predicate_raw", aliases = ["sand::cmd::Target::predicate_raw", "sand::prelude::Target::predicate_raw", "sand::prelude::cmd::Target::predicate_raw"], module = "sand::command", summary = "Adds an explicitly raw predicate selector filter.", context = "Escape hatch pending consolidation on the canonical predicate resource ID.", minecraft = "Emits predicate=<namespace:path> after resource-location validation.", use_when = ["Filtering by a predicate before a shared typed ID is available at this layer"], avoid_when = ["Passing unchecked user input"], params(predicate = "The predicate resource location text."), returns = "The same typed target with the predicate filter applied.", example = "let target = sand::command::Target::entities().predicate_raw(\"demo:is_enemy\");")]
    pub fn predicate_raw(mut self, predicate: impl Into<String>) -> Self {
        self.raw = self.raw.predicate_raw(predicate);
        self
    }

    /// Restricts the target through a canonical predicate resource identifier.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::predicate", aliases = ["sand::cmd::Target::predicate", "sand::prelude::Target::predicate", "sand::prelude::cmd::Target::predicate"], module = "sand::command", summary = "Filters a target through a named predicate resource.", context = "Accepts the canonical PredicateId directly through its Display representation, avoiding a command-local identifier wrapper.", minecraft = "Emits predicate=<namespace:path>.", use_when = ["Filtering entities through a reusable predicate resource"], avoid_when = ["Supplying unsupported raw selector syntax; use predicate_raw"], params(predicate = "The canonical predicate resource identifier."), returns = "The same target with the predicate filter applied.", example = "let target = sand::command::Target::entities().predicate(predicate_id);")]
    pub fn predicate(mut self, predicate: impl fmt::Display) -> Self {
        self.raw = self.raw.predicate(predicate.to_string());
        self
    }

    /// Excludes entities matching a canonical predicate resource identifier.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::not_predicate", aliases = ["sand::cmd::Target::not_predicate", "sand::prelude::Target::not_predicate", "sand::prelude::cmd::Target::not_predicate"], module = "sand::command", summary = "Excludes entities matching a named predicate resource.", context = "Negation is a capability of Target rather than a second predicate-ID wrapper state.", minecraft = "Emits predicate=!<namespace:path>.", use_when = ["Excluding matches of a reusable predicate resource"], avoid_when = ["The predicate should be required; use predicate"], params(predicate = "The canonical predicate resource identifier to negate."), returns = "The same target with the negated predicate filter applied.", example = "let target = sand::command::Target::entities().not_predicate(predicate_id);")]
    pub fn not_predicate(mut self, predicate: impl fmt::Display) -> Self {
        self.raw = self.raw.predicate(format!("!{predicate}"));
        self
    }
}

impl<K, A> Clone for Target<K, A> {
    fn clone(&self) -> Self {
        Self::from_selector(self.raw.clone())
    }
}

impl Target<AnyTarget, Many> {
    /// `@e` — starts a target that may contain any number of entities.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::entities", aliases = ["sand::cmd::Target::entities", "sand::prelude::Target::entities", "sand::prelude::cmd::Target::entities"], module = "sand::command", summary = "Starts a many-entity target.", context = "This is the canonical entry point for filtering and iterating arbitrary entities.", minecraft = "Starts from @e.", use_when = ["Selecting arbitrary Minecraft entities"], avoid_when = ["Player-only filters are required; use players"], returns = "A typed target that may select many entities.", example = "let target = sand::command::Target::entities();")]
    pub fn entities() -> Self {
        Self::from_selector(Selector::all_entities())
    }

    /// Discoverable synonym for [`Target::entities`].
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::all_entities", aliases = ["sand::cmd::Target::all_entities", "sand::prelude::Target::all_entities", "sand::prelude::cmd::Target::all_entities"], module = "sand::command", summary = "Starts a target containing all entities.", context = "Discoverable synonym for Target::entities.", minecraft = "Starts from @e.", use_when = ["Selecting all entity categories"], avoid_when = ["Player-only filters are required"], returns = "A typed target that may select many entities.", example = "let target = sand::command::Target::all_entities();")]
    pub fn all_entities() -> Self {
        Self::entities()
    }

    /// Starts an entity target within `radius` blocks.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::nearby", aliases = ["sand::cmd::Target::nearby", "sand::prelude::Target::nearby", "sand::prelude::cmd::Target::nearby"], module = "sand::command", summary = "Starts a many-entity target inside a radius.", context = "Convenience constructor combining entities and within_blocks.", minecraft = "Emits @e[distance=..<radius>].", use_when = ["Selecting nearby entities"], avoid_when = ["Player-only selection is required"], params(radius = "The inclusive maximum distance."), returns = "A typed many-entity target with a distance filter.", example = "let target = sand::command::Target::nearby(8.0);")]
    pub fn nearby(radius: f64) -> Self {
        Self::entities().within_blocks(radius)
    }

    /// Explicit unchecked many-entity selector syntax.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::raw_many", aliases = ["sand::cmd::Target::raw_many", "sand::prelude::Target::raw_many", "sand::prelude::cmd::Target::raw_many"], module = "sand::command", summary = "Creates an unchecked target assumed to allow many entities.", context = "Advanced escape hatch for modded or future selector grammar; the caller supplies the cardinality assertion.", minecraft = "Emits the supplied selector text verbatim.", use_when = ["Using target grammar Sand cannot model"], avoid_when = ["A typed Target constructor can represent the selection"], params(selector = "The unchecked selector expression."), returns = "A target carrying a many-entity cardinality assertion.", example = "let target = sand::command::Target::raw_many(\"@e[modded=true]\");")]
    pub fn raw_many(selector: impl Into<String>) -> Self {
        Self::from_selector(Selector::raw(selector))
    }
}

impl Target<AnyTarget, One> {
    /// `@s` — the current executor as one entity.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::self_", aliases = ["sand::cmd::Target::self_", "sand::prelude::Target::self_", "sand::prelude::cmd::Target::self_"], module = "sand::command", summary = "Targets the current executor as one entity.", context = "Canonical single-entity constructor for @s; it does not claim that the executor is a player.", minecraft = "Emits @s.", use_when = ["Targeting the current command executor"], avoid_when = ["A player-only capability must be proven; use current_player in a player-bound context"], returns = "A statically single entity target.", example = "let target = sand::command::Target::self_();")]
    pub fn self_() -> Self {
        Self::from_selector(Selector::self_())
    }

    /// A literal player name represented as a single entity target.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::named", aliases = ["sand::cmd::Target::named", "sand::prelude::Target::named", "sand::prelude::cmd::Target::named"], module = "sand::command", summary = "Targets one literal player name as an entity.", context = "Creates a validated literal-name target with single cardinality.", minecraft = "Emits the player name token.", use_when = ["Targeting a known literal player name in an entity-capable command"], avoid_when = ["Selecting a filtered player set"], params(name = "The literal player name."), returns = "A statically single entity target.", example = "let target = sand::command::Target::named(\"Steve\");")]
    pub fn named(name: impl Into<String>) -> Self {
        Self::from_selector(Selector::player(name))
    }

    /// Explicit unchecked single-entity selector syntax.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::raw_single", aliases = ["sand::cmd::Target::raw_single", "sand::prelude::Target::raw_single", "sand::prelude::cmd::Target::raw_single"], module = "sand::command", summary = "Creates an unchecked target asserted to select at most one entity.", context = "Advanced escape hatch for modded or future selector grammar; the caller supplies the cardinality assertion.", minecraft = "Emits the supplied selector text verbatim.", use_when = ["Using single-target grammar Sand cannot model"], avoid_when = ["A typed narrowing method can prove cardinality"], params(selector = "The unchecked selector expression."), returns = "A target carrying a single-entity cardinality assertion.", example = "let target = sand::command::Target::raw_single(\"@e[modded=true,limit=1]\");")]
    pub fn raw_single(selector: impl Into<String>) -> Self {
        Self::from_selector(Selector::raw(selector))
    }
}

impl<A> Target<AnyTarget, A> {
    /// Restricts the target to an entity type.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::entity_type", aliases = ["sand::cmd::Target::entity_type", "sand::prelude::Target::entity_type", "sand::prelude::cmd::Target::entity_type"], module = "sand::command", summary = "Restricts an entity target to one entity type.", context = "Uses Sand's typed/generated entity-type conversion path and preserves cardinality.", minecraft = "Emits type=<entity-type>.", use_when = ["Filtering arbitrary entities by type"], avoid_when = ["The target is already statically player-only"], params(ty = "The typed vanilla, custom, or raw entity type."), returns = "The same entity target with the type filter applied.", example = "let target = sand::command::Target::entities().entity_type(\"minecraft:zombie\");")]
    pub fn entity_type(mut self, ty: impl IntoEntityType) -> Self {
        self.raw = self.raw.entity_type(ty);
        self
    }

    /// Excludes an entity type.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::not_entity_type", aliases = ["sand::cmd::Target::not_entity_type", "sand::prelude::Target::not_entity_type", "sand::prelude::cmd::Target::not_entity_type"], module = "sand::command", summary = "Excludes one entity type from an entity target.", context = "Uses Sand's typed/generated entity-type conversion path and preserves cardinality.", minecraft = "Emits type=!<entity-type>.", use_when = ["Excluding an entity category by type"], avoid_when = ["The target is statically player-only"], params(ty = "The typed vanilla, custom, or raw entity type to exclude."), returns = "The same entity target with the exclusion applied.", example = "let target = sand::command::Target::entities().not_entity_type(\"minecraft:player\");")]
    pub fn not_entity_type(mut self, ty: impl IntoEntityType) -> Self {
        self.raw = self.raw.not_type(ty);
        self
    }

    /// Alias for [`Target::not_entity_type`].
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::not_type", aliases = ["sand::cmd::Target::not_type", "sand::prelude::Target::not_type", "sand::prelude::cmd::Target::not_type"], module = "sand::command", summary = "Alias for Target::not_entity_type.", context = "Provides the compact selector-filter spelling on the canonical Target value.", minecraft = "Emits type=!<entity-type>.", use_when = ["Using symmetric entity_type/not_type naming"], avoid_when = ["A single canonical spelling is preferred; use not_entity_type"], params(ty = "The entity type to exclude."), returns = "The same entity target with the exclusion applied.", example = "let target = sand::command::Target::entities().not_type(\"minecraft:player\");")]
    pub fn not_type(self, ty: impl IntoEntityType) -> Self {
        self.not_entity_type(ty)
    }

    /// Excludes players from an entity target.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::excluding_players", aliases = ["sand::cmd::Target::excluding_players", "sand::prelude::Target::excluding_players", "sand::prelude::cmd::Target::excluding_players"], module = "sand::command", summary = "Excludes players from an entity target.", context = "Discoverable typed convenience for a negated minecraft:player entity-type filter.", minecraft = "Emits type=!minecraft:player.", use_when = ["Selecting only non-player entities"], avoid_when = ["Selecting players"], returns = "The same entity target with players excluded.", example = "let target = sand::command::Target::entities().excluding_players();")]
    pub fn excluding_players(self) -> Self {
        self.not_entity_type("minecraft:player")
    }
}

impl Target<PlayersOnly, Many> {
    /// `@a` — starts a target that may contain any number of players.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::players", aliases = ["sand::cmd::Target::players", "sand::prelude::Target::players", "sand::prelude::cmd::Target::players"], module = "sand::command", summary = "Starts the canonical target used to select or query players.", context = "Canonical player-query entry point that enables player-only target filters without a separate PlayerQuery wrapper.", minecraft = "Starts from @a.", use_when = ["Selecting, filtering, or querying Minecraft players"], avoid_when = ["Non-player entities must be selectable"], returns = "A statically player-only target that may select many players.", example = "let target = sand::command::Target::players();")]
    pub fn players() -> Self {
        Self::from_selector(Selector::all_players())
    }

    /// Discoverable synonym for [`Target::players`].
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::all_players", aliases = ["sand::cmd::Target::all_players", "sand::prelude::Target::all_players", "sand::prelude::cmd::Target::all_players"], module = "sand::command", summary = "Starts a target containing all players.", context = "Discoverable synonym for Target::players.", minecraft = "Starts from @a.", use_when = ["Selecting every online player"], avoid_when = ["Non-player entities must be selectable"], returns = "A statically player-only target that may select many players.", example = "let target = sand::command::Target::all_players();")]
    pub fn all_players() -> Self {
        Self::players()
    }
}

impl Target<PlayersOnly, One> {
    /// `@p` — the nearest player.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::nearest_player", aliases = ["sand::cmd::Target::nearest_player", "sand::prelude::Target::nearest_player", "sand::prelude::cmd::Target::nearest_player"], module = "sand::command", summary = "Targets the nearest player with single cardinality.", context = "Canonical player-only constructor for @p.", minecraft = "Emits @p.", use_when = ["Targeting the nearest player"], avoid_when = ["Selecting several players"], returns = "A statically single, player-only target.", example = "let target = sand::command::Target::nearest_player();")]
    pub fn nearest_player() -> Self {
        Self::from_selector(Selector::nearest_player())
    }

    /// `@r` — a random player.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::random_player", aliases = ["sand::cmd::Target::random_player", "sand::prelude::Target::random_player", "sand::prelude::cmd::Target::random_player"], module = "sand::command", summary = "Targets one random player.", context = "Canonical player-only constructor for @r.", minecraft = "Emits @r.", use_when = ["Selecting one random online player"], avoid_when = ["Deterministic nearest or filtered selection is required"], returns = "A statically single, player-only target.", example = "let target = sand::command::Target::random_player();")]
    pub fn random_player() -> Self {
        Self::from_selector(Selector::random_player())
    }

    /// A literal player name.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::named_player", aliases = ["sand::cmd::Target::named_player", "sand::prelude::Target::named_player", "sand::prelude::cmd::Target::named_player"], module = "sand::command", summary = "Targets one literal player name with player-only capability.", context = "Creates a validated literal player token while retaining player-only method availability.", minecraft = "Emits the supplied player name.", use_when = ["Targeting a known player name in a player-only command"], avoid_when = ["Selecting a filtered player set"], params(name = "The literal player name."), returns = "A statically single, player-only target.", example = "let target = sand::command::Target::named_player(\"Steve\");")]
    pub fn named_player(name: impl Into<String>) -> Self {
        Self::from_selector(Selector::player(name))
    }

    /// `@s` asserted by the author to be a player-bound executor.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::current_player", aliases = ["sand::cmd::Target::current_player", "sand::prelude::Target::current_player", "sand::prelude::cmd::Target::current_player"], module = "sand::command", summary = "Targets @s with an explicit player-only assertion.", context = "Use inside a player-bound event/query context when a command requires a player target; Target::self_ is the honest general entity form.", minecraft = "Emits @s.", use_when = ["The current executor is guaranteed to be a player"], avoid_when = ["The executor may be a non-player entity"], returns = "A statically single, player-only target.", example = "let target = sand::command::Target::current_player();")]
    pub fn current_player() -> Self {
        Self::from_selector(Selector::self_())
    }
}

impl<A> Target<PlayersOnly, A> {
    /// Restricts the target to a typed game mode.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::gamemode", aliases = ["sand::cmd::Target::gamemode", "sand::prelude::Target::gamemode", "sand::prelude::cmd::Target::gamemode"], module = "sand::command", summary = "Restricts a player target to a typed game mode.", context = "This method exists only on targets constructed as player-only; GameMode is the canonical vanilla value.", minecraft = "Emits gamemode=<mode>.", use_when = ["Filtering players by a known vanilla game mode"], avoid_when = ["Filtering a target that may contain non-player entities"], params(mode = "The required game mode."), returns = "The same player target with the game-mode filter applied.", example = "let target = sand::command::Target::players().gamemode(sand::command::GameMode::Survival);")]
    pub fn gamemode(mut self, mode: GameMode) -> Self {
        self.raw = self.raw.gamemode_typed(mode);
        self
    }

    /// Excludes a typed game mode.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::not_gamemode", aliases = ["sand::cmd::Target::not_gamemode", "sand::prelude::Target::not_gamemode", "sand::prelude::cmd::Target::not_gamemode"], module = "sand::command", summary = "Excludes a typed game mode from a player target.", context = "This method exists only on targets constructed as player-only.", minecraft = "Emits gamemode=!<mode>.", use_when = ["Excluding players in one vanilla game mode"], avoid_when = ["Filtering a target that may contain non-player entities"], params(mode = "The excluded game mode."), returns = "The same player target with the exclusion applied.", example = "let target = sand::command::Target::players().not_gamemode(sand::command::GameMode::Spectator);")]
    pub fn not_gamemode(mut self, mode: GameMode) -> Self {
        self.raw = self.raw.not_gamemode_typed(mode);
        self
    }

    /// Explicit raw escape hatch for a game-mode selector filter.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::gamemode_raw", aliases = ["sand::cmd::Target::gamemode_raw", "sand::prelude::Target::gamemode_raw", "sand::prelude::cmd::Target::gamemode_raw"], module = "sand::command", summary = "Adds a raw game-mode filter to a player target.", context = "Explicit escape hatch for future or modded game modes; prefer gamemode for vanilla modes.", minecraft = "Emits gamemode=<mode> after validation.", use_when = ["Using game-mode syntax not represented by GameMode"], avoid_when = ["A GameMode variant is available"], params(mode = "The raw game-mode token."), returns = "The same player target with the filter applied.", example = "let target = sand::command::Target::players().gamemode_raw(\"mod:mode\");")]
    pub fn gamemode_raw(mut self, mode: impl Into<String>) -> Self {
        self.raw = self.raw.gamemode(mode);
        self
    }

    /// Restricts the target to players between two experience levels.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::level_range", aliases = ["sand::cmd::Target::level_range", "sand::prelude::Target::level_range", "sand::prelude::cmd::Target::level_range"], module = "sand::command", summary = "Restricts a player target to an inclusive experience-level range.", context = "Accepts bounds directly so authors do not need a selector-specific range wrapper.", minecraft = "Emits level=<min>..<max>.", use_when = ["Filtering players by an experience-level interval"], avoid_when = ["Filtering non-player entities"], params(min = "The inclusive minimum level.", max = "The inclusive maximum level."), returns = "The same player target with the level filter applied.", example = "let target = sand::command::Target::players().level_range(10.0, 30.0);")]
    pub fn level_range(mut self, min: f64, max: f64) -> Self {
        self.raw = self.raw.level_typed(TargetRange::between(min, max));
        self
    }

    /// Restricts the target to players at or above an experience level.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::level_min", aliases = ["sand::cmd::Target::level_min", "sand::prelude::Target::level_min", "sand::prelude::cmd::Target::level_min"], module = "sand::command", summary = "Restricts a player target to a minimum experience level.", context = "Accepts the numeric bound directly on Target.", minecraft = "Emits level=<min>...", use_when = ["Selecting players at or above a level"], avoid_when = ["Filtering non-player entities"], params(min = "The inclusive minimum level."), returns = "The same player target with the level filter applied.", example = "let target = sand::command::Target::players().level_min(10.0);")]
    pub fn level_min(mut self, min: f64) -> Self {
        self.raw = self.raw.level_typed(TargetRange::at_least(min));
        self
    }

    /// Restricts the target to players at or below an experience level.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::level_max", aliases = ["sand::cmd::Target::level_max", "sand::prelude::Target::level_max", "sand::prelude::cmd::Target::level_max"], module = "sand::command", summary = "Restricts a player target to a maximum experience level.", context = "Accepts the numeric bound directly on Target.", minecraft = "Emits level=..<max>.", use_when = ["Selecting players at or below a level"], avoid_when = ["Filtering non-player entities"], params(max = "The inclusive maximum level."), returns = "The same player target with the level filter applied.", example = "let target = sand::command::Target::players().level_max(30.0);")]
    pub fn level_max(mut self, max: f64) -> Self {
        self.raw = self.raw.level_typed(TargetRange::at_most(max));
        self
    }

    /// Explicit raw escape hatch for an experience-level range.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::level_raw", aliases = ["sand::cmd::Target::level_raw", "sand::prelude::Target::level_raw", "sand::prelude::cmd::Target::level_raw"], module = "sand::command", summary = "Adds a raw experience-level filter to a player target.", context = "Explicit escape hatch for future level syntax; prefer level_min, level_max, or level_range.", minecraft = "Emits level=<range> after validation.", use_when = ["Using future level-range syntax"], avoid_when = ["The numeric Target methods can represent the range"], params(range = "The raw experience-level range."), returns = "The same player target with the level filter applied.", example = "let target = sand::command::Target::players().level_raw(\"10..30\");")]
    pub fn level_raw(mut self, range: impl Into<String>) -> Self {
        self.raw = self.raw.level(range);
        self
    }
}

impl<K> Target<K, Many> {
    /// Narrows a many-target expression to `limit=1`.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::limit", aliases = ["sand::cmd::Target::limit", "sand::prelude::Target::limit", "sand::prelude::cmd::Target::limit"], module = "sand::command", summary = "Narrows a many-target expression to one target.", context = "Changes the hidden cardinality state only when the requested limit is exactly one.", minecraft = "Emits limit=1.", use_when = ["Passing a filtered target to a command that requires one entity"], avoid_when = ["Keeping a many-target expression"], params(n = "The limit, which must be exactly one for static narrowing."), returns = "A statically single target or a validation error.", example = "let target = sand::command::Target::entities().limit(1)?;")]
    pub fn limit(mut self, n: i32) -> CommandResult<Target<K, One>> {
        if n != 1 {
            return Err(CommandError::new(
                "Target::limit",
                "limit",
                format!("single-target narrowing requires `limit=1`, got `{n}`"),
            ));
        }
        self.raw = self.raw.limit(1);
        Ok(Target::from_selector(self.raw))
    }

    /// Sorts by nearest and narrows to one target.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::command::Target::nearest", aliases = ["sand::cmd::Target::nearest", "sand::prelude::Target::nearest", "sand::prelude::cmd::Target::nearest"], module = "sand::command", summary = "Selects the nearest match and narrows cardinality to one.", context = "Preserves whether the source target is entity-wide or player-only while changing its cardinality state.", minecraft = "Adds sort=nearest and limit=1.", use_when = ["Selecting one nearest match from a filtered target"], avoid_when = ["Every matching target should remain selected"], returns = "A statically single target with the same category.", example = "let target = sand::command::Target::entities().tag(\"enemy\").nearest();")]
    pub fn nearest(mut self) -> Target<K, One> {
        self.raw = self.raw.sort(SortOrder::Nearest).limit(1);
        Target::from_selector(self.raw)
    }
}

impl<K, A> fmt::Display for Target<K, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl<K, A> Validate for Target<K, A> {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.raw.validate(profile)
    }
}

impl<K, A> RenderCommand for Target<K, A> {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

impl<K, A> From<Target<K, A>> for Selector {
    fn from(target: Target<K, A>) -> Self {
        target.raw
    }
}

impl<A> From<Target<PlayersOnly, A>> for Target<AnyTarget, A> {
    fn from(target: Target<PlayersOnly, A>) -> Self {
        Self::from_selector(target.raw)
    }
}

mod target_argument_sealed {
    pub trait Sealed {}
}

/// Internal capability accepted by command APIs that can target entities.
#[doc(hidden)]
pub trait TargetArgument:
    target_argument_sealed::Sealed + fmt::Display + Clone + Validate + RenderCommand + Into<Selector>
{
    #[doc(hidden)]
    fn into_target_selector(self) -> Selector;
}

impl target_argument_sealed::Sealed for Selector {}
impl TargetArgument for Selector {
    fn into_target_selector(self) -> Selector {
        self
    }
}

impl<K, A> target_argument_sealed::Sealed for Target<K, A> {}
impl<K, A> TargetArgument for Target<K, A> {
    fn into_target_selector(self) -> Selector {
        self.raw
    }
}

mod single_target_argument_sealed {
    pub trait Sealed {}
}

/// Internal capability accepted by commands that require at most one entity.
#[doc(hidden)]
pub trait SingleTargetArgument:
    single_target_argument_sealed::Sealed + Clone + Into<Selector> + Sized
{
    #[doc(hidden)]
    fn into_single_target_selector(self) -> Selector {
        self.into()
    }
}

impl single_target_argument_sealed::Sealed for Target<AnyTarget, One> {}
impl SingleTargetArgument for Target<AnyTarget, One> {}

impl single_target_argument_sealed::Sealed for Target<PlayersOnly, One> {}
impl SingleTargetArgument for Target<PlayersOnly, One> {}

/// Sort order for entity selection in `@a`/`@e` selectors.
///
/// Determines the order entities are iterated when using commands like `execute as`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Sort by distance from executor (nearest first).
    Nearest,
    /// Sort by distance from executor (furthest first).
    Furthest,
    /// Randomize the order.
    Random,
    /// No specific order (performance optimized).
    Arbitrary,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Nearest => write!(f, "nearest"),
            SortOrder::Furthest => write!(f, "furthest"),
            SortOrder::Random => write!(f, "random"),
            SortOrder::Arbitrary => write!(f, "arbitrary"),
        }
    }
}

/// A single selector argument key=value pair.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum SelectorArg {
    Tag(String),
    NotTag(String),
    Team(String),
    NotTeam(String),
    Name(String),
    NotName(String),
    Type(String),
    NotType(String),
    Limit(i32),
    Sort(SortOrder),
    Distance(String),
    Level(String),
    XRotation(String),
    YRotation(String),
    Gamemode(String),
    Scores(String),
    Nbt(String),
    Predicate(String),
    X(f64),
    Y(f64),
    Z(f64),
    Dx(f64),
    Dy(f64),
    Dz(f64),
}

impl fmt::Display for SelectorArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tag(v) => write!(f, "tag={v}"),
            Self::NotTag(v) => write!(f, "tag=!{v}"),
            Self::Team(v) => write!(f, "team={v}"),
            Self::NotTeam(v) => write!(f, "team=!{v}"),
            Self::Name(v) => write!(f, "name={v}"),
            Self::NotName(v) => write!(f, "name=!{v}"),
            Self::Type(v) => write!(f, "type={v}"),
            Self::NotType(v) => write!(f, "type=!{v}"),
            Self::Limit(v) => write!(f, "limit={v}"),
            Self::Sort(v) => write!(f, "sort={v}"),
            Self::Distance(v) => write!(f, "distance={v}"),
            Self::Level(v) => write!(f, "level={v}"),
            Self::XRotation(v) => write!(f, "x_rotation={v}"),
            Self::YRotation(v) => write!(f, "y_rotation={v}"),
            Self::Gamemode(v) => write!(f, "gamemode={v}"),
            Self::Scores(v) => write!(f, "scores={{{v}}}"),
            Self::Nbt(v) => write!(f, "nbt={v}"),
            Self::Predicate(v) => write!(f, "predicate={v}"),
            Self::X(v) => write!(f, "x={v}"),
            Self::Y(v) => write!(f, "y={v}"),
            Self::Z(v) => write!(f, "z={v}"),
            Self::Dx(v) => write!(f, "dx={v}"),
            Self::Dy(v) => write!(f, "dy={v}"),
            Self::Dz(v) => write!(f, "dz={v}"),
        }
    }
}

// ── Constructor methods ───────────────────────────────────────────────────────

impl Selector {
    /// `@a` — all players currently connected to the server.
    pub fn all_players() -> Self {
        Self {
            base: TargetBase::AllPlayers,
            args: vec![],
        }
    }

    /// `@e` — all entities in the world.
    pub fn all_entities() -> Self {
        Self {
            base: TargetBase::AllEntities,
            args: vec![],
        }
    }

    /// `@p` — the nearest player to the command executor.
    pub fn nearest_player() -> Self {
        Self {
            base: TargetBase::NearestPlayer,
            args: vec![],
        }
    }

    /// `@s` — the entity currently executing the command.
    pub fn self_() -> Self {
        Self {
            base: TargetBase::Self_,
            args: vec![],
        }
    }

    /// `@r` — a random player from the current players.
    pub fn random_player() -> Self {
        Self {
            base: TargetBase::RandomPlayer,
            args: vec![],
        }
    }

    /// A specific player by exact name.
    pub fn player(name: impl Into<String>) -> Self {
        Self {
            base: TargetBase::Player(name.into()),
            args: vec![],
        }
    }

    /// Wrap advanced selector syntax without typed validation.
    ///
    /// Prefer the typed builder methods for normal selectors. Raw selectors
    /// are preserved verbatim and should be limited to syntax Sand cannot yet
    /// model.
    pub fn raw(selector: impl Into<String>) -> Self {
        Self {
            base: TargetBase::Raw(selector.into()),
            args: vec![],
        }
    }
}

// ── Builder methods ───────────────────────────────────────────────────────────

impl Selector {
    /// `tag=<tag>` — select only entities that have the given tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Tag(tag.into()));
        self
    }

    /// `tag=!<tag>` — select only entities that do NOT have the given tag.
    pub fn not_tag(mut self, tag: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotTag(tag.into()));
        self
    }

    /// `team=<team>` — select only entities on the given team.
    pub fn team(mut self, team: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Team(team.into()));
        self
    }

    /// `team=!<team>` — select only entities NOT on the given team.
    pub fn not_team(mut self, team: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotTeam(team.into()));
        self
    }

    /// `name=<name>` — select only entities with the exact display name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Name(name.into()));
        self
    }

    /// `name=!<name>` — select only entities WITHOUT the given display name.
    pub fn not_name(mut self, name: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotName(name.into()));
        self
    }

    /// `type=<entity_type>` — select only entities of the given type.
    pub fn entity_type(mut self, ty: impl IntoEntityType) -> Self {
        self.args.push(SelectorArg::Type(ty.into_entity_type()));
        self
    }

    /// `type=!<entity_type>` — select only entities NOT of the given type.
    pub fn not_type(mut self, ty: impl IntoEntityType) -> Self {
        self.args.push(SelectorArg::NotType(ty.into_entity_type()));
        self
    }

    /// `limit=<n>` — select at most `n` entities.
    pub fn limit(mut self, n: i32) -> Self {
        self.args.push(SelectorArg::Limit(n));
        self
    }

    /// `sort=<order>` — set the sort order before applying limit.
    pub fn sort(mut self, order: SortOrder) -> Self {
        self.args.push(SelectorArg::Sort(order));
        self
    }

    /// `distance=<range>` — select only entities within a distance range.
    pub fn distance(mut self, range: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Distance(range.into()));
        self
    }

    /// `distance=..<max>` — select only entities at most `max` blocks away.
    pub fn distance_max(mut self, max: f64) -> Self {
        self.args.push(SelectorArg::Distance(format!("..{max}")));
        self
    }

    /// `distance=<min>..` — select only entities at least `min` blocks away.
    pub fn distance_min(mut self, min: f64) -> Self {
        self.args.push(SelectorArg::Distance(format!("{min}..")));
        self
    }

    /// `distance=<min>..<max>` — select only entities between `min` and `max` blocks away.
    pub fn distance_range(mut self, min: f64, max: f64) -> Self {
        self.args
            .push(SelectorArg::Distance(format!("{min}..{max}")));
        self
    }

    /// `type=!minecraft:player` — exclude all players from the selection.
    pub fn not_player(mut self) -> Self {
        self.args
            .push(SelectorArg::NotType("minecraft:player".into()));
        self
    }

    /// `level=<range>` — select only players within the given XP level range.
    pub fn level(mut self, range: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Level(range.into()));
        self
    }

    /// `gamemode=<mode>` — select only players in the given gamemode.
    ///
    /// Raw/compatibility: `mode` is a string, validated against the vanilla
    /// gamemode set at [`Selector::try_build`] time rather than at the type
    /// level. Prefer [`Selector::gamemode_typed`] in normal code — see
    /// [#173](https://github.com/ThatOneToast/sand/issues/173).
    pub fn gamemode(mut self, mode: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Gamemode(mode.into()));
        self
    }

    /// `gamemode=<mode>` — select only players in the given gamemode, using
    /// the canonical typed [`GameMode`] enum instead of a validated string.
    pub fn gamemode_typed(mut self, mode: GameMode) -> Self {
        self.args.push(SelectorArg::Gamemode(mode.to_string()));
        self
    }

    /// `gamemode=!<mode>` — exclude players in the given gamemode.
    pub fn not_gamemode_typed(mut self, mode: GameMode) -> Self {
        self.args.push(SelectorArg::Gamemode(format!("!{mode}")));
        self
    }

    /// `scores=<objective>=<range>` — select only entities with matching scoreboard score.
    ///
    /// Raw/compatibility: `scores` is a single pre-formatted fragment (e.g.
    /// `"kills=1..10,deaths=0"`), validated at [`Selector::try_build`] time
    /// rather than at the type level. Prefer [`Selector::scores_typed`] in
    /// normal code — see [#200](https://github.com/ThatOneToast/sand/issues/200).
    /// Equivalent to [`Selector::scores_raw`].
    pub fn scores(mut self, scores: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Scores(scores.into()));
        self
    }

    /// Explicit raw escape hatch for `scores=...` syntax, e.g. hand-formatted
    /// fragments this crate has no typed representation for yet. Equivalent
    /// to [`Selector::scores`] — use whichever name best documents intent at
    /// the call site.
    pub fn scores_raw(self, scores: impl Into<String>) -> Self {
        self.scores(scores)
    }

    /// `scores={<objective>=<range>,...}` — select only entities with
    /// matching scoreboard scores from ordinary objective/range pairs.
    pub(crate) fn scores_typed(
        mut self,
        scores: impl IntoIterator<Item = (String, ScoreRange)>,
    ) -> Self {
        let scores = scores
            .into_iter()
            .map(|(objective, range)| format!("{objective}={range}"))
            .collect::<Vec<_>>()
            .join(",");
        self.args.push(SelectorArg::Scores(scores));
        self
    }

    /// Add one typed scoreboard filter to the selector's score map.
    ///
    /// Repeated calls merge into one `scores={...}` argument. Reusing an
    /// objective is rejected so higher-level typed state queries cannot emit
    /// ambiguous filters.
    pub fn score_typed(
        mut self,
        objective: crate::ObjectiveName,
        range: ScoreRange,
    ) -> CommandResult<Self> {
        objective.validate(&CommandProfile::unprofiled())?;
        let objective = objective.as_str();
        if let Some(SelectorArg::Scores(scores)) = self
            .args
            .iter_mut()
            .find(|argument| matches!(argument, SelectorArg::Scores(_)))
        {
            if scores.split(',').any(|entry| {
                entry
                    .split_once('=')
                    .is_some_and(|(existing, _)| existing == objective)
            }) {
                return Err(CommandError::new(
                    "Selector",
                    "scores",
                    format!("duplicate typed score predicate for objective `{objective}`"),
                )
                .with_code("SAND-SELECTOR-SCORE-DUPLICATE"));
            }
            scores.push(',');
            scores.push_str(objective);
            scores.push('=');
            scores.push_str(&range.to_string());
        } else {
            self.args
                .push(SelectorArg::Scores(format!("{objective}={range}")));
        }
        Ok(self)
    }

    /// `nbt=<nbt>` — select only entities matching the given NBT compound.
    ///
    /// Raw escape hatch: no typed SNBT representation exists yet in this
    /// crate, so this remains the normal path for NBT filters. Equivalent to
    /// [`Selector::nbt_raw`], kept for readability at call sites that prefer
    /// the shorter name.
    pub fn nbt(mut self, nbt: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Nbt(nbt.into()));
        self
    }

    /// Explicit raw escape hatch for `nbt=...` syntax. Equivalent to
    /// [`Selector::nbt`].
    pub fn nbt_raw(self, nbt: impl Into<String>) -> Self {
        self.nbt(nbt)
    }

    /// `predicate=<predicate>` — select only entities matching a loot table predicate.
    ///
    /// Raw/compatibility: `predicate` is a string, validated for
    /// resource-location shape at [`Selector::try_build`] time. Prefer
    /// [`Target::predicate`] in normal code. Equivalent to
    /// [`Selector::predicate_raw`].
    pub fn predicate(mut self, predicate: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Predicate(predicate.into()));
        self
    }

    /// Explicit raw escape hatch for `predicate=...` syntax. Equivalent to
    /// [`Selector::predicate`].
    pub fn predicate_raw(self, predicate: impl Into<String>) -> Self {
        self.predicate(predicate)
    }

    /// `distance=<range>` — select only entities within a typed distance
    /// range using the internal numeric range representation.
    pub(crate) fn distance_typed(mut self, range: TargetRange) -> Self {
        self.args.push(SelectorArg::Distance(range.to_string()));
        self
    }

    /// `level=<range>` — select only players within a typed XP level range
    /// using the internal numeric range representation.
    pub(crate) fn level_typed(mut self, range: TargetRange) -> Self {
        self.args.push(SelectorArg::Level(range.to_string()));
        self
    }

    /// `dx/dy/dz` — set a bounding box volume filter.
    pub fn volume(mut self, dx: f64, dy: f64, dz: f64) -> Self {
        self.args.push(SelectorArg::Dx(dx));
        self.args.push(SelectorArg::Dy(dy));
        self.args.push(SelectorArg::Dz(dz));
        self
    }

    /// `x/y/z` — set the origin point for distance and volume checks.
    pub fn at_pos(mut self, x: f64, y: f64, z: f64) -> Self {
        self.args.push(SelectorArg::X(x));
        self.args.push(SelectorArg::Y(y));
        self.args.push(SelectorArg::Z(z));
        self
    }

    fn exclude_self_distance(mut self) -> Self {
        for arg in &mut self.args {
            if let SelectorArg::Distance(range) = arg
                && let Some(max) = range.strip_prefix("..")
            {
                *range = format!("0.1..{max}");
                return self;
            }
        }
        self.args.push(SelectorArg::Distance("0.1..".to_string()));
        self
    }
}

// ── Display ───────────────────────────────────────────────────────────────────

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base = match &self.base {
            TargetBase::AllPlayers => "@a",
            TargetBase::AllEntities => "@e",
            TargetBase::NearestPlayer => "@p",
            TargetBase::Self_ => "@s",
            TargetBase::RandomPlayer => "@r",
            TargetBase::Player(n) => return write!(f, "{n}"),
            TargetBase::Raw(raw) => return write!(f, "{raw}"),
        };
        if self.args.is_empty() {
            write!(f, "{base}")
        } else {
            let args = self
                .args
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(",");
            write!(f, "{base}[{args}]")
        }
    }
}

impl Selector {
    pub(crate) fn is_statically_single(&self) -> bool {
        !matches!(self.base, TargetBase::Raw(_))
            && (matches!(
                self.base,
                TargetBase::NearestPlayer
                    | TargetBase::Self_
                    | TargetBase::RandomPlayer
                    | TargetBase::Player(_)
            ) || self
                .args
                .iter()
                .any(|arg| matches!(arg, SelectorArg::Limit(1))))
    }
}

impl Validate for Selector {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        if let TargetBase::Raw(_) = self.base {
            if !self.args.is_empty() {
                return Err(CommandError::new(
                    "Selector",
                    "arguments",
                    "raw selectors cannot be combined with typed arguments",
                ));
            }
            return Ok(());
        }
        if let TargetBase::Player(ref name) = self.base {
            if !self.args.is_empty() {
                return Err(CommandError::new(
                    "Selector",
                    "arguments",
                    "literal player names cannot be combined with selector arguments",
                ));
            }
            if name.is_empty()
                || name.len() > 16
                || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                return Err(CommandError::new(
                    "Selector",
                    "player_name",
                    format!("must be 1..=16 ASCII letters, digits, or `_`, got `{name}`"),
                ));
            }
        }

        let mut singleton_keys = std::collections::BTreeSet::new();
        let mut positive_type = false;
        for arg in &self.args {
            let (key, value): (&str, Option<&str>) = match arg {
                SelectorArg::Tag(v) | SelectorArg::NotTag(v) => {
                    validate_optional_token(v, "tag")?;
                    ("tag*", None)
                }
                SelectorArg::Team(v) | SelectorArg::NotTeam(v) => {
                    validate_optional_token(v, "team")?;
                    ("team*", None)
                }
                SelectorArg::Name(v) | SelectorArg::NotName(v) => ("name*", Some(v)),
                SelectorArg::Type(v) => {
                    if positive_type {
                        return Err(CommandError::new(
                            "Selector",
                            "type",
                            "duplicate positive `type` arguments are contradictory",
                        ));
                    }
                    positive_type = true;
                    validate::resource_location_shape(
                        v.strip_prefix('#').unwrap_or(v),
                        "Selector",
                        "type",
                    )?;
                    ("type+", None)
                }
                SelectorArg::NotType(v) => {
                    validate::resource_location_shape(
                        v.strip_prefix('#').unwrap_or(v),
                        "Selector",
                        "type",
                    )?;
                    ("type-", None)
                }
                SelectorArg::Limit(v) => {
                    if !matches!(self.base, TargetBase::AllPlayers | TargetBase::AllEntities) {
                        return Err(CommandError::new(
                            "Selector",
                            "limit",
                            "`limit` is only applicable to `@a` and `@e` selector bases",
                        ));
                    }
                    if *v <= 0 {
                        return Err(CommandError::new(
                            "Selector",
                            "limit",
                            format!("selector limits must be greater than zero, got `{v}`"),
                        ));
                    }
                    ("limit", None)
                }
                SelectorArg::Sort(_) => {
                    if !matches!(self.base, TargetBase::AllPlayers | TargetBase::AllEntities) {
                        return Err(CommandError::new(
                            "Selector",
                            "sort",
                            "`sort` is only applicable to `@a` and `@e` selector bases",
                        ));
                    }
                    ("sort", None)
                }
                SelectorArg::Distance(v) => {
                    validate_range(v, "distance", true)?;
                    ("distance", None)
                }
                SelectorArg::Level(v) => {
                    validate_range(v, "level", false)?;
                    ("level", None)
                }
                SelectorArg::XRotation(v) => {
                    validate_range(v, "x_rotation", true)?;
                    ("x_rotation", None)
                }
                SelectorArg::YRotation(v) => {
                    validate_range(v, "y_rotation", true)?;
                    ("y_rotation", None)
                }
                SelectorArg::Gamemode(v) => {
                    if !matches!(
                        v.strip_prefix('!').unwrap_or(v),
                        "survival" | "creative" | "adventure" | "spectator"
                    ) {
                        return Err(CommandError::new(
                            "Selector",
                            "gamemode",
                            format!("unknown vanilla gamemode `{v}`"),
                        ));
                    }
                    ("gamemode", None)
                }
                SelectorArg::Scores(v) => {
                    validate_scores(v)?;
                    ("scores", None)
                }
                SelectorArg::Nbt(v) => {
                    validate_snbt_compound(v)?;
                    ("nbt", None)
                }
                SelectorArg::Predicate(v) => {
                    validate::resource_location_shape(
                        v.strip_prefix('!').unwrap_or(v),
                        "Selector",
                        "predicate",
                    )?;
                    ("predicate", None)
                }
                SelectorArg::X(v) => {
                    validate::finite(*v, "Selector", "x")?;
                    ("x", None)
                }
                SelectorArg::Y(v) => {
                    validate::finite(*v, "Selector", "y")?;
                    ("y", None)
                }
                SelectorArg::Z(v) => {
                    validate::finite(*v, "Selector", "z")?;
                    ("z", None)
                }
                SelectorArg::Dx(v) => {
                    validate::finite(*v, "Selector", "dx")?;
                    ("dx", None)
                }
                SelectorArg::Dy(v) => {
                    validate::finite(*v, "Selector", "dy")?;
                    ("dy", None)
                }
                SelectorArg::Dz(v) => {
                    validate::finite(*v, "Selector", "dz")?;
                    ("dz", None)
                }
            };
            if let Some(v) = value {
                validate::no_whitespace_or_control(v, "Selector", key)?;
            }
            if !key.ends_with('*')
                && !key.ends_with('-')
                && !key.ends_with('+')
                && !singleton_keys.insert(key)
            {
                return Err(CommandError::new(
                    "Selector",
                    "arguments",
                    format!("duplicate `{key}` argument"),
                ));
            }
        }
        Ok(())
    }
}

impl RenderCommand for Selector {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

fn validate_range(value: &str, field: &'static str, allow_float: bool) -> CommandResult<()> {
    validate::non_empty(value, "Selector", field)?;
    let parse = |part: &str| -> CommandResult<Option<f64>> {
        if part.is_empty() {
            return Ok(None);
        }
        let n = part.parse::<f64>().map_err(|_| {
            CommandError::new("Selector", field, format!("invalid range bound `{part}`"))
        })?;
        validate::finite(n, "Selector", field)?;
        if !allow_float && n.fract() != 0.0 {
            return Err(CommandError::new(
                "Selector",
                field,
                "range requires integer bounds",
            ));
        }
        Ok(Some(n))
    };
    let (min, max) = if let Some((a, b)) = value.split_once("..") {
        if b.contains("..") {
            return Err(CommandError::new(
                "Selector",
                field,
                "range contains more than one `..`",
            ));
        }
        (parse(a)?, parse(b)?)
    } else {
        let exact = parse(value)?;
        (exact, exact)
    };
    if min.is_none() && max.is_none() {
        return Err(CommandError::new(
            "Selector",
            field,
            "range must contain at least one bound",
        ));
    }
    if let (Some(a), Some(b)) = (min, max)
        && a > b
    {
        return Err(CommandError::new(
            "Selector",
            field,
            format!("range lower bound `{a}` exceeds upper bound `{b}`"),
        ));
    }
    if matches!(field, "distance" | "level")
        && (min.is_some_and(|v| v < 0.0) || max.is_some_and(|v| v < 0.0))
    {
        return Err(CommandError::new(
            "Selector",
            field,
            format!("{field} cannot be negative"),
        ));
    }
    Ok(())
}

fn validate_optional_token(value: &str, field: &'static str) -> CommandResult<()> {
    if value.chars().any(|c| c.is_whitespace() || c.is_control()) {
        Err(CommandError::new(
            "Selector",
            field,
            format!("must not contain whitespace or control characters, got `{value}`"),
        ))
    } else {
        Ok(())
    }
}

fn validate_snbt_compound(value: &str) -> CommandResult<()> {
    validate::non_empty(value, "Selector", "nbt")?;
    if !(value.starts_with('{') && value.ends_with('}')) {
        return Err(CommandError::new(
            "Selector",
            "nbt",
            "typed NBT filters must be an SNBT compound wrapped in `{...}`",
        ));
    }
    if value.contains(['\0', '\n', '\r']) {
        return Err(CommandError::new(
            "Selector",
            "nbt",
            "SNBT selector fragments must remain on one command line",
        ));
    }
    let mut delimiters = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    for character in value.chars() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '{' | '[' => delimiters.push(character),
            '}' if delimiters.pop() == Some('{') => {}
            ']' if delimiters.pop() == Some('[') => {}
            '}' | ']' => {
                return Err(CommandError::new(
                    "Selector",
                    "nbt",
                    "SNBT selector fragment has an unmatched closing delimiter",
                ));
            }
            _ => {}
        }
    }
    if quote.is_some() || !delimiters.is_empty() {
        return Err(CommandError::new(
            "Selector",
            "nbt",
            "SNBT selector fragment has unbalanced quotes or delimiters",
        ));
    }
    Ok(())
}

fn validate_scores(value: &str) -> CommandResult<()> {
    validate::non_empty(value, "Selector", "scores")?;
    let mut objectives = std::collections::BTreeSet::new();
    for entry in value.split(',') {
        let Some((objective, range)) = entry.split_once('=') else {
            return Err(CommandError::new(
                "Selector",
                "scores",
                format!("expected `objective=range`, got `{entry}`"),
            ));
        };
        validate::no_whitespace_or_control(objective, "Selector", "scores.objective")?;
        if objective.len() > 16 {
            return Err(CommandError::new(
                "Selector",
                "scores.objective",
                format!("objective `{objective}` exceeds 16 characters"),
            ));
        }
        if !objectives.insert(objective) {
            return Err(CommandError::new(
                "Selector",
                "scores",
                format!("duplicate objective `{objective}`"),
            ));
        }
        validate_range(range, "scores.range", false)?;
    }
    Ok(())
}

// ── TargetRange ─────────────────────────────────────────────────────────────

/// A typed numeric range for selector arguments such as `distance` and
/// `level` (see [#200](https://github.com/ThatOneToast/sand/issues/200)).
///
/// Renders to vanilla's `min..max` range syntax. At least one bound must be
/// present; use [`TargetRange::at_least`]/[`TargetRange::at_most`] for
/// open-ended ranges. Impossible ranges (`min > max`) and non-finite bounds
/// are not rejected at construction — they are diagnosed uniformly with all
/// other selector arguments at [`Selector::try_build`] time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TargetRange {
    min: Option<f64>,
    max: Option<f64>,
}

impl TargetRange {
    /// `n..` — at least `n`.
    pub(crate) fn at_least(n: f64) -> Self {
        Self {
            min: Some(n),
            max: None,
        }
    }

    /// `..n` — at most `n`.
    pub(crate) fn at_most(n: f64) -> Self {
        Self {
            min: None,
            max: Some(n),
        }
    }

    /// `min..max` — an inclusive range.
    pub(crate) fn between(min: f64, max: f64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

impl fmt::Display for TargetRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_bound(v: f64, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{v}")
        }
        match (self.min, self.max) {
            (Some(a), Some(b)) if a == b => fmt_bound(a, f),
            (Some(a), Some(b)) => {
                fmt_bound(a, f)?;
                write!(f, "..")?;
                fmt_bound(b, f)
            }
            (Some(a), None) => {
                fmt_bound(a, f)?;
                write!(f, "..")
            }
            (None, Some(b)) => {
                write!(f, "..")?;
                fmt_bound(b, f)
            }
            (None, None) => Ok(()),
        }
    }
}

// ── ScoreRange ───────────────────────────────────────────────────────────────

/// A typed integer range for `scores={...}` selector entries (see
/// [#200](https://github.com/ThatOneToast/sand/issues/200)).
///
/// Deliberately distinct from [`TargetRange`]: Minecraft scoreboard scores
/// are always 32-bit integers, so `scores={obj=1.5..3.2}` is not legal
/// vanilla syntax even though the same `min..max` grammar shape is used for
/// `distance`/`level` (which *are* floating-point). Using an `i32`-based
/// type here at the API boundary makes a fractional score range a compile
/// error instead of a malformed-selector diagnostic discovered at
/// `try_build` time.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ScoreRange",
    aliases = ["sand::cmd::ScoreRange", "sand::prelude::cmd::ScoreRange"],
    module = "sand::command",
    summary = "A typed integer range for `scores={...}` selector entries (see [#200](https://github.com/ThatOneToast/sand/issues/200)).",
    context = "A typed integer range for `scores={...}` selector entries (see [#200](https://github.com/ThatOneToast/sand/issues/200)). Deliberately distinct from [`TargetRange`]: Minecraft scoreboard scores are always 32-bit integers, so `scores={obj=1.5..3.2}` is not legal vanilla syntax even though the same `min..max` grammar shape is used for `distance`/`level` (which *are* floating-point). Using an `i32`-based type here at the API boundary makes a fractional score range a compile error instead of a malformed-selector diagnostic discovered at `try_build` time.",
    minecraft = "Deliberately distinct from [`TargetRange`]: Minecraft scoreboard scores are always 32-bit integers, so `scores={obj=1.5..3.2}` is not legal vanilla syntax even though the same `min..max` grammar shape is used for `distance`/`level` (which *are* floating-point). Using an `i32`-based type here at the API boundary makes a fractional score range a compile error instead of a malformed-selector diagnostic discovered at `try_build` time.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ScoreRange;",
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreRange {
    min: Option<i32>,
    max: Option<i32>,
}

impl ScoreRange {
    /// An exact value: `n..n`, rendered as `n`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreRange::exact",
        aliases = ["sand::cmd::ScoreRange::exact", "sand::prelude::cmd::ScoreRange::exact"],
        module = "sand::command",
        kind = "method",
        summary = "An exact value: `n..n`, rendered as `n`.",
        context = "An exact value: `n..n`, rendered as `n`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(n = "An exact value: `n..n`, rendered as `n`."),
        returns = "A `ScoreRange` configured for an exact value: `n..n`, rendered as `n`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(n: i32)  {\n    let score_range = sand::command::ScoreRange::exact(n);\n}",
    )]
    pub fn exact(n: i32) -> Self {
        Self {
            min: Some(n),
            max: Some(n),
        }
    }

    /// `n..` — at least `n`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreRange::at_least",
        aliases = ["sand::cmd::ScoreRange::at_least", "sand::prelude::cmd::ScoreRange::at_least"],
        module = "sand::command",
        kind = "method",
        summary = "`n..` — at least `n`.",
        context = "`n..` — at least `n`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(n = "`n..` — at least `n`."),
        returns = "A `ScoreRange` that emits the documented `n..` — at least `n` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(n: i32)  {\n    let score_range = sand::command::ScoreRange::at_least(n);\n}",
    )]
    pub fn at_least(n: i32) -> Self {
        Self {
            min: Some(n),
            max: None,
        }
    }

    /// `..n` — at most `n`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreRange::at_most",
        aliases = ["sand::cmd::ScoreRange::at_most", "sand::prelude::cmd::ScoreRange::at_most"],
        module = "sand::command",
        kind = "method",
        summary = "`..n` — at most `n`.",
        context = "`..n` — at most `n`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(n = "`..n` — at most `n`."),
        returns = "A `ScoreRange` that emits the documented `..n` — at most `n` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(n: i32)  {\n    let score_range = sand::command::ScoreRange::at_most(n);\n}",
    )]
    pub fn at_most(n: i32) -> Self {
        Self {
            min: None,
            max: Some(n),
        }
    }

    /// `min..max` — an inclusive range.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreRange::between",
        aliases = ["sand::cmd::ScoreRange::between", "sand::prelude::cmd::ScoreRange::between"],
        module = "sand::command",
        kind = "method",
        summary = "`min..max` — an inclusive range.",
        context = "`min..max` — an inclusive range. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(min = "`min` provides the inclusive lower bound used to emit the documented `min..max` — an inclusive range form.", max = "`max` provides the inclusive upper bound used to emit the documented `min..max` — an inclusive range form."),
        returns = "A `ScoreRange` that emits the documented `min..max` — an inclusive range form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(min: i32, max: i32)  {\n    let score_range = sand::command::ScoreRange::between(min, max);\n}",
    )]
    pub fn between(min: i32, max: i32) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

impl fmt::Display for ScoreRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.min, self.max) {
            (Some(a), Some(b)) if a == b => write!(f, "{a}"),
            (Some(a), Some(b)) => write!(f, "{a}..{b}"),
            (Some(a), None) => write!(f, "{a}.."),
            (None, Some(b)) => write!(f, "..{b}"),
            (None, None) => Ok(()),
        }
    }
}

// ── GameMode ──────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::GameMode",
    aliases = ["sand::cmd::GameMode", "sand::prelude::GameMode", "sand::prelude::cmd::GameMode"],
    module = "sand::command",
    summary = "Minecraft player game mode.",
    context = "Minecraft player game mode. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::GameMode;",
    variants(Adventure = "`adventure` — survival-like with block-break restrictions.", Creative = "`creative` — infinite resources and flight.", Spectator = "`spectator` — observe-only mode.", Survival = "`survival` — normal gameplay."),
)]
/// Minecraft player game mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// `survival` — normal gameplay.
    Survival,
    /// `creative` — infinite resources and flight.
    Creative,
    /// `adventure` — survival-like with block-break restrictions.
    Adventure,
    /// `spectator` — observe-only mode.
    Spectator,
}

impl fmt::Display for GameMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameMode::Survival => write!(f, "survival"),
            GameMode::Creative => write!(f, "creative"),
            GameMode::Adventure => write!(f, "adventure"),
            GameMode::Spectator => write!(f, "spectator"),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_selectors() {
        assert_eq!(Selector::all_players().to_string(), "@a");
        assert_eq!(Selector::all_entities().to_string(), "@e");
        assert_eq!(Selector::self_().to_string(), "@s");
        assert_eq!(Selector::nearest_player().to_string(), "@p");
        assert_eq!(Selector::random_player().to_string(), "@r");
        assert_eq!(Selector::player("Steve").to_string(), "Steve");
    }

    #[test]
    fn with_args() {
        let s = Selector::all_players().tag("ready").limit(1);
        assert_eq!(s.to_string(), "@a[tag=ready,limit=1]");
    }

    #[test]
    fn multiple_args() {
        let s = Selector::all_entities()
            .entity_type("minecraft:zombie")
            .not_tag("killed")
            .limit(5);
        assert_eq!(
            s.to_string(),
            "@e[type=minecraft:zombie,tag=!killed,limit=5]"
        );
    }

    #[test]
    fn negation() {
        assert_eq!(
            Selector::all_players().not_team("red").to_string(),
            "@a[team=!red]"
        );
    }

    #[test]
    fn typed_entity_targets_render_stably() {
        let targets = Target::nearby(5.0).excluding_players().excluding_self();
        assert_eq!(
            targets.to_string(),
            "@e[distance=0.1..5,type=!minecraft:player]"
        );
    }

    #[test]
    fn many_entity_limit_converts_to_single() {
        let target = Target::entities().entity_type("minecraft:zombie").nearest();
        assert_eq!(
            target.to_string(),
            "@e[type=minecraft:zombie,sort=nearest,limit=1]"
        );
    }

    // ── Selector argument golden tests ────────────────────────────────────────

    #[test]
    fn scores_arg() {
        // scores() wraps the argument in { } automatically
        let s = Selector::all_players().scores("kills=1..10,deaths=0");
        assert_eq!(s.to_string(), "@a[scores={kills=1..10,deaths=0}]");
    }

    #[test]
    fn nbt_arg() {
        let s = Selector::all_entities().nbt("{CustomName:\"Boss\"}");
        assert_eq!(s.to_string(), r#"@e[nbt={CustomName:"Boss"}]"#);
    }

    #[test]
    fn predicate_arg() {
        let s = Selector::all_players().predicate("my_pack:is_sneaking");
        assert_eq!(s.to_string(), "@a[predicate=my_pack:is_sneaking]");
    }

    #[test]
    fn gamemode_arg() {
        let s = Selector::all_players().gamemode("survival");
        assert_eq!(s.to_string(), "@a[gamemode=survival]");
    }

    #[test]
    fn level_range_arg() {
        let s = Selector::all_players().level("10..30");
        assert_eq!(s.to_string(), "@a[level=10..30]");
    }

    #[test]
    fn distance_range_arg() {
        let s = Selector::all_entities().distance_range(0.5, 10.0);
        assert_eq!(s.to_string(), "@e[distance=0.5..10]");
    }

    #[test]
    fn distance_max_arg() {
        let s = Selector::nearest_player().distance_max(16.0);
        assert_eq!(s.to_string(), "@p[distance=..16]");
    }

    #[test]
    fn sort_random_arg() {
        let s = Selector::all_entities()
            .entity_type("minecraft:cow")
            .sort(SortOrder::Random)
            .limit(1);
        assert_eq!(s.to_string(), "@e[type=minecraft:cow,sort=random,limit=1]");
    }

    #[test]
    fn volume_box_arg() {
        let s = Selector::all_entities().volume(3.0, 1.0, 3.0);
        assert_eq!(s.to_string(), "@e[dx=3,dy=1,dz=3]");
    }

    #[test]
    fn at_pos_shifts_origin() {
        let s = Selector::all_entities().at_pos(10.0, 64.0, -20.0);
        assert_eq!(s.to_string(), "@e[x=10,y=64,z=-20]");
    }

    #[test]
    fn not_player_type_arg() {
        let s = Selector::all_entities()
            .not_player()
            .limit(3)
            .sort(SortOrder::Nearest);
        assert_eq!(
            s.to_string(),
            "@e[type=!minecraft:player,limit=3,sort=nearest]"
        );
    }

    #[test]
    fn name_and_not_name() {
        let s = Selector::all_players().name("Steve");
        assert_eq!(s.to_string(), "@a[name=Steve]");

        let s = Selector::all_players().not_name("Notch");
        assert_eq!(s.to_string(), "@a[name=!Notch]");
    }

    #[test]
    fn validation_rejects_invalid_limits_ranges_and_names() {
        assert!(Selector::all_players().limit(0).try_build().is_err());
        assert!(
            Selector::all_entities()
                .distance_range(5.0, 1.0)
                .try_build()
                .is_err()
        );
        assert!(
            Selector::all_entities()
                .distance_max(f64::NAN)
                .try_build()
                .is_err()
        );
        assert!(Selector::player("").try_build().is_err());
        assert!(Selector::player("has space").try_build().is_err());
        assert!(
            Selector::all_entities()
                .distance_max(-1.0)
                .try_build()
                .is_err()
        );
        assert!(Selector::all_players().level("-1..").try_build().is_err());
        assert!(
            Selector::all_entities()
                .nbt("{broken:[1,2}")
                .try_build()
                .is_err()
        );
        assert!(
            Selector::all_entities()
                .gamemode("!creative")
                .try_build()
                .is_ok()
        );
        assert!(
            Selector::all_entities()
                .predicate("!pack:ready")
                .try_build()
                .is_ok()
        );
        assert!(
            Selector::all_entities()
                .entity_type("#pack:mobs")
                .try_build()
                .is_ok()
        );
        assert!(Selector::all_entities().tag("").try_build().is_ok());
        assert!(Selector::self_().limit(1).try_build().is_err());
    }

    #[test]
    fn gamemode_typed_matches_string_gamemode_for_valid_input() {
        assert_eq!(
            Selector::all_players()
                .gamemode_typed(GameMode::Survival)
                .try_build()
                .unwrap(),
            Selector::all_players()
                .gamemode("survival")
                .try_build()
                .unwrap()
        );
    }

    #[test]
    fn not_gamemode_typed_renders_negation() {
        let selector = Selector::all_players().not_gamemode_typed(GameMode::Creative);
        assert_eq!(selector.try_build().unwrap(), "@a[gamemode=!creative]");
    }

    #[test]
    fn narrowing_is_fallible_and_safe_widening_remains_infallible() {
        assert!(Target::entities().limit(2).is_err());
        let entity: Target<AnyTarget, One> = Target::current_player().into();
        assert_eq!(entity.to_string(), "@s");
    }

    #[test]
    fn raw_selector_escape_hatch_remains_verbatim() {
        assert_eq!(
            Selector::raw("@e[modded_filter={x:1}]")
                .try_build()
                .unwrap(),
            "@e[modded_filter={x:1}]"
        );
    }

    // ── #200: typed selector filter helpers ───────────────────────────────

    #[test]
    fn distance_typed_matches_string_variants() {
        assert_eq!(
            Selector::all_entities()
                .distance_typed(TargetRange::at_most(16.0))
                .try_build()
                .unwrap(),
            Selector::all_entities()
                .distance_max(16.0)
                .try_build()
                .unwrap()
        );
        assert_eq!(
            Selector::all_entities()
                .distance_typed(TargetRange::between(0.5, 10.0))
                .try_build()
                .unwrap(),
            "@e[distance=0.5..10]"
        );
        assert_eq!(
            Selector::all_entities()
                .distance_typed(TargetRange::at_least(2.0))
                .try_build()
                .unwrap(),
            "@e[distance=2..]"
        );
    }

    #[test]
    fn level_typed_renders_same_as_string_level() {
        assert_eq!(
            Selector::all_players()
                .level_typed(TargetRange::between(10.0, 30.0))
                .try_build()
                .unwrap(),
            Selector::all_players().level("10..30").try_build().unwrap()
        );
    }

    #[test]
    fn selector_range_impossible_range_is_a_diagnostic_not_a_panic() {
        let err = Selector::all_entities()
            .distance_typed(TargetRange::between(10.0, 1.0))
            .try_build()
            .unwrap_err();
        assert!(err.to_string().contains("distance"), "{err}");
    }

    #[test]
    fn scores_typed_matches_string_scores() {
        let typed = Selector::all_players()
            .scores_typed([
                ("kills".to_owned(), ScoreRange::between(1, 10)),
                ("deaths".to_owned(), ScoreRange::exact(0)),
            ])
            .try_build()
            .unwrap();
        let stringly = Selector::all_players()
            .scores("kills=1..10,deaths=0")
            .try_build()
            .unwrap();
        assert_eq!(typed, stringly);
        assert_eq!(typed, "@a[scores={kills=1..10,deaths=0}]");
    }

    #[test]
    fn scores_typed_duplicate_objective_is_a_diagnostic() {
        let err = Selector::all_players()
            .scores_typed([
                ("kills".to_owned(), ScoreRange::exact(1)),
                ("kills".to_owned(), ScoreRange::exact(2)),
            ])
            .try_build()
            .unwrap_err();
        assert!(err.to_string().contains("duplicate"), "{err}");
    }

    #[test]
    fn score_range_is_integer_typed_not_float() {
        // #200 review finding: `scores={...}` values must be integers in
        // vanilla Minecraft — `ScoreRange` uses `i32` bounds so a fractional
        // score range is a compile error, not a malformed-selector
        // diagnostic discovered later at `try_build` time. This test just
        // pins the rendered (always-integer) output shape.
        assert_eq!(ScoreRange::exact(0).to_string(), "0");
        assert_eq!(ScoreRange::between(-5, 5).to_string(), "-5..5");
        assert_eq!(ScoreRange::at_least(3).to_string(), "3..");
        assert_eq!(ScoreRange::at_most(-1).to_string(), "..-1");
    }

    #[test]
    fn predicate_filters_render_stably() {
        assert_eq!(
            Selector::all_players()
                .predicate("my_pack:is_sneaking")
                .try_build()
                .unwrap(),
            Selector::all_players()
                .predicate("my_pack:is_sneaking")
                .try_build()
                .unwrap()
        );
        assert_eq!(
            Selector::all_players()
                .predicate("!my_pack:is_sneaking")
                .try_build()
                .unwrap(),
            "@a[predicate=!my_pack:is_sneaking]"
        );
    }

    #[test]
    fn tag_and_team_filters_render_stably() {
        assert_eq!(
            Selector::all_players().tag("ready").try_build().unwrap(),
            "@a[tag=ready]"
        );
        assert_eq!(
            Selector::all_players().team("red").try_build().unwrap(),
            "@a[team=red]"
        );
    }

    #[test]
    fn raw_aliases_render_identically_to_normal_methods() {
        assert_eq!(
            Selector::all_entities()
                .nbt_raw("{CustomName:\"Boss\"}")
                .to_string(),
            Selector::all_entities()
                .nbt("{CustomName:\"Boss\"}")
                .to_string()
        );
        assert_eq!(
            Selector::all_players().scores_raw("kills=1..").to_string(),
            Selector::all_players().scores("kills=1..").to_string()
        );
        assert_eq!(
            Selector::all_players()
                .predicate_raw("pack:ready")
                .to_string(),
            Selector::all_players().predicate("pack:ready").to_string()
        );
    }

    #[test]
    fn selector_construction_order_is_deterministic() {
        // #200/#173: rebuilding the same selector from scratch, in the same
        // call order, must always render identically — no run-to-run
        // variance. `Selector`'s args and the supplied score pairs preserve
        // insertion order rather than using a hasher-seeded map,
        // so this is not merely "the same closure returns the same string
        // twice": each `build()` call constructs fresh `Vec`s from scratch,
        // and a `HashMap`-backed regression (each instance gets an
        // independently randomized iteration order in std) would show up as
        // flaky inequality across iterations. Loop many times for
        // confidence instead of relying on two calls that could coincide by
        // chance.
        let build = || {
            Selector::all_entities()
                .entity_type("minecraft:zombie")
                .tag("elite")
                .distance_typed(TargetRange::at_most(20.0))
                .scores_typed([
                    ("threat".to_owned(), ScoreRange::at_least(5)),
                    ("armor".to_owned(), ScoreRange::between(0, 3)),
                    ("kills".to_owned(), ScoreRange::exact(0)),
                ])
                .limit(3)
                .to_string()
        };
        let expected = "@e[type=minecraft:zombie,tag=elite,distance=..20,scores={threat=5..,armor=0..3,kills=0},limit=3]";
        let first = build();
        assert_eq!(first, expected);
        for _ in 0..64 {
            assert_eq!(build(), first);
        }

        // Construction order is caller-controlled and semantically
        // significant for score pairs (they are not canonicalized/sorted),
        // so two *different* insertion orders are expected to render
        // differently from each other — while each remains internally
        // deterministic across repeated builds.
        let reordered = || {
            Selector::all_entities()
                .entity_type("minecraft:zombie")
                .tag("elite")
                .distance_typed(TargetRange::at_most(20.0))
                .scores_typed([
                    ("kills".to_owned(), ScoreRange::exact(0)),
                    ("armor".to_owned(), ScoreRange::between(0, 3)),
                    ("threat".to_owned(), ScoreRange::at_least(5)),
                ])
                .limit(3)
                .to_string()
        };
        let reordered_first = reordered();
        assert_ne!(reordered_first, first);
        for _ in 0..64 {
            assert_eq!(reordered(), reordered_first);
        }
    }
}
