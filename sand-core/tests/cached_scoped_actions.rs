//! Cached action values must retain every anonymous helper across exports.
use sand::prelude::*;
use std::sync::OnceLock;

static BODY: OnceLock<Actions> = OnceLock::new();

fn body() -> Actions {
    BODY.get_or_init(|| {
        mcfunction![
        when(sand_core::state::Flag::new("cached_ready").of("@s").is_true()).then_all(["say cached when"]);
        unless(sand_core::state::Flag::new("cached_ready").of("@s").is_true()).then_all(["say cached unless"]);
        if_(sand_core::state::Flag::new("cached_ready").of("@s").is_true()).then_all(["say cached if"]);
        if_(sand_core::state::Flag::new("cached_ready").of("@s").is_true())
            .then_all(["say cached then", "return fail"])
            .else_all(["say cached else"]);
        when(sand_core::state::Flag::new("cached_ready").of("@s").is_true()).and_then("say staged first").then("say staged last");
        Target::players().each(|player| {
            EntityScope::bind(player, |bound| {
                bound
                    .owner()
                    .if_present(|_| mcfunction!["say retained body"; "return 0"])
            })
        });
        ]
    })
    .clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor {
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "cached",
        make: body,
    }
}

#[test]
fn prebuilt_and_reused_scopes_export_all_owned_helpers() {
    // Evaluate before the export registry scope begins, then reuse this value.
    let _cached = body();
    let first = sand_core::try_export_components_json("cached").unwrap();
    let second = sand_core::try_export_components_json("cached").unwrap();
    assert_eq!(first, second);
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let functions: std::collections::BTreeMap<_, _> = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .map(|record| {
            (
                format!(
                    "{}:{}",
                    record["namespace"].as_str().unwrap(),
                    record["path"].as_str().unwrap()
                ),
                record["content"].as_str().unwrap(),
            )
        })
        .collect();
    for prefix in [
        "sand/branches/",
        "sand/entity_scope/",
        "sand/entity_relation/owner/",
        "sand/entity_query/",
    ] {
        assert!(
            functions.keys().any(|name| name.contains(prefix)),
            "missing {prefix}"
        );
    }
    assert!(
        functions
            .values()
            .any(|body| body.contains("say retained body"))
    );
    for message in [
        "cached when",
        "cached unless",
        "cached if",
        "cached then",
        "cached else",
        "staged first",
        "staged last",
    ] {
        assert!(
            functions
                .values()
                .any(|body| body.lines().any(|line| line == format!("say {message}"))),
            "missing cached branch body {message}"
        );
    }
    for body in functions.values() {
        for line in body.lines() {
            if let Some((_, target)) = line.rsplit_once("function ") {
                let target = target.split_whitespace().next().unwrap();
                assert!(functions.contains_key(target), "dangling call: {line}");
            }
        }
    }
}
