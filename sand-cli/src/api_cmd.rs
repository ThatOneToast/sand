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
// Force the author-facing facade into the final binary so its distributed
// contract registrations are present in the installed catalog.
use sand as _;
use sand_api_contract::{ApiCatalog, ApiEntry, ApiKind, ApiParameter, CoverageStatus};

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
        #[arg(required = true, num_args = 1..)]
        query: Vec<String>,
        /// Maximum results to print (defaults to 20)
        #[arg(long, value_parser = parse_positive_usize, conflicts_with = "all")]
        limit: Option<usize>,
        /// Print every matching result
        #[arg(long)]
        all: bool,
        /// Restrict results to one canonical module or its descendants
        #[arg(long)]
        module: Option<String>,
        /// Restrict results to one API kind (for example method or struct)
        #[arg(long)]
        kind: Option<String>,
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

fn parse_positive_usize(value: &str) -> std::result::Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "expected a positive integer".to_owned())?;
    if parsed == 0 {
        Err("expected a positive integer".to_owned())
    } else {
        Ok(parsed)
    }
}

/// Run an `api` command against the contracts linked into the installed CLI.
pub fn run(args: ApiArgs) -> Result<()> {
    let catalog = installed_catalog()?;

    match args.command {
        ApiCommand::Show { path } => {
            let output = show(&catalog, &path)?;
            print!("{output}");
        }
        ApiCommand::Search {
            query,
            limit,
            all,
            module,
            kind,
        } => {
            let output = search_with_options(
                &catalog,
                &query.join(" "),
                if all { None } else { Some(limit.unwrap_or(20)) },
                module.as_deref(),
                kind.as_deref(),
            )?;
            print!("{output}");
        }
        ApiCommand::Module { module_path } => {
            let output = module(&catalog, &module_path)?;
            print!("{output}");
        }
        ApiCommand::Export { output } => {
            if let Some(json) = export(&catalog, output.as_deref())? {
                print!("{json}");
            }
        }
    }

    Ok(())
}

fn installed_catalog() -> Result<ApiCatalog> {
    let coverage = sand::__private::api_contract::installed_coverage();
    let configuration = sand::__private::api_contract::installed_configuration();
    let mut entries = sand_api_contract::inventory::iter::<sand_api_contract::ApiRegistration>
        .into_iter()
        .map(ApiEntry::from)
        .collect::<Vec<_>>();
    for expected in sand::__private::api_contract::INSTALLED_FACADE_CONTRACTS {
        let expected = ApiEntry::from(expected);
        let actual = entries
            .iter()
            .filter(|entry| entry.canonical_path == expected.canonical_path)
            .collect::<Vec<_>>();
        let [actual] = actual.as_slice() else {
            bail!(
                "linked facade contract `{}` has {} runtime registrations; expected exactly one registration matching the build-validated source declaration",
                expected.canonical_path,
                actual.len()
            );
        };
        if **actual != expected {
            bail!(
                "linked facade contract `{}` differs from the build-validated source declaration",
                expected.canonical_path
            );
        }
    }

    for provider_json in sand::__private::api_contract::GENERATED_API_PROVIDER_CATALOGS {
        let provider: sand_build::ApiProviderCatalog = serde_json::from_str(provider_json)
            .context("failed to parse an installed generated API provider")?;
        provider
            .validate()
            .map_err(anyhow::Error::msg)
            .with_context(|| {
                format!(
                    "installed generated API provider `{}` is invalid",
                    provider.provider
                )
            })?;
        if provider.minecraft_version != configuration.minecraft_version
            || (provider.provider != "generated_registry_id_contracts"
                && provider.placeholder != configuration.placeholder_codegen)
        {
            bail!(
                "installed generated API provider `{}` targets Minecraft {} (placeholder={}), but the catalog configuration targets {} (placeholder={})",
                provider.provider,
                provider.minecraft_version,
                provider.placeholder,
                configuration.minecraft_version,
                configuration.placeholder_codegen
            );
        }
        entries.extend(provider.entries.into_iter().map(|entry| entry.contract));
    }

    let installed_paths = sand::__private::api_contract::INSTALLED_API_PATHS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    entries.retain(|entry| {
        installed_paths.contains(entry.canonical_path.as_str())
            || entry
                .aliases
                .iter()
                .any(|alias| installed_paths.contains(alias.as_str()))
    });
    for entry in &mut entries {
        let family_contract = sand::__private::api_contract::INSTALLED_FAMILY_API_PATHS
            .contains(&entry.canonical_path.as_str());
        if let Some(paths) = sand::__private::api_contract::INSTALLED_API_IDENTITIES
            .iter()
            .find(|paths| paths.contains(&entry.canonical_path.as_str()))
        {
            entry.aliases = paths
                .iter()
                .filter(|path| **path != entry.canonical_path)
                .map(|path| (*path).to_owned())
                .collect();
        }
        // A procedural macro's Rust implementation function is not its author-facing
        // invocation syntax, so its explicit facade contract remains authoritative.
        if entry.kind == ApiKind::Macro {
            continue;
        }
        if let Some((_, signature, parameters, return_type, documentation, has_receiver)) =
            sand::__private::api_contract::INSTALLED_API_SHAPES
                .iter()
                .find(|(paths, ..)| paths.contains(&entry.canonical_path.as_str()))
        {
            entry.signature = (*signature).to_owned();
            let source_summary = rustdoc_summary(documentation);
            let resolved_summary = source_summary
                .clone()
                .unwrap_or_else(|| semantic_fallback_summary(entry));
            let semantic_summary = resolved_summary.as_str();
            let authored = entry
                .parameters
                .iter()
                .map(|parameter| (parameter.name.as_str(), parameter.description.as_str()))
                .collect::<BTreeMap<_, _>>();
            entry.parameters = parameters
                .iter()
                .map(|(name, ty)| ApiParameter {
                    name: (*name).to_owned(),
                    rust_type: Some((*ty).to_owned()),
                    description: authored.get(name).map_or_else(
                        || semantic_parameter_description(name, ty, semantic_summary),
                        |description| (*description).to_owned(),
                    ),
                })
                .collect();
            entry.returns = return_type.map(|ty| {
                entry.returns.clone().unwrap_or_else(|| {
                    semantic_return_description(ty, semantic_summary, &entry.canonical_module)
                })
            });
            entry.return_type = return_type.map(|ty| (*ty).to_owned());
            if family_contract {
                let summary = resolved_summary;
                let prose = if documentation.trim().is_empty() {
                    format!(
                        "{summary} The exact source-derived Rust declaration is `{}`.",
                        entry.signature
                    )
                } else {
                    rustdoc_prose(documentation)
                };
                entry.summary = summary.clone();
                entry.context = prose.clone();
                entry.minecraft = format!(
                    "Minecraft and generated-output behavior follows the defining item's documented semantics: {prose}"
                );
                entry.use_when = vec![format!(
                    "When the defining item's documented behavior is required: {summary}"
                )];
                entry.avoid_when = vec![rustdoc_avoidance(&prose)];
            } else if let Some(summary) = source_summary {
                if is_family_template_summary(&entry.summary) {
                    entry.summary = summary.clone();
                }
                if !entry.context.contains(&summary) {
                    entry.context = format!("{summary} {}", entry.context);
                }
            }
            if is_import_only_example(&entry.example) {
                entry.example = rustdoc_example(documentation).unwrap_or_else(|| {
                    structural_example(
                        entry.kind,
                        &entry.canonical_path,
                        parameters,
                        *return_type,
                        *has_receiver,
                    )
                });
            }
        }
    }

    // Provider catalogs created before source-shape resolution may carry a
    // name-derived `Type()` example. A type declaration is not a function;
    // retain an exact, compilable import reference instead of fabricating a
    // constructor that may be private or may not exist.
    for entry in &mut entries {
        if !matches!(
            entry.kind,
            ApiKind::Function | ApiKind::Method | ApiKind::TraitMethod
        ) && entry.example.trim_end().ends_with("();")
        {
            entry.example = structural_example(entry.kind, &entry.canonical_path, &[], None, false);
        }
    }

    let catalog = ApiCatalog::from_entries_with_coverage(
        env!("CARGO_PKG_VERSION"),
        configuration,
        entries,
        coverage,
    )
    .context("failed to assemble the installed Sand API catalog")?;
    catalog
        .validate_quality()
        .context("installed Sand API catalog failed resolved quality validation")?;
    Ok(catalog)
}

