//! Typed condition DSL for `execute if/unless` generation.
//!
//! Conditions can be composed with [`Condition::all`], [`Condition::any`], and
//! the `!` operator ([`std::ops::Not`]) without writing any raw execute syntax.
//!
//! Nested `Any` inside `All` is correctly lowered into multiple execute commands
//! by Sand's typed execute and branch builders.
//!
//! # Example
//! ```rust,ignore
//! use sand_core::state::{ScoreVar, Flag, Cooldown, Ticks};
//! use sand_core::condition::Condition;
//!
//! static MANA: ScoreVar<i32> = ScoreVar::new("mana");
//! static CASTING: Flag = Flag::new("casting");
//! static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
//!
//! let cond = Condition::all([
//!     MANA.of("@s").gte(25),
//!     DASH.ready("@s"),
//!     CASTING.of("@s").is_false(),
//! ]);
//! ```

// ── ScoreRange ────────────────────────────────────────────────────────────────

/// A range used in `execute if score … matches <range>`.
///
/// `None` on either end of `Between` means the range is open on that side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ScoreRange {
    /// `matches <n>` — exactly equal.
    Eq(i32),
    /// `matches <n+1>..` — strictly greater than n.
    Gt(i32),
    /// `matches <n>..` — greater than or equal.
    Gte(i32),
    /// `matches ..<n-1>` — strictly less than n.
    Lt(i32),
    /// `matches ..<n>` — less than or equal.
    Lte(i32),
    /// `matches [lo]..[hi]` — inclusive range (either bound may be `None` = open).
    Between(Option<i32>, Option<i32>),
}

/// Vanilla operators accepted by `execute if score <left> <op> <right>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScoreCompareOp {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
}

impl ScoreRange {
    /// Render the range to a Minecraft matches string fragment.
    ///
    /// Uses saturating arithmetic for `Gt`/`Lt` so that `Gt(i32::MAX)` and
    /// `Lt(i32::MIN)` cannot overflow (panic in debug, silently wrap in
    /// release). Those two ranges cannot be satisfied by any `i32` score and
    /// are also rejected by [`ScoreRange::validate`] — `render` stays total
    /// (never panics) so it remains safe to call on a range that was
    /// constructed through an infallible/legacy path.
    pub fn render(&self) -> String {
        match self {
            ScoreRange::Eq(n) => n.to_string(),
            ScoreRange::Gt(n) => format!("{}..", n.saturating_add(1)),
            ScoreRange::Gte(n) => format!("{n}.."),
            ScoreRange::Lt(n) => format!("..{}", n.saturating_sub(1)),
            ScoreRange::Lte(n) => format!("..{n}"),
            ScoreRange::Between(lo, hi) => {
                let lo_s = lo.map(|n| n.to_string()).unwrap_or_default();
                let hi_s = hi.map(|n| n.to_string()).unwrap_or_default();
                format!("{lo_s}..{hi_s}")
            }
        }
    }

    /// `true` if some `i32` score value could satisfy this range.
    ///
    /// `Gt(i32::MAX)`, `Lt(i32::MIN)`, and `Between(lo, hi)` with `lo > hi`
    /// describe an empty range: every score is excluded, but the naive
    /// rendered `matches` fragment does not make that obvious (and for
    /// `Gt`/`Lt` at the `i32` boundary, naive rendering would overflow).
    pub fn is_satisfiable(&self) -> bool {
        match self {
            ScoreRange::Eq(_) | ScoreRange::Gte(_) | ScoreRange::Lte(_) => true,
            ScoreRange::Gt(n) => *n != i32::MAX,
            ScoreRange::Lt(n) => *n != i32::MIN,
            ScoreRange::Between(Some(lo), Some(hi)) => lo <= hi,
            ScoreRange::Between(_, _) => true,
        }
    }

    /// Validate this range using the shared `sand-commands` diagnostic type.
    ///
    /// Rejects ranges that cannot be satisfied by any `i32` score:
    /// `Gt(i32::MAX)`, `Lt(i32::MIN)`, and `Between(lo, hi)` with `lo > hi`.
    pub fn validate(&self) -> sand_commands::CommandResult<()> {
        if self.is_satisfiable() {
            Ok(())
        } else {
            Err(sand_commands::CommandError::new(
                "ScoreRange",
                "range",
                format!(
                    "range `{}` (from {self:?}) cannot be satisfied by any i32 score",
                    self.render()
                ),
            )
            .with_code("SAND-SCORE-RANGE"))
        }
    }
}

// ── Condition ─────────────────────────────────────────────────────────────────

