//! Entity/player selector (`@a`, `@e`, `@s`, etc.) with a typed builder API.

use std::fmt;

// ── Public types ──────────────────────────────────────────────────────────────

/// An entity/player selector for use in Minecraft commands.
///
/// Constructed via static factory methods and refined with builder methods.
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

/// The base target of a selector.
#[derive(Debug, Clone, PartialEq)]
enum TargetBase {
    AllPlayers,
    AllEntities,
    NearestPlayer,
    Self_,
    RandomPlayer,
    Player(String),
}

/// Sort order for `@a`/`@e` selectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Nearest,
    Furthest,
    Random,
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
    Distance(String),    // raw range string e.g. "0..10"
    Level(String),
    XRotation(String),
    YRotation(String),
    Gamemode(String),
    Scores(String),      // raw scores block e.g. "playtime=100.."
    Nbt(String),
    Predicate(String),
    X(f64), Y(f64), Z(f64),
    Dx(f64), Dy(f64), Dz(f64),
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
    /// `@a` — all players.
    pub fn all_players() -> Self {
        Self { base: TargetBase::AllPlayers, args: vec![] }
    }

    /// `@e` — all entities (including players).
    pub fn all_entities() -> Self {
        Self { base: TargetBase::AllEntities, args: vec![] }
    }

    /// `@p` — nearest player.
    pub fn nearest_player() -> Self {
        Self { base: TargetBase::NearestPlayer, args: vec![] }
    }

    /// `@s` — the entity executing the command.
    pub fn self_() -> Self {
        Self { base: TargetBase::Self_, args: vec![] }
    }

    /// `@r` — a random player.
    pub fn random_player() -> Self {
        Self { base: TargetBase::RandomPlayer, args: vec![] }
    }

    /// A specific player by name.
    pub fn player(name: impl Into<String>) -> Self {
        Self { base: TargetBase::Player(name.into()), args: vec![] }
    }
}

// ── Builder methods ───────────────────────────────────────────────────────────

impl Selector {
    /// `tag=<tag>` — only entities with this tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Tag(tag.into())); self
    }

    /// `tag=!<tag>` — only entities WITHOUT this tag.
    pub fn not_tag(mut self, tag: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotTag(tag.into())); self
    }

    /// `team=<team>` — only entities on this team.
    pub fn team(mut self, team: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Team(team.into())); self
    }

    /// `team=!<team>` — only entities NOT on this team.
    pub fn not_team(mut self, team: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotTeam(team.into())); self
    }

    /// `name=<name>` — only entities with this display name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Name(name.into())); self
    }

    /// `name=!<name>` — only entities WITHOUT this display name.
    pub fn not_name(mut self, name: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotName(name.into())); self
    }

    /// `type=<entity_type>` — only entities of this type (e.g. `"minecraft:zombie"`).
    pub fn entity_type(mut self, ty: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Type(ty.into())); self
    }

    /// `type=!<entity_type>` — only entities NOT of this type.
    pub fn not_type(mut self, ty: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotType(ty.into())); self
    }

    /// `limit=<n>` — at most `n` entities.
    pub fn limit(mut self, n: i32) -> Self {
        self.args.push(SelectorArg::Limit(n)); self
    }

    /// `sort=<order>` — sort order before applying `limit`.
    pub fn sort(mut self, order: SortOrder) -> Self {
        self.args.push(SelectorArg::Sort(order)); self
    }

    /// `distance=<range>` — only entities within the given distance range (e.g. `"0..10"`).
    pub fn distance(mut self, range: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Distance(range.into())); self
    }

    /// `distance=..<max>` — only entities at most `max` blocks away.
    ///
    /// Equivalent to `.distance(format!("..{max}"))` but typed.
    pub fn distance_max(mut self, max: f64) -> Self {
        self.args.push(SelectorArg::Distance(format!("..{max}"))); self
    }

    /// `distance=<min>..` — only entities at least `min` blocks away.
    pub fn distance_min(mut self, min: f64) -> Self {
        self.args.push(SelectorArg::Distance(format!("{min}.."))); self
    }

    /// `distance=<min>..<max>` — only entities between `min` and `max` blocks away.
    pub fn distance_range(mut self, min: f64, max: f64) -> Self {
        self.args.push(SelectorArg::Distance(format!("{min}..{max}"))); self
    }

    /// `type=!minecraft:player` — exclude players from the selection.
    ///
    /// Useful with `@e` to select only non-player entities.
    pub fn not_player(mut self) -> Self {
        self.args.push(SelectorArg::NotType("minecraft:player".into())); self
    }

    /// `level=<range>` — only players with XP level in range (e.g. `"0..10"`).
    pub fn level(mut self, range: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Level(range.into())); self
    }

    /// `gamemode=<mode>` — only players in the given gamemode.
    pub fn gamemode(mut self, mode: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Gamemode(mode.into())); self
    }

    /// `scores=<objective>=<range>` — only entities with matching score (e.g. `"playtime=100.."`).
    pub fn scores(mut self, scores: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Scores(scores.into())); self
    }

    /// `nbt=<nbt>` — only entities matching this NBT compound.
    pub fn nbt(mut self, nbt: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Nbt(nbt.into())); self
    }

    /// `predicate=<predicate>` — only entities matching this loot predicate.
    pub fn predicate(mut self, predicate: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Predicate(predicate.into())); self
    }

    /// `dx/dy/dz` bounding box filter. All three must be set for it to work.
    pub fn volume(mut self, dx: f64, dy: f64, dz: f64) -> Self {
        self.args.push(SelectorArg::Dx(dx));
        self.args.push(SelectorArg::Dy(dy));
        self.args.push(SelectorArg::Dz(dz));
        self
    }

    /// `x/y/z` origin for distance/volume checks.
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
            let args = self.args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(",");
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
        assert_eq!(s.to_string(), "@e[type=minecraft:zombie,tag=!killed,limit=5]");
    }

    #[test]
    fn negation() {
        assert_eq!(
            Selector::all_players().not_team("red").to_string(),
            "@a[team=!red]"
        );
    }
}
