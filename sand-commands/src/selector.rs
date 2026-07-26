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
/// typed vanilla/custom entity-type identifiers: `sand_core::generated::EntityType`
/// (generated vanilla entity types, e.g. `Marker`) and
/// `sand_components::registry::EntityTypeId` (validated custom/modded IDs).
/// Prefer the typed identifiers in normal code; the string forms remain for
/// compatibility and cases with no typed representation yet.
pub trait IntoEntityType {
    /// Convert to the entity type's resource location, e.g. `"minecraft:marker"`.
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
/// use sand_commands::selector::Selector;
///
/// // @a[tag=ready,limit=1]
/// let sel = Selector::all_players().tag("ready").limit(1);
/// assert_eq!(sel.to_string(), "@a[tag=ready,limit=1]");
///
/// // @s
/// assert_eq!(Selector::self_().to_string(), "@s");
/// ```
#[derive(Debug, Clone)]
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
pub enum TargetBase {
    AllPlayers,
    AllEntities,
    NearestPlayer,
    Self_,
    RandomPlayer,
    Player(String),
    /// Explicit unchecked selector syntax for advanced/modded grammar.
    Raw(String),
}

/// Marker for selector wrappers that are statically known to select one target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum One {}

/// Marker for selector wrappers that may select multiple targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Many {}

/// Entity selector with statically modeled arity.
#[derive(Debug, Clone)]
#[must_use = "targets do nothing until passed to a command"]
pub struct EntityTarget<A> {
    raw: Selector,
    _arity: PhantomData<A>,
}

/// Player selector with statically modeled arity.
#[derive(Debug, Clone)]
#[must_use = "targets do nothing until passed to a command"]
pub struct PlayerTarget<A> {
    raw: Selector,
    _arity: PhantomData<A>,
}

/// An entity target that resolves to at most one entity.
pub type SingleEntity = EntityTarget<One>;

/// An entity target that may resolve to zero or more entities.
pub type EntityTargets = EntityTarget<Many>;

/// A player target that resolves to at most one player.
pub type SinglePlayer = PlayerTarget<One>;

/// A player target that may resolve to zero or more players.
pub type PlayerTargets = PlayerTarget<Many>;

impl<A> EntityTarget<A> {
    /// Access the underlying selector.
    pub fn selector(&self) -> &Selector {
        &self.raw
    }

    /// Convert this typed target into the underlying selector.
    pub fn into_selector(self) -> Selector {
        self.raw
    }

    /// `tag=<tag>` — select only entities that have the given tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.raw = self.raw.tag(tag);
        self
    }

    /// `tag=!<tag>` — select only entities that do NOT have the given tag.
    pub fn not_tag(mut self, tag: impl Into<String>) -> Self {
        self.raw = self.raw.not_tag(tag);
        self
    }

    /// `type=<entity_type>` — select only entities of the given type.
    pub fn entity_type(mut self, ty: impl IntoEntityType) -> Self {
        self.raw = self.raw.entity_type(ty);
        self
    }

    /// `type=!<entity_type>` — select only entities NOT of the given type.
    pub fn not_type(mut self, ty: impl IntoEntityType) -> Self {
        self.raw = self.raw.not_type(ty);
        self
    }

    /// `type=!minecraft:player` — exclude players from the target set.
    pub fn excluding_players(self) -> Self {
        self.not_type("minecraft:player")
    }

    /// Add one typed scoreboard filter without formatting a selector score map.
    pub fn score(
        mut self,
        objective: crate::ObjectiveName,
        range: ScoreRange,
    ) -> CommandResult<Self> {
        self.raw = self.raw.score_typed(objective, range)?;
        Ok(self)
    }

    /// `distance=0.1..` — exclude the current executor when centered at `@s`.
    pub fn excluding_self(mut self) -> Self {
        self.raw = self.raw.exclude_self_distance();
        self
    }

    /// `distance=..<max>` — select targets within `max` blocks.
    pub fn within_blocks(mut self, max: f64) -> Self {
        self.raw = self.raw.distance_max(max);
        self
    }