fn rustdoc_prose(documentation: &str) -> String {
    let mut in_code = false;
    let mut paragraphs = Vec::new();
    let mut current = Vec::new();
    for line in documentation.lines() {
        let line = line.trim();
        if line.eq_ignore_ascii_case("# API Contract") {
            break;
        }
        if line.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code || line.starts_with('#') {
            continue;
        }
        if line.is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
            continue;
        }
        current.push(line.replace("**", ""));
    }
    if !current.is_empty() {
        paragraphs.push(current.join(" "));
    }
    let prose = paragraphs.into_iter().take(4).collect::<Vec<_>>().join(" ");
    if prose.chars().count() <= 1_200 {
        prose
    } else {
        let end = prose
            .char_indices()
            .nth(1_197)
            .map_or(prose.len(), |(index, _)| index);
        format!("{}...", prose[..end].trim_end())
    }
}

fn semantic_parameter_description(name: &str, rust_type: &str, summary: &str) -> String {
    let normalized = name.trim_start_matches('_').replace('_', " ");
    let compact_type = rust_type.replace(' ', "");
    if compact_type.contains("Text") {
        return "Supplies the typed player-visible Minecraft text rendered by this operation."
            .to_owned();
    }
    if compact_type.contains("Selector") || compact_type.contains("EntityTarget") {
        return "Selects the Minecraft entity or entities affected by this operation.".to_owned();
    }
    if compact_type.contains("Condition") || compact_type.contains("Predicate") {
        return "Defines the condition that must hold for the documented behavior to apply."
            .to_owned();
    }
    if compact_type.contains("ResourceLocation") || compact_type.ends_with("Id") {
        return "Supplies the validated Minecraft resource identifier used by this operation."
            .to_owned();
    }
    let purpose = match name.trim_start_matches('_') {
        "selector" | "target" | "targets" | "entity" | "entities" | "player" | "players" => {
            "Selects the Minecraft entity or entities affected by this operation."
        }
        "id" | "key" | "name" | "field_name" | "objective" | "tag" => {
            "Identifies the named Minecraft resource, field, objective, or tag used by this operation."
        }
        "path" | "source_path" | "root_path" | "full_path" => {
            "Selects the structured NBT or resource path addressed by this operation."
        }
        "value" | "default_value" | "default_score" => {
            "Supplies the typed value written, compared, or configured by this operation."
        }
        "condition" | "predicate" | "item_predicate" | "location_predicate"
        | "position_predicate" => {
            "Defines the condition that must hold for the documented behavior to apply."
        }
        "duration" | "ticks" | "seconds" | "fade_in" | "stay" | "fade_out" => {
            "Controls the documented Minecraft duration or timing interval."
        }
        "index" | "count" | "amount" | "limit" | "max" | "min" | "scale" => {
            "Supplies the numeric bound, position, amount, or scale used by this operation."
        }
        "text" | "message" | "name_text" | "description" => {
            "Supplies the typed player-visible Minecraft text rendered by this operation."
        }
        "source" => "Selects the typed source from which this operation reads or copies data.",
        "profile" => "Selects the Minecraft command profile used for validation and rendering.",
        _ if rust_type.contains("bool") => {
            "Enables or disables the documented behavior for the resulting value."
        }
        _ => {
            return format!(
                "Supplies `{normalized}` to the documented operation: {}",
                summary.trim_end_matches('.')
            );
        }
    };
    purpose.to_owned()
}

fn semantic_fallback_summary(entry: &ApiEntry) -> String {
    let generic = entry.summary.starts_with("Configures or performs ")
        || entry.summary.starts_with("Builds or resolves ")
        || entry
            .summary
            .contains("on this typed datapack component definition");
    if !generic {
        return entry.summary.clone();
    }
    let (owner_path, member) = entry
        .canonical_path
        .rsplit_once("::")
        .unwrap_or((&entry.canonical_path, &entry.canonical_path));
    let owner = owner_path.rsplit("::").next().unwrap_or(owner_path);
    let member_words = member.replace('_', " ");
    match entry.kind {
        ApiKind::Field => {
            format!("Stores the `{member}` value used by the typed `{owner}` definition.")
        }
        ApiKind::AssociatedConst | ApiKind::Constant => {
            format!("Defines the `{member}` constant used by the typed `{owner}` API.")
        }
        _ => {
            format!("Performs the documented {member_words} operation for the typed `{owner}` API.")
        }
    }
}

