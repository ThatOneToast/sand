//! Intent-discovery acceptance corpus for `sand api search` (Phase 4/5 of
//! the API-discovery work).
//!
//! Each case is a realistic "I know what I want to do but not the type or
//! method name" query, verified by hand against the real installed catalog
//! (not a mock) before being encoded here. Every query has a genuine
//! matching API in the current catalog; none are invented to force a pass.
//!
//! Acceptance criteria (matches the plan this corpus implements):
//! - `Expect::Exact` — the named canonical path appears in the top 3 results.
//! - `Expect::Family` — some result under the given canonical-module prefix
//!   appears in the top 5 results, used when no single API is canonical for
//!   the intent (e.g. a whole family of `AdvancementTrigger` variants).
//!
//! Ordering must also be byte-for-byte deterministic across repeated runs.

use std::process::Command;

enum Expect {
    /// The exact canonical path must be in the top 3 results.
    Exact(&'static str),
    /// Some result whose canonical path starts with this module prefix
    /// (`prefix` or `prefix::...`) must be in the top 5 results.
    Family(&'static str),
}

struct Case {
    query: &'static str,
    expect: Expect,
}

const CORPUS: &[Case] = &[
    Case {
        query: "nearby entities",
        expect: Expect::Exact("sand::command::Target::nearby"),
    },
    Case {
        query: "query players",
        expect: Expect::Exact("sand::command::Target::players"),
    },
    Case {
        query: "detect equipped armor",
        expect: Expect::Exact("sand::events::ArmorEquipEvent"),
    },
    Case {
        query: "custom armor equip event",
        expect: Expect::Exact("sand::events::ArmorEquipEvent"),
    },
    Case {
        query: "create a predicate",
        expect: Expect::Family("sand::predicate"),
    },
    Case {
        query: "create custom item",
        expect: Expect::Exact("sand::component::CustomItem::new"),
    },
    Case {
        query: "give a player an item",
        expect: Expect::Exact("sand::command::give"),
    },
    Case {
        query: "construct a condition",
        expect: Expect::Family("sand::condition"),
    },
    Case {
        query: "schedule tick behavior",
        expect: Expect::Exact("sand::schedule"),
    },
    Case {
        query: "entity relationships",
        expect: Expect::Family("sand::entity"),
    },
    Case {
        query: "datapack component",
        expect: Expect::Family("sand::component"),
    },
    Case {
        query: "reference a datapack component",
        expect: Expect::Family("sand::component"),
    },
    Case {
        query: "equip an item on a player",
        expect: Expect::Family("sand::inventory"),
    },
    Case {
        query: "iterate over players",
        expect: Expect::Family("sand::entity"),
    },
    Case {
        query: "react to an advancement",
        expect: Expect::Family("sand::component"),
    },
    Case {
        query: "player advancement trigger",
        expect: Expect::Family("sand::component"),
    },
    Case {
        query: "define custom state",
        expect: Expect::Family("sand::state"),
    },
];

/// Runs `sand api search "<query>" --limit 5` against the real installed
/// catalog and returns the ordered canonical paths of the results shown.
fn run_search(query: &str) -> Vec<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_sand"))
        .args(["api", "search", query, "--limit", "5"])
        .output()
        .expect("run `sand api search`");
    assert!(
        output.status.success(),
        "`sand api search {query:?}` failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter_map(|line| {
            let trimmed = line.strip_prefix("  ")?;
            if trimmed.starts_with(' ') {
                return None; // indented summary line, not a result header
            }
            let (path, _) = trimmed.split_once("  [")?;
            Some(path.to_owned())
        })
        .collect()
}

#[test]
fn intent_corpus_resolves_to_the_expected_api() {
    let mut failures = Vec::new();
    for case in CORPUS {
        let results = run_search(case.query);
        let ok = match case.expect {
            Expect::Exact(path) => results.iter().take(3).any(|result| result == path),
            Expect::Family(prefix) => results.iter().take(5).any(|result| {
                result == prefix
                    || result
                        .strip_prefix(prefix)
                        .is_some_and(|rest| rest.starts_with("::"))
            }),
        };
        if !ok {
            let label = match case.expect {
                Expect::Exact(path) => format!("expected `{path}` in top 3"),
                Expect::Family(prefix) => format!("expected a `{prefix}` result in top 5"),
            };
            failures.push(format!(
                "query {:?}: {label}, got {:?}",
                case.query, results
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "intent-search corpus regressions:\n{}",
        failures.join("\n")
    );
}

#[test]
fn intent_corpus_ordering_is_byte_for_byte_deterministic() {
    for case in CORPUS {
        let first = run_search(case.query);
        let second = run_search(case.query);
        assert_eq!(
            first, second,
            "query {:?} produced different result ordering across two runs",
            case.query
        );
    }
}
