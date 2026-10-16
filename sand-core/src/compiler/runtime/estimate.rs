//! Conservative explicit-call estimates, separate from Minecraft's command budget.
//!
//! These estimates do not measure MSPT, selector search cost, or terminal-command
//! multiplicity. Resource-triggered callbacks require separate graph edges.

use std::collections::{BTreeMap, BTreeSet};

const MAX_UNKNOWN_REASONS: usize = 8;
const MAX_UNKNOWN_REASON_BYTES: usize = 256;
const OMITTED_REASONS: &str = "additional unknown causes omitted";

/// A range of explicit command evaluations for one invocation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Work {
    pub(crate) minimum: u64,
    pub(crate) maximum: Option<u64>,
    pub(crate) unknown: BTreeSet<String>,
    /// Cycle/depth summaries depend on the active traversal, even if their
    /// diagnostic text is omitted by the payload limit.
    #[serde(skip)]
    path_dependent: bool,
}

impl Work {
    /// Construct a known number of explicit command evaluations.
    fn exact(count: u64) -> Self {
        Self {
            minimum: count,
            maximum: Some(count),
            unknown: BTreeSet::new(),
            path_dependent: false,
        }
    }

    /// Preserve an unresolved cost without treating it as zero work.
    fn unknown(mut reason: String) -> Self {
        if reason.len() > MAX_UNKNOWN_REASON_BYTES {
            let mut end = MAX_UNKNOWN_REASON_BYTES - "…".len();
            while !reason.is_char_boundary(end) {
                end -= 1;
            }
            reason.truncate(end);
            reason.push('…');
        }
        Self {
            minimum: 0,
            maximum: None,
            unknown: BTreeSet::from([reason]),
            path_dependent: false,
        }
    }

    /// Combine sequential work, retaining unknown causes and checked upper bounds.
    fn add(&mut self, other: Self) {
        self.minimum = self.minimum.saturating_add(other.minimum);
        self.maximum = self
            .maximum
            .zip(other.maximum)
            .and_then(|(a, b)| a.checked_add(b));
        self.path_dependent |= other.path_dependent;
        self.unknown.extend(other.unknown);
        if self.unknown.len() > MAX_UNKNOWN_REASONS {
            while self.unknown.len() >= MAX_UNKNOWN_REASONS {
                self.unknown.pop_last();
            }
            self.unknown.insert(OMITTED_REASONS.into());
        }
        if self.maximum.is_none() && self.unknown.is_empty() {
            self.unknown
                .insert("evaluation count exceeds the report's integer range".into());
        }
    }

    /// A guard or an earlier return can skip this entire contribution.
    fn optional(mut self) -> Self {
        self.minimum = 0;
        self
    }

    /// Remove the terminal entry already counted by its enclosing execute/return.
    fn without_entry(mut self) -> Self {
        self.minimum = self.minimum.saturating_sub(1);
        self.maximum = self.maximum.map(|maximum| maximum.saturating_sub(1));
        self
    }

