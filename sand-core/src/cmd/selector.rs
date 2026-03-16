//! Entity/player selector (`@a`, `@e`, `@s`, etc.) with a typed builder API.

use std::fmt;

// ── Public types ──────────────────────────────────────────────────────────────

/// An entity/player selector for use in Minecraft commands.
///
/// Selectors target entities in the world. Construct with a base selector (e.g., `all_players()`)
/// then refine with builder methods to add filters (tags, distance, team, etc.).
///
/// # Examples
/// ```
/// use sand_core::cmd::Selector;
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

/// The base target variant of a selector.
#[derive(Debug, Clone, PartialEq)]
enum TargetBase {
    AllPlayers,
    AllEntities,
    NearestPlayer,
    Self_,
    RandomPlayer,
    Player(String),
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
    Distance(String), // raw range string e.g. "0..10"
    Level(String),
    XRotation(String),
    YRotation(String),
    Gamemode(String),
    Scores(String), // raw scores block e.g. "playtime=100.."
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
    ///
    /// The most common base for targeting players. Can be filtered by team, distance, tag, etc.
    pub fn all_players() -> Self {
        Self {
            base: TargetBase::AllPlayers,
            args: vec![],
        }
    }

    /// `@e` — all entities in the world, including players, mobs, armor stands, etc.
    ///
    /// Use `.not_player()` to exclude players, or `.entity_type()` to filter by specific types.
    pub fn all_entities() -> Self {
        Self {
            base: TargetBase::AllEntities,
            args: vec![],
        }
    }

    /// `@p` — the nearest player to the command executor.
    ///
    /// Returns a single player. Respects distance filters and other selector arguments.
    pub fn nearest_player() -> Self {
        Self {
            base: TargetBase::NearestPlayer,
            args: vec![],
        }
    }

    /// `@s` — the entity currently executing the command.
    ///
    /// In the context of an `execute as` chain, this refers to each iterated entity.
    /// Outside of execute context, this is the command executor (player or command block).
    pub fn self_() -> Self {
        Self {
            base: TargetBase::Self_,
            args: vec![],
        }
    }

    /// `@r` — a random player from the current players.
    ///
    /// Picks one player at random. Distribution respects filters like distance.
    pub fn random_player() -> Self {
        Self {
            base: TargetBase::RandomPlayer,
            args: vec![],
        }
    }

