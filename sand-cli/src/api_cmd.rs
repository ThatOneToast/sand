//! Query and rendering support for the installed Sand API contract catalog.
//!
//! The catalog is collected by `sand-api-contract` from compile-time
//! registrations. This module deliberately does not inspect Rust source or
//! contact a remote service: the CLI reports the contracts linked into this
//! exact Sand installation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use sand_api_contract::{ApiCatalog, ApiEntry, ApiKind};

/// Inspect the supported public API bundled with this Sand installation.
#[derive(Debug, Args)]
pub struct ApiArgs {
    #[command(subcommand)]
    command: ApiCommand,
}

#[derive(Debug, Subcommand)]
enum ApiCommand {
    /// Show the complete contract for one API path or alias
    Show {
        /// Canonical API path or a registered re-export alias
        path: String,
    },
    /// Search API contracts using stable, local keyword matching
    Search {
        /// Words to find in paths, summaries, parameters, and behavior
        query: String,
    },
    /// List the direct API contents and nested modules of a module
    Module {
        /// Canonical module path, for example `sand::predicate`
        module_path: String,
    },
    /// Export the installed machine-readable API catalog as JSON
    Export {
        /// Write to this file instead of stdout
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
}

/// Run an `api` command against the contracts linked into the installed CLI.
pub fn run(args: ApiArgs) -> Result<()> {
    let catalog = ApiCatalog::installed(env!("CARGO_PKG_VERSION"))
        .context("failed to collect the installed Sand API contracts")?;

    match args.command {
        ApiCommand::Show { path } => {
            let output = show(&catalog, &path)?;
            print!("{output}");
        }
        ApiCommand::Search { query } => {
            let output = search(&catalog, &query)?;
            print!("{output}");
        }
        ApiCommand::Module { module_path } => {
            let output = module(&catalog, &module_path)?;
            print!("{output}");
        }
        ApiCommand::Export { output } => {
            let json = catalog
                .to_json_pretty()
                .context("failed to serialize the installed API catalog")?;
            if let Some(path) = output {
                std::fs::write(&path, json.as_bytes())
                    .with_context(|| format!("failed to write `{}`", path.display()))?;
            } else {
                print!("{json}");
            }
        }
    }

    Ok(())
}

fn show(catalog: &ApiCatalog, requested_path: &str) -> Result<String> {
    let requested_path = requested_path.trim();
    if requested_path.is_empty() {
        bail!("API path cannot be empty");
    }

    let Some(entry) = find_entry(catalog, requested_path) else {
        let suggestions = suggestions(catalog, requested_path, 3);
        if suggestions.is_empty() {
            bail!("unknown API path `{requested_path}`");
        }
        bail!(
            "unknown API path `{requested_path}`; nearby APIs: {}",
            suggestions.join(", ")
        );
    };

    Ok(render_entry(entry))
}

fn find_entry<'a>(catalog: &'a ApiCatalog, path: &str) -> Option<&'a ApiEntry> {
    catalog.entries.iter().find(|entry| {
        entry.canonical_path == path || entry.aliases.iter().any(|alias| alias == path)
    })
}

fn render_entry(entry: &ApiEntry) -> String {
    let mut output = String::new();
    writeln!(output, "{}", entry.canonical_path).unwrap();
    writeln!(output, "{}", entry.signature).unwrap();
    writeln!(output).unwrap();
    writeln!(output, "{}", entry.summary).unwrap();
    section(&mut output, "Context", &entry.context);
    section(&mut output, "Minecraft behavior", &entry.minecraft);

    if !entry.parameters.is_empty() {
        writeln!(output, "\nParameters").unwrap();
        for parameter in &entry.parameters {
            writeln!(output, "  {}: {}", parameter.name, parameter.description).unwrap();
        }
    }
    if let Some(returns) = &entry.returns {
        section(&mut output, "Returns", returns);
    }
    list_section(&mut output, "Use when", &entry.use_when);
    list_section(&mut output, "Avoid when", &entry.avoid_when);
    section(&mut output, "Example", &entry.example);

    if !entry.availability.is_empty() {
        list_section(&mut output, "Availability", &entry.availability);
    }
    if !entry.aliases.is_empty() {
        list_section(&mut output, "Aliases", &entry.aliases);
    }
    section(
        &mut output,
        "API Contract",
        &format!("sand api show {}", entry.canonical_path),
    );
    output
}

