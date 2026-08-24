//! Query and rendering support for the installed Sand API contract catalog.
//!
//! The catalog is collected by `sand-api-contract` from compile-time
//! registrations. This module deliberately does not inspect Rust source or
//! contact a remote service: the CLI reports the contracts linked into this
//! exact Sand installation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::OnceLock;

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
    let entry_kinds = entries
        .iter()
        .map(|entry| (entry.canonical_path.clone(), entry.kind))
        .collect::<BTreeMap<_, _>>();
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
        if let Some((
            identity,
            _,
            signature,
            parameters,
            return_type,
            documentation,
            has_receiver,
            ..,
        )) = sand::__private::api_contract::INSTALLED_API_SHAPES
            .iter()
            .find(|(_, paths, ..)| paths.contains(&entry.canonical_path.as_str()))
        {
            entry.signature = normalize_shape_paths(signature, Some(identity));
            let source_summary = rustdoc_summary(documentation)
                .map(|summary| normalize_shape_paths(&summary, Some(identity)));
            if family_contract
                && matches!(
                    entry.kind,
                    ApiKind::Function | ApiKind::Method | ApiKind::TraitMethod
                )
                && source_summary.is_none()
            {
                bail!(
                    "supported family callable `{}` has no source-authored semantic summary",
                    entry.canonical_path
                );
            }
            let resolved_summary = source_summary
                .clone()
                .filter(|summary| !summary.starts_with("Carries the "))
                .unwrap_or_else(|| non_placeholder_summary(entry));
            let authored = entry
                .parameters
                .iter()
                .map(|parameter| (parameter.name.as_str(), parameter.description.as_str()))
                .collect::<BTreeMap<_, _>>();
            entry.parameters = parameters
                .iter()
                .map(|(name, ty)| {
                    let rust_type = normalize_shape_paths(ty, Some(identity));
                    ApiParameter {
                        name: (*name).to_owned(),
                        rust_type: Some(rust_type.clone()),
                        description: authored
                            .get(name)
                            .filter(|description| !description.trim().is_empty())
                            .map(|description| (*description).to_owned())
                            .or_else(|| {
                                let description = source_parameter_description(name, documentation);
                                (!description.is_empty()).then_some(description)
                            })
                            .unwrap_or_else(|| {
                                semantic_parameter_description(name, &rust_type, &resolved_summary)
                            }),
                    }
                })
                .collect();
            let callable = matches!(
                entry.kind,
                ApiKind::Function | ApiKind::Method | ApiKind::TraitMethod
            );
            entry.return_type = if callable {
                return_type.map(|ty| normalize_shape_paths(ty, Some(identity)))
            } else {
                None
            };
            entry.returns = if !callable {
                None
            } else if family_contract {
                source_return_description(documentation).or_else(|| {
                    entry.return_type.as_deref().and_then(|return_type| {
                        semantic_return_description(
                            return_type,
                            &resolved_summary,
                            *has_receiver,
                            &entry.canonical_path,
                        )
                    })
                })
            } else {
                entry
                    .returns
                    .clone()
                    .or_else(|| source_return_description(documentation))
            };
            if family_contract {
                let summary = resolved_summary.clone();
                let prose = normalize_shape_paths(&rustdoc_prose(documentation), Some(identity));
                entry.summary = summary.clone();
                if !prose.trim().is_empty() && prose.trim() != summary.trim() {
                    entry.context = prose;
                } else if !entry.context.contains(&summary) {
                    entry.context = format!("{summary} {}", entry.context);
                }
                if let Some(minecraft) = source_minecraft_behavior(documentation) {
                    entry.minecraft = normalize_shape_paths(&minecraft, Some(identity));
                }
                let use_when = source_guidance(documentation, GuidanceKind::Use)
                    .into_iter()
                    .map(|value| normalize_shape_paths(&value, Some(identity)))
                    .collect::<Vec<_>>();
                if !use_when.is_empty() {
                    entry.use_when = use_when;
                }
                let avoid_when = source_guidance(documentation, GuidanceKind::Avoid)
                    .into_iter()
                    .map(|value| normalize_shape_paths(&value, Some(identity)))
                    .collect::<Vec<_>>();
                if !avoid_when.is_empty() {
                    entry.avoid_when = avoid_when;
                }
            } else if let Some(summary) = source_summary {
                if is_family_template_summary(&entry.summary) {
                    entry.summary = summary.clone();
                }
                if !entry.context.contains(&summary) {
                    entry.context = format!("{summary} {}", entry.context);
                }
            }
            if matches!(
                entry.kind,
                ApiKind::Function | ApiKind::Method | ApiKind::TraitMethod
            ) && (is_import_only_example(&entry.example)
                || !example_exercises_member(&entry.example, &entry.canonical_path))
            {
                entry.example = rustdoc_example(documentation)
                    .map(|example| normalize_shape_paths(&example, Some(identity)))
                    .filter(|example| example_exercises_member(example, &entry.canonical_path))
                    .unwrap_or_default();
            }
        }
    }

    // Provider catalogs created before source-shape resolution may carry a
    // name-derived `Type()` example. A type declaration is not a function;
    // retain an exact, compilable import reference instead of fabricating a
    // constructor that may be private or may not exist.
    let entry_shapes = entries
        .iter()
        .map(|entry| {
            let installed = sand::__private::api_contract::INSTALLED_API_SHAPES
                .iter()
                .find(|(_, paths, ..)| paths.contains(&entry.canonical_path.as_str()));
            (
                entry.canonical_path.clone(),
                (
                    entry.kind,
                    entry.signature.clone(),
                    installed.map(|(identity, ..)| *identity),
                    installed.and_then(|(_, _, _, _, _, _, _, self_type, ..)| *self_type),
                    installed.and_then(|(_, _, _, _, _, _, _, _, generics, ..)| *generics),
                    installed.and_then(|(_, _, _, _, _, _, _, _, _, clause)| *clause),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for entry in &mut entries {
        let source_identity = installed_source_identity(&entry.canonical_path);
        entry.signature = normalize_shape_paths(&entry.signature, source_identity);
        entry.summary = normalize_shape_paths(&entry.summary, source_identity);
        entry.context = normalize_shape_paths(&entry.context, source_identity);
        entry.minecraft = normalize_shape_paths(&entry.minecraft, source_identity);
        entry.use_when = entry
            .use_when
            .iter()
            .map(|value| normalize_shape_paths(value, source_identity))
            .collect();
        entry.avoid_when = entry
            .avoid_when
            .iter()
            .map(|value| normalize_shape_paths(value, source_identity))
            .collect();
        entry.returns = entry
            .returns
            .as_deref()
            .map(|value| normalize_shape_paths(value, source_identity));
        entry.example = normalize_shape_paths(&entry.example, source_identity);
        for parameter in &mut entry.parameters {
            parameter.description = normalize_shape_paths(&parameter.description, source_identity);
            if let Some(rust_type) = &mut parameter.rust_type {
                *rust_type = normalize_shape_paths(rust_type, source_identity);
            }
        }
        if let Some(return_type) = &mut entry.return_type {
            *return_type = normalize_shape_paths(return_type, source_identity);
        }
        if matches!(
            entry.kind,
            ApiKind::Function | ApiKind::Method | ApiKind::TraitMethod
        ) && (entry.example.trim().is_empty()
            || !example_exercises_member(&entry.example, &entry.canonical_path))
        {
            let has_receiver = sand::__private::api_contract::INSTALLED_API_SHAPES
                .iter()
                .find(|(_, paths, ..)| paths.contains(&entry.canonical_path.as_str()))
                .is_some_and(|(_, _, _, _, _, _, has_receiver, ..)| *has_receiver);
            entry.example = semantic_callable_example(entry, has_receiver, &entry_shapes);
        }
        if entry.kind == ApiKind::Field {
            entry.example = semantic_field_example(entry, &entry_kinds);
        } else if !matches!(
            entry.kind,
            ApiKind::Function | ApiKind::Method | ApiKind::TraitMethod
        ) && (entry.example.trim_end().ends_with("();")
            || is_import_only_example(&entry.example))
        {
            entry.example = declaration_reference_example(entry.kind, &entry.canonical_path);
        }
    }
    validate_exported_references(&entries)?;

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
    let prose = rustdoc_prose_paragraphs(documentation)
        .into_iter()
        .take(4)
        .collect::<Vec<_>>()
        .join(" ");
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

fn rustdoc_prose_paragraphs(documentation: &str) -> Vec<String> {
    sand_api_contract::rustdoc_prose_paragraphs(documentation)
        .into_iter()
        .collect()
}

fn source_parameter_description(name: &str, documentation: &str) -> String {
    let bare = name.trim_start_matches('_');
    let backticked = format!("`{bare}`");
    rustdoc_prose_paragraphs(documentation)
        .into_iter()
        .find(|paragraph| paragraph.contains(&backticked))
        .unwrap_or_default()
}

fn source_return_description(documentation: &str) -> Option<String> {
    rustdoc_prose_paragraphs(documentation)
        .into_iter()
        .find(|paragraph| {
            let prose = paragraph.trim_start().to_ascii_lowercase();
            prose.starts_with("return")
                || prose.starts_with("on success")
                || prose.contains(" returns ")
        })
}

fn semantic_parameter_description(name: &str, rust_type: &str, summary: &str) -> String {
    let bare = name.trim_start_matches('_');
    let lower_type = rust_type.to_ascii_lowercase();
    let role = match bare {
        "x" => "the x-coordinate",
        "y" => "the y-coordinate",
        "z" => "the z-coordinate",
        "dx" => "the x-axis offset or spread",
        "dy" => "the y-axis offset or spread",
        "dz" => "the z-axis offset or spread",
        "selector" | "players" | "targets" => "the Minecraft target selection",
        "target" => "the entity, block, or command target",
        "player" => "the player participant",
        "entity" => "the entity participant or predicate",
        "json" => "the raw JSON payload",
        "nbt" => "the NBT payload",
        "text" | "message" | "name" => "the author-visible text value",
        "value" => "the value being applied or compared",
        "key" => "the key that identifies the setting or entry",
        "g" if summary.to_ascii_lowercase().contains("group") => "the recipe group name",
        "id" | "location" | "path" => "the typed resource identifier or location",
        "item" => "the item value or item predicate",
        "block" => "the block value or block predicate",
        "condition" | "cond" => "the condition that gates the operation",
        "predicate" => "the predicate that must match",
        "callback" | "handler" | "function" => "the callback invoked by this operation",
        "duration" | "ticks" => "the Minecraft tick duration",
        "count" | "amount" => "the requested numeric amount",
        "min" => "the inclusive lower bound",
        "max" => "the inclusive upper bound",
        "inputs" => "the runtime score inputs",
        "fixed" => "the fixed-value inputs",
        "archetype" => "the entity archetype supplying the property",
        "derivation" => "the derived-stat selector",
        _ if lower_type.contains("selector") => "the Minecraft target selection",
        _ if lower_type.contains("predicate") => "the typed predicate that must match",
        _ if lower_type.contains("resource") || lower_type.ends_with("id") => {
            "the typed Minecraft resource identifier"
        }
        _ if lower_type.contains("bool") => "the switch that enables or disables the behavior",
        _ if lower_type.contains("range") => "the accepted numeric range",
        _ if lower_type.contains("text") => "the player-visible text value",
        _ => {
            let label = bare.replace('_', " ");
            let label = if label.ends_with(" value") {
                label
            } else {
                format!("{label} value")
            };
            return format!(
                "`{bare}` supplies the {label} used to {}.",
                summary_purpose(summary)
            );
        }
    };
    format!(
        "`{bare}` provides {role} used to {}.",
        summary_purpose(summary)
    )
}

fn semantic_return_description(
    return_type: &str,
    summary: &str,
    has_receiver: bool,
    canonical_path: &str,
) -> Option<String> {
    let compact = return_type.split_whitespace().collect::<String>();
    if compact.is_empty() || compact == "()" || compact == "!" {
        return None;
    }
    let owner = canonical_path
        .rsplit_once("::")
        .and_then(|(owner, _)| owner.rsplit("::").next())
        .unwrap_or("value");
    let purpose = summary_purpose(summary);
    let description = if compact == "Self" {
        if has_receiver {
            format!("The `{owner}` value with the documented change applied to {purpose}.")
        } else {
            format!("A newly constructed `{owner}` configured to {purpose}.")
        }
    } else if compact.starts_with("Result<")
        || compact.starts_with("CommandResult<")
        || compact.starts_with("SandResult<")
    {
        format!(
            "On success, the value produced to {purpose}; otherwise, the documented validation or export diagnostic."
        )
    } else if compact.starts_with("Option<") {
        format!("The matching value used to {purpose}, or `None` when that value is unavailable.")
    } else if compact == "bool" {
        format!("`true` when the documented condition holds to {purpose}; otherwise `false`.")
    } else if compact == "String" || compact == "&str" || compact == "&'staticstr" {
        if summary.to_ascii_lowercase().contains("command") {
            format!("The rendered Minecraft command text produced to {purpose}.")
        } else {
            format!("The string value produced to {purpose}.")
        }
    } else if compact.starts_with("Vec<") {
        format!("The ordered values produced to {purpose}.")
    } else {
        format!("The `{return_type}` value produced to {purpose}.")
    };
    Some(description)
}

fn summary_purpose(summary: &str) -> String {
    let summary = summary.trim().trim_end_matches('.');
    let mut words = summary.split_whitespace();
    let first = words.next().unwrap_or("perform the documented behavior");
    let rest = words.collect::<Vec<_>>().join(" ");
    let verb = match first.to_ascii_lowercase().as_str() {
        "add" => "add",
        "adds" => "add",
        "append" => "append",
        "apply" => "apply",
        "applies" => "apply",
        "attach" => "attach",
        "begin" => "begin",
        "bind" | "binds" => "bind",
        "build" => "build",
        "builds" => "build",
        "check" => "check",
        "checks" => "check",
        "compare" => "compare",
        "configure" => "configure",
        "configures" => "configure",
        "construct" => "construct",
        "constructs" => "construct",
        "create" => "create",
        "creates" => "create",
        "emit" => "emit",
        "emits" => "emit",
        "evaluate" => "evaluate",
        "evaluates" => "evaluate",
        "extend" | "extends" => "extend",
        "filter" => "filter",
        "get" => "get",
        "gets" => "get",
        "guard" => "guard",
        "map" | "maps" => "map",
        "observe" => "observe",
        "parse" => "parse",
        "parses" => "parse",
        "query" => "query",
        "queries" => "query",
        "register" | "registers" => "register",
        "remove" => "remove",
        "removes" => "remove",
        "render" => "render",
        "renders" => "render",
        "reset" => "reset",
        "resets" => "reset",
        "restrict" => "restrict",
        "resolve" => "resolve",
        "resolves" => "resolve",
        "return" => "return",
        "returns" => "return",
        "select" => "select",
        "selects" => "select",
        "serialize" => "serialize",
        "serializes" => "serialize",
        "show" => "show",
        "start" | "starts" => "start",
        "set" => "set",
        "sets" => "set",
        "store" => "store",
        "stores" => "store",
        "update" | "updates" => "update",
        "use" => "use",
        "validate" => "validate",
        "validates" => "validate",
        "wrap" => "wrap",
        _ if first.starts_with('`') => return format!("emit the documented {summary} form"),
        "whether" => return format!("determine whether {rest}"),
        "how" => return format!("represent how {rest}"),
        _ if matches!(
            first.to_ascii_lowercase().as_str(),
            "a" | "an"
                | "the"
                | "alias"
                | "compatibility"
                | "convenience"
                | "explicit"
                | "fallible"
                | "like"
                | "shorthand:"
                | "validated"
        ) =>
        {
            return format!("use {}", lower_first(summary));
        }
        _ if matches!(
            first.to_ascii_lowercase().as_str(),
            "absolute"
                | "always"
                | "author"
                | "colored"
                | "color-transitioning"
                | "const-compatible"
                | "custom"
                | "ergonomic"
                | "exact"
                | "filled"
                | "horizontal"
                | "legacy"
                | "minecraft"
                | "optional"
                | "outward"
                | "plain"
                | "raw"
                | "relative"
                | "rising"
                | "selected"
                | "stable"
                | "straight"
                | "two"
        ) =>
        {
            return format!("use {}", lower_first(summary));
        }
        _ => {
            let verb = first
                .strip_suffix('s')
                .filter(|stem| stem.len() >= 3)
                .unwrap_or(first)
                .trim_end_matches(':');
            return if rest.is_empty() {
                verb.to_ascii_lowercase()
            } else {
                format!("{} {rest}", verb.to_ascii_lowercase())
            };
        }
    };
    if rest.is_empty() {
        verb.to_owned()
    } else {
        format!("{verb} {rest}")
    }
}

fn lower_first(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_lowercase().chain(characters).collect()
}

type InstalledEntryShape = (
    ApiKind,
    String,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
);

fn semantic_callable_example(
    entry: &ApiEntry,
    has_receiver: bool,
    entry_shapes: &BTreeMap<String, InstalledEntryShape>,
) -> String {
    let Some((owner_path, member)) = entry.canonical_path.rsplit_once("::") else {
        return String::new();
    };
    let owner_name = owner_path.rsplit("::").next().unwrap_or("value");
    let owner_variable = format!("{}_value", to_snake_case(owner_name));
    let (owner_kind, owner_signature) = entry_shapes
        .get(owner_path)
        .map_or((ApiKind::Struct, ""), |(kind, signature, ..)| {
            (*kind, signature.as_str())
        });
    let method_shape = entry_shapes.get(&entry.canonical_path);
    let source_identity = method_shape.and_then(|(_, _, identity, ..)| *identity);
    let impl_self_type = method_shape
        .and_then(|(_, _, _, self_type, ..)| *self_type)
        .map(|self_type| normalize_shape_paths(self_type, source_identity));
    let impl_generics = method_shape
        .and_then(|(_, _, _, _, generics, ..)| *generics)
        .unwrap_or_default();
    let impl_where_clause = method_shape
        .and_then(|(_, _, _, _, _, clause)| *clause)
        .unwrap_or_default();
    let (owner_generics, owner_arguments) = if impl_self_type.is_some() {
        (impl_generics.to_owned(), String::new())
    } else {
        signature_generics(owner_signature, owner_name)
    };
    let (method_generics, method_arguments) = signature_generics(&entry.signature, member);
    let method_turbofish = explicit_method_arguments(&method_arguments);
    let owner_use = if let Some(self_type) = &impl_self_type {
        normalize_example_type(self_type, owner_path, source_identity)
    } else if owner_arguments.is_empty() {
        owner_path.to_owned()
    } else {
        format!("{owner_path}{owner_arguments}")
    };
    let arguments = entry
        .parameters
        .iter()
        .map(|parameter| parameter.name.trim_start_matches('_'))
        .collect::<Vec<_>>()
        .join(", ");
    let inputs = entry
        .parameters
        .iter()
        .map(|parameter| {
            format!(
                "{}: {}",
                parameter.name.trim_start_matches('_'),
                normalize_example_type(
                    parameter.rust_type.as_deref().unwrap_or("impl Sized"),
                    &owner_use,
                    source_identity,
                )
            )
        })
        .collect::<Vec<_>>();
    let mut parameters = Vec::new();
    let mut generic_declarations = generic_declaration_items(&owner_generics);
    generic_declarations.extend(generic_declaration_items(&method_generics));
    let owner_call = if impl_self_type.is_some() {
        expression_type_path(&owner_use)
    } else if owner_arguments.is_empty() {
        owner_path.to_owned()
    } else {
        format!("{owner_path}::{owner_arguments}")
    };
    let call = if has_receiver {
        let receiver_type = if owner_kind == ApiKind::Trait {
            generic_declarations.push(format!("T: {owner_use}"));
            if entry.signature.contains("& mut self") {
                "&mut T".to_owned()
            } else if entry.signature.contains("& self") {
                "&T".to_owned()
            } else {
                "T".to_owned()
            }
        } else if entry.signature.contains("& mut self") {
            format!("&mut {owner_use}")
        } else if entry.signature.contains("& self") {
            format!("&{owner_use}")
        } else {
            owner_use.clone()
        };
        parameters.push(format!("{owner_variable}: {receiver_type}"));
        parameters.extend(inputs);
        format!("{owner_variable}.{member}{method_turbofish}({arguments})")
    } else {
        parameters.extend(inputs);
        if owner_kind == ApiKind::Trait {
            generic_declarations.push(format!("T: {owner_use}"));
            format!("<T as {owner_use}>::{member}{method_turbofish}({arguments})")
        } else {
            format!("{owner_call}::{member}{method_turbofish}({arguments})")
        }
    };
    let result_name = semantic_result_name(entry, owner_name, member, has_receiver);
    let std_import = if entry.signature.contains("fmt ::") {
        "use std::fmt;\n"
    } else {
        ""
    };
    let generic_declarations = if generic_declarations.is_empty() {
        String::new()
    } else {
        generic_declarations = generic_declarations
            .into_iter()
            .map(|parameter| normalize_example_type(&parameter, &owner_use, source_identity))
            .collect();
        generic_declarations.sort_by_key(|parameter| !parameter.trim_start().starts_with('\''));
        format!("<{}>", generic_declarations.join(", "))
    };
    let method_where_clause = entry
        .signature
        .find(" where ")
        .map(|position| {
            normalize_example_type(
                entry.signature[position..].trim_end_matches(','),
                &owner_use,
                source_identity,
            )
        })
        .unwrap_or_default();
    let where_clause = merge_where_clauses(
        &normalize_example_type(impl_where_clause, &owner_use, source_identity),
        &method_where_clause,
    );
    format!(
        "{std_import}use sand::prelude::*;\n\nfn demonstrate{generic_declarations}({}) {where_clause} {{\n    let {result_name} = {call};\n}}",
        parameters.join(", ")
    )
}

fn generic_declaration_items(declaration: &str) -> Vec<String> {
    if declaration.is_empty() {
        return Vec::new();
    }
    split_top_level(declaration.trim_matches(&['<', '>'][..]), ',')
        .into_iter()
        .filter_map(|parameter| {
            let parameter = parameter.trim();
            if parameter.is_empty() {
                return None;
            }
            let without_default = split_top_level(parameter, '=')
                .first()
                .copied()
                .unwrap_or(parameter)
                .trim();
            if without_default.is_empty() {
                None
            } else if without_default.starts_with('\'')
                || without_default.starts_with("const ")
                || without_default.contains("'static")
            {
                Some(without_default.to_owned())
            } else if without_default.contains(':') {
                Some(format!("{without_default} + 'static"))
            } else {
                Some(format!("{without_default}: 'static"))
            }
        })
        .collect()
}

fn explicit_method_arguments(arguments: &str) -> String {
    if arguments.is_empty() {
        return String::new();
    }
    let arguments = split_top_level(arguments.trim_matches(&['<', '>'][..]), ',')
        .into_iter()
        // Explicit late-bound lifetime arguments are rejected by rustc. Type
        // and const arguments are safe to spell and keep zero-argument generic
        // methods (for example event dependency groups) inferable.
        .filter(|argument| !argument.trim_start().starts_with('\''))
        .map(str::trim)
        .filter(|argument| !argument.is_empty())
        .collect::<Vec<_>>();
    if arguments.is_empty() {
        String::new()
    } else {
        format!("::<{}>", arguments.join(", "))
    }
}

fn expression_type_path(path: &str) -> String {
    path.find('<').map_or_else(
        || path.to_owned(),
        |position| format!("{}::{}", &path[..position], &path[position..]),
    )
}

fn merge_where_clauses(left: &str, right: &str) -> String {
    let predicates = [left, right]
        .into_iter()
        .map(str::trim)
        .filter(|clause| !clause.is_empty())
        .map(|clause| clause.strip_prefix("where ").unwrap_or(clause).trim())
        .filter(|clause| !clause.is_empty())
        .collect::<Vec<_>>();
    if predicates.is_empty() {
        String::new()
    } else {
        format!("where {}", predicates.join(", "))
    }
}

fn signature_generics(signature: &str, name: &str) -> (String, String) {
    let Some(name_position) = signature.find(name) else {
        return (String::new(), String::new());
    };
    let tail = &signature[name_position + name.len()..];
    let Some(start) = tail.find('<') else {
        return (String::new(), String::new());
    };
    if !tail[..start].trim().is_empty() {
        return (String::new(), String::new());
    }
    let mut depth = 0_i32;
    let mut end = None;
    for (index, character) in tail[start..].char_indices() {
        match character {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(start + index + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(end) = end else {
        return (String::new(), String::new());
    };
    let declaration = tail[start..end].to_owned();
    let arguments = split_top_level(&declaration[1..declaration.len() - 1], ',')
        .into_iter()
        .filter_map(|parameter| {
            let parameter = parameter.trim();
            if parameter.is_empty() {
                return None;
            }
            let parameter = parameter.strip_prefix("const ").unwrap_or(parameter);
            Some(
                parameter
                    .split([':', '='])
                    .next()
                    .unwrap_or(parameter)
                    .trim()
                    .to_owned(),
            )
        })
        .collect::<Vec<_>>();
    (declaration, format!("<{}>", arguments.join(", ")))
}

fn split_top_level(value: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut angle = 0_i32;
    let mut paren = 0_i32;
    let mut bracket = 0_i32;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        match character {
            '<' => angle += 1,
            '>' => angle -= 1,
            '(' => paren += 1,
            ')' => paren -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            _ if character == separator && angle == 0 && paren == 0 && bracket == 0 => {
                parts.push(&value[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&value[start..]);
    parts
}

fn normalize_example_type(
    rust_type: &str,
    owner_type: &str,
    source_identity: Option<&str>,
) -> String {
    let normalized = normalize_two_argument_result(rust_type.replace(" :: ", "::"));
    let bytes = normalized.as_bytes();
    let mut output = String::with_capacity(normalized.len());
    let mut cursor = 0;
    while cursor < bytes.len() {
        if !is_path_ident_start(bytes[cursor]) {
            output.push(bytes[cursor] as char);
            cursor += 1;
            continue;
        }
        let start = cursor;
        cursor += 1;
        while cursor < bytes.len() && is_path_ident_continue(bytes[cursor]) {
            cursor += 1;
        }
        let identifier = &normalized[start..cursor];
        let next_non_space = bytes[cursor..]
            .iter()
            .copied()
            .find(|byte| !byte.is_ascii_whitespace());
        if identifier == "Self" {
            output.push_str(owner_type);
        } else if next_non_space == Some(b'=')
            || cursor + 1 < bytes.len() && bytes[cursor..].starts_with(b"::")
            || start >= 2 && bytes[start - 2..start].starts_with(b"::")
        {
            output.push_str(identifier);
        } else if let Some(canonical) = standard_example_type(identifier) {
            output.push_str(canonical);
        } else if let Some(canonical) = resolve_bare_example_type(identifier, source_identity) {
            output.push_str(canonical);
        } else if let Some(canonical) = installed_type_suffix_mappings().get(identifier) {
            output.push_str(canonical);
        } else {
            output.push_str(identifier);
        }
    }
    output
}

fn resolve_bare_example_type(
    identifier: &str,
    source_identity: Option<&str>,
) -> Option<&'static str> {
    // The contextual scan includes variants, constants, and fields in the
    // complete path table. Only terminals independently classified as types
    // may participate, otherwise names such as `String` and `i32` can be
    // captured by an enum variant or associated constant.
    if !installed_type_path_mappings()
        .keys()
        .any(|path| path.rsplit("::").next() == Some(identifier))
    {
        return None;
    }
    let source_identity = source_identity?;
    installed_type_path_mappings()
        .iter()
        .filter(|(implementation, _)| implementation.rsplit("::").next() == Some(identifier))
        .max_by_key(|(implementation, _)| common_path_prefix_len(implementation, source_identity))
        .and_then(|(implementation, canonical)| {
            (common_path_prefix_len(implementation, source_identity) > 0).then_some(*canonical)
        })
}

fn common_path_prefix_len(left: &str, right: &str) -> usize {
    left.split("::")
        .zip(right.split("::"))
        .take_while(|(left, right)| left == right)
        .count()
}

fn standard_example_type(identifier: &str) -> Option<&'static str> {
    match identifier {
        "HashMap" => Some("std::collections::HashMap"),
        "RangeBounds" => Some("std::ops::RangeBounds"),
        "Display" => Some("std::fmt::Display"),
        "Value" => Some("serde_json::Value"),
        _ => None,
    }
}

fn normalize_two_argument_result(mut value: String) -> String {
    for spelling in ["sand::component::Result", "Result"] {
        let mut search_from = 0;
        while let Some(relative) = value[search_from..].find(spelling) {
            let start = search_from + relative;
            let after_name = start + spelling.len();
            if (start > 0 && is_path_ident_continue(value.as_bytes()[start - 1]))
                || (spelling == "Result" && start >= 2 && &value[start - 2..start] == "::")
                || value.as_bytes()[after_name..]
                    .first()
                    .is_some_and(|byte| is_path_ident_continue(*byte))
            {
                search_from = after_name;
                continue;
            }
            let Some(open_offset) = value[after_name..].find('<') else {
                break;
            };
            let open = after_name + open_offset;
            if !value[after_name..open].trim().is_empty() {
                search_from = after_name;
                continue;
            }
            let mut depth = 0_i32;
            let mut has_top_level_comma = false;
            let mut close = None;
            for (offset, character) in value[open..].char_indices() {
                match character {
                    '<' => depth += 1,
                    '>' => {
                        depth -= 1;
                        if depth == 0 {
                            close = Some(open + offset);
                            break;
                        }
                    }
                    ',' if depth == 1 => has_top_level_comma = true,
                    _ => {}
                }
            }
            if has_top_level_comma && close.is_some() {
                value.replace_range(start..after_name, "std::result::Result");
                search_from = start + "std::result::Result".len();
            } else {
                search_from = after_name;
            }
        }
    }
    value
}

fn semantic_field_example(entry: &ApiEntry, entry_kinds: &BTreeMap<String, ApiKind>) -> String {
    let Some((parent_path, field)) = entry.canonical_path.rsplit_once("::") else {
        return declaration_reference_example(entry.kind, &entry.canonical_path);
    };
    let field_name = if field.chars().all(|character| character.is_ascii_digit()) {
        "payload".to_owned()
    } else {
        field.to_owned()
    };
    if entry_kinds.get(parent_path) == Some(&ApiKind::Variant) {
        let Some((enum_path, variant)) = parent_path.rsplit_once("::") else {
            return declaration_reference_example(entry.kind, &entry.canonical_path);
        };
        let pattern = if let Ok(index) = field.parse::<usize>() {
            let mut fields = vec!["_"; index];
            fields.push("payload");
            fields.push("..");
            format!("{enum_path}::{variant}({})", fields.join(", "))
        } else {
            format!("{enum_path}::{variant} {{ {field}, .. }}")
        };
        return format!(
            "use sand::prelude::*;\n\nfn inspect(value: {enum_path}) {{\n    if let {pattern} = value {{\n        let {field_name}_value = {field_name};\n    }}\n}}"
        );
    }
    format!(
        "use sand::prelude::*;\n\nfn inspect(value: {parent_path}) {{\n    let {field_name}_value = value.{field};\n}}"
    )
}

fn semantic_result_name(
    entry: &ApiEntry,
    owner_name: &str,
    member: &str,
    has_receiver: bool,
) -> String {
    let return_type = entry
        .return_type
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .collect::<String>();
    if return_type == "Self" {
        if has_receiver {
            return format!("updated_{}", to_snake_case(owner_name));
        }
        return to_snake_case(owner_name);
    }
    if matches!(member, "new" | "parse" | "try_new") {
        return format!("{}_result", to_snake_case(owner_name));
    }
    if return_type == "bool" {
        return format!("is_{}", to_snake_case(member));
    }
    if return_type == "String" && entry.summary.to_ascii_lowercase().contains("command") {
        return "command".to_owned();
    }
    if return_type.starts_with("Vec<") {
        return "values".to_owned();
    }
    to_snake_case(member)
}

fn to_snake_case(value: &str) -> String {
    let mut output = String::new();
    for (index, character) in value.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index > 0 {
                output.push('_');
            }
            output.push(character.to_ascii_lowercase());
        } else {
            output.push(character);
        }
    }
    output
}

fn non_placeholder_summary(entry: &ApiEntry) -> String {
    let generic = entry.summary.starts_with("Configures or performs ")
        || entry.summary.starts_with("Builds or resolves ")
        || entry.summary.starts_with("Carries the ")
        || entry
            .summary
            .contains("on this typed datapack component definition");
    if !generic {
        return entry.summary.clone();
    }
    String::new()
}

#[derive(Clone, Copy)]
enum GuidanceKind {
    Use,
    Avoid,
}

fn source_guidance(documentation: &str, kind: GuidanceKind) -> Vec<String> {
    rustdoc_prose_paragraphs(documentation)
        .into_iter()
        .filter(|paragraph| {
            let lower = paragraph.trim_start().to_ascii_lowercase();
            match kind {
                GuidanceKind::Use => {
                    lower.starts_with("use ")
                        || lower.starts_with("call ")
                        || lower.starts_with("choose ")
                        || lower.starts_with("prefer ")
                }
                GuidanceKind::Avoid => {
                    lower.starts_with("avoid ")
                        || lower.starts_with("do not ")
                        || lower.starts_with("don't ")
                }
            }
        })
        .collect()
}

fn source_minecraft_behavior(documentation: &str) -> Option<String> {
    let paragraphs = rustdoc_prose_paragraphs(documentation)
        .into_iter()
        .skip(1)
        .collect::<Vec<_>>();
    paragraphs
        .iter()
        .find(|paragraph| {
            paragraph
                .trim_start()
                .to_ascii_lowercase()
                .starts_with("minecraft ")
        })
        .cloned()
        .or_else(|| {
            paragraphs.into_iter().find(|paragraph| {
                let lower = paragraph.to_ascii_lowercase();
                [
                    "minecraft",
                    " command",
                    "scoreboard",
                    "nbt",
                    "datapack",
                    "resource pack",
                    "json",
                    "export",
                ]
                .iter()
                .any(|token| lower.contains(token))
            })
        })
}

fn is_family_template_summary(summary: &str) -> bool {
    (summary.contains("typed ") && summary.ends_with(" API."))
        || summary.starts_with("Provides ") && summary.contains(" author API")
        || summary.starts_with("Exposes ") && summary.contains(" author API")
}

fn rustdoc_summary(documentation: &str) -> Option<String> {
    let paragraphs = rustdoc_prose_paragraphs(documentation);
    let first = paragraphs.first()?.trim().to_owned();
    if first.len() >= 48 || paragraphs.len() == 1 {
        return Some(first);
    }
    let second = &paragraphs[1];
    Some(format!(
        "{}. {}.",
        first.trim_end_matches('.'),
        second.trim_end_matches('.')
    ))
}

fn rustdoc_example(documentation: &str) -> Option<String> {
    let mut code = Vec::new();
    let mut fence: Option<bool> = None;
    for line in documentation.lines() {
        let trimmed = line.trim();
        if fence.is_none() {
            let Some(info) = trimmed.strip_prefix("```") else {
                continue;
            };
            let language = info.split(',').next().unwrap_or_default().trim();
            fence = Some(language.is_empty() || matches!(language, "rust" | "ignore" | "no_run"));
            continue;
        }
        if trimmed.starts_with("```") {
            if fence == Some(true) && !code.is_empty() {
                break;
            }
            fence = None;
            continue;
        }
        if fence == Some(true) && !trimmed.is_empty() && !trimmed.starts_with('#') {
            code.push(trimmed);
        }
    }
    (!code.is_empty()).then(|| code.join("\n"))
}

fn installed_source_identity(canonical_path: &str) -> Option<&'static str> {
    sand::__private::api_contract::INSTALLED_API_SHAPES
        .iter()
        .find(|(_, paths, ..)| paths.contains(&canonical_path))
        .map(|(identity, ..)| *identity)
        .or_else(|| {
            installed_path_mappings()
                .iter()
                .find_map(|(identity, canonical)| {
                    (*canonical == canonical_path).then_some(*identity)
                })
        })
}

fn installed_path_mappings() -> &'static BTreeMap<&'static str, &'static str> {
    static MAPPINGS: OnceLock<BTreeMap<&'static str, &'static str>> = OnceLock::new();
    MAPPINGS.get_or_init(|| {
        sand::__private::api_contract::INSTALLED_API_PATH_MAPPINGS
            .iter()
            .copied()
            .collect()
    })
}

fn installed_suffix_mappings() -> &'static BTreeMap<&'static str, &'static str> {
    static MAPPINGS: OnceLock<BTreeMap<&'static str, &'static str>> = OnceLock::new();
    MAPPINGS.get_or_init(|| {
        sand::__private::api_contract::INSTALLED_API_SUFFIX_MAPPINGS
            .iter()
            .copied()
            .collect()
    })
}

fn installed_type_suffix_mappings() -> &'static BTreeMap<&'static str, &'static str> {
    static MAPPINGS: OnceLock<BTreeMap<&'static str, &'static str>> = OnceLock::new();
    MAPPINGS.get_or_init(|| {
        sand::__private::api_contract::INSTALLED_API_TYPE_SUFFIX_MAPPINGS
            .iter()
            .copied()
            .collect()
    })
}

fn installed_type_path_mappings() -> &'static BTreeMap<&'static str, &'static str> {
    static MAPPINGS: OnceLock<BTreeMap<&'static str, &'static str>> = OnceLock::new();
    MAPPINGS.get_or_init(|| {
        sand::__private::api_contract::INSTALLED_API_TYPE_PATH_MAPPINGS
            .iter()
            .copied()
            .collect()
    })
}

fn installed_implementation_crates() -> &'static BTreeSet<&'static str> {
    static CRATES: OnceLock<BTreeSet<&'static str>> = OnceLock::new();
    CRATES.get_or_init(|| {
        installed_path_mappings()
            .keys()
            .filter_map(|path| path.split("::").next())
            .filter(|root| *root != "sand")
            .collect()
    })
}

fn normalize_shape_paths(value: &str, source_identity: Option<&str>) -> String {
    let mut normalized = value.to_owned();
    if let Some(source_crate) = source_identity.and_then(|identity| identity.split("::").next()) {
        normalized = normalized
            .replace("crate::", &format!("{source_crate}::"))
            .replace("crate ::", &format!("{source_crate} ::"));
    }
    if normalized.contains("super::")
        || normalized.contains("super ::")
        || normalized.contains("self::")
        || normalized.contains("self ::")
    {
        for (suffix, canonical) in installed_type_suffix_mappings() {
            normalized = normalized
                .replace(&format!("super::{suffix}"), canonical)
                .replace(&format!("super :: {suffix}"), canonical)
                .replace(&format!("self::{suffix}"), canonical)
                .replace(&format!("self :: {suffix}"), canonical);
        }
    }
    normalized = rewrite_braced_use_paths(&normalized, source_identity);
    rewrite_qualified_paths(&normalized, source_identity)
}

fn rewrite_braced_use_paths(value: &str, source_identity: Option<&str>) -> String {
    value
        .lines()
        .map(|line| {
            let indentation = &line[..line.len() - line.trim_start().len()];
            let trimmed = line.trim();
            let Some(body) = trimmed
                .strip_prefix("use ")
                .and_then(|body| body.strip_suffix(';'))
            else {
                return line.to_owned();
            };
            let mut leaves = Vec::new();
            if !flatten_use_tree("", body, &mut leaves) || !body.contains("::{") {
                return line.to_owned();
            }
            let resolved = leaves
                .into_iter()
                .map(|(implementation, rename)| {
                    resolve_qualified_path(&implementation, source_identity).map(|canonical| {
                        rename.map_or_else(
                            || canonical.to_owned(),
                            |rename| format!("{canonical} as {rename}"),
                        )
                    })
                })
                .collect::<Option<Vec<_>>>();
            let Some(resolved) = resolved else {
                return line.to_owned();
            };
            format!("{indentation}use {{{}}};", resolved.join(", "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn flatten_use_tree(prefix: &str, tree: &str, leaves: &mut Vec<(String, Option<String>)>) -> bool {
    let tree = tree.trim();
    if tree.starts_with('{') && tree.ends_with('}') {
        return split_use_members(&tree[1..tree.len() - 1])
            .into_iter()
            .all(|member| flatten_use_tree(prefix, member, leaves));
    }
    if let Some(position) = top_level_group_position(tree) {
        let owner = join_use_path(prefix, tree[..position].trim());
        let group = &tree[position + 2..];
        return flatten_use_tree(&owner, group, leaves);
    }
    let (leaf, rename) = tree
        .split_once(" as ")
        .map_or((tree, None), |(leaf, rename)| {
            (leaf.trim(), Some(rename.trim().to_owned()))
        });
    if leaf.is_empty() || leaf == "*" {
        return false;
    }
    let path = if leaf == "self" {
        prefix.to_owned()
    } else {
        join_use_path(prefix, leaf)
    };
    if path.is_empty() {
        return false;
    }
    leaves.push((path, rename));
    true
}

fn split_use_members(group: &str) -> Vec<&str> {
    let mut members = Vec::new();
    let mut depth = 0_u32;
    let mut start = 0;
    for (index, character) in group.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                if !group[start..index].trim().is_empty() {
                    members.push(group[start..index].trim());
                }
                start = index + 1;
            }
            _ => {}
        }
    }
    if !group[start..].trim().is_empty() {
        members.push(group[start..].trim());
    }
    members
}

fn top_level_group_position(tree: &str) -> Option<usize> {
    let bytes = tree.as_bytes();
    let mut depth = 0_u32;
    let mut cursor = 0;
    while cursor + 2 < bytes.len() {
        match bytes[cursor] {
            b'{' => depth += 1,
            b'}' => depth = depth.saturating_sub(1),
            b':' if depth == 0 && bytes[cursor + 1] == b':' && bytes[cursor + 2] == b'{' => {
                return Some(cursor);
            }
            _ => {}
        }
        cursor += 1;
    }
    None
}

fn join_use_path(prefix: &str, suffix: &str) -> String {
    if prefix.is_empty() {
        suffix.to_owned()
    } else {
        format!("{prefix}::{suffix}")
    }
}

fn rewrite_qualified_paths(value: &str, source_identity: Option<&str>) -> String {
    let mut output = String::with_capacity(value.len());
    let mut copied = 0;
    for (start, end, compact_path) in qualified_paths(value) {
        let raw_path = &value[start..end];
        if let Some(canonical) = resolve_qualified_path(&compact_path, source_identity) {
            output.push_str(&value[copied..start]);
            if raw_path.contains(" :: ") {
                output.push_str(&canonical.replace("::", " :: "));
            } else {
                output.push_str(canonical);
            }
            copied = end;
        }
    }
    output.push_str(&value[copied..]);
    output
}

fn qualified_paths(value: &str) -> Vec<(usize, usize, String)> {
    let bytes = value.as_bytes();
    let mut paths = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if !is_path_ident_start(bytes[cursor])
            || cursor > 0 && is_path_ident_continue(bytes[cursor - 1])
        {
            cursor += 1;
            continue;
        }
        let start = cursor;
        cursor += 1;
        while cursor < bytes.len() && is_path_ident_continue(bytes[cursor]) {
            cursor += 1;
        }
        let mut segments = 1;
        loop {
            let separator_start = cursor;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if bytes.get(cursor..cursor + 2) != Some(b"::") {
                cursor = separator_start;
                break;
            }
            cursor += 2;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if bytes.get(cursor) == Some(&b'{') {
                if segments == 1 {
                    paths.push((
                        start,
                        separator_start,
                        value[start..separator_start].to_owned(),
                    ));
                }
                cursor = separator_start;
                break;
            }
            if cursor >= bytes.len() || !is_path_ident_start(bytes[cursor]) {
                cursor = separator_start;
                break;
            }
            cursor += 1;
            while cursor < bytes.len() && is_path_ident_continue(bytes[cursor]) {
                cursor += 1;
            }
            segments += 1;
        }
        if segments >= 2 {
            paths.push((
                start,
                cursor,
                value[start..cursor].split_whitespace().collect(),
            ));
        }
    }
    paths
}

fn validate_exported_references(entries: &[ApiEntry]) -> Result<()> {
    let public_paths = entries
        .iter()
        .flat_map(|entry| {
            std::iter::once(entry.canonical_path.as_str())
                .chain(entry.aliases.iter().map(String::as_str))
        })
        .collect::<BTreeSet<_>>();
    let public_kinds = entries
        .iter()
        .flat_map(|entry| {
            std::iter::once((entry.canonical_path.as_str(), entry.kind)).chain(
                entry
                    .aliases
                    .iter()
                    .map(move |alias| (alias.as_str(), entry.kind)),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut invalid = BTreeSet::new();
    let mut invalid_types = BTreeSet::new();
    for entry in entries {
        let parameter_text = entry.parameters.iter().flat_map(|parameter| {
            std::iter::once(parameter.description.as_str()).chain(parameter.rust_type.as_deref())
        });
        let texts = [
            entry.signature.as_str(),
            entry.summary.as_str(),
            entry.context.as_str(),
            entry.minecraft.as_str(),
            entry.example.as_str(),
        ]
        .into_iter()
        .chain(entry.use_when.iter().map(String::as_str))
        .chain(entry.avoid_when.iter().map(String::as_str))
        .chain(entry.returns.as_deref())
        .chain(entry.return_type.as_deref())
        .chain(parameter_text);
        for text in texts {
            for (_, _, path) in qualified_paths(text) {
                let root = path.split("::").next().unwrap_or_default();
                if installed_implementation_crates().contains(root)
                    || path.starts_with("crate::")
                    || path.starts_with("sand::")
                        && !is_meaningful_public_path(&path, &public_paths)
                {
                    invalid.insert(format!("{} -> {path}", entry.canonical_path));
                }
            }
        }
        let structural_types = entry
            .parameters
            .iter()
            .filter_map(|parameter| parameter.rust_type.as_deref())
            .chain(entry.return_type.as_deref())
            // Constant signatures may include an initializer expression. A
            // path in that expression names a value, not a type identity.
            .chain(
                (!matches!(entry.kind, ApiKind::Constant | ApiKind::AssociatedConst))
                    .then_some(entry.signature.as_str()),
            );
        for text in structural_types {
            for (_, _, path) in qualified_paths(text) {
                if path.starts_with("sand::")
                    && !public_kinds
                        .get(path.as_str())
                        .is_some_and(|kind| type_capable_kind(*kind))
                {
                    invalid_types.insert(format!("{} -> {path}", entry.canonical_path));
                }
            }
        }
    }
    if invalid.is_empty() && invalid_types.is_empty() {
        Ok(())
    } else {
        let count = invalid.len() + invalid_types.len();
        let details = invalid
            .into_iter()
            .map(|item| format!("unresolved reference: {item}"))
            .chain(
                invalid_types
                    .into_iter()
                    .map(|item| format!("non-type structural reference: {item}")),
            )
            .take(50)
            .collect::<Vec<_>>()
            .join("\n");
        bail!("installed API catalog contains {count} invalid exported API references:\n{details}")
    }
}

fn type_capable_kind(kind: ApiKind) -> bool {
    matches!(
        kind,
        ApiKind::Struct
            | ApiKind::Enum
            | ApiKind::Trait
            | ApiKind::TypeAlias
            | ApiKind::AssociatedType
    )
}

fn is_meaningful_public_path(path: &str, public_paths: &BTreeSet<&str>) -> bool {
    public_paths.contains(path)
        || public_paths
            .iter()
            .any(|candidate| candidate.starts_with(&format!("{path}::")))
}

fn resolve_qualified_path(path: &str, source_identity: Option<&str>) -> Option<&'static str> {
    if let Some(canonical) = installed_path_mappings().get(path) {
        return Some(*canonical);
    }
    let implementation_crate = path.split("::").next()?;
    if !installed_implementation_crates().contains(implementation_crate) {
        return None;
    }
    let path_terminal = path.rsplit("::").next()?;
    if path.split("::").count() == 2
        && let Some(canonical) = installed_type_suffix_mappings().get(path_terminal)
    {
        return Some(*canonical);
    }
    if let Some(mut source_owner) = source_identity {
        while let Some((owner, _)) = source_owner.rsplit_once("::") {
            source_owner = owner;
            if source_owner.rsplit("::").next() == Some(path_terminal)
                && let Some(canonical) = installed_path_mappings().get(source_owner)
            {
                return Some(*canonical);
            }
        }
    }
    let mut suffix = path;
    while let Some((_, remainder)) = suffix.split_once("::") {
        suffix = remainder;
        if let Some(canonical) = installed_suffix_mappings().get(suffix) {
            return Some(*canonical);
        }
    }
    let descendant_prefix = format!("{path}::");
    let descendant_modules = installed_path_mappings()
        .iter()
        .filter_map(|(implementation, canonical)| {
            implementation
                .starts_with(&descendant_prefix)
                .then_some(*canonical)
                .map(|canonical| {
                    canonical
                        .rsplit_once("::")
                        .map_or(canonical, |(owner, _)| owner)
                })
        })
        .collect::<Vec<_>>();
    if let Some(module) = common_canonical_module(descendant_modules.iter().copied()) {
        return Some(module);
    }
    let preferred_domain = source_identity
        .and_then(|identity| installed_path_mappings().get(identity).copied())
        .and_then(|canonical| {
            let mut segments = canonical.split("::");
            Some(format!("{}::{}", segments.next()?, segments.next()?))
        });
    common_canonical_module(descendant_modules.into_iter().filter(|module| {
        preferred_domain
            .as_ref()
            .is_some_and(|domain| module.starts_with(domain))
    }))
}

fn common_canonical_module(
    modules: impl IntoIterator<Item = &'static str>,
) -> Option<&'static str> {
    let mut canonical_module = None;
    for module in modules {
        canonical_module = match canonical_module {
            None => Some(module),
            Some(existing)
                if module == existing || module.starts_with(&format!("{existing}::")) =>
            {
                Some(existing)
            }
            Some(existing) if existing.starts_with(&format!("{module}::")) => Some(module),
            Some(_) => return None,
        };
    }
    canonical_module
}

const fn is_path_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

const fn is_path_ident_continue(byte: u8) -> bool {
    is_path_ident_start(byte) || byte.is_ascii_digit() || byte == b'#'
}

fn is_import_only_example(example: &str) -> bool {
    let trimmed = example.trim();
    trimmed.starts_with("use sand::") && trimmed.lines().count() == 1
}

fn example_exercises_member(example: &str, canonical_path: &str) -> bool {
    let member = canonical_path
        .rsplit_once("::")
        .map_or(canonical_path, |(_, member)| member);
    [
        format!(".{member}"),
        format!("::{member}"),
        canonical_path.to_owned(),
    ]
    .iter()
    .any(|needle| {
        example.match_indices(needle).any(|(start, _)| {
            let tail = &example[start + needle.len()..];
            tail.starts_with('(')
                || tail
                    .strip_prefix("::<")
                    .and_then(|generic_tail| {
                        generic_tail.rfind('>').map(|end| &generic_tail[end + 1..])
                    })
                    .is_some_and(|after_generics| after_generics.starts_with('('))
        })
    })
}

fn declaration_reference_example(kind: ApiKind, canonical_path: &str) -> String {
    let import_path = match kind {
        ApiKind::Variant | ApiKind::Field | ApiKind::AssociatedConst | ApiKind::AssociatedType => {
            canonical_path
                .rsplit_once("::")
                .map_or(canonical_path, |(owner, _)| owner)
        }
        _ => canonical_path,
    };
    format!("use {import_path};")
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
    if !entry.summary.is_empty() {
        writeln!(output).unwrap();
        writeln!(output, "{}", entry.summary).unwrap();
    }
    if !entry.context.is_empty() {
        section(&mut output, "Context", &entry.context);
    }
    if !entry.minecraft.is_empty() {
        section(&mut output, "Minecraft behavior", &entry.minecraft);
    }

    if !entry.parameters.is_empty() {
        writeln!(output, "\nParameters").unwrap();
        for parameter in &entry.parameters {
            if let Some(rust_type) = &parameter.rust_type {
                if parameter.description.is_empty() {
                    writeln!(output, "  {} (`{}`)", parameter.name, rust_type).unwrap();
                } else {
                    writeln!(
                        output,
                        "  {} (`{}`): {}",
                        parameter.name, rust_type, parameter.description
                    )
                    .unwrap();
                }
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
    } else if let Some(return_type) = &entry.return_type {
        section(&mut output, "Returns", &format!("`{return_type}`"));
    }
    list_section(&mut output, "Use when", &entry.use_when);
    list_section(&mut output, "Avoid when", &entry.avoid_when);
    if !entry.example.is_empty() {
        section(&mut output, "Example", &entry.example);
    }

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

    #[test]
    fn source_paths_resolve_through_installed_facade_identities() {
        let cases = [
            (
                "crate::selector::EntityTarget<Player>",
                "sand_commands::selector::EntityTarget",
                "sand::command::EntityTarget<Player>",
            ),
            (
                "crate :: worldgen :: ConfiguredCarver",
                "sand_components::worldgen::ConfiguredCarver",
                "sand :: component :: ConfiguredCarver",
            ),
            (
                "crate::participant::EntityParticipant",
                "sand_core::events::SandEventParticipants::entity",
                "sand::participant::EntityParticipant",
            ),
            (
                "crate::cmd::IntoGiveItem",
                "sand_core::cmd::IntoGiveItem",
                "sand::command::IntoGiveItem",
            ),
            (
                "crate::event::handle::EventHandle<E>",
                "sand_core::event::handle::EventHandle",
                "sand::event::handle::EventHandle<E>",
            ),
        ];
        for (source, identity, expected) in cases {
            assert_eq!(normalize_shape_paths(source, Some(identity)), expected);
        }
        assert_eq!(
            normalize_shape_paths(
                "use sand_components::{AdvancementDisplay, AdvancementIcon, ItemId};",
                Some("sand_components::advancement::AdvancementDisplay::new"),
            ),
            "use {sand::component::AdvancementDisplay, sand::component::AdvancementIcon, sand::registry::ItemId};"
        );
        assert_eq!(
            normalize_shape_paths(
                "use sand_components::{advancement::{AdvancementDisplay, AdvancementIcon}, ItemId};",
                Some("sand_components::advancement::AdvancementDisplay::new"),
            ),
            "use {sand::component::AdvancementDisplay, sand::component::AdvancementIcon, sand::registry::ItemId};"
        );
        assert_eq!(
            normalize_shape_paths(
                "sand_components::worldgen::Noise",
                Some("sand_components::worldgen::noise::Noise::id"),
            ),
            "sand::component::Noise"
        );
        assert_eq!(
            normalize_shape_paths(
                "sand_components::private_model::Text",
                Some("sand_components::private_model::Owner"),
            ),
            "sand_components::private_model::Text",
            "an ambiguous terminal name must not be assigned to an arbitrary public owner",
        );

        let public_paths = BTreeSet::from(["sand::component", "sand::component::ItemId"]);
        assert!(is_meaningful_public_path(
            "sand::component::ItemId",
            &public_paths
        ));
        assert!(!is_meaningful_public_path(
            "sand::component::DefinitelyMissing",
            &public_paths
        ));
    }

    #[test]
    fn opaque_event_dispatch_constructor_uses_the_public_trigger_type() {
        let entry = generated_catalog()
            .find("sand::events::SandEventDispatch::AdvancementTrigger")
            .expect("opaque advancement constructor is installed");
        assert_eq!(
            entry.parameters[0]
                .rust_type
                .as_deref()
                .map(|value| value.split_whitespace().collect::<String>()),
            Some("sand::component::AdvancementTrigger".to_owned())
        );
        assert!(
            !entry.parameters[0]
                .rust_type
                .as_deref()
                .unwrap()
                .contains("SandEventDispatch::AdvancementTrigger")
        );
    }

    #[test]
    fn structural_sand_paths_must_resolve_to_type_capable_identities() {
        let mut trigger_type = entry(
            "sand::component::AdvancementTrigger",
            ApiKind::Enum,
            "A typed advancement trigger.",
            "sand::component",
        );
        trigger_type.signature = "pub enum AdvancementTrigger".into();
        trigger_type.returns = None;
        trigger_type.return_type = None;

        let mut constructor = entry(
            "sand::events::SandEventDispatch::AdvancementTrigger",
            ApiKind::Method,
            "Creates an event dispatch.",
            "sand::events",
        );
        constructor.signature =
            "pub fn AdvancementTrigger(trigger: sand::component::AdvancementTrigger) -> Self"
                .into();
        constructor.parameters = vec![ApiParameter {
            name: "trigger".into(),
            rust_type: Some("sand::component::AdvancementTrigger".into()),
            description: "The typed advancement trigger.".into(),
        }];
        constructor.return_type = Some("Self".into());
        validate_exported_references(&[trigger_type.clone(), constructor.clone()])
            .expect("an enum is valid in structural type metadata");

        constructor.signature = "pub fn AdvancementTrigger(trigger: sand::events::SandEventDispatch::AdvancementTrigger) -> Self".into();
        constructor.parameters[0].rust_type =
            Some("sand::events::SandEventDispatch::AdvancementTrigger".into());
        let error = validate_exported_references(&[trigger_type, constructor])
            .expect_err("a method identity cannot masquerade as a parameter type");
        assert!(error.to_string().contains("non-type structural reference"));
    }

    #[test]
    fn example_type_normalization_handles_std_types_and_generic_methods() {
        assert_eq!(
            normalize_example_type("HashMap<String, String>", "sand::component::Owner", None),
            "std::collections::HashMap<String, String>"
        );
        assert_eq!(
            normalize_example_type("impl RangeBounds<i32>", "sand::component::Owner", None),
            "impl std::ops::RangeBounds<i32>"
        );
        assert_eq!(
            normalize_example_type("impl Display", "sand::component::Owner", None),
            "impl std::fmt::Display"
        );
        assert_eq!(
            normalize_example_type(
                "sand::component::Result<Value, Error>",
                "sand::component::Owner",
                None
            ),
            "std::result::Result<serde_json::Value, Error>"
        );
        assert_eq!(explicit_method_arguments("<'a, G, N>"), "::<G, N>");
    }

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
        assert_eq!(json["schema_version"], 3);
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
            let compact_signature = entry.signature.replace(' ', "");
            assert!(
                !entry.signature.contains("# [doc")
                    && !entry.signature.contains("#[doc")
                    && !entry.signature.contains("```")
                    && !compact_signature.contains("sand_core::")
                    && !compact_signature.contains("sand_commands::")
                    && !compact_signature.contains("sand_components::")
                    && !compact_signature.contains("sand_resourcepack::")
                    && !compact_signature.contains("sand_version::")
                    && !compact_signature.contains("crate::"),
                "signature leaks attributes or implementation paths: {} => {}",
                entry.canonical_path,
                entry.signature
            );
            assert!(
                !entry
                    .returns
                    .as_deref()
                    .is_some_and(|returns| returns.contains("`a diagnostic`")),
                "return prose quotes a fabricated error type: {}",
                entry.canonical_path
            );
            assert!(
                entry.avoid_when.iter().all(|guidance| {
                    !guidance.contains("When this documented limitation applies")
                        && !guidance.contains("when the goal is not to ")
                }),
                "avoidance uses inferred or ungrammatical filler: {}",
                entry.canonical_path
            );
            assert!(!entry.summary.starts_with("Configures or performs "));
            assert!(!entry.summary.starts_with("Builds or resolves "));
            assert!(!entry.summary.starts_with("Carries the "));
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
            let serialized = serde_json::to_string(entry).unwrap();
            assert!(
                !serialized.contains("crate::") && !serialized.contains("crate ::"),
                "contract prose leaks an implementation-relative path: {}",
                entry.canonical_path
            );
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
        assert_eq!(path.return_type.as_deref(), Some("& str"));

        let documented_field = catalog.find("sand::command::BlockPos::x").unwrap();
        assert_eq!(documented_field.signature, "pub x: Coord");
        assert!(documented_field.summary.contains('x'));
        assert!(documented_field.summary.contains("coordinate"));
        assert!(documented_field.example.contains("value.x"));

        assert!(
            catalog
                .find("sand::entity::CurveInputs::get")
                .unwrap()
                .signature
                .starts_with("pub fn get")
        );
        #[cfg(feature = "resourcepack")]
        assert_eq!(
            catalog
                .find("sand::resourcepack::AssetOutput::path")
                .unwrap()
                .signature,
            "pub path: String"
        );
        assert_eq!(
            catalog
                .find("sand::command::Coord::Absolute::0")
                .unwrap()
                .signature,
            "f64"
        );
        assert!(
            catalog
                .find("sand::command::RenderCommand")
                .unwrap()
                .signature
                .contains(": Validate")
        );
        assert!(
            catalog
                .find("sand::version::LATEST_KNOWN")
                .unwrap()
                .signature
                .contains("= sand :: version :: LATEST_KNOWN")
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
                    entry.return_type.is_none() && entry.returns.is_none(),
                    "non-callable carries callable return metadata: {}",
                    entry.canonical_path
                );
                assert!(
                    !entry.example.trim_end().ends_with("();"),
                    "non-callable is presented as a constructor: {}",
                    entry.canonical_path
                );
            } else {
                assert_ne!(
                    entry.context, entry.summary,
                    "callable context only repeats its summary: {}",
                    entry.canonical_path
                );
                assert!(!entry.minecraft.starts_with(
                    "Minecraft and generated-output behavior follows the defining item's documented semantics:"
                ));
                assert!(entry.use_when.iter().all(|guidance| {
                    !guidance
                        .starts_with("When the defining item's documented behavior is required:")
                }));
                assert!(entry.avoid_when.iter().all(|guidance| guidance
                    != "When the defining item's documented preconditions or scope do not apply."));
                assert!(
                    !entry.example.trim_start().starts_with("// Call `"),
                    "callable has only a comment synopsis instead of a usage expression: {}",
                    entry.canonical_path
                );
            }
        }

        let trim_name = catalog.find("sand::component::TrimAssetName::new").unwrap();
        assert_eq!(
            trim_name.return_type.as_deref(),
            Some("SandResult < Self >")
        );
        let block_pos = catalog.find("sand::command::BlockPos::new").unwrap();
        assert_eq!(block_pos.return_type.as_deref(), Some("Self"));
        let from_score = catalog
            .find("sand::state::TypedGameState::from_score")
            .unwrap();
        assert_eq!(from_score.return_type.as_deref(), Some("Option < Self >"));
        let insert_score = catalog
            .find("sand::entity::CurveInputs::insert_score")
            .unwrap();
        assert!(
            insert_score
                .return_type
                .as_deref()
                .is_some_and(|returns| returns.contains("Result < Option < FixedValue >"))
        );

        for entry in &catalog.entries {
            if sand::__private::api_contract::INSTALLED_FAMILY_API_PATHS
                .contains(&entry.canonical_path.as_str())
                && matches!(
                    entry.kind,
                    ApiKind::Function | ApiKind::Method | ApiKind::TraitMethod
                )
            {
                if !entry.example.is_empty() {
                    assert!(
                        example_exercises_member(&entry.example, &entry.canonical_path),
                        "family callable example does not exercise its member: {} => {}",
                        entry.canonical_path,
                        entry.example
                    );
                }
                assert!(
                    entry
                        .avoid_when
                        .iter()
                        .all(|guidance| !guidance.contains("does not provide the required")),
                    "family avoidance is tautological: {}",
                    entry.canonical_path
                );
            }
        }
        assert!(catalog.entries.iter().all(|entry| {
            entry
                .parameters
                .iter()
                .all(|parameter| !parameter.description.starts_with("The source-derived `"))
        }));
        assert!(catalog.entries.iter().all(|entry| {
            entry
                .use_when
                .iter()
                .all(|guidance| !guidance.starts_with("Use `sand::"))
        }));

        let raw_description = catalog
            .find("sand::component::AdvancementDisplay::raw_description")
            .unwrap();
        assert!(raw_description.parameters[0].description.contains("raw"));
        assert!(
            !raw_description.parameters[0]
                .description
                .contains("player-visible Minecraft text")
        );
        let click_copy = catalog
            .find("sand::text::TextComponent::click_copy")
            .unwrap();
        assert!(click_copy.parameters[0].description.contains("clipboard"));
        let hover = catalog
            .find("sand::text::TextComponent::hover_entity_with_id")
            .unwrap();
        assert!(hover.parameters[1].description.contains("entity UUID"));
        let into_event = catalog
            .find("sand::event::IntoEventId::into_event_resource_location")
            .unwrap();
        assert!(example_exercises_member(
            &into_event.example,
            &into_event.canonical_path
        ));

        let participant_example = &catalog
            .find("sand::participant::PlayerParticipant")
            .unwrap()
            .example;
        assert_eq!(
            participant_example,
            "use sand::participant::PlayerParticipant;"
        );
        assert!(!participant_example.contains("sand_core::"));
        let actionbar_example = &catalog.find("sand::command::Actionbar").unwrap().example;
        assert_eq!(actionbar_example, "use sand::command::Actionbar;");
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
        assert_eq!(
            rustdoc_example("Shape:\n\n```text\nnot Rust\n```\nTrailing prose."),
            None
        );
        assert_eq!(
            rustdoc_example("Shape:\n\n```text\nnot Rust\n```\n\n```rust\nlet actual = 3;\n```"),
            Some("let actual = 3;".to_owned())
        );
        assert_eq!(
            rustdoc_example(
                "Rejected:\n\n```compile_fail\nlet wrong = missing();\n```\n\n```rust\nlet right = 4;\n```"
            ),
            Some("let right = 4;".to_owned())
        );
        assert_eq!(
            rustdoc_summary(
                "Short heading.\n\n```rust\nlet internal = sand_components::Thing;\n```\n\nExplains the author-facing behavior in enough detail."
            ),
            Some("Short heading. Explains the author-facing behavior in enough detail.".to_owned())
        );
        assert_eq!(
            rustdoc_summary("Fake scoreboard holder.\n\nConvention: prefix with # (e.g. #global)."),
            Some("Fake scoreboard holder. Convention: prefix with # (e.g. #global).".to_owned())
        );

        let catalog = generated_catalog();
        assert!(
            !catalog
                .find("sand::state::ScoreVar::clamp")
                .unwrap()
                .example
                .contains("Trailing prose")
        );
        for path in ["sand::event::Event", "sand::events::SandEvent"] {
            let example = &catalog.find(path).unwrap().example;
            assert!(!example.contains("sand_macros::"), "{path}: {example}");
        }
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
        assert!(entity.contains("selector : sand :: command :: Selector"));
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
        assert_eq!(entries.len(), 66);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.canonical_path.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            66
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

    #[test]
    fn entity_archetype_constructor_exposes_structural_and_semantic_contract_details() {
        let contract = show(generated_catalog(), "sand::entity::EntityArchetype::new").unwrap();
        assert!(contract.contains("pub fn new (id : ResourceLocation) -> Self"));
        assert!(contract.contains("id (`ResourceLocation`):"));
        assert!(contract.contains("Returns\n  `Self`"));
        assert!(contract.contains("Minecraft receives the resulting objectives"));
        assert!(contract.contains("Avoid creating multiple archetypes"));
    }

    #[test]
    fn all_generated_callable_examples_compile_in_a_downstream_crate() {
        use std::fs;
        use std::process::Command;

        let catalog = generated_catalog();
        let paths = [
            // Standard-library collection and trait paths must be nameable
            // without relying on the Sand prelude.
            "sand::component::AdvancementTrigger::enter_block",
            "sand::entity::EntityScore::matches",
            "sand::component::Ingredient::item",
            // Two-argument `Result` is the standard result, not the
            // component serialization alias with the same suffix.
            "sand::entity::StatCurve::custom",
            // A zero-argument generic method needs an explicit turbofish.
            "sand::events::SandEventDispatch::after_any",
        ];
        for path in paths {
            let entry = catalog
                .find(path)
                .unwrap_or_else(|| panic!("representative callable `{path}` is installed"));
            assert!(
                entry.example.contains("fn demonstrate"),
                "`{path}` should use a generated callable example"
            );
        }

        let generated = catalog
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry.kind,
                    ApiKind::Function | ApiKind::Method | ApiKind::TraitMethod
                ) && entry.example.contains("fn demonstrate")
            })
            .collect::<Vec<_>>();
        assert!(
            generated.len() > 1_900,
            "expected repository-wide generated callable coverage, found {}",
            generated.len()
        );
        let mut source = String::new();
        for (index, entry) in generated.iter().enumerate() {
            source.push_str(&format!(
                "#[allow(dead_code, unused_imports, unused_variables, unreachable_code)]\nmod example_{index} {{\n{}\n}}\n",
                entry.example
            ));
        }

        let project = tempfile::tempdir().expect("create downstream example crate");
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("sand-cli is inside the workspace");
        fs::create_dir(project.path().join("src")).expect("create source directory");
        fs::write(
            project.path().join("Cargo.toml"),
            format!(
                "[package]\nname = \"sand-contract-example-check\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\nsand = {{ path = {:?}, features = [\"systems-all\", \"resourcepack\"] }}\nserde_json = \"1\"\n",
                workspace.join("sand")
            ),
        )
        .expect("write downstream manifest");
        fs::write(project.path().join("src/lib.rs"), source).expect("write downstream examples");
        let target = workspace.join("target/generated-api-example-check");
        let output = Command::new(env!("CARGO"))
            .current_dir(project.path())
            .env("CARGO_TARGET_DIR", &target)
            .args(["check", "--offline", "--quiet"])
            .output()
            .expect("run downstream cargo check");
        assert!(
            output.status.success(),
            "all {} generated callable examples must compile:\n{}",
            generated.len(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn all_exported_event_marker_examples_compile_in_a_downstream_crate() {
        use std::fs;
        use std::process::Command;

        let catalog = generated_catalog();
        let item_examples = catalog
            .entries
            .iter()
            .filter(|entry| {
                let example = entry.example.trim_start();
                entry.canonical_path.starts_with("sand::events::")
                    && example.starts_with("#[sand::on_event")
                    && example.contains('\n')
            })
            .collect::<Vec<_>>();
        assert!(
            item_examples.len() >= 60,
            "expected repository-wide event-marker example coverage, found {}",
            item_examples.len()
        );

        let mut source = String::new();
        for (index, entry) in item_examples.iter().enumerate() {
            source.push_str(&format!(
                "#[allow(dead_code, unused_imports, unused_variables, unreachable_code)]\nmod example_{index} {{\nuse sand::prelude::*;\n{}\n}}\n",
                entry.example
            ));
        }

        let project = tempfile::tempdir().expect("create downstream event-example crate");
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("sand-cli is inside the workspace");
        fs::create_dir(project.path().join("src")).expect("create source directory");
        fs::write(
            project.path().join("Cargo.toml"),
            format!(
                "[package]\nname = \"sand-contract-event-example-check\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\nsand = {{ path = {:?}, features = [\"systems-all\", \"resourcepack\"] }}\nserde_json = \"1\"\n",
                workspace.join("sand")
            ),
        )
        .expect("write downstream manifest");
        fs::write(project.path().join("src/lib.rs"), source)
            .expect("write downstream event examples");
        let target = workspace.join("target/exported-api-event-example-check");
        let output = Command::new(env!("CARGO"))
            .current_dir(project.path())
            .env("CARGO_TARGET_DIR", &target)
            .args(["check", "--offline", "--quiet"])
            .output()
            .expect("run downstream cargo check");
        assert!(
            output.status.success(),
            "all {} exported event-marker examples must compile:\n{}",
            item_examples.len(),
            String::from_utf8_lossy(&output.stderr)
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