    /// A specific player by exact name (no selector, just the player name).
    ///
    /// Use this for targeting a single known player: `Selector::player("Steve")` → `Steve`.
    /// The player must be online for the command to affect them.
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
    ///
    /// Tags are added via `tag add` and removed via `tag remove`. An entity can have multiple tags.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Tag(tag.into()));
        self
    }

    /// `tag=!<tag>` — select only entities that do NOT have the given tag.
    ///
    /// Useful for excluding specific entities without affecting other filters.
    pub fn not_tag(mut self, tag: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotTag(tag.into()));
        self
    }

    /// `team=<team>` — select only entities on the given team.
    ///
    /// Teams are registered via `team add` and entities join via `team join`. Used for PvP and grouping.
    pub fn team(mut self, team: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Team(team.into()));
        self
    }

    /// `team=!<team>` — select only entities NOT on the given team.
    ///
    /// Useful for targeting enemies or neutral entities.
    pub fn not_team(mut self, team: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotTeam(team.into()));
        self
    }

    /// `name=<name>` — select only entities with the exact display name.
    ///
    /// The name is the custom display name set via commands or name tags, not the player username.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Name(name.into()));
        self
    }

    /// `name=!<name>` — select only entities WITHOUT the given display name.
    ///
    /// Negation of the `name` filter.
    pub fn not_name(mut self, name: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotName(name.into()));
        self
    }

    /// `type=<entity_type>` — select only entities of the given type.
    ///
    /// Types use the namespaced format: `"minecraft:zombie"`, `"minecraft:armor_stand"`, etc.
    pub fn entity_type(mut self, ty: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Type(ty.into()));
        self
    }

    /// `type=!<entity_type>` — select only entities NOT of the given type.
    ///
    /// Useful for excluding specific mobs or entities.
    pub fn not_type(mut self, ty: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotType(ty.into()));
        self
    }

    /// `limit=<n>` — select at most `n` entities.
    ///
    /// Combined with `sort`, this lets you pick the nearest/random/etc entity. Default is unlimited.
    pub fn limit(mut self, n: i32) -> Self {
        self.args.push(SelectorArg::Limit(n));
        self
    }

    /// `sort=<order>` — set the sort order before applying limit.
    ///
    /// Determines which entities are picked when using `limit`. `nearest` for closest, `random` for random, etc.
    pub fn sort(mut self, order: SortOrder) -> Self {
        self.args.push(SelectorArg::Sort(order));
        self
    }

    /// `distance=<range>` — select only entities within a distance range.
    ///
    /// Range format: `"5"` (exact), `"5.."` (5 or more), `"..5"` (5 or less), `"1..10"` (between).
    pub fn distance(mut self, range: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Distance(range.into()));
        self
    }

    /// `distance=..<max>` — select only entities at most `max` blocks away.
    ///
    /// Convenience method; equivalent to `.distance(format!("..{max}"))`.
    pub fn distance_max(mut self, max: f64) -> Self {
        self.args.push(SelectorArg::Distance(format!("..{max}")));
        self
    }

    /// `distance=<min>..` — select only entities at least `min` blocks away.
    ///
    /// Convenience method; equivalent to `.distance(format!("{min}.."))`.
    pub fn distance_min(mut self, min: f64) -> Self {
        self.args.push(SelectorArg::Distance(format!("{min}..")));
        self
    }

    /// `distance=<min>..<max>` — select only entities between `min` and `max` blocks away.
    ///
    /// Convenience method; equivalent to `.distance(format!("{min}..{max}"))`.
    pub fn distance_range(mut self, min: f64, max: f64) -> Self {
        self.args
            .push(SelectorArg::Distance(format!("{min}..{max}")));
        self
    }

    /// `type=!minecraft:player` — exclude all players from the selection.
    ///
    /// Shorthand for `.not_type("minecraft:player")`. Use with `@e` to select only mobs/objects.
    pub fn not_player(mut self) -> Self {
        self.args
            .push(SelectorArg::NotType("minecraft:player".into()));
        self
    }

    /// `level=<range>` — select only players within the given XP level range.
    ///
    /// Range format: `"0..10"`, `"10"`, `"10.."`, etc. Only applies to players.
    pub fn level(mut self, range: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Level(range.into()));
        self
    }

    /// `gamemode=<mode>` — select only players in the given gamemode.
    ///
    /// Valid modes: `"survival"`, `"creative"`, `"adventure"`, `"spectator"`. Only applies to players.
    pub fn gamemode(mut self, mode: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Gamemode(mode.into()));
        self
    }

    /// `scores=<objective>=<range>` — select only entities with matching scoreboard score.
    ///
    /// Example: `.scores("damage=10..20")` selects entities with damage score between 10 and 20.
    pub fn scores(mut self, scores: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Scores(scores.into()));
        self
    }

    /// `nbt=<nbt>` — select only entities matching the given NBT compound.
    ///
    /// NBT format: `"{Health:10f}"` or `"{Custom:{Value:42}}"`. Matches entities with matching NBT.
    pub fn nbt(mut self, nbt: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Nbt(nbt.into()));
        self
    }

    /// `predicate=<predicate>` — select only entities matching a loot table predicate.
    ///
    /// Predicates are defined in JSON files and can check complex conditions.
    pub fn predicate(mut self, predicate: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Predicate(predicate.into()));
        self
    }

    /// `dx/dy/dz` — set a bounding box volume filter (half-extents).
    ///
    /// All three must be set together. Used with `at_pos` to define an axis-aligned cuboid.
    /// Entities within the box relative to the origin are selected.
    pub fn volume(mut self, dx: f64, dy: f64, dz: f64) -> Self {
        self.args.push(SelectorArg::Dx(dx));
        self.args.push(SelectorArg::Dy(dy));
        self.args.push(SelectorArg::Dz(dz));
        self
    }

    /// `x/y/z` — set the origin point for distance and volume checks.
    ///
    /// All three coordinates must be set together. If not set, the executor's position is used.
    pub fn at_pos(mut self, x: f64, y: f64, z: f64) -> Self {
        self.args.push(SelectorArg::X(x));
        self.args.push(SelectorArg::Y(y));
        self.args.push(SelectorArg::Z(z));
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
}