/// A typed datapack condition, suitable for use in `execute if/unless`.
///
/// Produce conditions from [`ScoreVar::of`](crate::state::ScoreVar::of),
/// [`Flag::of`](crate::state::Flag::of), or the static constructors below.
///
/// Use [`when`](crate::execute_when::when) / [`unless`](crate::execute_when::unless)
/// to turn a `Condition` into complete execute commands.
///
/// Nested `Any` inside `All` is automatically distributed into multiple execute
/// commands by Sand's typed execute and branch builders.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::condition::Condition",
    aliases = ["sand::prelude::Condition"],
    summary = "Represents a typed boolean test evaluated by Minecraft commands.",
    context = "State, inventory, entity, and resource helpers produce Conditions so authors can compose gameplay tests without assembling execute syntax by hand.",
    minecraft = "Lowers to one or more execute if or execute unless command plans; disjunctions may require multiple plans because vanilla execute has no direct OR clause.",
    use_when = ["Guarding commands with Minecraft runtime state", "Combining typed score, entity, item, data, or predicate tests"],
    avoid_when = ["The decision can be made once while generating the datapack", "A dedicated typed helper already performs the complete operation"],
    example = "let ready = MANA.of(\"@s\").gte(25).and(CASTING.of(\"@s\").is_false());"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Condition {
    kind: ConditionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConditionKind {
    /// `if score <selector> <objective> matches <range>`
    Score {
        selector: String,
        objective: String,
        range: ScoreRange,
    },
    /// `if score <left selector> <left objective> <op> <right selector> <right objective>`.
    ScoreCompare {
        left: crate::state::score::ScoreOperand,
        op: ScoreCompareOp,
        right: crate::state::score::ScoreOperand,
    },
    /// `if score <selector> <flag_objective> matches 1` (or `0` when `value = false`)
    Flag {
        selector: String,
        objective: String,
        value: bool,
    },
    /// `if predicate <namespace:path>`
    Predicate(String),
    /// `if entity <selector>`
    Entity(String),
    /// `if data <storage|entity|block> <path>` using the unified NBT model.
    NbtExists {
        target: sand_commands::DataTarget,
        path: sand_commands::NbtPath,
    },
    /// `execute if items entity <target> <slot> <item>`.
    ItemsEntity {
        target: sand_commands::Selector,
        slot: sand_commands::ItemSlot,
        item: String,
    },
    /// `execute if items block <position> <slot> <item>`.
    ItemsBlock {
        position: sand_commands::BlockPos,
        slot: sand_commands::ItemSlot,
        item: String,
    },
    /// Invert this condition (flips `if` ↔ `unless`).
    Not(Box<Condition>),
    /// All sub-conditions must hold (chained `if … if …`).
    All(Vec<Condition>),
    /// At least one sub-condition must hold (generates one execute per sub-condition).
    Any(Vec<Condition>),
    /// Explicit raw escape hatch: `if/unless <fragment>` verbatim.
    ///
    /// The fragment must be a valid Minecraft `execute if`/`unless` sub-command
    /// *without* the leading `if`/`unless` keyword, e.g. `"score @s sync_jumps < @s jumps"`
    /// or `"predicate my_pack:some_predicate"`.
    ///
    /// This is an intentionally explicit escape hatch — there is no `From<&str>`/
    /// `From<String>` impl for `Condition`, so raw fragments never enter a typed
    /// condition chain silently. Prefer the typed constructors above; reach for
    /// `Condition::raw` only when no typed equivalent exists yet.
    Raw(String),
}

impl Condition {
    pub(crate) fn kind(&self) -> &ConditionKind {
        &self.kind
    }

    pub(crate) fn score(selector: String, objective: String, range: ScoreRange) -> Self {
        Self {
            kind: ConditionKind::Score {
                selector,
                objective,
                range,
            },
        }
    }

    pub(crate) fn score_compare(
        left: crate::state::score::ScoreOperand,
        op: ScoreCompareOp,
        right: crate::state::score::ScoreOperand,
    ) -> Self {
        Self {
            kind: ConditionKind::ScoreCompare { left, op, right },
        }
    }

    pub(crate) fn flag(selector: String, objective: String, value: bool) -> Self {
        Self {
            kind: ConditionKind::Flag {
                selector,
                objective,
                value,
            },
        }
    }

    pub(crate) fn predicate_raw(location: impl Into<String>) -> Self {
        Self {
            kind: ConditionKind::Predicate(location.into()),
        }
    }

    pub(crate) fn entity_raw(selector: impl Into<String>) -> Self {
        Self {
            kind: ConditionKind::Entity(selector.into()),
        }
    }

    pub(crate) fn nbt_exists(
        target: sand_commands::DataTarget,
        path: sand_commands::NbtPath,
    ) -> Self {
        Self {
            kind: ConditionKind::NbtExists { target, path },
        }
    }

    pub(crate) fn items_entity(
        target: sand_commands::Selector,
        slot: sand_commands::ItemSlot,
        item: String,
    ) -> Self {
        Self {
            kind: ConditionKind::ItemsEntity { target, slot, item },
        }
    }

    pub(crate) fn items_block(
        position: sand_commands::BlockPos,
        slot: sand_commands::ItemSlot,
        item: String,
    ) -> Self {
        Self {
            kind: ConditionKind::ItemsBlock {
                position,
                slot,
                item,
            },
        }
    }

    /// Invert a condition.
    ///
    /// Also available as the `!` operator via [`std::ops::Not`].
    #[sand_macros::api(
        registry = sand_api_contract,
        kind = "method",
        path = "sand::condition::Condition::negate",
        aliases = ["sand::prelude::Condition::negate"],
        summary = "Inverts a typed runtime condition.",
        context = "The named constructor is useful when an inverted condition must be passed or returned as a value; the ! operator provides equivalent expression syntax.",
        minecraft = "Swaps execute-if and execute-unless polarity and applies De Morgan lowering to composed conditions.",
        use_when = ["Passing an inverted condition to another API", "Constructing a negation without operator syntax"],
        avoid_when = ["The ! operator is clearer at the call site", "Rust generation-time control flow is intended"],
        params(cond = "The typed condition whose truth value is inverted."),
        returns = "A condition that succeeds exactly when the input condition fails.",
        example = "Condition::negate(READY.of(\"@s\").is_true())"
    )]
    pub fn negate(cond: Condition) -> Self {
        Self {
            kind: ConditionKind::Not(Box::new(cond)),
        }
    }

    /// All of the given conditions must hold.
    #[sand_macros::api(
        registry = sand_api_contract,
        kind = "method",
        path = "sand::condition::Condition::all",
        aliases = ["sand::prelude::Condition::all"],
        summary = "Requires every supplied runtime condition to succeed.",
        context = "This constructor forms a reusable conjunction from a dynamic or fixed-size collection; an empty collection represents the boolean identity true.",
        minecraft = "Chains compatible execute-if clauses and distributes nested disjunctions into multiple command plans when necessary.",
        use_when = ["Every condition must guard the same command or branch", "Building a conjunction from an iterator"],
        avoid_when = ["Only two conditions are being combined and and is clearer", "The tests guard different commands"],
        params(conds = "The conditions that must all succeed; an empty iterator creates an unconditional condition."),
        returns = "A conjunction that succeeds when every supplied condition succeeds.",
        example = "Condition::all([MANA.of(\"@s\").gte(25), READY.of(\"@s\").is_true()])"
    )]
    pub fn all(conds: impl IntoIterator<Item = Condition>) -> Self {
        Self {
            kind: ConditionKind::All(conds.into_iter().collect()),
        }
    }

    /// Any of the given conditions must hold.
    #[sand_macros::api(
        registry = sand_api_contract,
        kind = "method",
        path = "sand::condition::Condition::any",
        aliases = ["sand::prelude::Condition::any"],
        summary = "Allows any supplied runtime condition to satisfy a branch.",
        context = "This constructor forms a disjunction from a dynamic or fixed-size collection; an empty collection represents false. Because vanilla execute lacks OR, overlapping alternatives can run the guarded command more than once.",
        minecraft = "Lowers alternatives into separate execute command plans; negating the result produces a conjunction of unless clauses.",
        use_when = ["Independent alternatives may enable the same idempotent branch", "Building a disjunction from an iterator"],
        avoid_when = ["Multiple alternatives may succeed and repeating the guarded effect would be incorrect", "Each alternative should run different commands"],
        params(conds = "The alternative conditions; an empty iterator creates a condition that never succeeds."),
        returns = "A disjunction represented by one command plan per alternative.",
        example = "Condition::any([HAS_KEY.of(\"@s\").is_true(), IS_ADMIN.of(\"@s\").is_true()])"
    )]
    pub fn any(conds: impl IntoIterator<Item = Condition>) -> Self {
        Self {
            kind: ConditionKind::Any(conds.into_iter().collect()),
        }
    }

    /// Condition on a named predicate resource.
    ///
    /// ```rust,ignore
    /// let predicate = PredicateId::custom("my_pack:can_cast".parse()?);
    /// let c = Condition::predicate(predicate);
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        kind = "method",
        path = "sand::condition::Condition::predicate",
        aliases = ["sand::prelude::Condition::predicate"],
        summary = "Tests a typed named Minecraft predicate resource.",
        context = "Predicate resources package reusable loot-condition logic under a validated identity that commands can evaluate without duplicating the condition tree.",
        minecraft = "Renders execute if predicate <namespace:path>, or the corresponding execute-unless form when negated.",
        use_when = ["Reusing a predicate resource as a command guard", "Testing loot-condition behavior exposed through a named predicate"],
        avoid_when = ["Building the predicate JSON resource itself", "Mutable scoreboard or storage state is the actual condition"],
        params(predicate = "The typed namespaced reference to the predicate resource to evaluate."),
        returns = "A runtime condition referencing the named predicate.",
        example = "Condition::predicate(PredicateId::custom(\"demo:can_cast\".parse()?))"
    )]
    pub fn predicate(predicate: crate::resource_ref::PredicateId) -> Self {
        Self::predicate_raw(predicate.to_string())
    }

    /// Condition on an entity selector.
    ///
    /// ```rust,ignore
    /// let c = Condition::entity(Target::self_().tag("ready"));
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        kind = "method",
        path = "sand::condition::Condition::entity",
        aliases = ["sand::prelude::Condition::entity"],
        summary = "Tests whether a typed selector matches at least one entity.",
        context = "The typed selector preserves its filters and validation while the Condition records only the existence test, not an execution-context change.",
        minecraft = "Renders execute if entity <selector>, succeeding when the selector finds one or more entities.",
        use_when = ["Checking whether a filtered entity set is non-empty", "Guarding a branch on an entity tag, type, distance, or score filter"],
        avoid_when = ["Commands must execute as or at each matched entity", "An unchecked raw selector fragment is required"],
        params(selector = "The typed entity selector whose match set is tested for existence."),
        returns = "A condition that succeeds when the selector matches at least one entity.",
        example = "Condition::entity(Target::players().tag(\"ready\"))"
    )]
    pub fn entity(selector: impl sand_commands::TargetArgument) -> Self {
        Self::entity_raw(selector.to_string())
    }

    /// Condition on a typed NBT reference existing.
    ///
    /// ```rust,ignore
    /// let mana = Nbt::storage("example:state").path("player.mana");
    /// let c = Condition::data_exists(&mana);
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        kind = "method",
        path = "sand::condition::Condition::data_exists",
        aliases = ["sand::prelude::Condition::data_exists"],
        summary = "Tests whether a typed Minecraft NBT path exists.",
        context = "NbtRef retains both the typed data target and parsed path so storage, entity, and block data can share one existence-check API.",
        minecraft = "Renders execute if data <target> <path>, or execute unless data when the condition is negated.",
        use_when = ["Guarding commands on optional storage, entity, or block NBT", "Checking whether a typed state field has been materialized"],
        avoid_when = ["Reading or comparing the value stored at the path", "Testing whether a live inventory slot contains an item"],
        params(reference = "The typed NBT target and path whose existence Minecraft should test."),
        returns = "A condition that succeeds when the referenced NBT path exists.",
        example = "Condition::data_exists(&Nbt::storage(\"demo:state\").path(\"player.mana\"))"
    )]
    pub fn data_exists<T>(reference: &sand_commands::NbtRef<T>) -> Self {
        Self::nbt_exists(reference.location().clone(), reference.path_value().clone())
    }

    /// Explicit raw `execute if/unless` fragment escape hatch.
    ///
    /// The fragment is used verbatim **after** the `if`/`unless` keyword,
    /// which is added automatically when rendering — do not include it
    /// yourself. Use this only when no typed condition constructor covers
    /// your case.
    ///
    /// ```rust,ignore
    /// let c = Condition::raw("score @s sync_jumps < @s jumps");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `fragment` already starts with a leading `if `/`unless `
    /// keyword — that would render as a malformed doubled keyword (e.g.
    /// `"if if score ..."`). This is checked eagerly, at construction, rather
    /// than silently accepted and only visible in generated datapack output.
    #[sand_macros::api(
        registry = sand_api_contract,
        kind = "method",
        path = "sand::condition::Condition::raw",
        aliases = ["sand::prelude::Condition::raw"],
        summary = "Creates an explicit escape hatch for unsupported execute-condition syntax.",
        context = "Raw fragments keep uncommon or newly added Minecraft grammar usable without pretending it has Sand's typed or validation guarantees.",
        minecraft = "Places the fragment verbatim after the generated if or unless keyword in an execute command.",
        use_when = ["Minecraft supports a condition form Sand has not typed yet", "A modded execute-condition grammar must pass through unchanged"],
        avoid_when = ["A typed Condition constructor is available", "The fragment comes from untrusted input or requires Minecraft grammar validation"],
        params(fragment = "The execute-condition fragment without a leading if or unless keyword."),
        returns = "A raw condition containing the supplied command fragment.",
        example = "Condition::raw(\"block ~ ~-1 ~ minecraft:white_wool\")"
    )]
    pub fn raw(fragment: impl Into<String>) -> Self {
        let fragment = fragment.into();
        let trimmed = fragment.trim_start();
        assert!(
            !trimmed.starts_with("if ")
                && !trimmed.starts_with("unless ")
                && trimmed != "if"
                && trimmed != "unless",
            "Condition::raw fragment must not include a leading `if`/`unless` keyword — it is \
             added automatically when rendering: {fragment:?}"
        );
        Self {
            kind: ConditionKind::Raw(fragment),
        }
    }
}

