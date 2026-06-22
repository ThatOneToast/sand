//! Typed scoreboard variable — wraps a scoreboard objective for clean access.

use std::marker::PhantomData;
use std::ops::RangeBounds;

use crate::condition::{Condition, ScoreRange};

// ── Name utilities ────────────────────────────────────────────────────────────

/// Minecraft scoreboard objective names are limited to 16 characters.
/// If the requested name exceeds that limit, a stable FNV-1a hash is used
/// to produce a deterministic 16-character name prefixed with `"s"`.
pub(super) fn objective_name(name: &str) -> String {
    if name.len() <= 16 {
        name.to_string()
    } else {
        let hash = fnv1a(name);
        format!("s{:015x}", hash & 0x0FFF_FFFF_FFFF_FFFF)
    }
}

fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 14_695_981_039_346_656_037;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1_099_511_628_211);
    }
    h
}

// ── ScoreVar ──────────────────────────────────────────────────────────────────

/// A typed scoreboard variable backed by a single scoreboard objective.
///
/// Declare once as a `static` and use throughout your datapack:
///
/// ```rust,ignore
/// use sand_core::state::ScoreVar;
///
/// static MANA: ScoreVar<i32> = ScoreVar::new("mana");
///
/// let cmds = vec![
///     MANA.define(),
///     MANA.set("@s", 100),
///     MANA.add("@s", 5),
/// ];
/// ```
pub struct ScoreVar<T = i32> {
    name: &'static str,
    _marker: PhantomData<T>,
}

impl<T> ScoreVar<T> {
    /// Create a new `ScoreVar` with the given objective name.
    ///
    /// Names longer than 16 characters are automatically hashed to a stable
    /// 16-character objective name (see [`ScoreVar::objective_name`]).
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    /// Return the actual scoreboard objective name used in commands.
    ///
    /// This is either `name` directly (≤16 chars) or a stable hash (>16 chars).
    pub fn objective_name(&self) -> String {
        objective_name(self.name)
    }

    /// `scoreboard objectives add <obj> dummy` — register the objective.
    ///
    /// Call this in your `load` function.
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective_name())
    }

    /// `scoreboard players set <selector> <obj> <value>`
    pub fn set(&self, selector: impl std::fmt::Display, value: i32) -> String {
        format!(
            "scoreboard players set {} {} {}",
            selector,
            self.objective_name(),
            value
        )
    }

    /// `scoreboard players add <selector> <obj> <amount>`
    pub fn add(&self, selector: impl std::fmt::Display, amount: i32) -> String {
        format!(
            "scoreboard players add {} {} {}",
            selector,
            self.objective_name(),
            amount
        )
    }

    /// `scoreboard players remove <selector> <obj> <amount>`
    pub fn remove(&self, selector: impl std::fmt::Display, amount: i32) -> String {
        format!(
            "scoreboard players remove {} {} {}",
            selector,
            self.objective_name(),
            amount
        )
    }

    /// `scoreboard players reset <selector> <obj>`
    pub fn reset(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players reset {} {}",
            selector,
            self.objective_name()
        )
    }

    /// Clamp the score for `selector` to `[min, max]`.
    ///
    /// Returns two commands: one to enforce the lower bound and one for the upper.
    ///
    /// Generated commands:
    /// ```text
    /// execute if score <selector> <obj> matches ..<min-1> run scoreboard players set <selector> <obj> <min>
    /// execute if score <selector> <obj> matches <max+1>.. run scoreboard players set <selector> <obj> <max>
    /// ```
    pub fn clamp(&self, selector: impl std::fmt::Display, min: i32, max: i32) -> Vec<String> {
        let selector = selector.to_string();
        let obj = self.objective_name();
        vec![
            format!(
                "execute if score {selector} {obj} matches ..{} run scoreboard players set {selector} {obj} {min}",
                min.saturating_sub(1)
            ),
            format!(
                "execute if score {selector} {obj} matches {}.. run scoreboard players set {selector} {obj} {max}",
                max.saturating_add(1)
            ),
        ]
    }

    /// Bind this variable to a selector to produce a condition builder.
    ///
    /// ```rust,ignore
    /// let cond = MANA.of("@s").gte(25);
    /// ```
    pub fn of<'a>(&'a self, selector: &str) -> ScoreRef<'a, T> {
        ScoreRef {
            objective: self.name,
            selector: selector.to_string(),
            _marker: PhantomData,
        }
    }
}

// ── ScoreRef ──────────────────────────────────────────────────────────────────

/// A [`ScoreVar`] bound to a selector — used to build [`Condition`]s.
///
/// Produced by [`ScoreVar::of`].
pub struct ScoreRef<'a, T = i32> {
    /// The underlying `ScoreVar` name (used to derive the objective name).
    objective: &'a str,
    selector: String,
    _marker: PhantomData<T>,
}

impl<'a, T> ScoreRef<'a, T> {
    fn obj(&self) -> String {
        objective_name(self.objective)
    }