fn semantic_return_description(rust_type: &str, summary: &str, module: &str) -> String {
    let compact = rust_type.replace(' ', "");
    if compact.contains("Result<") {
        return "The validated result, or a diagnostic describing why the input cannot be represented safely.".to_owned();
    }
    if compact == "bool" {
        return "Whether the documented condition holds for this value.".to_owned();
    }
    if compact.starts_with("Option<") {
        return "The documented value when it is present; otherwise `None`.".to_owned();
    }
    if compact == "Self" {
        return "The updated typed builder, ready for further chained configuration.".to_owned();
    }
    if module.starts_with("sand::command") && compact == "String" {
        return "The rendered Minecraft command text.".to_owned();
    }
    if compact.starts_with('&') && compact.contains("str") {
        return "A borrowed textual representation of the documented value, without allocation."
            .to_owned();
    }
    if compact.starts_with('&') {
        return "A borrowed view of the documented value.".to_owned();
    }
    if compact.starts_with("Vec<") || compact.contains("Iterator") {
        return "The ordered values produced by the documented operation.".to_owned();
    }
    format!(
        "The typed result of the documented operation: {}",
        summary.trim_end_matches('.')
    )
}

fn rustdoc_avoidance(prose: &str) -> String {
    prose
        .split(". ")
        .find(|sentence| {
            let sentence = sentence.replace("**", "").to_ascii_lowercase();
            [
                " do not ",
                " does not ",
                " not ",
                " only ",
                " instead",
                " rejected",
            ]
            .iter()
            .any(|needle| sentence.contains(needle))
        })
        .map(|sentence| {
            format!(
                "When this documented limitation applies: {}.",
                sentence.trim_end_matches('.')
            )
        })
        .unwrap_or_else(|| {
            "When the defining item's documented preconditions or scope do not apply.".to_owned()
        })
}

fn is_family_template_summary(summary: &str) -> bool {
    (summary.contains("typed ") && summary.ends_with(" API."))
        || summary.starts_with("Provides ") && summary.contains(" author API")
        || summary.starts_with("Exposes ") && summary.contains(" author API")
}

fn rustdoc_summary(documentation: &str) -> Option<String> {
    let paragraphs = documentation
        .split("\n\n")
        .map(|paragraph| {
            paragraph
                .lines()
                .map(str::trim)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|paragraph| !paragraph.is_empty() && !paragraph.starts_with('#'))
        .collect::<Vec<_>>();
    let first = paragraphs.first()?.trim().to_owned();
    if first.len() >= 48 || paragraphs.len() == 1 {
        return Some(first);
    }
    let second = paragraphs[1]
        .split_once(". ")
        .map_or(paragraphs[1].as_str(), |(sentence, _)| sentence);
    Some(format!(
        "{}. {}.",
        first.trim_end_matches('.'),
        second.trim_end_matches('.')
    ))
}

fn rustdoc_example(documentation: &str) -> Option<String> {
    let mut code = Vec::new();
    let mut in_rust_fence = false;
    for line in documentation.lines() {
        let trimmed = line.trim();
        if !in_rust_fence {
            let Some(info) = trimmed.strip_prefix("```") else {
                continue;
            };
            let language = info.split(',').next().unwrap_or_default().trim();
            in_rust_fence = language.is_empty()
                || matches!(language, "rust" | "ignore" | "no_run" | "compile_fail");
            continue;
        }
        if trimmed.starts_with("```") {
            break;
        }
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            code.push(trimmed);
        }
    }
    (!code.is_empty()).then(|| {
        code.join("\n")
            .replace("sand_core::", "sand::")
            .replace("sand_commands::", "sand::command::")
            .replace("sand_components::", "sand::component::")
            .replace("sand_resourcepack::", "sand::resourcepack::")
    })
}

fn is_import_only_example(example: &str) -> bool {
    let trimmed = example.trim();
    trimmed.starts_with("use sand::") && trimmed.lines().count() == 1
}