impl std::ops::Not for Condition {
    type Output = Condition;
    fn not(self) -> Self::Output {
        Condition::negate(self)
    }
}

// ── Ergonomic chaining ────────────────────────────────────────────────────────

impl Condition {
    /// Both `self` and `other` must hold (`All`).
    ///
    /// Flattens adjacent `All` chains so `a.and(b).and(c)` produces a single
    /// `All([a, b, c])` rather than nested `All([All([a, b]), c])`.
    ///
    /// ```rust,ignore
    /// let cond = MANA.of("@s").gte(25).and(DASH.ready("@s"));
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        kind = "method",
        path = "sand::condition::Condition::and",
        aliases = ["sand::prelude::Condition::and"],
        summary = "Requires this condition and another condition to both succeed.",
        context = "Binary conjunction provides readable fluent composition and flattens adjacent conjunctions without exposing the internal condition tree.",
        minecraft = "Combines both tests into the same execute plan, distributing nested alternatives when required.",
        use_when = ["Adding one required guard to an existing condition", "Building a fluent all-of expression"],
        avoid_when = ["Either test succeeding should be sufficient", "The conditions guard different commands"],
        params(other = "The additional typed condition that must also succeed."),
        returns = "A condition requiring both operands.",
        example = "has_mana.and(cooldown_ready)"
    )]
    pub fn and(self, other: Condition) -> Condition {
        match self.kind {
            ConditionKind::All(mut conds) => {
                conds.push(other);
                Condition::all(conds)
            }
            kind => Condition::all([Condition { kind }, other]),
        }
    }

    /// Either `self` or `other` must hold (`Any`).
    ///
    /// Flattens adjacent `Any` chains so `a.or(b).or(c)` produces a single
    /// `Any([a, b, c])`.
    ///
    /// ```rust,ignore
    /// let cond = MANA.of("@s").gte(100).or(SHIELD.of("@s").is_true());
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        kind = "method",
        path = "sand::condition::Condition::or",
        aliases = ["sand::prelude::Condition::or"],
        summary = "Allows this condition or another condition to satisfy a branch.",
        context = "Binary disjunction provides fluent composition, but overlapping alternatives can repeat the guarded effect because vanilla execute has no direct OR clause.",
        minecraft = "Lowers the operands into alternative execute plans rather than a single short-circuiting clause.",
        use_when = ["Either alternative may enable the same idempotent branch", "Building a fluent any-of expression"],
        avoid_when = ["Both alternatives may succeed and duplicate execution would be incorrect", "The alternatives should run different commands"],
        params(other = "The alternative typed condition that may satisfy the branch."),
        returns = "A condition represented by the alternative plans of both operands.",
        example = "has_key.or(is_admin)"
    )]
    pub fn or(self, other: Condition) -> Condition {
        match self.kind {
            ConditionKind::Any(mut conds) => {
                conds.push(other);
                Condition::any(conds)
            }
            kind => Condition::any([Condition { kind }, other]),
        }
    }

    /// `self` must hold and `other` must **not** hold.
    ///
    /// Equivalent to `self.and(!other)`.
    ///
    /// ```rust,ignore
    /// let cond = MANA.of("@s").gte(25).and_not(CASTING.of("@s").is_true());
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        kind = "method",
        path = "sand::condition::Condition::and_not",
        aliases = ["sand::prelude::Condition::and_not"],
        summary = "Requires this condition to succeed and another condition to fail.",
        context = "This fluent convenience expresses self.and(!other) while preserving typed boolean composition.",
        minecraft = "Combines this operand's execute-if clauses with the other operand's execute-unless clauses, distributing nested alternatives when required.",
        use_when = ["Adding an exclusion to an existing condition", "Expressing an allowed-unless-blocked guard"],
        avoid_when = ["The exclusion needs separate commands", "Explicit and plus ! syntax is clearer for a complex operand"],
        params(other = "The typed condition that must not succeed."),
        returns = "A condition requiring this operand and the negation of the other operand.",
        example = "has_mana.and_not(is_silenced)"
    )]
    pub fn and_not(self, other: Condition) -> Condition {
        self.and(!other)
    }

    /// Either `self` holds or `other` must **not** hold.
    ///
    /// Equivalent to `self.or(!other)`.
    #[sand_macros::api(
        registry = sand_api_contract,
        kind = "method",
        path = "sand::condition::Condition::or_not",
        aliases = ["sand::prelude::Condition::or_not"],
        summary = "Allows this condition to succeed or another condition to fail.",
        context = "This fluent convenience expresses self.or(!other); like every disjunction, overlapping alternatives can repeat the guarded effect.",
        minecraft = "Lowers this operand's plans and the negated operand's plans as execute alternatives.",
        use_when = ["A branch is allowed by a positive test or by absence of a blocker"],
        avoid_when = ["Both alternatives may succeed and duplicate execution would be incorrect", "The boolean intent is clearer as an explicit or plus ! expression"],
        params(other = "The typed condition whose failure provides the alternative."),
        returns = "A condition satisfied by this operand or the negation of the other operand.",
        example = "is_admin.or_not(is_locked)"
    )]
    pub fn or_not(self, other: Condition) -> Condition {
        self.or(!other)
    }
}