    /// `distance=<range>` — select only entities within a distance range.
    pub fn distance(mut self, range: impl Into<String>) -> Self {
        self.raw = self.raw.distance(range);
        self
    }

    /// `distance=<min>..<max>` — select only entities between `min` and `max`.
    pub fn distance_range(mut self, min: f64, max: f64) -> Self {
        self.raw = self.raw.distance_range(min, max);
        self
    }
}

impl<A> Validate for EntityTarget<A> {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.raw.validate(profile)
    }
}

impl<A> RenderCommand for EntityTarget<A> {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

impl EntityTargets {
    /// `@e` — all entities.
    pub fn all() -> Self {
        Self::from_selector(Selector::all_entities())
    }

    /// `@e[distance=..<radius>]` — all entities within a radius of the executor.
    pub fn nearby(radius: f64) -> Self {
        Self::all().within_blocks(radius)
    }

    /// Add `limit=1` and convert to a single-entity target.
    pub fn limit(mut self, n: i32) -> CommandResult<SingleEntity> {
        if n != 1 {
            return Err(CommandError::new(
                "EntityTargets::limit",
                "limit",
                format!("single-entity narrowing requires `limit=1`, got `{n}`"),
            ));
        }
        self.raw = self.raw.limit(n);
        Ok(SingleEntity::from_selector(self.raw))
    }

    /// Pick the nearest matching entity as a single target.
    pub fn nearest(mut self) -> SingleEntity {
        self.raw = self.raw.sort(SortOrder::Nearest).limit(1);
        SingleEntity::from_selector(self.raw)
    }
}

impl SingleEntity {
    /// `@s` — the current executor as a single entity.
    pub fn self_() -> Self {
        Self::from_selector(Selector::self_())
    }

    /// Explicit unchecked single-entity selector syntax.
    ///
    /// This opts out of Sand's cardinality proof. Use only when advanced or
    /// modded syntax guarantees zero or one result.
    pub fn raw(selector: impl Into<String>) -> Self {
        Self::from_selector(Selector::raw(selector))
    }
}

impl<A> PlayerTarget<A> {
    /// Access the underlying selector.
    pub fn selector(&self) -> &Selector {
        &self.raw
    }

    /// Convert this typed target into the underlying selector.
    pub fn into_selector(self) -> Selector {
        self.raw
    }

    /// `tag=<tag>` — select only players that have the given tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.raw = self.raw.tag(tag);
        self
    }

    /// `tag=!<tag>` — select only players that do NOT have the given tag.
    pub fn not_tag(mut self, tag: impl Into<String>) -> Self {
        self.raw = self.raw.not_tag(tag);
        self
    }

    /// `distance=..<max>` — select players within `max` blocks.
    pub fn within_blocks(mut self, max: f64) -> Self {
        self.raw = self.raw.distance_max(max);
        self
    }

    /// `distance=<min>..<max>` — select only players between `min` and `max`.
    pub fn distance_range(mut self, min: f64, max: f64) -> Self {
        self.raw = self.raw.distance_range(min, max);
        self
    }
}

impl<A> Validate for PlayerTarget<A> {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.raw.validate(profile)
    }
}

impl<A> RenderCommand for PlayerTarget<A> {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

impl PlayerTargets {
    /// `@a` — all players.
    pub fn all() -> Self {
        Self::from_selector(Selector::all_players())
    }

    /// Add `limit=1` and convert to a single-player target.
    pub fn limit(mut self, n: i32) -> CommandResult<SinglePlayer> {
        if n != 1 {
            return Err(CommandError::new(
                "PlayerTargets::limit",
                "limit",
                format!("single-player narrowing requires `limit=1`, got `{n}`"),
            ));
        }
        self.raw = self.raw.limit(n);
        Ok(SinglePlayer::from_selector(self.raw))
    }

    /// Pick the nearest matching player as a single target.
    pub fn nearest(mut self) -> SinglePlayer {
        self.raw = self.raw.sort(SortOrder::Nearest).limit(1);
        SinglePlayer::from_selector(self.raw)
    }
}

impl SinglePlayer {
    /// `@s` — the current executor as a single player.
    pub fn self_() -> Self {
        Self::from_selector(Selector::self_())
    }