fn section(output: &mut String, heading: &str, value: &str) {
    writeln!(output, "\n{heading}\n  {value}").unwrap();
}

fn list_section(output: &mut String, heading: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    writeln!(output, "\n{heading}").unwrap();
    for value in values {
        writeln!(output, "  - {value}").unwrap();
    }
}

#[derive(Debug)]
struct SearchHit<'a> {
    entry: &'a ApiEntry,
    score: u32,
}

fn search(catalog: &ApiCatalog, query: &str) -> Result<String> {
    let normalized = normalize_query(query)?;
    let mut hits: Vec<_> = catalog
        .entries
        .iter()
        .filter_map(|entry| {
            let score = search_score(entry, &normalized);
            (score > 0).then_some(SearchHit { entry, score })
        })
        .collect();
    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.entry.canonical_path.cmp(&b.entry.canonical_path))
    });

    let mut output = String::new();
    if hits.is_empty() {
        writeln!(output, "No APIs matched `{}`.", query.trim()).unwrap();
        return Ok(output);
    }

    writeln!(output, "API matches for `{}`:", query.trim()).unwrap();
    for hit in hits {
        writeln!(
            output,
            "  {}  [{}]\n    {}",
            hit.entry.canonical_path,
            kind_name(hit.entry.kind),
            hit.entry.summary
        )
        .unwrap();
    }
    Ok(output)
}

fn normalize_query(query: &str) -> Result<Vec<String>> {
    let words: Vec<_> = query
        .split_whitespace()
        .map(|word| word.to_lowercase())
        .filter(|word| !word.is_empty())
        .collect();
    if words.is_empty() {
        bail!("search query cannot be empty");
    }
    Ok(words)
}

/// Stable, explainable lexical rank. Every query word must occur somewhere.
/// Path matches carry the most weight, then aliases and summary, followed by
/// domain prose. Exact path/alias matches always sort first.
fn search_score(entry: &ApiEntry, words: &[String]) -> u32 {
    let path = entry.canonical_path.to_lowercase();
    let aliases: Vec<_> = entry
        .aliases
        .iter()
        .map(|alias| alias.to_lowercase())
        .collect();
    let summary = entry.summary.to_lowercase();
    let parameters = entry
        .parameters
        .iter()
        .map(|parameter| format!("{} {}", parameter.name, parameter.description))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let guidance = entry
        .use_when
        .iter()
        .chain(entry.avoid_when.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let minecraft = entry.minecraft.to_lowercase();

    let exact_query = words.join(" ");
    if path == exact_query || aliases.iter().any(|alias| alias == &exact_query) {
        return 100_000;
    }

    let mut total = 0;
    for word in words {
        let word_score = if path.split("::").any(|segment| segment == word) {
            8_000
        } else if path.contains(word) {
            6_000
        } else if aliases.iter().any(|alias| alias.contains(word)) {
            5_000
        } else if summary.contains(word) {
            4_000
        } else if parameters.contains(word) {
            2_000
        } else if guidance.contains(word) {
            1_500
        } else if minecraft.contains(word) {
            1_000
        } else {
            return 0;
        };
        total += word_score;
    }
    total
}

fn module(catalog: &ApiCatalog, requested_module: &str) -> Result<String> {
    let requested_module = requested_module.trim().trim_end_matches("::");
    if requested_module.is_empty() {
        bail!("module path cannot be empty");
    }

    let mut direct: BTreeMap<&str, Vec<&ApiEntry>> = BTreeMap::new();
    let mut nested: BTreeMap<String, usize> = BTreeMap::new();
    let nested_prefix = format!("{requested_module}::");

    for entry in &catalog.entries {
        if entry.canonical_module == requested_module {
            direct
                .entry(kind_heading(entry.kind))
                .or_default()
                .push(entry);
        } else if let Some(remainder) = entry.canonical_module.strip_prefix(&nested_prefix) {
            if let Some(segment) = remainder.split("::").next() {
                *nested
                    .entry(format!("{requested_module}::{segment}"))
                    .or_default() += 1;
            }
        }
    }

    if direct.is_empty() && nested.is_empty() {
        let modules: BTreeSet<_> = catalog
            .entries
            .iter()
            .map(|entry| entry.canonical_module.as_str())
            .collect();
        let suggestions = nearest_strings(modules.into_iter(), requested_module, 3);
        if suggestions.is_empty() {
            bail!("unknown API module `{requested_module}`");
        }
        bail!(
            "unknown API module `{requested_module}`; nearby modules: {}",
            suggestions.join(", ")
        );
    }

    let mut output = format!("Module {requested_module}\n");
    for (heading, entries) in &mut direct {
        entries.sort_by(|a, b| a.canonical_path.cmp(&b.canonical_path));
        writeln!(output, "\n{heading}").unwrap();
        for entry in entries {
            writeln!(output, "  {}\n    {}", entry.canonical_path, entry.summary).unwrap();
        }
    }
    if !nested.is_empty() {
        writeln!(output, "\nNested modules").unwrap();
        for (path, count) in nested {
            writeln!(
                output,
                "  {path} ({count} {})",
                if count == 1 { "API" } else { "APIs" }
            )
            .unwrap();
        }
    }
    Ok(output)
}

fn kind_name(kind: ApiKind) -> &'static str {
    match kind {
        ApiKind::Module => "module",
        ApiKind::Struct => "struct",
        ApiKind::Enum => "enum",
        ApiKind::Variant => "variant",
        ApiKind::Trait => "trait",
        ApiKind::Function => "function",
        ApiKind::Method => "method",
        ApiKind::TraitMethod => "trait method",
        ApiKind::TypeAlias => "type alias",
        ApiKind::Constant => "constant",
        ApiKind::Macro => "macro",
    }
}