// ── Execute plan lowering ─────────────────────────────────────────────────────

/// One normalized typed condition clause.
#[derive(Debug, Clone)]
pub(crate) struct ExecuteClause {
    /// Whether this clause renders with `unless` instead of `if`.
    negated: bool,
    /// Structured condition body.
    pub(crate) condition: sand_commands::ConditionIr,
}

impl ExecuteClause {
    /// Convert this clause into an ordered execute operation.
    pub(crate) fn into_operation(self) -> sand_commands::ExecuteOp {
        if self.negated {
            sand_commands::ExecuteOp::Unless(self.condition)
        } else {
            sand_commands::ExecuteOp::If(self.condition)
        }
    }

    /// Compatibility renderer for callers that still consume clause strings.
    pub(crate) fn render(&self) -> String {
        format!("{} {}", if_kw(self.negated), self.condition.render())
    }
}

/// A normalized typed execute plan. Multiple plans are OR-alternatives.
pub(crate) type ExecuteIrPlan = Vec<ExecuteClause>;

impl Condition {
    /// Expand this condition into normalized typed execute plans.
    ///
    /// Each plan is a list of `if/unless …` clause strings to chain in a single
    /// `execute … run <cmd>` command. Multiple plans are OR-alternatives — the
    /// command is emitted once per plan.
    ///
    /// | Condition | negated=false | negated=true |
    /// |---|---|---|
    /// | Leaf | `[[if clause]]` | `[[unless clause]]` |
    /// | `Not(c)` | `c.rendered_plans(true)` | `c.rendered_plans(false)` |
    /// | `All(cs)` | Cartesian product of children | Union of negated children |
    /// | `Any(cs)` | Union of children | Cartesian product of negated children |
    ///
    /// The Cartesian product of `[[a], [b]]` and `[[c], [d]]` is
    /// `[[a, c], [a, d], [b, c], [b, d]]`.
    pub(crate) fn to_ir_plans(&self, negated: bool) -> Vec<ExecuteIrPlan> {
        use sand_commands::{ConditionIr, ScoreCmp, Selector};

        let clause = |condition| vec![vec![ExecuteClause { negated, condition }]];
        match &self.kind {
            ConditionKind::Score {
                selector,
                objective,
                range,
            } => clause(ConditionIr::ScoreMatches {
                holder: sand_commands::__private::score_holder_compat(selector.clone()),
                objective: objective.clone(),
                range: range.render(),
            }),
            ConditionKind::ScoreCompare { left, op, right } => {
                let op = match op {
                    ScoreCompareOp::Eq => ScoreCmp::Eq,
                    ScoreCompareOp::Gt => ScoreCmp::Gt,
                    ScoreCompareOp::Gte => ScoreCmp::Ge,
                    ScoreCompareOp::Lt => ScoreCmp::Lt,
                    ScoreCompareOp::Lte => ScoreCmp::Le,
                };
                clause(ConditionIr::ScoreCompare {
                    left: sand_commands::__private::score_holder_compat(left.selector.clone()),
                    left_objective: left.objective.clone(),
                    op,
                    right: sand_commands::__private::score_holder_compat(right.selector.clone()),
                    right_objective: right.objective.clone(),
                })
            }
            ConditionKind::Flag {
                selector,
                objective,
                value,
            } => clause(ConditionIr::ScoreMatches {
                holder: sand_commands::__private::score_holder_compat(selector.clone()),
                objective: objective.clone(),
                range: if *value { "1" } else { "0" }.to_string(),
            }),
            ConditionKind::Predicate(loc) => clause(ConditionIr::Predicate(loc.clone())),
            ConditionKind::Entity(sel) => clause(ConditionIr::Entity(Selector::raw(sel.clone()))),
            ConditionKind::NbtExists { target, path } => clause(ConditionIr::Data {
                target: target.clone(),
                path: path.as_str().to_string(),
            }),
            ConditionKind::ItemsEntity { target, slot, item } => clause(ConditionIr::ItemsEntity {
                target: target.clone(),
                slot: slot.clone(),
                item: item.clone(),
            }),
            ConditionKind::ItemsBlock {
                position,
                slot,
                item,
            } => clause(ConditionIr::ItemsBlock {
                position: position.clone(),
                slot: slot.clone(),
                item: item.clone(),
            }),
            ConditionKind::Raw(fragment) => clause(ConditionIr::Raw(fragment.clone())),

            // Not: flip the negated flag and delegate
            ConditionKind::Not(inner) => inner.to_ir_plans(!negated),

            // All(cs) negated=false → AND  → Cartesian product of each child's plans
            // All(cs) negated=true  → NOT(AND) = OR of NOTs → union of negated children
            ConditionKind::All(conds) => {
                if negated {
                    // NOT(a AND b) = NOT a OR NOT b
                    conds.iter().flat_map(|c| c.to_ir_plans(true)).collect()
                } else {
                    let sub_plan_sets: Vec<Vec<ExecuteIrPlan>> =
                        conds.iter().map(|c| c.to_ir_plans(false)).collect();
                    cartesian_product_plans(sub_plan_sets)
                }
            }

            // Any(cs) negated=false → OR  → union of children's plans
            // Any(cs) negated=true  → NOT(OR) = AND of NOTs → Cartesian product of negated children
            ConditionKind::Any(conds) => {
                if negated {
                    // NOT(a OR b) = NOT a AND NOT b
                    let sub_plan_sets: Vec<Vec<ExecuteIrPlan>> =
                        conds.iter().map(|c| c.to_ir_plans(true)).collect();
                    cartesian_product_plans(sub_plan_sets)
                } else {
                    conds.iter().flat_map(|c| c.to_ir_plans(false)).collect()
                }
            }
        }
    }