    /// `@p` — the nearest player.
    pub fn nearest() -> Self {
        Self::from_selector(Selector::nearest_player())
    }

    /// Explicit unchecked single-player selector syntax.
    pub fn raw(selector: impl Into<String>) -> Self {
        Self::from_selector(Selector::raw(selector))
    }
}

impl SingleEntity {
    fn from_selector(raw: Selector) -> Self {
        Self {
            raw,
            _arity: PhantomData,
        }
    }
}

impl EntityTargets {
    fn from_selector(raw: Selector) -> Self {
        Self {
            raw,
            _arity: PhantomData,
        }
    }
}

impl SinglePlayer {
    fn from_selector(raw: Selector) -> Self {
        Self {
            raw,
            _arity: PhantomData,
        }
    }
}

impl PlayerTargets {
    fn from_selector(raw: Selector) -> Self {
        Self {
            raw,
            _arity: PhantomData,
        }
    }
}

impl TryFrom<Selector> for SingleEntity {
    type Error = CommandError;
    fn try_from(raw: Selector) -> CommandResult<Self> {
        raw.validate_single("SingleEntity")?;
        Ok(Self::from_selector(raw))
    }
}

impl TryFrom<Selector> for EntityTargets {
    type Error = CommandError;
    fn try_from(raw: Selector) -> CommandResult<Self> {
        raw.validate(&CommandProfile::unprofiled())?;
        Ok(Self::from_selector(raw))
    }
}

impl TryFrom<Selector> for SinglePlayer {
    type Error = CommandError;
    fn try_from(raw: Selector) -> CommandResult<Self> {
        raw.validate_player("SinglePlayer")?;
        raw.validate_single("SinglePlayer")?;
        Ok(Self::from_selector(raw))
    }
}

impl TryFrom<Selector> for PlayerTargets {
    type Error = CommandError;
    fn try_from(raw: Selector) -> CommandResult<Self> {
        raw.validate_player("PlayerTargets")?;
        Ok(Self::from_selector(raw))
    }
}

impl From<SinglePlayer> for SingleEntity {
    fn from(player: SinglePlayer) -> Self {
        SingleEntity::from_selector(player.raw)
    }
}

impl From<PlayerTargets> for EntityTargets {
    fn from(players: PlayerTargets) -> Self {
        EntityTargets::from_selector(players.raw)
    }
}

impl<A> fmt::Display for EntityTarget<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl<A> fmt::Display for PlayerTarget<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

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
    /// matching scoreboard scores, built from typed [`SelectorScores`]
    /// entries instead of a hand-formatted string.
    ///
    /// ```
    /// use sand_commands::selector::{Selector, SelectorScores, ScoreRange};
    ///
    /// let sel = Selector::all_players().scores_typed(
    ///     SelectorScores::new()
    ///         .with("kills", ScoreRange::between(1, 10))
    ///         .with("deaths", ScoreRange::exact(0)),
    /// );
    /// assert_eq!(sel.to_string(), "@a[scores={kills=1..10,deaths=0}]");
    /// ```
    pub fn scores_typed(mut self, scores: SelectorScores) -> Self {
        self.args.push(SelectorArg::Scores(scores.to_string()));
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
    /// [`Selector::predicate_id`] in normal code — see
    /// [#200](https://github.com/ThatOneToast/sand/issues/200). Equivalent to
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

    /// `predicate=<id>` — select only entities matching a loot table
    /// predicate, using a typed [`PredicateId`] instead of a raw string.
    ///
    /// ```
    /// use sand_commands::selector::{Selector, PredicateId};
    ///
    /// let sel = Selector::all_players().predicate_id(PredicateId::new("my_pack:is_sneaking"));
    /// assert_eq!(sel.to_string(), "@a[predicate=my_pack:is_sneaking]");
    /// ```
    pub fn predicate_id(mut self, id: PredicateId) -> Self {
        self.args.push(SelectorArg::Predicate(id.to_string()));
        self
    }

    /// `distance=<range>` — select only entities within a typed distance
    /// range, using [`SelectorRange`] instead of a hand-formatted string.
    ///
    /// ```
    /// use sand_commands::selector::{Selector, SelectorRange};
    ///
    /// let sel = Selector::all_entities().distance_typed(SelectorRange::at_most(16.0));
    /// assert_eq!(sel.to_string(), "@e[distance=..16]");
    /// ```
    pub fn distance_typed(mut self, range: SelectorRange) -> Self {
        self.args.push(SelectorArg::Distance(range.to_string()));
        self
    }

    /// `level=<range>` — select only players within a typed XP level range,
    /// using [`SelectorRange`] instead of a hand-formatted string.
    ///
    /// ```
    /// use sand_commands::selector::{Selector, SelectorRange};
    ///
    /// let sel = Selector::all_players().level_typed(SelectorRange::between(10.0, 30.0));
    /// assert_eq!(sel.to_string(), "@a[level=10..30]");
    /// ```
    pub fn level_typed(mut self, range: SelectorRange) -> Self {
        self.args.push(SelectorArg::Level(range.to_string()));
        self
    }

    /// `tag=<tag>` — select only entities with the given tag, using a typed
    /// [`EntityTag`] instead of a raw string.
    pub fn tag_typed(mut self, tag: EntityTag) -> Self {
        self.args.push(SelectorArg::Tag(tag.into_inner()));
        self
    }

    /// `team=<team>` — select only entities on the given team, using a typed
    /// [`TeamName`] instead of a raw string.
    pub fn team_typed(mut self, team: TeamName) -> Self {
        self.args.push(SelectorArg::Team(team.into_inner()));
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

    fn validate_single(&self, helper: &'static str) -> CommandResult<()> {
        self.validate(&CommandProfile::unprofiled())?;
        if self.is_statically_single() {
            Ok(())
        } else {
            Err(CommandError::new(
                helper,
                "selector",
                "target may match multiple entities; add `limit=1` or use a many-target type",
            ))
        }
    }

    fn validate_player(&self, helper: &'static str) -> CommandResult<()> {
        self.validate(&CommandProfile::unprofiled())?;
        if matches!(
            self.base,
            TargetBase::AllPlayers
                | TargetBase::NearestPlayer
                | TargetBase::Self_
                | TargetBase::RandomPlayer
                | TargetBase::Player(_)
        ) {
            Ok(())
        } else {
            Err(CommandError::new(
                helper,
                "selector",
                "selector is not statically player-targeting",
            ))
        }
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

// ── SelectorRange ─────────────────────────────────────────────────────────────

/// A typed numeric range for selector arguments such as `distance` and
/// `level` (see [#200](https://github.com/ThatOneToast/sand/issues/200)).
///
/// Renders to vanilla's `min..max` range syntax. At least one bound must be
/// present; use [`SelectorRange::at_least`]/[`SelectorRange::at_most`] for
/// open-ended ranges. Impossible ranges (`min > max`) and non-finite bounds
/// are not rejected at construction — they are diagnosed uniformly with all
/// other selector arguments at [`Selector::try_build`] time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectorRange {
    min: Option<f64>,
    max: Option<f64>,
}

impl SelectorRange {
    /// An exact value: `n..n`, rendered as `n`.
    pub fn exact(n: f64) -> Self {
        Self {
            min: Some(n),
            max: Some(n),
        }
    }

    /// `n..` — at least `n`.
    pub fn at_least(n: f64) -> Self {
        Self {
            min: Some(n),
            max: None,
        }
    }

    /// `..n` — at most `n`.
    pub fn at_most(n: f64) -> Self {
        Self {
            min: None,
            max: Some(n),
        }
    }

    /// `min..max` — an inclusive range.
    pub fn between(min: f64, max: f64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

impl fmt::Display for SelectorRange {
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
/// Deliberately distinct from [`SelectorRange`]: Minecraft scoreboard scores
/// are always 32-bit integers, so `scores={obj=1.5..3.2}` is not legal
/// vanilla syntax even though the same `min..max` grammar shape is used for
/// `distance`/`level` (which *are* floating-point). Using an `i32`-based
/// type here at the API boundary makes a fractional score range a compile
/// error instead of a malformed-selector diagnostic discovered at
/// `try_build` time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreRange {
    min: Option<i32>,
    max: Option<i32>,
}

impl ScoreRange {
    /// An exact value: `n..n`, rendered as `n`.
    pub fn exact(n: i32) -> Self {
        Self {
            min: Some(n),
            max: Some(n),
        }
    }

    /// `n..` — at least `n`.
    pub fn at_least(n: i32) -> Self {
        Self {
            min: Some(n),
            max: None,
        }
    }

    /// `..n` — at most `n`.
    pub fn at_most(n: i32) -> Self {
        Self {
            min: None,
            max: Some(n),
        }
    }

    /// `min..max` — an inclusive range.
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

// ── SelectorScores ───────────────────────────────────────────────────────────

/// A typed `scores={...}` selector filter map (see
/// [#200](https://github.com/ThatOneToast/sand/issues/200)).
///
/// Entries are rendered in insertion order, so equivalent construction order
/// always produces an identical rendered selector. Values are [`ScoreRange`]
/// (integer), not [`SelectorRange`] (float) — scoreboard scores are always
/// integers in vanilla Minecraft.
///
/// ```
/// use sand_commands::selector::{SelectorScores, ScoreRange};
///
/// let scores = SelectorScores::new()
///     .with("kills", ScoreRange::between(1, 10))
///     .with("deaths", ScoreRange::exact(0));
/// assert_eq!(scores.to_string(), "kills=1..10,deaths=0");
/// ```
#[derive(Debug, Clone, Default)]
pub struct SelectorScores {
    entries: Vec<(String, ScoreRange)>,
}

impl SelectorScores {
    /// Create an empty score filter map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `objective=range` to this filter map.
    pub fn with(mut self, objective: impl Into<String>, range: ScoreRange) -> Self {
        self.entries.push((objective.into(), range));
        self
    }
}

impl fmt::Display for SelectorScores {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = self
            .entries
            .iter()
            .map(|(objective, range)| format!("{objective}={range}"))
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{rendered}")
    }
}

// ── PredicateId ──────────────────────────────────────────────────────────────

/// A typed `predicate=<namespace:path>` identifier (see
/// [#200](https://github.com/ThatOneToast/sand/issues/200)).
///
/// Resource-location shape is validated at [`Selector::try_build`] time
/// (consistent with [`Selector::predicate`]/[`Selector::predicate_raw`]),
/// not at construction, so this stays const/static-friendly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredicateId {
    location: String,
    negated: bool,
}

impl PredicateId {
    /// Create a predicate ID from a `namespace:path` resource location.
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            negated: false,
        }
    }

    /// `predicate=!<id>` — negate this predicate filter.
    pub fn negated(mut self) -> Self {
        self.negated = true;
        self
    }
}

impl fmt::Display for PredicateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.negated {
            write!(f, "!{}", self.location)
        } else {
            write!(f, "{}", self.location)
        }
    }
}

