//! Entity/player selector (`@a`, `@e`, `@s`, etc.) with a typed builder API.

use std::fmt;
use std::marker::PhantomData;

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
pub struct Selector {
    base: TargetBase,
    args: Vec<SelectorArg>,
}

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
}

/// Marker for selector wrappers that are statically known to select one target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum One {}

/// Marker for selector wrappers that may select multiple targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Many {}

/// Entity selector with statically modeled arity.
#[derive(Debug, Clone)]
pub struct EntityTarget<A> {
    raw: Selector,
    _arity: PhantomData<A>,
}

/// Player selector with statically modeled arity.
#[derive(Debug, Clone)]
pub struct PlayerTarget<A> {
    raw: Selector,
    _arity: PhantomData<A>,
}

/// Exactly one entity target.
pub type SingleEntity = EntityTarget<One>;

/// One or more entity targets.
pub type EntityTargets = EntityTarget<Many>;

/// Exactly one player target.
pub type SinglePlayer = PlayerTarget<One>;

/// One or more player targets.
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
    pub fn entity_type(mut self, ty: impl Into<String>) -> Self {
        self.raw = self.raw.entity_type(ty);
        self
    }

    /// `type=!<entity_type>` — select only entities NOT of the given type.
    pub fn not_type(mut self, ty: impl Into<String>) -> Self {
        self.raw = self.raw.not_type(ty);
        self
    }

    /// `type=!minecraft:player` — exclude players from the target set.
    pub fn excluding_players(self) -> Self {
        self.not_type("minecraft:player")
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

impl EntityTargets {
    /// `@e` — all entities.
    pub fn all() -> Self {
        Self::from(Selector::all_entities())
    }

    /// `@e[distance=..<radius>]` — all entities within a radius of the executor.
    pub fn nearby(radius: f64) -> Self {
        Self::all().within_blocks(radius)
    }

    /// Add `limit=1` and convert to a single-entity target.
    pub fn limit(mut self, n: i32) -> SingleEntity {
        self.raw = self.raw.limit(n);
        SingleEntity::from(self.raw)
    }

    /// Pick the nearest matching entity as a single target.
    pub fn nearest(mut self) -> SingleEntity {
        self.raw = self.raw.sort(SortOrder::Nearest).limit(1);
        SingleEntity::from(self.raw)
    }
}

impl SingleEntity {
    /// `@s` — the current executor as a single entity.
    pub fn self_() -> Self {
        Self::from(Selector::self_())
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

impl PlayerTargets {
    /// `@a` — all players.
    pub fn all() -> Self {
        Self::from(Selector::all_players())
    }

    /// Add `limit=1` and convert to a single-player target.
    pub fn limit(mut self, n: i32) -> SinglePlayer {
        self.raw = self.raw.limit(n);
        SinglePlayer::from(self.raw)
    }

    /// Pick the nearest matching player as a single target.
    pub fn nearest(mut self) -> SinglePlayer {
        self.raw = self.raw.sort(SortOrder::Nearest).limit(1);
        SinglePlayer::from(self.raw)
    }
}

impl SinglePlayer {
    /// `@s` — the current executor as a single player.
    pub fn self_() -> Self {
        Self::from(Selector::self_())
    }

    /// `@p` — the nearest player.
    pub fn nearest() -> Self {
        Self::from(Selector::nearest_player())
    }
}

impl From<Selector> for SingleEntity {
    fn from(raw: Selector) -> Self {
        Self {
            raw,
            _arity: PhantomData,
        }
    }
}

impl From<Selector> for EntityTargets {
    fn from(raw: Selector) -> Self {
        Self {
            raw,
            _arity: PhantomData,
        }
    }
}

impl From<Selector> for SinglePlayer {
    fn from(raw: Selector) -> Self {
        Self {
            raw,
            _arity: PhantomData,
        }
    }
}

impl From<Selector> for PlayerTargets {
    fn from(raw: Selector) -> Self {
        Self {
            raw,
            _arity: PhantomData,
        }
    }
}

impl From<SinglePlayer> for SingleEntity {
    fn from(player: SinglePlayer) -> Self {
        SingleEntity::from(player.raw)
    }
}

impl From<PlayerTargets> for EntityTargets {
    fn from(players: PlayerTargets) -> Self {
        EntityTargets::from(players.raw)
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
    pub fn entity_type(mut self, ty: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Type(ty.into()));
        self
    }

    /// `type=!<entity_type>` — select only entities NOT of the given type.
    pub fn not_type(mut self, ty: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotType(ty.into()));
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
    pub fn gamemode(mut self, mode: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Gamemode(mode.into()));
        self
    }

    /// `scores=<objective>=<range>` — select only entities with matching scoreboard score.
    pub fn scores(mut self, scores: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Scores(scores.into()));
        self
    }

    /// `nbt=<nbt>` — select only entities matching the given NBT compound.
    pub fn nbt(mut self, nbt: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Nbt(nbt.into()));
        self
    }

    /// `predicate=<predicate>` — select only entities matching a loot table predicate.
    pub fn predicate(mut self, predicate: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Predicate(predicate.into()));
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
}