    /// `if score <sel> <obj> matches <n>` — equal to `n`.
    pub fn eq(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Eq(n),
        }
    }

    /// `unless score <sel> <obj> matches <n>` — not equal to `n`.
    pub fn ne(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::Not(Box::new(Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Eq(n),
        }))
    }

    /// `if score <sel> <obj> matches <n+1>..` — strictly greater than `n`.
    pub fn gt(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Gt(n),
        }
    }

    /// `if score <sel> <obj> matches <n>..` — greater than or equal to `n`.
    pub fn gte(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Gte(n),
        }
    }

    /// `if score <sel> <obj> matches ..<n-1>` — strictly less than `n`.
    pub fn lt(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Lt(n),
        }
    }

    /// `if score <sel> <obj> matches ..<n>` — less than or equal to `n`.
    pub fn lte(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Lte(n),
        }
    }

    /// `if score <sel> <obj> matches <min>..<max>` — inside an inclusive range.
    pub fn between(self, min: i32, max: i32) -> Condition {
        let objective = self.obj();
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Between(Some(min), Some(max)),
        }
    }

    /// `unless score <sel> <obj> matches <min>..<max>` — outside an inclusive range.
    pub fn outside(self, min: i32, max: i32) -> Condition {
        Condition::Not(Box::new(self.between(min, max)))
    }

    /// `if score <sel> <obj> matches <lo>..<hi>` — within an inclusive range.
    ///
    /// Accepts any `RangeBounds<i32>`: `1..=100`, `0..`, `..100`, etc.
    pub fn matches(self, range: impl RangeBounds<i32>) -> Condition {
        use std::ops::Bound;
        let lo = match range.start_bound() {
            Bound::Included(&n) => Some(n),
            Bound::Excluded(&n) => Some(n + 1),
            Bound::Unbounded => None,
        };
        let hi = match range.end_bound() {
            Bound::Included(&n) => Some(n),
            Bound::Excluded(&n) => Some(n - 1),
            Bound::Unbounded => None,
        };
        let objective = self.obj();
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Between(lo, hi),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::Condition;

    static MANA: ScoreVar<i32> = ScoreVar::new("mana");
    static LONG: ScoreVar<i32> = ScoreVar::new("this_is_a_very_long_name_that_exceeds_limit");

    #[test]
    fn short_name_unchanged() {
        assert_eq!(MANA.objective_name(), "mana");
    }

    #[test]
    fn long_name_hashed_stable() {
        let a = LONG.objective_name();
        let b = LONG.objective_name();
        assert_eq!(a, b, "hash must be deterministic");
        assert!(
            a.len() <= 16,
            "hashed name must be ≤16 chars, got {}",
            a.len()
        );
        assert!(a.starts_with('s'), "hashed name must start with 's'");
    }

    #[test]
    fn define_cmd() {
        assert_eq!(MANA.define(), "scoreboard objectives add mana dummy");
    }

    #[test]
    fn set_cmd() {
        assert_eq!(MANA.set("@s", 100), "scoreboard players set @s mana 100");
    }

    #[test]
    fn add_cmd() {
        assert_eq!(MANA.add("@s", 5), "scoreboard players add @s mana 5");
    }

    #[test]
    fn remove_cmd() {
        assert_eq!(
            MANA.remove("@s", 10),
            "scoreboard players remove @s mana 10"
        );
    }

    #[test]
    fn reset_cmd() {
        assert_eq!(MANA.reset("@s"), "scoreboard players reset @s mana");
    }

    #[test]
    fn clamp_cmds() {
        let cmds = MANA.clamp("@s", 0, 100);
        assert_eq!(cmds.len(), 2);
        assert!(cmds[0].contains("matches ..-1"), "got: {}", cmds[0]);
        assert!(cmds[1].contains("matches 101.."), "got: {}", cmds[1]);
    }

    #[test]
    fn condition_gte() {
        let cond = MANA.of("@s").gte(25);
        match cond {
            Condition::Score {
                selector,
                objective,
                range: ScoreRange::Gte(25),
            } => {
                assert_eq!(selector, "@s");
                assert_eq!(objective, "mana");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn condition_lte() {
        let cond = MANA.of("@s").lte(100);
        match cond {
            Condition::Score {
                range: ScoreRange::Lte(100),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn condition_ne_wraps_not() {
        let cond = MANA.of("@s").ne(0);
        assert!(matches!(cond, Condition::Not(_)));
    }

    #[test]
    fn condition_matches_range() {
        let cond = MANA.of("@s").matches(1..=100);
        match cond {
            Condition::Score {
                range: ScoreRange::Between(Some(1), Some(100)),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn condition_between() {
        let cond = MANA.of("@s").between(10, 100);
        match cond {
            Condition::Score {
                range: ScoreRange::Between(Some(10), Some(100)),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn condition_outside_wraps_not() {
        let cond = MANA.of("@s").outside(10, 100);
        assert!(matches!(cond, Condition::Not(_)));
    }

    #[test]
    fn condition_matches_open_end() {
        let cond = MANA.of("@s").matches(25..);
        match cond {
            Condition::Score {
                range: ScoreRange::Between(Some(25), None),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }
}
