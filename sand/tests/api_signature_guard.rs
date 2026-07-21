//! API-signature regression guard (issue #277).
//!
//! Scans the public façade source files for `pub fn` parameters whose *name*
//! indicates a known identifier/target concept (item, block, entity type,
//! function/dialog ref, ...) but whose *type* is still an untyped
//! `impl Into<String>` / `impl Display` / bare `String` — i.e. a case where a
//! meaningful typed concept exists (or should) but the normal-path signature
//! doesn't use it yet.
//!
//! This is deliberately source-aware rather than a blind repository-wide
//! grep: it only reads a curated list of façade files (the modules ordinary
//! datapack authors actually import), only matches `pub fn` signatures (not
//! doc comments or private helpers), and skips explicit raw escape hatches
//! (`*_raw` function names) and a small allowlist of known, tracked
//! exceptions. Growing the allowlist is expected as the API is grooomed
//! further; the guard's job is only to stop *new*, unlisted occurrences from
//! landing silently.
//!
//! Free-form text parameters (chat messages, lore, display names) are not
//! flagged: this guard only matches parameter names that look like typed
//! identifier/target concepts, via [`LOOKS_LIKE_TYPED_CONCEPT`].

use std::path::{Path, PathBuf};

/// Parameter name substrings that indicate a typed identifier/target
/// concept exists (or should exist) rather than genuine free-form text.
/// Matched case-sensitively against the parameter name only.
const LOOKS_LIKE_TYPED_CONCEPT: &[&str] = &[
    "item",
    "block",
    "entity_type",
    "function_ref",
    "dialog_ref",
    "advancement_ref",
    "loot_table_ref",
    "predicate_ref",
];

/// Function names known to accept a typed identifier/target concept even
/// though their parameter is conventionally named something generic like
/// `ty` rather than spelling out the concept (`entity_type`/`not_type` on
/// `EntityTarget`/`Selector`/`EntityQuery`, `summon*` builders). Every
/// non-`self` parameter of a matching function is checked, regardless of
/// its name — this exists because [`LOOKS_LIKE_TYPED_CONCEPT`]'s
/// name-based matching alone would silently miss these (parameter name
/// `ty`, not `entity_type`).
const TYPED_CONCEPT_FUNCTIONS: &[&str] = &[
    "entity_type",
    "not_entity_type",
    "not_type",
    "summon",
    "summon_here",
    "summon_at",
    "summon_at_with_nbt",
];

/// `(file relative to the workspace root, function name)` pairs that are
/// known, tracked exceptions to the rule above — either genuine free-form
/// text, a deliberate raw escape hatch not already caught by the `*_raw`
/// suffix check, or a pre-existing gap tracked by a follow-up issue rather
/// than fixed in this PR.
const ALLOWLIST: &[(&str, &str)] = &[
    // `dimension` in `Execute::in_` is a target concept without a typed
    // `DimensionId`-accepting normal path yet; tracked as a remaining
    // signature gap in issue #277's follow-up scope rather than fixed here.
    ("sand-commands/src/execute.rs", "in_"),
    // This PR's #277 scope covers `give`/`entity_type`/`summon` (see
    // sand_core::cmd::IntoGiveItem, sand_commands::selector::IntoEntityType).
    // These related item/block-matching signatures were not converted in
    // this pass and remain tracked as follow-up scope, not fixed here:
    // item/block predicate matchers used by `execute if/unless items`, and
    // the clear/give/setblock item-or-block builders in `builtins.rs`.
    ("sand-commands/src/builtins.rs", "clear_item"),
    ("sand-commands/src/builtins.rs", "try_clear_item"),
    ("sand-commands/src/builtins.rs", "give"),
    ("sand-commands/src/builtins.rs", "try_give"),
    ("sand-commands/src/builtins.rs", "give_count"),
    ("sand-commands/src/builtins.rs", "try_give_count"),
    ("sand-commands/src/builtins.rs", "setblock_abs"),
    ("sand-commands/src/builtins.rs", "try_setblock_abs"),
    ("sand-commands/src/execute.rs", "if_block"),
    ("sand-commands/src/execute.rs", "unless_block"),
    ("sand-commands/src/execute.rs", "if_items_entity"),
    ("sand-commands/src/execute.rs", "unless_items_entity"),
    ("sand-commands/src/execute.rs", "if_items_block"),
    ("sand-commands/src/execute.rs", "unless_items_block"),
    ("sand-commands/src/execute.rs", "if_items"),
    ("sand-commands/src/execute.rs", "unless_items"),
];

/// Façade source files scanned by this guard: the modules reachable from
/// `sand::prelude`, `sand::cmd`, `sand::vanilla`, and the entity/selector
/// builders most commonly used by datapack authors.
const FACADE_FILES: &[&str] = &[
    "sand-core/src/cmd/mod.rs",
    "sand-commands/src/builtins.rs",
    "sand-commands/src/selector.rs",
    "sand-commands/src/execute.rs",
    "sand-core/src/entity/query.rs",
];

// Fields are read via the derived `Debug` impl in the failure message,
// which dead-code analysis doesn't see through.
#[derive(Debug)]
#[allow(dead_code)]
struct Violation {
    file: String,
    function: String,
    param: String,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("sand/ has a workspace root parent")
        .to_path_buf()
}