// ── EntityTag / TeamName ─────────────────────────────────────────────────────

/// A typed selector `tag` value (see
/// [#200](https://github.com/ThatOneToast/sand/issues/200)). Whitespace/
/// control-character validity is checked at [`Selector::try_build`] time,
/// matching [`Selector::tag`]'s existing validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTag(String);

impl EntityTag {
    /// Wrap a tag value.
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for EntityTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A typed selector `team` value (see
/// [#200](https://github.com/ThatOneToast/sand/issues/200)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamName(String);

impl TeamName {
    /// Wrap a team name.
    pub fn new(team: impl Into<String>) -> Self {
        Self(team.into())
    }

    fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for TeamName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── GameMode ──────────────────────────────────────────────────────────────────

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
        let targets = EntityTargets::nearby(5.0)
            .excluding_players()
            .excluding_self();
        assert_eq!(
            targets.to_string(),
            "@e[distance=0.1..5,type=!minecraft:player]"
        );
    }

    #[test]
    fn many_entity_limit_converts_to_single() {
        let target = EntityTargets::all()
            .entity_type("minecraft:zombie")
            .nearest();
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
        assert!(SingleEntity::try_from(Selector::all_entities()).is_err());
        assert!(SinglePlayer::try_from(Selector::all_entities().limit(1)).is_err());
        assert!(SingleEntity::try_from(Selector::all_entities().limit(1)).is_ok());
        assert!(EntityTargets::all().limit(2).is_err());
        let entity: SingleEntity = SinglePlayer::self_().into();
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
                .distance_typed(SelectorRange::at_most(16.0))
                .try_build()
                .unwrap(),
            Selector::all_entities()
                .distance_max(16.0)
                .try_build()
                .unwrap()
        );
        assert_eq!(
            Selector::all_entities()
                .distance_typed(SelectorRange::between(0.5, 10.0))
                .try_build()
                .unwrap(),
            "@e[distance=0.5..10]"
        );
        assert_eq!(
            Selector::all_entities()
                .distance_typed(SelectorRange::at_least(2.0))
                .try_build()
                .unwrap(),
            "@e[distance=2..]"
        );
    }