    /// Repeat nested work for an unknown number of selected execution contexts.
    fn unbounded(mut self, reason: String) -> Self {
        self.minimum = 0;
        if self.maximum != Some(0) {
            self.maximum = None;
            self.add(Self::unknown(reason));
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReturnFlow {
    Never,
    Maybe,
    Always,
}

#[derive(Debug, Clone)]
struct Summary {
    work: Work,
    returns: ReturnFlow,
}

/// Analyze immutable collected function bodies with deterministic memoization.
pub(crate) struct Estimator<'a> {
    functions: &'a BTreeMap<String, String>,
    active: BTreeSet<String>,
    cache: BTreeMap<String, Summary>,
    remaining_commands: usize,
    command_depth: usize,
}

impl<'a> Estimator<'a> {
    /// Start an isolated analysis with a bounded traversal budget.
    pub(crate) fn new(functions: &'a BTreeMap<String, String>) -> Self {
        Self {
            functions,
            active: BTreeSet::new(),
            cache: BTreeMap::new(),
            remaining_commands: 100_000,
            command_depth: 0,
        }
    }

    /// Estimate one body invocation, excluding the caller's function command.
    pub(crate) fn function(&mut self, id: &str) -> Work {
        self.summary(id).work
    }

    /// Follow explicit calls while retaining early-return flow and cycle uncertainty.
    fn summary(&mut self, id: &str) -> Summary {
        if id.starts_with('#') {
            return Self::unsupported(format!("function-tag return flow is not analyzed: {id}"));
        }
        let (namespace, path) = id.split_once(':').unwrap_or(("minecraft", id));
        let Ok(id) = sand_components::ResourceLocation::new(namespace, path) else {
            return Self::unsupported(format!("unknown function reference: {id}"));
        };
        let id = id.to_string();
        if let Some(summary) = self.cache.get(&id) {
            return summary.clone();
        }
        if self.active.contains(&id) || self.active.len() >= 128 {
            return Self::path_dependent(format!("recursive or excessively deep call graph: {id}"));
        }
        let Some(body) = self.functions.get(&id).cloned() else {
            return Self::unsupported(format!("external or unavailable function: {id}"));
        };
        if body.lines().any(|line| line.trim_start().starts_with('$')) {
            return Self::unsupported(format!("macro instantiation is not analyzed: {id}"));
        }
        self.active.insert(id.clone());
        let mut work = Work::exact(0);
        let mut returns = ReturnFlow::Never;
        for line in body
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            if self.remaining_commands == 0 {
                work.add(Work::unknown("analysis work limit reached".into()));
                returns = ReturnFlow::Maybe;
                break;
            }
            let summary = self.line(line);
            work.add(if returns == ReturnFlow::Maybe {
                summary.work.optional()
            } else {
                summary.work
            });
            match summary.returns {
                ReturnFlow::Always => {
                    returns = ReturnFlow::Always;
                    break;
                }
                ReturnFlow::Maybe => returns = ReturnFlow::Maybe,
                ReturnFlow::Never => {}
            }
        }
        self.active.remove(&id);
        let summary = Summary { work, returns };
        // Cycle/depth results depend on the active call path. Never cache them
        // as though they were a context-independent function summary.
        if self.remaining_commands > 0 && !summary.work.path_dependent {
            self.cache.insert(id, summary.clone());
        }
        summary
    }

    /// Unsupported control flow may return early as well as perform unknown work.
    fn unsupported(reason: String) -> Summary {
        Summary {
            work: Work::unknown(reason),
            returns: ReturnFlow::Maybe,
        }
    }

    /// Keep path-sensitive uncertainty separate from bounded diagnostic text.
    fn path_dependent(reason: String) -> Summary {
        let mut summary = Self::unsupported(reason);
        summary.work.path_dependent = true;
        summary
    }

    /// Bound nested execute/return analysis independently of function graph depth.
    fn line(&mut self, line: &str) -> Summary {
        if self.remaining_commands == 0 {
            return Self::unsupported("analysis work limit reached".into());
        }
        if self.command_depth >= 128 {
            return Self::path_dependent("command nesting analysis limit reached".into());
        }
        self.remaining_commands -= 1;
        self.command_depth += 1;
        let summary = self.line_inner(line);
        self.command_depth -= 1;
        summary
    }

    /// Count the command entry plus explicit nested function and predicate work.
    fn line_inner(&mut self, line: &str) -> Summary {
        use sand_commands::render::{
            collected_execute, collected_return_command, collected_tokens,
        };
        let Some(tokens) = collected_tokens(line) else {
            let mut summary = Self::unsupported(format!("unsupported command grammar: {line}"));
            summary.work.minimum = 1;
            return summary;
        };
        let mut work = Work::exact(1);
        let mut returns = ReturnFlow::Never;
        match tokens.first().copied() {
            Some("function") if tokens.len() == 2 => work.add(self.summary(tokens[1]).work),
            Some("function") => work.add(Work::unknown(format!(
                "function arguments are not analyzed: {line}"
            ))),
            Some("return") => {
                returns = ReturnFlow::Always;
                if let Some(command) = collected_return_command(line) {
                    work = self.line(command).work;
                } else if tokens.len() != 2
                    || (tokens[1] != "fail" && tokens[1].parse::<i32>().is_err())
                {
                    work.add(Work::unknown(format!("unsupported return grammar: {line}")));
                    returns = ReturnFlow::Maybe;
                }
            }
            Some("execute") => {
                let Some(execute) = collected_execute(line) else {
                    work.add(Work::unknown(format!(
                        "unsupported execute grammar: {line}"
                    )));
                    return Summary {
                        work,
                        returns: ReturnFlow::Maybe,
                    };
                };
                let tail = self.line(execute.command);
                let mut optional = false;
                let mut unbounded = false;
                for operation in execute.operations {
                    let args = operation.arguments;
                    if matches!(operation.keyword, "if" | "unless") {
                        if args.first() == Some(&"function") {
                            let mut predicate = self.summary(args[1]).work;
                            if optional {
                                predicate = predicate.optional();
                            }
                            if unbounded {
                                predicate = predicate
                                    .unbounded(format!("unknown selector multiplicity: {line}"));
                            }
                            work.add(predicate);
                        }
                        optional = true;
                    } else if matches!(operation.keyword, "as" | "at" | "on")
                        || (matches!(operation.keyword, "positioned" | "rotated")
                            && args.first() == Some(&"as"))
                        || (operation.keyword == "facing" && args.first() == Some(&"entity"))
                    {
                        optional = true;
                        // Cardinality is deliberately unknown until the report
                        // supplies a supported selector/context assumption.
                        unbounded = true;
                    } else {
                        // A known context operation can still fail at runtime,
                        // for example when a referenced dimension is absent.
                        optional = true;
                    }
                }
                let mut nested = tail.work.without_entry();
                if optional {
                    nested = nested.optional();
                }
                if unbounded {
                    nested = nested.unbounded(format!("unknown selector multiplicity: {line}"));
                }
                work.add(nested);
                if tail.returns != ReturnFlow::Never {
                    returns = ReturnFlow::Maybe;
                }
            }
            Some("advancement") if tokens.get(1) == Some(&"grant") => {
                work.add(Work::unknown(
                    "advancement reward callbacks require resource-graph analysis".into(),
                ));
            }
            _ => {}
        }
        Summary { work, returns }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Repeated calls count repeatedly even when one resource is shared.
    #[test]
    fn repeated_calls_and_returns_follow_observable_evaluations() {
        let functions = BTreeMap::from([
            ("demo:leaf".into(), "say leaf".into()),
            (
                "demo:twice".into(),
                "function demo:leaf\nfunction demo:leaf".into(),
            ),
            (
                "demo:return".into(),
                "return run function demo:leaf\nsay unreachable".into(),
            ),
            (
                "demo:conditional".into(),
                "execute if score @s flag matches 1 run return 1\nsay maybe".into(),
            ),
        ]);
        let mut estimator = Estimator::new(&functions);
        assert_eq!(estimator.function("demo:twice"), Work::exact(4));
        assert_eq!(estimator.function("demo:return"), Work::exact(2));
        let conditional = estimator.function("demo:conditional");
        assert_eq!((conditional.minimum, conditional.maximum), (1, Some(2)));
    }

    /// Unknown calls, macros, tags and implicit rewards never become zero cost.
    #[test]
    fn unresolved_work_remains_explicit() {
        let functions = BTreeMap::from([
            ("demo:external".into(), "function other:missing".into()),
            ("demo:cycle".into(), "function demo:cycle".into()),
            ("demo:macro".into(), "$say $(value)".into()),
            ("demo:tag".into(), "function #demo:handlers".into()),
            (
                "demo:reward".into(),
                "advancement grant @a only demo:rewarded".into(),
            ),
        ]);
        let mut estimator = Estimator::new(&functions);
        for id in functions.keys() {
            let estimate = estimator.function(id);
            assert_eq!(estimate.maximum, None, "{id}");
            assert!(!estimate.unknown.is_empty(), "{id}");
        }
    }

    /// A forked ordinary terminal still contributes one debug command entry.
    #[test]
    fn selector_work_is_not_terminal_execution_multiplicity() {
        let functions = BTreeMap::from([
            ("demo:ordinary".into(), "execute as @e run say hello".into()),
            ("demo:leaf".into(), "say leaf".into()),
            (
                "demo:call".into(),
                "execute as @e run function demo:leaf".into(),
            ),
        ]);
        let mut estimator = Estimator::new(&functions);
        assert_eq!(estimator.function("demo:ordinary"), Work::exact(1));
        let calls = estimator.function("demo:call");
        assert_eq!((calls.minimum, calls.maximum), (1, None));
    }

    /// Context modifiers do not prove that the nested call will run.
    #[test]
    fn context_changes_keep_optional_nested_work() {
        let functions = BTreeMap::from([
            ("demo:leaf".into(), "say leaf".into()),
            (
                "demo:dimension".into(),
                "execute in demo:world run function demo:leaf".into(),
            ),
        ]);
        let work = Estimator::new(&functions).function("demo:dimension");
        assert_eq!((work.minimum, work.maximum), (1, Some(2)));
    }
    /// Cyclic branching must not turn report generation into unbounded work.
    #[test]
    fn analysis_limit_keeps_unvisited_commands_unknown() {
        let functions = BTreeMap::from([
            ("demo:a".into(), "function demo:b\nfunction demo:b".into()),
            ("demo:b".into(), "function demo:a\nfunction demo:a".into()),
        ]);
        let mut estimator = Estimator::new(&functions);
        estimator.remaining_commands = 2;
        let work = estimator.function("demo:a");
        assert_eq!(work.maximum, None);
        assert!(work.unknown.contains("analysis work limit reached"));
        assert!(estimator.cache.is_empty());
    }
    /// Truncation remains valid UTF-8 and never turns uncertainty into a bound.
    #[test]
    fn long_unknown_reasons_preserve_unicode_and_unknown_bounds() {
        let work = Work::unknown("🌍".repeat(4096));
        let reason = work.unknown.first().unwrap();
        assert!(reason.len() <= MAX_UNKNOWN_REASON_BYTES);
        assert!(reason.ends_with('…'));
        assert_eq!((work.minimum, work.maximum), (0, None));
        assert!(serde_json::to_string(&work).is_ok());
    }

    /// Dropping excess diagnostic text must not make cycle summaries cacheable.
    #[test]
    fn bounded_diagnostics_retain_path_dependent_analysis_state() {
        let mut commands: Vec<_> = (0..40)
            .map(|index| format!("function other:missing_{index}"))
            .collect();
        commands.push("function demo:root".into());
        let functions = BTreeMap::from([("demo:root".into(), commands.join("\n"))]);
        let mut estimator = Estimator::new(&functions);
        let work = estimator.function("demo:root");
        assert!(work.unknown.len() <= MAX_UNKNOWN_REASONS);
        assert!(work.unknown.contains(OMITTED_REASONS));
        assert!(!estimator.cache.contains_key("demo:root"));
    }
}