/// Extract `(function_name, parameter_list_text)` for every `pub fn` in
/// `source`, joining multi-line signatures into one string per function.
fn extract_pub_fn_signatures(source: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;

    while let Some(rel) = source[i..].find("pub fn ") {
        let start = i + rel + "pub fn ".len();
        let name_end = source[start..]
            .find(|c: char| c == '(' || c.is_whitespace() || c == '<')
            .map(|n| start + n)
            .unwrap_or(source.len());
        let name = source[start..name_end].to_string();

        let Some(paren_rel) = source[name_end..].find('(') else {
            i = name_end;
            continue;
        };
        let paren_start = name_end + paren_rel;

        let mut depth = 0i32;
        let mut j = paren_start;
        let mut paren_end = None;
        while j < bytes.len() {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        paren_end = Some(j);
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }

        let Some(paren_end) = paren_end else {
            break;
        };
        let params = source[paren_start + 1..paren_end].to_string();
        out.push((name, params));
        i = paren_end + 1;
    }

    out
}

/// Split a parameter list on top-level commas (ignoring commas nested
/// inside `<...>` generic argument lists) and return `(name, type_text)`
/// for each `name: type` parameter.
fn split_params(params: &str) -> Vec<(String, String)> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for c in params.chars() {
        match c {
            '<' => {
                depth += 1;
                current.push(c);
            }
            '>' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => {
                parts.push(std::mem::take(&mut current));
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current);
    }

    parts
        .into_iter()
        .filter_map(|p| {
            let p = p.trim();
            if p.is_empty() || p == "&self" || p == "self" || p == "mut self" {
                return None;
            }
            let (name, ty) = p.split_once(':')?;
            Some((name.trim().to_string(), ty.trim().to_string()))
        })
        .collect()
}

fn is_untyped_string_conversion(ty: &str) -> bool {
    let normalized: String = ty.chars().filter(|c| !c.is_whitespace()).collect();
    normalized.contains("implInto<String>")
        || normalized.contains("implstd::fmt::Display")
        || normalized.contains("implDisplay")
        || normalized == "String"
}

#[test]
fn facade_signatures_do_not_regress_typed_identifier_parameters() {
    let root = workspace_root();
    let mut violations = Vec::new();

    for file in FACADE_FILES {
        let path = root.join(file);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

        for (function, params) in extract_pub_fn_signatures(&source) {
            if function.ends_with("_raw") {
                continue;
            }
            if ALLOWLIST.contains(&(*file, function.as_str())) {
                continue;
            }

            let function_is_typed_concept = TYPED_CONCEPT_FUNCTIONS.contains(&function.as_str());

            for (param_name, param_type) in split_params(&params) {
                // For functions known to accept an entity type, only the
                // entity-type parameter itself is in scope (by convention
                // named `entity_type` or `ty`) — not unrelated parameters
                // like NBT/position data on the same builder.
                let is_entity_type_position = function_is_typed_concept
                    && matches!(param_name.as_str(), "entity_type" | "ty");
                let looks_typed = is_entity_type_position
                    || LOOKS_LIKE_TYPED_CONCEPT
                        .iter()
                        .any(|needle| param_name.contains(needle));
                if looks_typed && is_untyped_string_conversion(&param_type) {
                    violations.push(Violation {
                        file: (*file).to_string(),
                        function: function.clone(),
                        param: format!("{param_name}: {param_type}"),
                    });
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "found {} public façade parameter(s) that look like a typed identifier/target \
         concept but still accept an untyped string conversion. Either use a typed \
         parameter (see sand_commands::selector::IntoEntityType, sand_core::cmd::IntoGiveItem \
         for precedent), rename the function to end in `_raw` as an explicit escape hatch, \
         or add `(file, function)` to ALLOWLIST in this test with a comment explaining why:\n{:#?}",
        violations.len(),
        violations
    );
}

#[test]
fn guard_actually_detects_a_known_bad_pattern() {
    let source = "pub fn give(selector: Selector, item: impl Into<String>) -> String {}";
    let sigs = extract_pub_fn_signatures(source);
    assert_eq!(sigs.len(), 1);
    let (name, params) = &sigs[0];
    assert_eq!(name, "give");
    let parsed = split_params(params);
    let item_param = parsed.iter().find(|(n, _)| n == "item").unwrap();
    assert!(is_untyped_string_conversion(&item_param.1));
}

#[test]
fn guard_catches_ty_named_entity_type_regression_even_though_name_alone_would_miss_it() {
    // `entity_type`/`not_type` conventionally name their parameter `ty`,
    // which `LOOKS_LIKE_TYPED_CONCEPT`'s substring match on the parameter
    // name would not catch on its own — this is exactly the false
    // negative TYPED_CONCEPT_FUNCTIONS exists to close.
    let source = "pub fn entity_type(mut self, ty: impl Into<String>) -> Self {}";
    let (function, params) = &extract_pub_fn_signatures(source)[0];
    assert!(TYPED_CONCEPT_FUNCTIONS.contains(&function.as_str()));
    let parsed = split_params(params);
    let ty_param = parsed.iter().find(|(n, _)| n == "ty").unwrap();
    assert!(is_untyped_string_conversion(&ty_param.1));
    assert!(!LOOKS_LIKE_TYPED_CONCEPT.iter().any(|n| "ty".contains(n)));
}