fn structural_example(
    kind: ApiKind,
    canonical_path: &str,
    parameters: &[(&str, &str)],
    return_type: Option<&str>,
    has_receiver: bool,
) -> String {
    if !matches!(
        kind,
        ApiKind::Function | ApiKind::Method | ApiKind::TraitMethod
    ) {
        let import_path = match kind {
            ApiKind::Variant
            | ApiKind::Field
            | ApiKind::AssociatedConst
            | ApiKind::AssociatedType => canonical_path
                .rsplit_once("::")
                .map_or(canonical_path, |(owner, _)| owner),
            _ => canonical_path,
        };
        return format!("use {import_path};");
    }

    let arguments = parameters
        .iter()
        .map(|(name, ty)| format!("`{name}: {ty}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let receiver = if has_receiver {
        " on an existing receiver"
    } else {
        ""
    };
    let arguments = if arguments.is_empty() {
        "no explicit arguments".to_owned()
    } else {
        arguments
    };
    return_type.map_or_else(
        || {
            format!(
                "// Call `{canonical_path}`{receiver} using {arguments} to perform the documented operation."
            )
        },
        |ty| {
            format!(
                "// Call `{canonical_path}`{receiver} using {arguments}; it returns `{ty}`."
            )
        },
    )
}

fn export(catalog: &ApiCatalog, output: Option<&std::path::Path>) -> Result<Option<String>> {
    let json = catalog
        .to_json_pretty()
        .context("failed to serialize the installed API catalog")?;
    if let Some(path) = output {
        std::fs::write(path, json.as_bytes())
            .with_context(|| format!("failed to write `{}`", path.display()))?;
        Ok(None)
    } else {
        Ok(Some(json))
    }
}

fn coverage_notice(catalog: &ApiCatalog) -> String {
    if catalog.coverage.status == CoverageStatus::Complete {
        String::new()
    } else {
        format!(
            "API contract migration is partial: {} static items and {} scopes remain pending.\n\n",
            catalog.coverage.pending_item_ceiling, catalog.coverage.pending_scope_ceiling
        )
    }
}

fn show(catalog: &ApiCatalog, requested_path: &str) -> Result<String> {
    let requested_path = requested_path.trim();
    if requested_path.is_empty() {
        bail!("API path cannot be empty");
    }

    let Some(entry) = catalog.find(requested_path) else {
        let suggestions = suggestions(catalog, requested_path, 3);
        if suggestions.is_empty() {
            bail!("unknown API path `{requested_path}`");
        }
        bail!(
            "unknown API path `{requested_path}`; nearby APIs: {}",
            suggestions.join(", ")
        );
    };

    Ok(format!(
        "{}{}",
        coverage_notice(catalog),
        render_entry(entry)
    ))
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
            if let Some(rust_type) = &parameter.rust_type {
                writeln!(
                    output,
                    "  {} (`{}`): {}",
                    parameter.name, rust_type, parameter.description
                )
                .unwrap();
            } else {
                writeln!(output, "  {}: {}", parameter.name, parameter.description).unwrap();
            }
        }
    }
    if let Some(returns) = &entry.returns {
        let rendered = entry
            .return_type
            .as_ref()
            .map_or_else(|| returns.clone(), |ty| format!("`{ty}` — {returns}"));
        section(&mut output, "Returns", &rendered);
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

#[cfg(test)]
fn search(catalog: &ApiCatalog, query: &str) -> Result<String> {
    search_with_options(catalog, query, Some(20), None, None)
}

fn search_with_options(
    catalog: &ApiCatalog,
    query: &str,
    limit: Option<usize>,
    module_filter: Option<&str>,
    kind_filter: Option<&str>,
) -> Result<String> {
    if query.trim().is_empty() {
        bail!("search query cannot be empty");
    }
    let requested_kind = kind_filter.map(parse_kind).transpose()?;
    let hits = catalog
        .search(query)
        .into_iter()
        .filter(|entry| {
            module_filter.is_none_or(|module| {
                entry.canonical_module == module
                    || entry
                        .canonical_module
                        .strip_prefix(module)
                        .is_some_and(|suffix| suffix.starts_with("::"))
            }) && requested_kind.is_none_or(|kind| entry.kind == kind)
        })
        .collect::<Vec<_>>();

    let mut output = coverage_notice(catalog);
    if hits.is_empty() {
        writeln!(output, "No APIs matched `{}`.", query.trim()).unwrap();
        return Ok(output);
    }

    let total = hits.len();
    let shown = limit.map_or(total, |limit| total.min(limit));
    writeln!(
        output,
        "API matches for `{}` (showing {shown} of {total}):",
        query.trim()
    )
    .unwrap();
    for entry in hits.into_iter().take(shown) {
        writeln!(
            output,
            "  {}  [{}]\n    {}",
            entry.canonical_path,
            kind_name(entry.kind),
            entry.summary
        )
        .unwrap();
    }
    Ok(output)
}

fn parse_kind(value: &str) -> Result<ApiKind> {
    let normalized = value.trim().to_ascii_lowercase().replace(['-', ' '], "_");
    let kind = match normalized.as_str() {
        "module" => ApiKind::Module,
        "struct" => ApiKind::Struct,
        "enum" => ApiKind::Enum,
        "variant" => ApiKind::Variant,
        "trait" => ApiKind::Trait,
        "function" | "fn" => ApiKind::Function,
        "method" => ApiKind::Method,
        "trait_method" => ApiKind::TraitMethod,
        "type_alias" => ApiKind::TypeAlias,
        "constant" | "const" => ApiKind::Constant,
        "associated_const" => ApiKind::AssociatedConst,
        "associated_type" => ApiKind::AssociatedType,
        "field" => ApiKind::Field,
        "macro" => ApiKind::Macro,
        _ => bail!("unknown API kind `{value}`"),
    };
    Ok(kind)
}

fn module(catalog: &ApiCatalog, requested_module: &str) -> Result<String> {
    let requested_module = requested_module.trim().trim_end_matches("::");
    if requested_module.is_empty() {
        bail!("module path cannot be empty");
    }

    let direct_by_kind = catalog.module(requested_module);
    let mut direct: BTreeMap<&str, Vec<&ApiEntry>> = BTreeMap::new();
    for (kind, entries) in direct_by_kind {
        direct.insert(kind_heading(kind), entries);
    }
    let mut nested: BTreeMap<String, usize> = BTreeMap::new();
    let nested_prefix = format!("{requested_module}::");

    for entry in &catalog.entries {
        if entry.canonical_module != requested_module
            && let Some(remainder) = entry.canonical_module.strip_prefix(&nested_prefix)
            && let Some(segment) = remainder.split("::").next()
        {
            *nested
                .entry(format!("{requested_module}::{segment}"))
                .or_default() += 1;
        }
    }

    let contracted_module = catalog
        .find(requested_module)
        .is_some_and(|entry| entry.kind == ApiKind::Module);
    if direct.is_empty() && nested.is_empty() && !contracted_module {
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

    let mut output = format!("{}Module {requested_module}\n", coverage_notice(catalog));
    if direct.is_empty() && nested.is_empty() {
        output.push_str("\nNo direct APIs are registered for this module.\n");
        return Ok(output);
    }
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
        ApiKind::AssociatedConst => "associated constant",
        ApiKind::AssociatedType => "associated type",
        ApiKind::Field => "field",
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
        ApiKind::AssociatedConst => "Associated constants",
        ApiKind::AssociatedType => "Associated types",
        ApiKind::Field => "Fields",
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
    use std::sync::OnceLock;

    use super::*;
    use sand_api_contract::ApiParameter;

    fn generated_catalog() -> &'static ApiCatalog {
        static CATALOG: OnceLock<ApiCatalog> = OnceLock::new();
        CATALOG.get_or_init(|| installed_catalog().unwrap())
    }

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
            return_type: Some("Predicate".into()),
            example: "Predicate::new(id, condition)".into(),
            availability: Vec::new(),
            canonical_module: module.into(),
        }
    }

    fn catalog(mut entries: Vec<ApiEntry>) -> ApiCatalog {
        entries.sort_by(|a, b| a.canonical_path.cmp(&b.canonical_path));
        let compiled_surface_items = entries.len();
        ApiCatalog {
            schema_version: sand_api_contract::SCHEMA_VERSION,
            sand_version: "0.1.0".into(),
            configuration: sand_api_contract::ApiConfiguration {
                surface_profile: "test".into(),
                minecraft_version: "test".into(),
                cargo_features: Vec::new(),
                placeholder_codegen: false,
                compiled_surface_items,
            },
            coverage: sand_api_contract::ApiCoverage {
                status: CoverageStatus::Complete,
                static_surface_items: entries.len(),
                pending_item_ceiling: 0,
                pending_scope_ceiling: 0,
                pending_scopes: Vec::new(),
            },
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
            rust_type: Some("ResourceLocation".into()),
            description: "The namespaced predicate identifier.".into(),
        }];
        predicate.availability = vec!["all configurations".into()];
        let catalog = catalog(vec![predicate]);

        let rendered = show(&catalog, "sand::prelude::Predicate::new").unwrap();
        assert!(rendered.contains("sand::predicate::Predicate::new"));
        assert!(rendered.contains(
            "Parameters\n  id (`ResourceLocation`): The namespaced predicate identifier."
        ));
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
    fn exploratory_search_matches_any_word_and_reports_partial_results() {
        let catalog = catalog(vec![entry(
            "sand::equipment::Equipment",
            ApiKind::Struct,
            "A typed loadout.",
            "sand::equipment",
        )]);
        assert!(
            search(&catalog, "equipment missing")
                .unwrap()
                .contains("sand::equipment::Equipment")
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
    fn module_accepts_a_contracted_module_without_children() {
        let catalog = catalog(vec![entry(
            "sand::inventory",
            ApiKind::Module,
            "Typed inventory locations.",
            "sand",
        )]);
        assert_eq!(
            module(&catalog, "sand::inventory").unwrap(),
            "Module sand::inventory\n\nNo direct APIs are registered for this module.\n"
        );
    }

    #[test]
    fn edit_distance_handles_unicode_without_byte_indexing() {
        assert_eq!(edit_distance("café", "cafe"), 1);
        assert_eq!(edit_distance("predicate", "predicat"), 1);
    }

    #[test]
    fn export_to_stdout_or_file_uses_identical_deterministic_json() {
        let catalog = catalog(vec![entry(
            "sand::predicate::Predicate",
            ApiKind::Struct,
            "A reusable predicate.",
            "sand::predicate",
        )]);
        let stdout = export(&catalog, None).unwrap().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("first.json");
        let second = directory.path().join("second.json");
        assert!(export(&catalog, Some(&first)).unwrap().is_none());
        assert!(export(&catalog, Some(&second)).unwrap().is_none());

        let first_bytes = std::fs::read(first).unwrap();
        let second_bytes = std::fs::read(second).unwrap();
        assert_eq!(first_bytes, second_bytes);
        assert_eq!(first_bytes, stdout.as_bytes());
        let json: serde_json::Value = serde_json::from_slice(&first_bytes).unwrap();
        assert_eq!(json["schema_version"], 2);
        assert!(json["configuration"]["minecraft_version"].is_string());
        assert_eq!(json["sand_version"], "0.1.0");
        assert_eq!(json["coverage"]["status"], "complete");
        assert_eq!(
            json["entries"][0]["canonical_path"],
            "sand::predicate::Predicate"
        );
    }

    #[test]
    fn partial_catalogs_are_never_rendered_as_complete() {
        let mut catalog = catalog(vec![entry(
            "sand::predicate::Predicate",
            ApiKind::Struct,
            "A reusable predicate.",
            "sand::predicate",
        )]);
        catalog.coverage = sand_api_contract::ApiCoverage {
            status: CoverageStatus::Partial,
            static_surface_items: 11_736,
            pending_item_ceiling: 11_613,
            pending_scope_ceiling: 38,
            pending_scopes: vec!["predicate-source".into()],
        };

        let shown = show(&catalog, "sand::predicate::Predicate").unwrap();
        assert!(shown.starts_with(
            "API contract migration is partial: 11613 static items and 38 scopes remain pending."
        ));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&catalog.to_json_pretty().unwrap()).unwrap()
                ["coverage"]["pending_scopes"][0],
            "predicate-source"
        );
    }

    #[test]
    fn installed_show_includes_generated_command_and_registry_contracts() {
        let catalog = generated_catalog();
        let command = show(catalog, "sand::command::say").unwrap();
        assert!(command.contains("pub fn say(message: impl Into<String>) -> Say"));
        assert!(command.contains("Minecraft `/say <message>` command"));

        let registry = show(catalog, "sand::vanilla::Item::Diamond").unwrap();
        assert!(registry.contains("Selects the vanilla registry entry `minecraft:diamond`."));
        assert!(registry.contains("minecraft = "));

        let prelude_registry = show(catalog, "sand::prelude::vanilla::Item::Diamond").unwrap();
        assert!(prelude_registry.contains("sand::vanilla::Item::Diamond"));
        assert!(prelude_registry.contains("sand::prelude::vanilla::Item::Diamond"));
    }

    #[test]
    fn installed_catalog_rejects_structural_filler_as_semantic_documentation() {
        let catalog = generated_catalog();
        for entry in &catalog.entries {
            assert!(!entry.summary.starts_with("Configures or performs "));
            assert!(!entry.summary.starts_with("Builds or resolves "));
            assert!(
                !entry
                    .summary
                    .contains("on this typed datapack component definition")
            );
            assert!(entry.parameters.iter().all(|parameter| {
                !parameter
                    .description
                    .starts_with("Rust parameter with type `")
            }));
            assert!(
                !entry
                    .returns
                    .as_deref()
                    .is_some_and(|returns| returns.starts_with("A value with Rust type `"))
            );
        }

        let path = catalog
            .find("sand::data::NbtPath::as_str")
            .expect("NbtPath::as_str is installed");
        assert_eq!(
            path.summary,
            "Borrows the rendered NBT path text without allocating."
        );
        assert!(
            path.returns
                .as_deref()
                .is_some_and(|returns| returns.contains("borrowed textual representation"))
        );

        for entry in &catalog.entries {
            let return_type = entry.return_type.as_deref().unwrap_or_default();
            if return_type.contains("Result") || return_type.contains("Option") {
                assert_ne!(
                    entry.returns.as_deref(),
                    Some("The updated typed builder, ready for further chained configuration."),
                    "fallible/optional return documented as an infallible builder: {}",
                    entry.canonical_path
                );
            }
            if !matches!(
                entry.kind,
                ApiKind::Function | ApiKind::Method | ApiKind::TraitMethod
            ) {
                assert!(
                    !entry.example.trim_end().ends_with("();"),
                    "non-callable is presented as a constructor: {}",
                    entry.canonical_path
                );
            }
        }

        let participant_example = &catalog
            .find("sand::participant::PlayerParticipant")
            .unwrap()
            .example;
        assert!(participant_example.contains("PlayerParticipant::subject()"));
        assert!(participant_example.contains("use sand::participant::"));
        assert!(!participant_example.contains("sand_core::"));
        assert_eq!(
            catalog.find("sand::command::Actionbar").unwrap().example,
            "use sand::command::Actionbar;"
        );
    }

    #[test]
    fn rustdoc_prose_stops_before_the_contract_lookup_footer() {
        let documentation = "Builds a typed value.\n\nMore behavioral detail.\n\n# API Contract\n\n`sand api show sand::topic::Value::new`";
        assert_eq!(
            rustdoc_prose(documentation),
            "Builds a typed value. More behavioral detail."
        );

        let archetype = generated_catalog()
            .find("sand::entity::EntityArchetype::new")
            .expect("EntityArchetype::new is installed");
        assert!(!archetype.context.contains("sand api show"));
        assert!(!archetype.minecraft.contains("sand api show"));

        assert_eq!(
            rustdoc_example("Example:\n\n```\nlet value = 42;\n```"),
            Some("let value = 42;".to_owned())
        );
        assert_eq!(
            rustdoc_example("Example:\n\n```rust,ignore\n# let hidden = 1;\nlet shown = 2;\n```"),
            Some("let shown = 2;".to_owned())
        );
    }

    #[test]
    fn installed_search_finds_generated_registry_contracts() {
        let rendered = search(generated_catalog(), "diamond registry").unwrap();
        assert!(rendered.contains("sand::vanilla::Item::Diamond"));
    }

    #[test]
    fn installed_module_groups_generated_command_contracts() {
        let rendered = module(generated_catalog(), "sand::command").unwrap();
        assert!(rendered.contains("Functions\n"));
        assert!(rendered.contains("sand::command::say"));
    }

    #[test]
    fn installed_generated_export_is_byte_deterministic() {
        let first = export(generated_catalog(), None).unwrap().unwrap();
        let second = export(generated_catalog(), None).unwrap().unwrap();
        assert_eq!(first.as_bytes(), second.as_bytes());

        let json: serde_json::Value = serde_json::from_str(&first).unwrap();
        let paths = json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["canonical_path"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert!(paths.contains("sand::command::say"));
        assert!(paths.contains("sand::vanilla::Item::Diamond"));
    }

    #[test]
    fn installed_root_scope_distinguishes_topic_modules_from_attribute_macros() {
        let catalog = generated_catalog();

        let component_module = show(catalog, "sand::component").unwrap();
        assert!(component_module.contains("Builds datapack JSON components"));

        let component_attribute = show(catalog, "sand::prelude::datapack_component").unwrap();
        assert!(component_attribute.contains("sand::datapack_component"));
        assert!(component_attribute.contains("#[datapack_component(...)]"));

        let location = show(catalog, "sand::prelude::ResourceLocation").unwrap();
        assert!(location.contains("sand::ResourceLocation"));

        let macros = search(catalog, "typed event handler").unwrap();
        assert!(macros.contains("sand::on_event"));
    }

    #[test]
    fn installed_predicate_scope_exposes_source_and_generated_contracts() {
        let catalog = generated_catalog();
        let constructor = show(catalog, "sand::prelude::Predicate::new").unwrap();
        assert!(constructor.contains("sand::predicate::Predicate::new"));
        assert!(constructor.contains("location : PredicateId"));

        let generated_id = show(catalog, "sand::component::PredicateId::minecraft").unwrap();
        assert!(generated_id.contains("sand::predicate::PredicateId::minecraft"));
        assert!(generated_id.contains("Validates the path and emits minecraft:<path>"));

        let equipment = search(catalog, "equipment predicate").unwrap();
        assert!(equipment.contains("sand::predicate::EntityEquipment"));

        let grouped = module(catalog, "sand::predicate").unwrap();
        assert!(grouped.contains("Structs\n"));
        assert!(grouped.contains("sand::predicate::Predicate"));
        assert!(!grouped.contains("Methods\n"));
        assert!(grouped.contains("sand::predicate::EntityEquipment (7 APIs)"));
        assert!(grouped.contains("sand::predicate::PredicateId (3 APIs)"));
    }

    #[test]
    fn installed_predicate_catalog_matches_the_enforced_identity_count() {
        let entries = generated_catalog()
            .entries
            .iter()
            .filter(|entry| {
                entry.canonical_path == "sand::predicate"
                    || entry.canonical_path.starts_with("sand::predicate::")
            })
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 123);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.canonical_path.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            123
        );
    }

    #[test]
    fn installed_execute_when_scope_exposes_exclusive_branch_contracts() {
        let catalog = generated_catalog();
        let when_contract = show(catalog, "sand::prelude::when").unwrap();
        assert!(when_contract.contains("sand::execute_when::when"));
        assert!(when_contract.contains("execute-if"));

        let else_contract = show(catalog, "sand::execute_when::IfThenBuilder::else_all").unwrap();
        assert!(else_contract.contains("else"));
        assert!(else_contract.contains("mutually exclusive"));

        let grouped = module(catalog, "sand::execute_when").unwrap();
        assert!(grouped.contains("Functions\n"));
        assert!(grouped.contains("sand::execute_when::if_"));
        assert!(grouped.contains("Structs\n"));
        assert!(grouped.contains("sand::execute_when::WhenBuilder"));
        assert!(grouped.contains("sand::execute_when::WhenBuilder (5 APIs)"));
    }

    #[test]
    fn installed_execute_when_catalog_matches_the_enforced_identity_count() {
        let entries = generated_catalog()
            .entries
            .iter()
            .filter(|entry| {
                entry.canonical_path == "sand::execute_when"
                    || entry.canonical_path.starts_with("sand::execute_when::")
            })
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 23);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.canonical_path.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            23
        );
    }

    #[test]
    fn installed_condition_scope_exposes_typed_opaque_contracts() {
        let catalog = generated_catalog();
        let entity = show(catalog, "sand::prelude::Condition::entity").unwrap();
        assert!(entity.contains("sand::condition::Condition::entity"));
        assert!(entity.contains("selector : sand_commands :: Selector"));
        assert!(entity.contains("at least one entity"));

        let raw = show(catalog, "sand::condition::Condition::raw").unwrap();
        assert!(raw.contains("escape hatch"));
        assert!(raw.contains("without a leading if or unless"));

        let search_results = search(catalog, "typed runtime condition").unwrap();
        assert!(search_results.contains("sand::condition::Condition"));

        let grouped = module(catalog, "sand::condition").unwrap();
        assert!(grouped.contains("Structs\n  sand::condition::Condition"));
        assert!(grouped.contains("sand::condition::Condition (11 APIs)"));
        assert!(!grouped.contains("ExecuteClause"));
        assert!(!grouped.contains("ExecutePlan"));
        assert!(!grouped.contains("ScoreRange"));
    }

    #[test]
    fn installed_condition_catalog_matches_the_enforced_identity_count() {
        let entries = generated_catalog()
            .entries
            .iter()
            .filter(|entry| {
                entry.canonical_path == "sand::condition"
                    || entry.canonical_path.starts_with("sand::condition::")
            })
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 13);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.canonical_path.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            13
        );
    }

    #[test]
    fn installed_resource_reference_scope_uses_canonical_ids_and_aliases() {
        let catalog = generated_catalog();
        let function = show(catalog, "sand::prelude::FunctionId::minecraft").unwrap();
        assert!(function.contains("sand::resource_ref::FunctionId::minecraft"));
        assert!(function.contains("function resource"));

        let predicate = show(catalog, "sand::resource_ref::PredicateId").unwrap();
        assert!(predicate.contains("sand::predicate::PredicateId"));

        let local = show(catalog, "sand::resource_ref::DialogId::local").unwrap();
        assert!(local.contains("trusted literal path"));
        assert!(local.contains("Minecraft Java 1.21.6+"));

        let search_results = search(catalog, "dialog namespace sentinel").unwrap();
        assert!(search_results.contains("sand::resource_ref::DialogId::local"));

        let grouped = module(catalog, "sand::resource_ref").unwrap();
        assert!(grouped.contains("Structs\n"));
        assert!(grouped.contains("sand::resource_ref::DialogId"));
        assert!(grouped.contains("sand::resource_ref::DialogId (5 APIs)"));
        assert!(grouped.contains("sand::resource_ref::FunctionId (3 APIs)"));
    }

    #[test]
    fn installed_resource_reference_catalog_matches_the_enforced_identity_count() {
        let entries = generated_catalog()
            .entries
            .iter()
            .filter(|entry| {
                entry.canonical_path == "sand::resource_ref"
                    || entry.canonical_path.starts_with("sand::resource_ref::")
            })
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 23);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.canonical_path.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            23
        );
    }

    #[test]
    fn installed_version_scope_uses_typed_capabilities_and_prelude_aliases() {
        let catalog = generated_catalog();
        let feature = show(catalog, "sand::prelude::VersionFeature::Dialogs").unwrap();
        assert!(feature.contains("sand::version::VersionFeature::Dialogs"));
        assert!(feature.contains("Data-driven Minecraft dialogs"));

        let supports = show(catalog, "sand::prelude::VersionProfile::supports").unwrap();
        assert!(supports.contains("sand::version::VersionProfile::supports"));
        assert!(supports.contains("typed Minecraft capability"));

        let search_results = search(catalog, "conservative future release fallback").unwrap();
        assert!(search_results.contains("sand::version::VersionProfile::is_fallback"));

        let grouped = module(catalog, "sand::version").unwrap();
        assert!(grouped.contains("sand::version::VersionFeature"));
        assert!(grouped.contains("sand::version::VersionFeature (14 APIs)"));
        assert!(grouped.contains("sand::version::VersionProfile"));
    }

    #[test]
    fn installed_version_catalog_matches_the_enforced_identity_count() {
        let entries = generated_catalog()
            .entries
            .iter()
            .filter(|entry| {
                entry.canonical_path == "sand::version"
                    || entry.canonical_path.starts_with("sand::version::")
            })
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 43);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.canonical_path.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            43
        );
    }

    #[test]
    fn installed_vfx_scope_exposes_typed_steps_and_inherited_aliases() {
        let catalog = generated_catalog();
        let raw_command = show(catalog, "sand::prelude::Vfx::command").unwrap();
        assert!(raw_command.contains("sand::vfx::Vfx::command"));
        assert!(raw_command.contains("RawCommand"));
        assert!(raw_command.contains("explicitly raw Minecraft command"));

        let visibility = show(catalog, "sand::cmd::VfxParticleVisibility::Normal").unwrap();
        assert!(visibility.contains("sand::vfx::VfxParticleVisibility::Normal"));
        assert!(visibility.contains("normal distance-limited"));

        let search_results = search(catalog, "forced particle visibility").unwrap();
        assert!(search_results.contains("sand::vfx::VfxParticleVisibility"));

        let grouped = module(catalog, "sand::vfx").unwrap();
        assert!(grouped.contains("Structs\n"));
        assert!(grouped.contains("Enums\n"));
        assert!(grouped.contains("sand::vfx::Vfx"));
        assert!(grouped.contains("sand::vfx::VfxParticleVisibility"));
        assert!(grouped.contains("sand::vfx::Vfx (12 APIs)"));
    }

    #[test]
    fn installed_vfx_catalog_matches_the_enforced_identity_count() {
        let entries = generated_catalog()
            .entries
            .iter()
            .filter(|entry| {
                entry.canonical_path == "sand::vfx"
                    || entry.canonical_path.starts_with("sand::vfx::")
            })
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 45);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.canonical_path.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            45
        );
    }

    #[test]
    fn installed_advanced_catalog_exposes_only_the_version_aware_export_hook() {
        let catalog = generated_catalog();
        let hook = show(catalog, "sand::advanced::try_export_components_json").unwrap();
        assert!(hook.contains("version-validated JSON"));
        assert!(hook.contains("mc_version"));

        let grouped = module(catalog, "sand::advanced").unwrap();
        assert!(grouped.contains("Functions\n  sand::advanced::try_export_components_json"));

        let entries = catalog
            .entries
            .iter()
            .filter(|entry| {
                entry.canonical_path == "sand::advanced"
                    || entry.canonical_path.starts_with("sand::advanced::")
            })
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    #[cfg(not(any(
        feature = "resourcepack",
        feature = "systems-damage",
        feature = "systems-cooldowns",
        feature = "systems-lifecycle",
        feature = "systems-player-data",
        feature = "systems-movement",
        feature = "systems-inventory",
        feature = "systems-entities"
    )))]
    fn default_catalog_contains_only_the_compiled_feature_surface() {
        let catalog = generated_catalog();
        assert!(
            catalog
                .find("sand::systems::damage::DamageTracker")
                .is_none()
        );
        assert!(
            catalog
                .entries
                .iter()
                .all(|entry| { !entry.canonical_path.starts_with("sand::resourcepack::") })
        );
        assert!(catalog.configuration.cargo_features.is_empty());
        assert_eq!(
            catalog.configuration.compiled_surface_items,
            catalog.entries.len()
        );
    }

    #[test]
    #[cfg(feature = "systems-damage")]
    fn damage_feature_catalog_contains_damage_apis_and_declares_the_feature() {
        let catalog = generated_catalog();
        assert!(
            catalog
                .find("sand::systems::damage::DamageTracker")
                .is_some()
        );
        assert!(
            catalog
                .configuration
                .cargo_features
                .contains(&"systems-damage".to_owned())
        );
    }

    #[test]
    #[cfg(feature = "resourcepack")]
    fn resourcepack_feature_catalog_contains_resourcepack_apis() {
        let catalog = generated_catalog();
        assert!(
            catalog
                .entries
                .iter()
                .any(|entry| entry.canonical_path.starts_with("sand::resourcepack::"))
        );
        assert!(
            catalog
                .configuration
                .cargo_features
                .contains(&"resourcepack".to_owned())
        );
    }

    #[test]
    fn installed_system_feature_matrix_matches_the_compiled_surface() {
        let catalog = generated_catalog();
        let systems_all = cfg!(feature = "systems-all");
        let expectations = [
            (
                "systems-cooldowns",
                "sand::systems::cooldowns::",
                cfg!(feature = "systems-cooldowns") || systems_all,
            ),
            (
                "systems-damage",
                "sand::systems::damage::",
                cfg!(feature = "systems-damage") || systems_all,
            ),
            (
                "systems-entities",
                "sand::systems::entities::",
                cfg!(feature = "systems-entities") || systems_all,
            ),
            (
                "systems-inventory",
                "sand::systems::inventory::",
                cfg!(feature = "systems-inventory") || systems_all,
            ),
            (
                "systems-lifecycle",
                "sand::systems::lifecycle::",
                cfg!(feature = "systems-lifecycle") || systems_all,
            ),
            (
                "systems-movement",
                "sand::systems::movement::",
                cfg!(feature = "systems-movement") || systems_all,
            ),
            (
                "systems-player-data",
                "sand::systems::player_data::",
                cfg!(feature = "systems-player-data") || systems_all,
            ),
        ];

        for (feature, prefix, expected) in expectations {
            assert_eq!(
                catalog
                    .configuration
                    .cargo_features
                    .iter()
                    .any(|item| item == feature),
                expected,
                "configuration mismatch for {feature}"
            );
            assert_eq!(
                catalog
                    .entries
                    .iter()
                    .any(|entry| entry.canonical_path.starts_with(prefix)),
                expected,
                "compiled surface mismatch for {feature}"
            );
            for entry in catalog
                .entries
                .iter()
                .filter(|entry| entry.canonical_path.starts_with(prefix))
            {
                assert_eq!(
                    entry.availability,
                    vec![format!("Cargo feature: {feature}")],
                    "availability mismatch for {}",
                    entry.canonical_path
                );
            }
        }
    }

    #[test]
    fn installed_catalog_has_canonical_registry_ownership_and_clean_prose() {
        let catalog = generated_catalog();
        let item = catalog.find("sand::prelude::ItemId").unwrap();
        assert_eq!(item.canonical_path, "sand::registry::ItemId");
        assert!(catalog.entries.iter().all(|entry| {
            entry.canonical_path == "sand::prelude"
                || !entry.canonical_path.starts_with("sand::prelude::")
        }));
        assert!(
            catalog
                .entries
                .iter()
                .all(|entry| !entry.summary.ends_with("(e"))
        );
    }

    #[test]
    fn exploratory_search_is_ranked_bounded_and_filterable() {
        let catalog = generated_catalog();
        let nearby = search_with_options(catalog, "nearby entities", Some(5), None, None).unwrap();
        assert!(nearby.contains("showing 5 of"));
        assert!(nearby.contains("sand::entity::EntityQuery::nearby"));
        let events = search_with_options(
            catalog,
            "advancement event",
            Some(3),
            Some("sand::event"),
            Some("trait"),
        )
        .unwrap();
        assert!(events.contains("sand::event::AdvancementEvent"));
        assert!(!events.contains("sand::vanilla::"));
    }

    #[test]
    fn exploratory_search_rejects_punctuation_only_queries() {
        let catalog = generated_catalog();
        assert_eq!(
            search_with_options(catalog, "!!!", Some(3), None, None).unwrap(),
            "No APIs matched `!!!`.\n"
        );
    }

    #[cfg(any(feature = "systems-player-data", feature = "systems-all"))]
    #[test]
    fn family_contracts_use_item_specific_source_documentation() {
        let contract = show(
            generated_catalog(),
            "sand::systems::player_data::PlayerDataSchema::define_all",
        )
        .unwrap();
        assert!(contract.contains("scoreboard objectives add"));
        assert!(contract.contains("Storage schemas do not generate commands"));
        assert!(contract.contains("idempotent"));
        assert!(!contract.contains("typed systems API"));
    }
}