    pub(crate) fn rendered_plans(&self, negated: bool) -> Vec<Vec<String>> {
        self.to_ir_plans(negated)
            .into_iter()
            .map(|plan| plan.into_iter().map(|clause| clause.render()).collect())
            .collect()
    }

    /// Build complete `execute … run <cmd>` command strings for this condition.
    ///
    /// Nested `Any` inside `All` correctly expands into multiple commands.
    ///
    /// - Simple conditions and `All`: typically one command.
    /// - `Any`: one command per sub-condition.
    /// - `Not(Any)`: one command with de Morgan–applied `unless` clauses.
    /// - `All([a, Any([b, c])])`: two commands.
    pub(crate) fn execute_commands(&self, negated: bool, run: &str) -> Vec<String> {
        self.to_ir_plans(negated)
            .into_iter()
            .map(|clauses| {
                if clauses.is_empty() {
                    run.to_string()
                } else {
                    clauses
                        .into_iter()
                        .fold(sand_commands::Execute::new(), |execute, clause| {
                            sand_commands::__private::execute_with_operation(
                                execute,
                                clause.into_operation(),
                            )
                        })
                        .run(run)
                }
            })
            .collect()
    }

    #[cfg(test)]
    fn render_clauses(&self, negated: bool) -> Vec<String> {
        self.rendered_plans(negated).into_iter().flatten().collect()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn if_kw(negated: bool) -> &'static str {
    if negated { "unless" } else { "if" }
}

/// Compute the Cartesian product of multiple plan sets.
///
/// Given `[[plan_a1, plan_a2], [plan_b1]]` produces every combination:
/// `[plan_a1 + plan_b1, plan_a2 + plan_b1]`.
fn cartesian_product_plans(plan_sets: Vec<Vec<ExecuteIrPlan>>) -> Vec<ExecuteIrPlan> {
    if plan_sets.is_empty() {
        return vec![vec![]];
    }
    let mut result: Vec<ExecuteIrPlan> = vec![vec![]];
    for plan_set in plan_sets {
        let mut new_result = Vec::new();
        for existing in &result {
            for plan in &plan_set {
                let mut combined = existing.clone();
                combined.extend_from_slice(plan);
                new_result.push(combined);
            }
        }
        result = new_result;
    }
    result
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn score(sel: &str, obj: &str, range: ScoreRange) -> Condition {
        Condition::score(sel.to_string(), obj.to_string(), range)
    }

    fn flag(sel: &str, obj: &str, value: bool) -> Condition {
        Condition::flag(sel.to_string(), obj.to_string(), value)
    }

    // ── ScoreRange rendering ──────────────────────────────────────────────────

    #[test]
    fn range_eq() {
        assert_eq!(ScoreRange::Eq(5).render(), "5");
    }

    #[test]
    fn range_gte() {
        assert_eq!(ScoreRange::Gte(25).render(), "25..");
    }

    #[test]
    fn range_lte() {
        assert_eq!(ScoreRange::Lte(100).render(), "..100");
    }

    #[test]
    fn range_gt() {
        assert_eq!(ScoreRange::Gt(10).render(), "11..");
    }

    #[test]
    fn range_gt_at_i32_max_does_not_overflow() {
        // Never panics/wraps; the impossible range is caught by `validate`.
        assert_eq!(ScoreRange::Gt(i32::MAX).render(), format!("{}..", i32::MAX));
        assert!(!ScoreRange::Gt(i32::MAX).is_satisfiable());
        assert!(ScoreRange::Gt(i32::MAX).validate().is_err());
    }

    #[test]
    fn range_lt_at_i32_min_does_not_overflow() {
        assert_eq!(ScoreRange::Lt(i32::MIN).render(), format!("..{}", i32::MIN));
        assert!(!ScoreRange::Lt(i32::MIN).is_satisfiable());
        assert!(ScoreRange::Lt(i32::MIN).validate().is_err());
    }

    #[test]
    fn range_between_min_greater_than_max_is_unsatisfiable() {
        let range = ScoreRange::Between(Some(100), Some(10));
        assert!(!range.is_satisfiable());
        assert!(range.validate().is_err());
    }

    #[test]
    fn range_validate_error_code_is_stable() {
        let err = ScoreRange::Gt(i32::MAX).validate().unwrap_err();
        assert_eq!(err.code, "SAND-SCORE-RANGE");
    }

    #[test]
    fn range_validate_accepts_normal_ranges() {
        assert!(ScoreRange::Eq(5).validate().is_ok());
        assert!(ScoreRange::Gt(10).validate().is_ok());
        assert!(ScoreRange::Lt(10).validate().is_ok());
        assert!(ScoreRange::Between(Some(1), Some(100)).validate().is_ok());
        assert!(ScoreRange::Between(None, Some(100)).validate().is_ok());
        assert!(ScoreRange::Between(Some(1), None).validate().is_ok());
    }

    #[test]
    fn range_lt() {
        assert_eq!(ScoreRange::Lt(10).render(), "..9");
    }

    #[test]
    fn range_between() {
        assert_eq!(ScoreRange::Between(Some(1), Some(100)).render(), "1..100");
    }

    #[test]
    fn range_between_open_end() {
        assert_eq!(ScoreRange::Between(Some(25), None).render(), "25..");
    }

    #[test]
    fn range_between_open_start() {
        assert_eq!(ScoreRange::Between(None, Some(100)).render(), "..100");
    }

    // ── Leaf execute_plans ────────────────────────────────────────────────────

    #[test]
    fn score_plan_if() {
        let c = score("@s", "mana", ScoreRange::Gte(25));
        let plans = c.rendered_plans(false);
        assert_eq!(plans, vec![vec!["if score @s mana matches 25.."]]);
    }

    #[test]
    fn normalization_retains_typed_condition_nodes() {
        let condition = Condition::all([
            score("@s", "mana", ScoreRange::Gte(10)),
            Condition::predicate_raw("demo:can_cast"),
            Condition::raw("block ~ ~-1 ~ minecraft:stone"),
        ]);
        let plans = condition.to_ir_plans(false);
        assert_eq!(plans.len(), 1);
        assert!(matches!(
            &plans[0][0].condition,
            sand_commands::ConditionIr::ScoreMatches { objective, .. } if objective == "mana"
        ));
        assert!(matches!(
            &plans[0][1].condition,
            sand_commands::ConditionIr::Predicate(value) if value == "demo:can_cast"
        ));
        assert!(matches!(
            &plans[0][2].condition,
            sand_commands::ConditionIr::Raw(value)
                if value == "block ~ ~-1 ~ minecraft:stone"
        ));
    }

    #[test]
    fn score_plan_unless() {
        let c = score("@s", "mana", ScoreRange::Gte(25));
        let plans = c.rendered_plans(true);
        assert_eq!(plans, vec![vec!["unless score @s mana matches 25.."]]);
    }

    #[test]
    fn storage_exists_plan() {
        let reference = sand_commands::Nbt::storage("ex:state").path("mana");
        let c = Condition::data_exists(&reference);
        let plans = c.rendered_plans(false);
        assert_eq!(plans, vec![vec!["if data storage ex:state mana"]]);
        let plans_neg = c.rendered_plans(true);
        assert_eq!(plans_neg, vec![vec!["unless data storage ex:state mana"]]);
    }

    // ── Condition rendering (backwards compat) ────────────────────────────────

    #[test]
    fn score_if_clause() {
        let c = score("@s", "mana", ScoreRange::Gte(25));
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["if score @s mana matches 25.."]);
    }