fn kind_heading(kind: ApiKind) -> &'static str {
    match kind {
        ApiKind::Module => "Modules",
        ApiKind::Struct => "Structs",
        ApiKind::Enum => "Enums",
        ApiKind::Variant => "Variants",
        ApiKind::Trait => "Traits",
        ApiKind::Function => "Functions",
        ApiKind::Method => "Methods",
        ApiKind::TraitMethod => "Trait methods",
        ApiKind::TypeAlias => "Type aliases",
        ApiKind::Constant => "Constants",
        ApiKind::Macro => "Macros",
    }
}

fn suggestions(catalog: &ApiCatalog, requested_path: &str, limit: usize) -> Vec<String> {
    let candidates = catalog.entries.iter().flat_map(|entry| {
        std::iter::once(entry.canonical_path.as_str())
            .chain(entry.aliases.iter().map(String::as_str))
    });
    nearest_strings(candidates, requested_path, limit)
}

fn nearest_strings<'a>(
    candidates: impl Iterator<Item = &'a str>,
    requested: &str,
    limit: usize,
) -> Vec<String> {
    let requested = requested.to_lowercase();
    let mut ranked: Vec<_> = candidates
        .map(|candidate| {
            (
                edit_distance(&candidate.to_lowercase(), &requested),
                candidate,
            )
        })
        .collect();
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
    ranked.dedup_by(|a, b| a.1 == b.1);
    ranked
        .into_iter()
        .take(limit)
        .map(|(_, candidate)| candidate.to_owned())
        .collect()
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right: Vec<_> = right.chars().collect();
    let mut previous: Vec<_> = (0..=right.len()).collect();
    for (left_index, left_char) in left.chars().enumerate() {
        let mut current = Vec::with_capacity(right.len() + 1);
        current.push(left_index + 1);
        for (right_index, right_char) in right.iter().enumerate() {
            current.push(std::cmp::min(
                std::cmp::min(current[right_index] + 1, previous[right_index + 1] + 1),
                previous[right_index] + usize::from(left_char != *right_char),
            ));
        }
        previous = current;
    }
    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use sand_api_contract::ApiParameter;

    fn entry(path: &str, kind: ApiKind, summary: &str, module: &str) -> ApiEntry {
        ApiEntry {
            canonical_path: path.into(),
            aliases: Vec::new(),
            kind,
            signature: format!("pub fn {}()", path.rsplit("::").next().unwrap()),
            summary: summary.into(),
            context: "Reusable author-facing context.".into(),
            minecraft: "Generates deterministic Minecraft data.".into(),
            use_when: vec!["Building an equipment predicate.".into()],
            avoid_when: vec!["Mutable scoreboard arithmetic.".into()],
            parameters: Vec::new(),
            returns: Some("A typed builder.".into()),
            example: "Predicate::new(id, condition)".into(),
            availability: Vec::new(),
            canonical_module: module.into(),
        }
    }

    fn catalog(mut entries: Vec<ApiEntry>) -> ApiCatalog {
        entries.sort_by(|a, b| a.canonical_path.cmp(&b.canonical_path));
        ApiCatalog {
            schema_version: 1,
            sand_version: "0.1.0".into(),
            entries,
        }
    }

    #[test]
    fn show_resolves_alias_and_renders_all_contract_sections() {
        let mut predicate = entry(
            "sand::predicate::Predicate::new",
            ApiKind::Method,
            "Creates a reusable Minecraft predicate resource.",
            "sand::predicate",
        );
        predicate.aliases = vec!["sand::prelude::Predicate::new".into()];
        predicate.parameters = vec![ApiParameter {
            name: "id".into(),
            description: "The namespaced predicate identifier.".into(),
        }];
        predicate.availability = vec!["all configurations".into()];
        let catalog = catalog(vec![predicate]);

        let rendered = show(&catalog, "sand::prelude::Predicate::new").unwrap();
        assert!(rendered.contains("sand::predicate::Predicate::new"));
        assert!(rendered.contains("Parameters\n  id: The namespaced predicate identifier."));
        assert!(rendered.contains("Minecraft behavior"));
        assert!(rendered.contains("sand api show sand::predicate::Predicate::new"));
    }

    #[test]
    fn unknown_path_suggestions_are_stable_and_use_aliases() {
        let mut predicate = entry(
            "sand::predicate::Predicate",
            ApiKind::Struct,
            "A reusable predicate.",
            "sand::predicate",
        );
        predicate.aliases = vec!["sand::prelude::Predicate".into()];
        let catalog = catalog(vec![
            predicate,
            entry(
                "sand::component::Recipe",
                ApiKind::Struct,
                "A recipe.",
                "sand::component",
            ),
        ]);

        let error = show(&catalog, "sand::prelude::Predicat").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("nearby APIs: sand::prelude::Predicate")
        );
    }

    #[test]
    fn search_ranking_prefers_path_then_summary_then_domain_prose() {
        let path_hit = entry(
            "sand::equipment::Equipment",
            ApiKind::Struct,
            "A typed loadout.",
            "sand::equipment",
        );
        let summary_hit = entry(
            "sand::item::Loadout",
            ApiKind::Struct,
            "Describes entity equipment.",
            "sand::item",
        );
        let prose_hit = entry(
            "sand::predicate::Predicate",
            ApiKind::Struct,
            "A reusable condition.",
            "sand::predicate",
        );
        let catalog = catalog(vec![prose_hit, summary_hit, path_hit]);

        let rendered = search(&catalog, "equipment").unwrap();
        let path_position = rendered.find("sand::equipment::Equipment").unwrap();
        let summary_position = rendered.find("sand::item::Loadout").unwrap();
        let prose_position = rendered.find("sand::predicate::Predicate").unwrap();
        assert!(path_position < summary_position);
        assert!(summary_position < prose_position);
    }

    #[test]
    fn every_search_word_must_match() {
        let catalog = catalog(vec![entry(
            "sand::equipment::Equipment",
            ApiKind::Struct,
            "A typed loadout.",
            "sand::equipment",
        )]);
        assert_eq!(
            search(&catalog, "equipment missing").unwrap(),
            "No APIs matched `equipment missing`.\n"
        );
    }

    #[test]
    fn module_groups_direct_items_and_distinguishes_nested_modules() {
        let catalog = catalog(vec![
            entry(
                "sand::predicate::Predicate",
                ApiKind::Struct,
                "A reusable predicate.",
                "sand::predicate",
            ),
            entry(
                "sand::predicate::create",
                ApiKind::Function,
                "Creates a predicate.",
                "sand::predicate",
            ),
            entry(
                "sand::predicate::condition::Entity",
                ApiKind::Struct,
                "An entity condition.",
                "sand::predicate::condition",
            ),
        ]);

        let rendered = module(&catalog, "sand::predicate").unwrap();
        assert!(rendered.contains("Functions\n  sand::predicate::create"));
        assert!(rendered.contains("Structs\n  sand::predicate::Predicate"));
        assert!(rendered.contains("Nested modules\n  sand::predicate::condition (1 API)"));
        assert!(!rendered.contains("sand::predicate::condition::Entity"));
    }

    #[test]
    fn edit_distance_handles_unicode_without_byte_indexing() {
        assert_eq!(edit_distance("café", "cafe"), 1);
        assert_eq!(edit_distance("predicate", "predicat"), 1);
    }
}