    #[test]
    fn level_typed_renders_same_as_string_level() {
        assert_eq!(
            Selector::all_players()
                .level_typed(SelectorRange::between(10.0, 30.0))
                .try_build()
                .unwrap(),
            Selector::all_players().level("10..30").try_build().unwrap()
        );
    }

    #[test]
    fn selector_range_impossible_range_is_a_diagnostic_not_a_panic() {
        let err = Selector::all_entities()
            .distance_typed(SelectorRange::between(10.0, 1.0))
            .try_build()
            .unwrap_err();
        assert!(err.to_string().contains("distance"), "{err}");
    }

    #[test]
    fn scores_typed_matches_string_scores() {
        let typed = Selector::all_players()
            .scores_typed(
                SelectorScores::new()
                    .with("kills", ScoreRange::between(1, 10))
                    .with("deaths", ScoreRange::exact(0)),
            )
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
            .scores_typed(
                SelectorScores::new()
                    .with("kills", ScoreRange::exact(1))
                    .with("kills", ScoreRange::exact(2)),
            )
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
    fn predicate_id_matches_string_predicate() {
        assert_eq!(
            Selector::all_players()
                .predicate_id(PredicateId::new("my_pack:is_sneaking"))
                .try_build()
                .unwrap(),
            Selector::all_players()
                .predicate("my_pack:is_sneaking")
                .try_build()
                .unwrap()
        );
        assert_eq!(
            Selector::all_players()
                .predicate_id(PredicateId::new("my_pack:is_sneaking").negated())
                .try_build()
                .unwrap(),
            "@a[predicate=!my_pack:is_sneaking]"
        );
    }

    #[test]
    fn tag_typed_and_team_typed_match_string_variants() {
        assert_eq!(
            Selector::all_players()
                .tag_typed(EntityTag::new("ready"))
                .try_build()
                .unwrap(),
            Selector::all_players().tag("ready").try_build().unwrap()
        );
        assert_eq!(
            Selector::all_players()
                .team_typed(TeamName::new("red"))
                .try_build()
                .unwrap(),
            Selector::all_players().team("red").try_build().unwrap()
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
        // variance. `Selector`'s args and `SelectorScores`'s entries are
        // both backed by `Vec` (insertion order), not a hasher-seeded map,
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
                .distance_typed(SelectorRange::at_most(20.0))
                .scores_typed(
                    SelectorScores::new()
                        .with("threat", ScoreRange::at_least(5))
                        .with("armor", ScoreRange::between(0, 3))
                        .with("kills", ScoreRange::exact(0)),
                )
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
        // significant for `SelectorScores` (it is not canonicalized/sorted),
        // so two *different* insertion orders are expected to render
        // differently from each other — while each remains internally
        // deterministic across repeated builds.
        let reordered = || {
            Selector::all_entities()
                .entity_type("minecraft:zombie")
                .tag("elite")
                .distance_typed(SelectorRange::at_most(20.0))
                .scores_typed(
                    SelectorScores::new()
                        .with("kills", ScoreRange::exact(0))
                        .with("armor", ScoreRange::between(0, 3))
                        .with("threat", ScoreRange::at_least(5)),
                )
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