    #[test]
    fn score_unless_clause() {
        let c = score("@s", "mana", ScoreRange::Gte(25));
        let clauses = c.render_clauses(true);
        assert_eq!(clauses, vec!["unless score @s mana matches 25.."]);
    }

    #[test]
    fn flag_true_clause() {
        let c = flag("@s", "casting", true);
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["if score @s casting matches 1"]);
    }

    #[test]
    fn flag_false_clause() {
        let c = flag("@s", "casting", false);
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["if score @s casting matches 0"]);
    }

    #[test]
    fn predicate_clause() {
        let c = Condition::predicate(crate::resource_ref::PredicateId::custom(
            "my_pack:can_cast".parse().unwrap(),
        ));
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["if predicate my_pack:can_cast"]);
    }

    #[test]
    fn entity_clause() {
        let c = Condition::entity(sand_commands::Selector::self_().tag("ready"));
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["if entity @s[tag=ready]"]);
    }

    #[test]
    fn not_flips_keyword() {
        let c = !(score("@s", "mana", ScoreRange::Gte(25)));
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["unless score @s mana matches 25.."]);
    }

    #[test]
    fn not_not_cancels() {
        let c = !(!(score("@s", "mana", ScoreRange::Eq(10))));
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["if score @s mana matches 10"]);
    }

    #[test]
    fn all_chains_clauses() {
        let c = Condition::all([
            score("@s", "mana", ScoreRange::Gte(25)),
            flag("@s", "casting", false),
        ]);
        let clauses = c.render_clauses(false);
        assert_eq!(clauses.len(), 2);
        assert_eq!(clauses[0], "if score @s mana matches 25..");
        assert_eq!(clauses[1], "if score @s casting matches 0");
    }

    // ── execute_commands ──────────────────────────────────────────────────────

    #[test]
    fn execute_score() {
        let c = score("@s", "mana", ScoreRange::Gte(25));
        let cmds = c.execute_commands(false, "say enough mana");
        assert_eq!(
            cmds,
            vec!["execute if score @s mana matches 25.. run say enough mana"]
        );
    }

    #[test]
    fn execute_unless() {
        let c = flag("@s", "casting", true);
        let cmds = c.execute_commands(true, "say ok");
        assert_eq!(
            cmds,
            vec!["execute unless score @s casting matches 1 run say ok"]
        );
    }

    #[test]
    fn execute_any_generates_multiple() {
        let c = Condition::any([
            score("@s", "mana", ScoreRange::Gte(25)),
            score("@s", "rage", ScoreRange::Gte(10)),
        ]);
        let cmds = c.execute_commands(false, "say ok");
        assert_eq!(
            cmds,
            vec![
                "execute if score @s mana matches 25.. run say ok",
                "execute if score @s rage matches 10.. run say ok",
            ]
        );
    }

    #[test]
    fn execute_not_any_de_morgan() {
        // NOT(a OR b) = (NOT a) AND (NOT b) — one chained command
        let c = !(Condition::any([flag("@s", "a", true), flag("@s", "b", true)]));
        let cmds = c.execute_commands(false, "say ok");
        assert_eq!(
            cmds,
            vec!["execute unless score @s a matches 1 unless score @s b matches 1 run say ok"],
            "de Morgan should produce one chained command"
        );
    }

    #[test]
    fn execute_all() {
        let c = Condition::all([
            score("@s", "mana", ScoreRange::Gte(25)),
            flag("@s", "casting", false),
            Condition::predicate_raw("my_pack:can_cast"),
        ]);
        let cmds = c.execute_commands(false, "say cast");
        assert_eq!(
            cmds,
            vec![
                "execute if score @s mana matches 25.. if score @s casting matches 0 if predicate my_pack:can_cast run say cast"
            ]
        );
    }

    // ── Nested Any lowering (the key bug fix) ────────────────────────────────

    #[test]
    fn nested_any_inside_all_expands() {
        // all![a, any![b, c]] → 2 commands:
        //   execute if a if b run cmd
        //   execute if a if c run cmd
        let a = score("@s", "mana", ScoreRange::Gte(25));
        let b = flag("@s", "casting", false);
        let c = flag("@s", "sprinting", true);
        let cond = Condition::all([a, Condition::any([b, c])]);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(
            cmds,
            vec![
                "execute if score @s mana matches 25.. if score @s casting matches 0 run say ok",
                "execute if score @s mana matches 25.. if score @s sprinting matches 1 run say ok",
            ],
            "nested Any should expand"
        );
    }

    #[test]
    fn any_inside_any_flattens() {
        let a = score("@s", "mana", ScoreRange::Gte(25));
        let b = score("@s", "rage", ScoreRange::Gte(10));
        let c = score("@s", "ki", ScoreRange::Gte(5));
        let cond = Condition::any([a, Condition::any([b, c])]);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 3, "Any(Any) should produce 3 commands");
    }

    #[test]
    fn not_all_de_morgan() {
        // NOT(a AND b) = NOT a OR NOT b → 2 separate commands
        let a = score("@s", "mana", ScoreRange::Gte(25));
        let b = flag("@s", "casting", true);
        let cond = !(Condition::all([a, b]));
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(
            cmds,
            vec![
                "execute unless score @s mana matches 25.. run say ok",
                "execute unless score @s casting matches 1 run say ok",
            ],
            "NOT(AND) should give 2 plans"
        );
    }

    #[test]
    fn all_of_any_cross_product() {
        // all![any![a, b], any![c, d]] → 4 commands
        let a = score("@s", "a", ScoreRange::Eq(1));
        let b = score("@s", "b", ScoreRange::Eq(2));
        let c = score("@s", "c", ScoreRange::Eq(3));
        let d = score("@s", "d", ScoreRange::Eq(4));
        let cond = Condition::all([Condition::any([a, b]), Condition::any([c, d])]);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(
            cmds.len(),
            4,
            "cross product of two Any(2) should be 4: got {cmds:?}"
        );
    }

    #[test]
    fn raw_condition_if() {
        let c = Condition::raw("score @s sync_jumps < @s jumps");
        let plans = c.rendered_plans(false);
        assert_eq!(plans, vec![vec!["if score @s sync_jumps < @s jumps"]]);
    }

    #[test]
    fn raw_condition_unless() {
        let c = Condition::raw("entity @s[tag=busy]");
        let plans = c.rendered_plans(true);
        assert_eq!(plans, vec![vec!["unless entity @s[tag=busy]"]]);
    }

    #[test]
    fn raw_condition_composes_with_typed() {
        let c = Condition::all([
            score("@s", "mana", ScoreRange::Gte(25)),
            Condition::raw("score @s sync_jumps < @s jumps"),
        ]);
        let cmds = c.execute_commands(false, "say ok");
        assert_eq!(
            cmds,
            vec![
                "execute if score @s mana matches 25.. if score @s sync_jumps < @s jumps run say ok"
            ]
        );
    }

    #[test]
    #[should_panic(expected = "must not include a leading `if`/`unless` keyword")]
    fn raw_condition_rejects_embedded_if_keyword() {
        Condition::raw("if score @s x matches 1");
    }

    #[test]
    #[should_panic(expected = "must not include a leading `if`/`unless` keyword")]
    fn raw_condition_rejects_embedded_unless_keyword() {
        Condition::raw("unless score @s x matches 1");
    }

    #[test]
    fn raw_condition_permits_fragments_merely_containing_the_word_if() {
        // Only a *leading* if/unless keyword is rejected — "iffy" or an
        // embedded "if" elsewhere in the fragment must not false-positive.
        let c = Condition::raw("score @s iffy_score matches 1");
        assert_eq!(
            c.rendered_plans(false),
            vec![vec!["if score @s iffy_score matches 1"]]
        );
    }

    // ── Empty Condition::all([])/any([]) rendering ────────────────────────────

    #[test]
    fn empty_all_renders_as_single_vacuously_true_plan() {
        // All([]) is a vacuous AND — always true — and must render as one
        // plan with zero clauses (an unconditional execute), not zero plans.
        let c = Condition::all([]);
        assert_eq!(c.rendered_plans(false), vec![Vec::<String>::new()]);
    }

    #[test]
    fn empty_any_renders_as_zero_plans() {
        // Any([]) is a vacuous OR — always false / unsatisfiable — and must
        // render as zero plans (never matches), not one vacuous plan.
        let c = Condition::any([]);
        let plans: Vec<Vec<String>> = c.rendered_plans(false);
        assert!(plans.is_empty(), "expected zero plans, got: {plans:?}");
    }

    #[test]
    fn storage_exists_execute() {
        let reference = sand_commands::Nbt::storage("ex:state").path("mana");
        let c = Condition::data_exists(&reference);
        let cmds = c.execute_commands(false, "say has mana");
        assert_eq!(
            cmds,
            vec!["execute if data storage ex:state mana run say has mana"]
        );
    }

    // ── Condition chaining ────────────────────────────────────────────────────

    #[test]
    fn and_produces_all() {
        let a = score("@s", "mana", ScoreRange::Gte(25));
        let b = flag("@s", "casting", false);
        let cond = a.and(b);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(
            cmds,
            vec!["execute if score @s mana matches 25.. if score @s casting matches 0 run say ok"]
        );
    }

    #[test]
    fn and_flattens_chain() {
        let a = score("@s", "mana", ScoreRange::Gte(25));
        let b = flag("@s", "casting", false);
        let c = flag("@s", "sprinting", true);
        // a.and(b).and(c) should be a flat All([a, b, c]), not All([All([a,b]), c])
        let cond = a.and(b).and(c);
        match cond.kind() {
            ConditionKind::All(v) => assert_eq!(v.len(), 3, "expected flat All([a,b,c])"),
            other => panic!("expected All, got {other:?}"),
        }
    }

    #[test]
    fn or_produces_any() {
        let a = score("@s", "mana", ScoreRange::Gte(100));
        let b = flag("@s", "shield", true);
        let cond = a.or(b);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 2, "Any should expand to 2 commands");
    }

    #[test]
    fn or_flattens_chain() {
        let a = score("@s", "mana", ScoreRange::Gte(100));
        let b = score("@s", "rage", ScoreRange::Gte(50));
        let c = score("@s", "ki", ScoreRange::Gte(10));
        let cond = a.or(b).or(c);
        match cond.kind() {
            ConditionKind::Any(v) => assert_eq!(v.len(), 3, "expected flat Any([a,b,c])"),
            other => panic!("expected Any, got {other:?}"),
        }
    }

    #[test]
    fn and_not_negates_rhs() {
        let a = score("@s", "mana", ScoreRange::Gte(25));
        let b = flag("@s", "casting", true);
        let cond = a.and_not(b);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(
            cmds,
            vec![
                "execute if score @s mana matches 25.. unless score @s casting matches 1 run say ok"
            ]
        );
    }

    #[test]
    fn chained_and_with_nested_or() {
        let mana = score("@s", "mana", ScoreRange::Gte(25));
        let dash = flag("@s", "dash", false);
        let shield = flag("@s", "shield", true);
        // mana AND (dash OR shield) → 2 commands
        let cond = mana.and(dash.or(shield));
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 2, "AND with nested OR: {cmds:?}");
        assert!(
            cmds.iter().all(|c| c.contains("if score @s mana")),
            "both commands include mana: {cmds:?}"
        );
    }

    #[test]
    fn event_guard_chaining_pattern() {
        static MANA2: crate::state::ScoreVar<i32> = crate::state::ScoreVar::new("mana");
        static DASH2: crate::state::Cooldown =
            crate::state::Cooldown::new("dash", crate::state::Ticks::new(60));
        static CASTING2: crate::state::Flag = crate::state::Flag::new("casting");
        let guard = MANA2
            .of("@s")
            .gte(25)
            .and(DASH2.ready("@s"))
            .and_not(CASTING2.of("@s").is_true());
        let cmds = guard.execute_commands(false, "function ns:handler");
        assert_eq!(
            cmds,
            vec![
                "execute if score @s mana matches 25.. if score @s dash matches 0 unless score @s casting matches 1 run function ns:handler"
            ]
        );
    }
}
