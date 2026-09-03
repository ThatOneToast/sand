use std::fmt::Write as FmtWrite;
use std::path::Path;

use heck::{ToPascalCase, ToSnakeCase};
use sand_api_contract::{ApiEntry, ApiKind, ApiParameter};
use serde_json::Value;

use crate::api_provider::{ApiProviderCatalog, GeneratedProviderEntry, write_contract_rustdoc};
use crate::error::Result;

/// Top-level commands to skip entirely (they use redirects or are aliases).
const SKIP_COMMANDS: &[&str] = &["effect", "execute", "tell", "tm", "tp", "w", "xp"];

/// Generated variants shadowed by Sand's intentional handwritten command API.
///
/// These variants are omitted instead of being emitted under a second Rust
/// identity at the same `sand::command` path. The handwritten versions carry
/// stronger domain types, validation, or richer builders than the raw command
/// tree can describe. Other variants in the same vanilla command family remain
/// generated normally.
const HANDWRITTEN_VARIANTS: &[&str] = &[
    "fill",
    "function",
    "give",
    "particle",
    "return_cmd",
    "return_fail",
    "tellraw",
    "title_times",
];

/// Maximum tree depth to prevent runaway generation.
const MAX_DEPTH: usize = 6;

// ---------------------------------------------------------------------------
// JSON helpers
// ---------------------------------------------------------------------------

fn node_type(node: &Value) -> &str {
    node.get("type").and_then(|v| v.as_str()).unwrap_or("")
}

fn is_executable(node: &Value) -> bool {
    node.get("executable")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn has_redirect(node: &Value) -> bool {
    node.get("redirect").is_some()
}

fn children(node: &Value) -> Vec<(&str, &Value)> {
    match node.get("children").and_then(|v| v.as_object()) {
        Some(map) => map.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        None => Vec::new(),
    }
}

fn parser_str(node: &Value) -> &str {
    node.get("parser").and_then(|v| v.as_str()).unwrap_or("")
}

// ---------------------------------------------------------------------------
// Type mapping
// ---------------------------------------------------------------------------

/// Returns (param_type, stored_type, needs_into)
/// `needs_into` means the param uses `impl Into<String>` and the field is `String`.
fn map_parser(parser: &str) -> (&'static str, &'static str, bool) {
    // Types are referenced without `crate::` prefix because the generated code
    // is included inside a module with `use super::*` bringing cmd types into scope.
    match parser {
        "brigadier:bool" => ("bool", "bool", false),
        "brigadier:integer" => ("i32", "i32", false),
        "brigadier:float" => ("f32", "f32", false),
        "brigadier:double" => ("f64", "f64", false),
        "minecraft:entity" | "minecraft:game_profile" => ("Selector", "Selector", false),
        "minecraft:block_pos" | "minecraft:column_pos" => ("BlockPos", "BlockPos", false),
        "minecraft:vec3" => ("Vec3", "Vec3", false),
        "minecraft:vec2" => ("Vec2", "Vec2", false),
        "minecraft:rotation" => ("Rotation", "Rotation", false),
        "minecraft:color" => ("ChatColor", "ChatColor", false),
        "minecraft:component" | "minecraft:style" => ("TextComponent", "TextComponent", false),
        "minecraft:resource_location"
        | "minecraft:dimension"
        | "minecraft:function"
        | "minecraft:loot_table"
        | "minecraft:loot_predicate"
        | "minecraft:loot_modifier" => {
            ("crate::ResourceLocation", "crate::ResourceLocation", false)
        }
        "minecraft:gamemode" => ("GameMode", "GameMode", false),
        "minecraft:entity_anchor" => ("Anchor", "Anchor", false),
        "minecraft:swizzle" => ("Swizzle", "Swizzle", false),
        // Everything else: impl Into<String>
        _ => ("impl Into<String>", "String", true),
    }
}

fn map_arg_parser(literals: &[&str], arg: &ArgInfo) -> (&'static str, &'static str, bool) {
    if literals == ["damage"] && arg.name == "target" && arg.parser == "minecraft:entity" {
        return ("impl Into<SingleEntity>", "SingleEntity", true);
    }

    map_parser(&arg.parser)
}

// ---------------------------------------------------------------------------
// Field name sanitization
// ---------------------------------------------------------------------------

fn sanitize_field_name(name: &str) -> String {
    let s = name.replace('-', "_");
    match s.as_str() {
        "type" => "kind".to_string(),
        "in" => "in_dim".to_string(),
        "return" => "return_val".to_string(),
        "fn" => "func".to_string(),
        "move" => "move_to".to_string(),
        "match" => "match_val".to_string(),
        "loop" => "loop_val".to_string(),
        _ => s,
    }
}

// ---------------------------------------------------------------------------
// Data structures for collected command variants
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct ArgInfo {
    /// The JSON name (sanitized for Rust).
    name: String,
    /// The parser string from JSON (e.g. "brigadier:integer").
    parser: String,
}

/// A segment of the full command path — either a literal keyword or a required
/// argument that appears at that position in the command string.
///
/// Tracking both together allows correct generation for commands like
/// `advancement revoke <targets> only <advancement>` where an argument
/// appears *between* two literal keywords.
#[derive(Debug, Clone)]
enum PathSegment {
    Literal(String),
    Arg(ArgInfo),
}

#[derive(Debug, Clone)]
struct CommandVariant {
    /// Full ordered path: literals and required args interleaved as they appear.
    full_path: Vec<PathSegment>,
    /// Optional arguments collected after the first executable node.
    optional_args: Vec<ArgInfo>,
}

impl CommandVariant {
    fn literal_segments(&self) -> Vec<&str> {
        self.full_path
            .iter()
            .filter_map(|s| {
                if let PathSegment::Literal(s) = s {
                    Some(s.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    fn required_args(&self) -> Vec<&ArgInfo> {
        self.full_path
            .iter()
            .filter_map(|s| {
                if let PathSegment::Arg(a) = s {
                    Some(a)
                } else {
                    None
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tree walking
// ---------------------------------------------------------------------------

/// Check if any node in this subtree contains a redirect.
fn subtree_has_redirect(node: &Value) -> bool {
    if has_redirect(node) {
        return true;
    }
    for (_name, child) in children(node) {
        if subtree_has_redirect(child) {
            return true;
        }
    }
    false
}

/// Walk the command tree and collect all executable variants.
///
/// `full_path`        — the ordered mix of literals and required args accumulated so far.
/// `found_executable` — whether we have already seen an executable node on this path.
/// `depth`            — current depth (bounded by MAX_DEPTH).
///
/// Using a single `full_path` of [`PathSegment`]s (rather than separate literal and
/// arg slices) ensures that arguments appearing *between* two literal keywords are
/// preserved at the correct position in the generated command string — e.g.
/// `advancement revoke <targets> only <advancement>` keeps `<targets>` before `only`.
fn walk(
    node: &Value,
    full_path: &[PathSegment],
    found_executable: bool,
    depth: usize,
    variants: &mut Vec<CommandVariant>,
) {
    if depth > MAX_DEPTH {
        return;
    }

    // If this node is executable AND we haven't emitted a variant for this
    // literal path yet, record it and collect optional args from deeper children.
    if is_executable(node) && !found_executable {
        let mut optional = Vec::new();
        collect_optional_args(node, depth, &mut optional);

        variants.push(CommandVariant {
            full_path: full_path.to_vec(),
            optional_args: optional,
        });
        // Recurse into literal children only to find deeper sub-commands.
        for (name, child) in children(node) {
            if node_type(child) == "literal" && !has_redirect(child) {
                let mut new_path = full_path.to_vec();
                new_path.push(PathSegment::Literal(name.to_string()));
                walk(child, &new_path, false, depth + 1, variants);
            }
        }
        return;
    }

    // Recurse into children, accumulating both literals and args into full_path.
    for (name, child) in children(node) {
        if has_redirect(child) {
            continue;
        }
        match node_type(child) {
            "literal" => {
                let mut new_path = full_path.to_vec();
                new_path.push(PathSegment::Literal(name.to_string()));
                walk(child, &new_path, found_executable, depth + 1, variants);
            }
            "argument" => {
                let mut new_path = full_path.to_vec();
                new_path.push(PathSegment::Arg(ArgInfo {
                    name: sanitize_field_name(name),
                    parser: parser_str(child).to_string(),
                }));
                walk(child, &new_path, found_executable, depth + 1, variants);
            }
            _ => {}
        }
    }
}

/// After finding the first executable node, collect optional argument children
/// (following only argument nodes, not literals, to avoid branching into separate commands).
/// Deduplicates by name — the test command has branching optional args that share the same names.
fn collect_optional_args(node: &Value, depth: usize, optional: &mut Vec<ArgInfo>) {
    if depth > MAX_DEPTH {
        return;
    }
    for (name, child) in children(node) {
        if node_type(child) == "argument" && !has_redirect(child) {
            let sanitized = sanitize_field_name(name);
            // Skip duplicates (can occur when multiple optional branches share arg names).
            if optional.iter().any(|a| a.name == sanitized) {
                continue;
            }
            optional.push(ArgInfo {
                name: sanitized,
                parser: parser_str(child).to_string(),
            });
            // Continue collecting deeper optional args.
            collect_optional_args(child, depth + 1, optional);
        }
        // Stop at literal children — those would be separate sub-commands.
    }
}

// ---------------------------------------------------------------------------
// Code generation
// ---------------------------------------------------------------------------

fn struct_name(literals: &[&str]) -> String {
    let joined = literals.join("_");
    let pascal = joined.replace('-', "_").to_pascal_case();
    if pascal.is_empty() {
        "UnknownCmd".to_string()
    } else if pascal.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{pascal}")
    } else {
        pascal
    }
}

fn fn_name(literals: &[&str]) -> String {
    let joined = literals.join("_");
    let snake = joined.replace('-', "_").to_snake_case();
    match snake.as_str() {
        "return" => "return_cmd".to_string(),
        "fn" => "fn_cmd".to_string(),
        "move" => "move_cmd".to_string(),
        "match" => "match_cmd".to_string(),
        "loop" => "loop_cmd".to_string(),
        "type" => "type_cmd".to_string(),
        "in" => "in_cmd".to_string(),
        _ => snake,
    }
}

/// Deduplication-safe wrapper: holds the original (immutable) CommandVariant alongside
/// the Rust identifier names that may have had a numeric suffix appended to resolve
/// collisions. The vanilla command path in `command.full_path` is never mutated.
struct EmittedVariant {
    command: CommandVariant,
    /// Deduplicated Rust struct name (e.g. `Kill`, `Kill2`).
    struct_name: String,
    /// Deduplicated Rust function name (e.g. `kill`, `kill_2`).
    fn_name: String,
}

/// Build `EmittedVariant`s from collected `CommandVariant`s, assigning unique
/// Rust identifiers without touching the vanilla command path.
fn assign_rust_names(variants: Vec<CommandVariant>) -> Vec<EmittedVariant> {
    let mut seen_structs: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut seen_fns: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    variants
        .into_iter()
        .map(|variant| {
            let literals = variant.literal_segments();
            let base_sn = struct_name(&literals);
            let base_fn = fn_name(&literals);

            // Read current counts (copies out of the map, ending the borrows).
            let s_count = *seen_structs.entry(base_sn.clone()).or_insert(0);
            let f_count = *seen_fns.entry(base_fn.clone()).or_insert(0);
            let idx = s_count.max(f_count);

            let final_sn = if idx == 0 {
                base_sn.clone()
            } else {
                format!("{}{}", base_sn, idx + 1)
            };
            let final_fn = if idx == 0 {
                base_fn.clone()
            } else {
                format!("{}_{}", base_fn, idx + 1)
            };

            // Advance both counters so the next collision gets the next suffix.
            let new_count = idx + 1;
            seen_structs.insert(base_sn, new_count);
            seen_fns.insert(base_fn, new_count);

            EmittedVariant {
                command: variant,
                struct_name: final_sn,
                fn_name: final_fn,
            }
        })
        .collect()
}

fn emit_variant(code: &mut String, ev: &EmittedVariant, contracts: &[GeneratedProviderEntry]) {
    let variant = &ev.command;
    let literals = variant.literal_segments();
    let required = variant.required_args();
    let sname = &ev.struct_name;
    let fname = &ev.fn_name;
    // Command string for docs: original vanilla literals only (never suffixed).
    let cmd_str = literals.join(" ");

    let has_required = !required.is_empty();
    let has_optional = !variant.optional_args.is_empty();

    // Build doc comment showing full usage with args in their correct positions.
    let mut usage = String::new();
    for (i, seg) in variant.full_path.iter().enumerate() {
        if i > 0 {
            usage.push(' ');
        }
        match seg {
            PathSegment::Literal(s) => usage.push_str(s),
            PathSegment::Arg(a) => write!(usage, "<{}>", a.name).unwrap(),
        }
    }
    for arg in &variant.optional_args {
        write!(usage, " [<{}>]", arg.name).unwrap();
    }

    writeln!(code, "// /{usage}").unwrap();
    let struct_contract = contracts
        .iter()
        .find(|entry| entry.contract.kind == ApiKind::Struct)
        .expect("command generator creates a struct contract");
    write_contract_rustdoc(code, "", &struct_contract.contract);

    if !has_required {
        writeln!(code, "#[derive(Debug, Clone, Default)]").unwrap();
    } else {
        writeln!(code, "#[derive(Debug, Clone)]").unwrap();
    }

    writeln!(code, "pub struct {sname} {{").unwrap();
    for arg in &required {
        let (_param_ty, stored_ty, _needs_into) = map_arg_parser(&literals, arg);
        writeln!(code, "    {}: {stored_ty},", arg.name).unwrap();
    }
    for arg in &variant.optional_args {
        let (_param_ty, stored_ty, _needs_into) = map_arg_parser(&literals, arg);
        writeln!(code, "    {}: Option<{stored_ty}>,", arg.name).unwrap();
    }
    writeln!(code, "}}").unwrap();
    writeln!(code).unwrap();

    writeln!(code, "impl {sname} {{").unwrap();

    if has_required {
        let mut params = Vec::new();
        let mut body_lines = Vec::new();
        for arg in &required {
            let (param_ty, _stored_ty, needs_into) = map_arg_parser(&literals, arg);
            if needs_into {
                params.push(format!("{}: {param_ty}", arg.name));
                body_lines.push(format!("{}: {}.into()", arg.name, arg.name));
            } else {
                params.push(format!("{}: {param_ty}", arg.name));
                body_lines.push(format!("{name}: {name}", name = arg.name));
            }
        }
        for arg in &variant.optional_args {
            body_lines.push(format!("{}: None", arg.name));
        }
        let params_str = params.join(", ");
        writeln!(code, "    pub(crate) fn new({params_str}) -> Self {{").unwrap();
        writeln!(code, "        Self {{").unwrap();
        for line in &body_lines {
            writeln!(code, "            {line},").unwrap();
        }
        writeln!(code, "        }}").unwrap();
        writeln!(code, "    }}").unwrap();
    }

    for arg in &variant.optional_args {
        let (param_ty, _stored_ty, needs_into) = map_arg_parser(&literals, arg);
        let method_path = format!("sand::command::{sname}::{}", arg.name);
        let method_contract = contracts
            .iter()
            .find(|entry| entry.contract.canonical_path == method_path)
            .expect("command generator creates an optional-argument contract");
        write_contract_rustdoc(code, "    ", &method_contract.contract);
        writeln!(
            code,
            "    pub fn {name}(mut self, {name}: {param_ty}) -> Self {{",
            name = arg.name
        )
        .unwrap();
        if needs_into {
            writeln!(
                code,
                "        self.{name} = Some({name}.into());",
                name = arg.name
            )
            .unwrap();
        } else {
            writeln!(code, "        self.{name} = Some({name});", name = arg.name).unwrap();
        }
        writeln!(code, "        self").unwrap();
        writeln!(code, "    }}").unwrap();
    }

    writeln!(code, "}}").unwrap();
    writeln!(code).unwrap();

    // Display impl — interleaves literals and required args in their actual order.
    writeln!(code, "impl std::fmt::Display for {sname} {{").unwrap();
    writeln!(
        code,
        "    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{"
    )
    .unwrap();

    if !has_required && !has_optional {
        writeln!(code, "        write!(f, \"{cmd_str}\")").unwrap();
    } else {
        // Build a single format string that interleaves literals and {} placeholders.
        let mut fmt_str = String::new();
        let mut fmt_args = Vec::new();
        for (i, seg) in variant.full_path.iter().enumerate() {
            if i > 0 {
                fmt_str.push(' ');
            }
            match seg {
                PathSegment::Literal(s) => fmt_str.push_str(s),
                PathSegment::Arg(a) => {
                    fmt_str.push_str("{}");
                    fmt_args.push(format!("self.{}", a.name));
                }
            }
        }
        let args_joined = if fmt_args.is_empty() {
            String::new()
        } else {
            format!(", {}", fmt_args.join(", "))
        };
        writeln!(code, "        write!(f, \"{fmt_str}\"{args_joined})?;").unwrap();
        for arg in &variant.optional_args {
            writeln!(
                code,
                "        if let Some(v) = &self.{name} {{ write!(f, \" {{v}}\")?; }}",
                name = arg.name
            )
            .unwrap();
        }
        writeln!(code, "        Ok(())").unwrap();
    }

    writeln!(code, "    }}").unwrap();
    writeln!(code, "}}").unwrap();
    writeln!(code).unwrap();

    writeln!(code, "impl Command for {sname} {{}}").unwrap();
    writeln!(code).unwrap();

    let function_contract = contracts
        .iter()
        .find(|entry| entry.contract.kind == ApiKind::Function)
        .expect("command generator creates a function contract");
    write_contract_rustdoc(code, "", &function_contract.contract);

    if has_required {
        let mut params = Vec::new();
        let mut call_args = Vec::new();

        for arg in &required {
            let (param_ty, _stored_ty, _needs_into) = map_arg_parser(&literals, arg);
            params.push(format!("{}: {param_ty}", arg.name));
            call_args.push(arg.name.clone());
        }

        let params_str = params.join(", ");
        let call_args_str = call_args.join(", ");
        writeln!(code, "pub fn {fname}({params_str}) -> {sname} {{").unwrap();
        writeln!(code, "    {sname}::new({call_args_str})").unwrap();
    } else {
        writeln!(code, "pub fn {fname}() -> {sname} {{").unwrap();
        if has_optional {
            writeln!(code, "    {sname}::default()").unwrap();
        } else {
            // True unit struct; but we declared it with braces, so use Default.
            writeln!(code, "    {sname}::default()").unwrap();
        }
    }

    writeln!(code, "}}").unwrap();
    writeln!(code).unwrap();
}

fn argument_description(arg: &ArgInfo, usage: &str) -> String {
    match arg.parser.as_str() {
        "brigadier:bool" => format!("The boolean `<{}>` argument in `/{usage}`.", arg.name),
        "brigadier:integer" => format!("The integer `<{}>` argument in `/{usage}`.", arg.name),
        "brigadier:float" | "brigadier:double" => {
            format!("The numeric `<{}>` argument in `/{usage}`.", arg.name)
        }
        "minecraft:entity" | "minecraft:game_profile" => format!(
            "The typed entity or player selection for `<{}>` in `/{usage}`.",
            arg.name
        ),
        "minecraft:block_pos" | "minecraft:column_pos" => {
            format!(
                "The typed block position for `<{}>` in `/{usage}`.",
                arg.name
            )
        }
        "minecraft:resource_location"
        | "minecraft:dimension"
        | "minecraft:function"
        | "minecraft:loot_table"
        | "minecraft:loot_predicate"
        | "minecraft:loot_modifier" => format!(
            "The validated resource location for `<{}>` in `/{usage}`.",
            arg.name
        ),
        parser => format!(
            "The `<{}>` command-tree argument parsed by Minecraft as `{parser}` in `/{usage}`.",
            arg.name
        ),
    }
}

fn command_api_entries(
    ev: &EmittedVariant,
    minecraft_version: &str,
) -> Vec<GeneratedProviderEntry> {
    let variant = &ev.command;
    let literals = variant.literal_segments();
    let command_name = literals.first().copied().unwrap_or("command");
    let mut usage = String::new();
    for (index, segment) in variant.full_path.iter().enumerate() {
        if index > 0 {
            usage.push(' ');
        }
        match segment {
            PathSegment::Literal(literal) => usage.push_str(literal),
            PathSegment::Arg(arg) => write!(usage, "<{}>", arg.name).unwrap(),
        }
    }
    for arg in &variant.optional_args {
        write!(usage, " [<{}>]", arg.name).unwrap();
    }

    let required = variant.required_args();
    let parameters = required
        .iter()
        .map(|arg| {
            let (param_type, _, _) = map_arg_parser(&literals, arg);
            ApiParameter {
                name: arg.name.clone(),
                rust_type: Some(param_type.to_owned()),
                description: argument_description(arg, &usage),
            }
        })
        .collect::<Vec<_>>();
    let params_signature = required
        .iter()
        .map(|arg| {
            let (param_type, _, _) = map_arg_parser(&literals, arg);
            format!("{}: {param_type}", arg.name)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let call_args = required
        .iter()
        .map(|arg| arg.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let struct_path = format!("sand::command::{}", ev.struct_name);
    let function_path = format!("sand::command::{}", ev.fn_name);
    let availability = vec![format!("minecraft = {minecraft_version}")];
    let context = format!(
        "Sand generates this typed API from Minecraft {minecraft_version}'s command tree for `/{usage}`."
    );
    let minecraft = format!("Renders the exact Minecraft command syntax `/{usage}`.");
    let use_when = vec![format!(
        "Emitting Minecraft's `/{command_name}` command with typed arguments"
    )];
    let avoid_when = vec![format!(
        "The target Minecraft version does not support the `/{usage}` command shape"
    )];

    let implementation_struct = format!("sand_core::cmd::_generated::{}", ev.struct_name);
    let mut result = vec![GeneratedProviderEntry {
        definition_identity: implementation_struct.clone(),
        definition_kind: ApiKind::Struct,
        parent_identity: None,
        member_name: None,
        contract: ApiEntry {
            canonical_path: struct_path.clone(),
            aliases: vec![
                format!("sand::cmd::{}", ev.struct_name),
                format!("sand::prelude::cmd::{}", ev.struct_name),
            ],
            canonical_module: "sand::command".into(),
            kind: ApiKind::Struct,
            signature: format!("pub struct {} {{ /* private fields */ }}", ev.struct_name),
            summary: format!("Builds and renders Minecraft's `/{usage}` command form."),
            context: context.clone(),
            minecraft: minecraft.clone(),
            use_when: use_when.clone(),
            avoid_when: avoid_when.clone(),
            parameters: Vec::new(),
            returns: None,
            return_type: None,
            example: format!("let command = sand::command::{}({call_args});", ev.fn_name),
            availability: availability.clone(),
        },
    }];

    result.push(GeneratedProviderEntry {
        definition_identity: format!("sand_core::cmd::_generated::{}", ev.fn_name),
        definition_kind: ApiKind::Function,
        parent_identity: None,
        member_name: None,
        contract: ApiEntry {
            canonical_path: function_path,
            aliases: vec![
                format!("sand::cmd::{}", ev.fn_name),
                format!("sand::prelude::cmd::{}", ev.fn_name),
            ],
            canonical_module: "sand::command".into(),
            kind: ApiKind::Function,
            signature: format!(
                "pub fn {}({params_signature}) -> {}",
                ev.fn_name, ev.struct_name
            ),
            summary: format!("Starts a typed Minecraft `/{usage}` command."),
            context: context.clone(),
            minecraft: minecraft.clone(),
            use_when: use_when.clone(),
            avoid_when: avoid_when.clone(),
            parameters,
            returns: Some(format!(
                "A `{}` builder that renders `/{usage}`.",
                ev.struct_name
            )),
            return_type: Some(ev.struct_name.clone()),
            example: format!("let command = sand::command::{}({call_args});", ev.fn_name),
            availability: availability.clone(),
        },
    });

    result.extend(variant.optional_args.iter().map(|arg| {
        let (param_type, _, _) = map_arg_parser(&literals, arg);
        GeneratedProviderEntry {
            definition_identity: format!("{implementation_struct}::{}", arg.name),
            definition_kind: ApiKind::Method,
            parent_identity: Some(implementation_struct.clone()),
            member_name: Some(arg.name.clone()),
            contract: ApiEntry {
            canonical_path: format!("{struct_path}::{}", arg.name),
            aliases: vec![
                format!("sand::cmd::{}::{}", ev.struct_name, arg.name),
                format!("sand::prelude::cmd::{}::{}", ev.struct_name, arg.name),
            ],
            canonical_module: "sand::command".into(),
            kind: ApiKind::Method,
            signature: format!(
                "pub fn {name}(self, {name}: {param_type}) -> Self",
                name = arg.name
            ),
            summary: format!(
                "Sets the optional `<{}>` argument for Minecraft's `/{usage}` command.",
                arg.name
            ),
            context: context.clone(),
            minecraft: minecraft.clone(),
            use_when: vec![format!(
                "Supplying Minecraft's optional `<{}>` argument to `/{command_name}`",
                arg.name
            )],
            avoid_when: vec![format!(
                "Leaving the optional `<{}>` argument unset so Minecraft uses its default behavior",
                arg.name
            )],
            parameters: vec![ApiParameter {
                name: arg.name.clone(),
                rust_type: Some(param_type.to_owned()),
                description: argument_description(arg, &usage),
            }],
            returns: Some("The command builder with the optional argument set.".into()),
            return_type: Some("Self".into()),
            example: format!(
                "let command = sand::command::{}({call_args}).{}({});",
                ev.fn_name, arg.name, arg.name
            ),
                availability: availability.clone(),
            },
        }
    }));

    result
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Parse `commands.json` and write `commands.rs` to `out_dir`.
pub fn generate(reports_dir: &Path, out_dir: &Path, minecraft_version: &str) -> Result<()> {
    let path = reports_dir.join("commands.json");
    let content = std::fs::read_to_string(&path)?;
    let root: Value = serde_json::from_str(&content)?;

    let top_children = children(&root);

    let mut all_variants: Vec<CommandVariant> = Vec::new();

    for (cmd_name, cmd_node) in &top_children {
        // Skip redirect-based aliases.
        if SKIP_COMMANDS.contains(cmd_name) {
            continue;
        }
        // Skip if top-level node itself is a redirect.
        if has_redirect(cmd_node) {
            continue;
        }
        // Skip commands that contain redirects at any depth.
        if subtree_has_redirect(cmd_node) {
            continue;
        }

        let full_path = vec![PathSegment::Literal(cmd_name.to_string())];
        walk(cmd_node, &full_path, false, 1, &mut all_variants);
    }

    // Assign unique Rust identifiers without mutating vanilla command paths.
    let emitted = assign_rust_names(all_variants)
        .into_iter()
        .filter(|variant| !HANDWRITTEN_VARIANTS.contains(&variant.fn_name.as_str()))
        .collect::<Vec<_>>();

    // Generate code.
    let mut code = String::new();
    let mut api_entries = Vec::new();
    writeln!(code, "// Generated by sand-build. Do not edit manually.").unwrap();
    writeln!(code).unwrap();

    for ev in &emitted {
        let contracts = command_api_entries(ev, minecraft_version);
        emit_variant(&mut code, ev, &contracts);
        api_entries.extend(contracts);
    }

    let out_path = out_dir.join("commands.rs");
    std::fs::write(out_path, code)?;
    ApiProviderCatalog::new("generated_commands", minecraft_version, api_entries)
        .write_json(&out_dir.join("commands.api.json"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_field_name() {
        assert_eq!(sanitize_field_name("type"), "kind");
        assert_eq!(sanitize_field_name("in"), "in_dim");
        assert_eq!(sanitize_field_name("ban-ip"), "ban_ip");
        assert_eq!(sanitize_field_name("targets"), "targets");
    }

    #[test]
    fn test_struct_name() {
        assert_eq!(struct_name(&["give"]), "Give");
        assert_eq!(struct_name(&["effect", "give"]), "EffectGive");
        assert_eq!(struct_name(&["ban-ip"]), "BanIp");
    }

    #[test]
    fn test_fn_name() {
        assert_eq!(fn_name(&["give"]), "give");
        assert_eq!(fn_name(&["effect", "give"]), "effect_give");
        assert_eq!(fn_name(&["ban-ip"]), "ban_ip");
    }

    #[test]
    fn test_map_parser() {
        let (p, s, n) = map_parser("brigadier:integer");
        assert_eq!(p, "i32");
        assert_eq!(s, "i32");
        assert!(!n);

        let (p, _s, n) = map_parser("minecraft:entity");
        assert_eq!(p, "Selector");
        assert!(!n);

        let (p, s, n) = map_parser("minecraft:message");
        assert_eq!(p, "impl Into<String>");
        assert_eq!(s, "String");
        assert!(n);
    }

    #[test]
    fn codegen_simple() {
        let dir = tempfile::tempdir().unwrap();
        let reports = dir.path().join("reports");
        std::fs::create_dir_all(&reports).unwrap();

        let fixture = serde_json::json!({
            "type": "root",
            "children": {
                "say": {
                    "type": "literal",
                    "children": {
                        "message": {
                            "type": "argument",
                            "executable": true,
                            "parser": "minecraft:message"
                        }
                    }
                },
                "kill": {
                    "type": "literal",
                    "children": {
                        "targets": {
                            "type": "argument",
                            "executable": true,
                            "parser": "minecraft:entity"
                        }
                    },
                    "executable": true
                }
            }
        });

        std::fs::write(reports.join("commands.json"), fixture.to_string()).unwrap();

        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();
        generate(&reports, &out, "test-version").unwrap();

        let generated = std::fs::read_to_string(out.join("commands.rs")).unwrap();
        assert!(generated.contains("pub struct Kill"), "missing Kill struct");
        assert!(generated.contains("pub struct Say"), "missing Say struct");
        assert!(generated.contains("pub fn say("), "missing say fn");
        assert!(generated.contains("pub fn kill("), "missing kill fn");
        assert!(generated.contains("#[doc = \"# Minecraft behavior\"]"));
        assert!(generated.contains("#[doc = \"# Parameters\"]"));
        assert!(generated.contains("#[doc = \"# Returns\"]"));
        assert!(generated.contains("#[doc = \"# Use when\"]"));
        assert!(generated.contains("#[doc = \"# Avoid when\"]"));
        assert!(generated.contains("#[doc = \"# Example\"]"));
        assert!(!generated.contains("sand api show"));
        assert!(
            generated.contains("impl Command for Say"),
            "missing Command impl"
        );

        let provider = crate::read_api_provider(&out.join("commands.api.json")).unwrap();
        crate::validate_api_provider_source(
            &provider,
            &out.join("commands.rs"),
            "sand_core::cmd::_generated",
        )
        .unwrap();
        assert_eq!(provider.provider, "generated_commands");
        assert_eq!(provider.minecraft_version, "test-version");
        assert!(provider.entries.iter().any(|entry| {
            entry.definition_identity == "sand_core::cmd::_generated::say"
                && entry.contract.canonical_path == "sand::command::say"
                && entry.contract.minecraft.contains("/say <message>")
        }));

        let second_out = dir.path().join("out-second");
        std::fs::create_dir_all(&second_out).unwrap();
        generate(&reports, &second_out, "test-version").unwrap();
        assert_eq!(
            std::fs::read(out.join("commands.api.json")).unwrap(),
            std::fs::read(second_out.join("commands.api.json")).unwrap(),
            "provider metadata must be byte-for-byte deterministic"
        );
    }

    /// Two variants whose literal paths normalize to the same Rust symbol must get
    /// unique Rust identifiers, but the vanilla command literals in Display output
    /// and doc comments must stay pristine (no `_2` leaking in).
    #[test]
    fn dedup_does_not_mutate_command_literals() {
        // "foo-bar" and "foo_bar" both normalize to `FooBar` / `foo_bar` via heck.
        let v1 = CommandVariant {
            full_path: vec![PathSegment::Literal("foo-bar".to_string())],
            optional_args: vec![],
        };
        let v2 = CommandVariant {
            full_path: vec![PathSegment::Literal("foo_bar".to_string())],
            optional_args: vec![],
        };

        let emitted = assign_rust_names(vec![v1, v2]);

        // Both Rust names must be distinct.
        assert_ne!(
            emitted[0].struct_name, emitted[1].struct_name,
            "struct names must be unique after deduplication"
        );
        assert_ne!(
            emitted[0].fn_name, emitted[1].fn_name,
            "fn names must be unique after deduplication"
        );

        // Vanilla command literals must not be modified by the dedup pass.
        assert_eq!(
            emitted[0].command.literal_segments(),
            vec!["foo-bar"],
            "first variant literal must remain 'foo-bar'"
        );
        assert_eq!(
            emitted[1].command.literal_segments(),
            vec!["foo_bar"],
            "second variant literal must remain 'foo_bar'"
        );

        // Rust suffixes must not appear in the vanilla literal segments.
        for ev in &emitted {
            for seg in &ev.command.full_path {
                if let PathSegment::Literal(s) = seg {
                    assert!(
                        !s.contains("_2") && !s.contains("_3"),
                        "Rust dedup suffix leaked into command literal: {s}"
                    );
                }
            }
        }
    }

    /// Verify that the generated code for colliding variants keeps vanilla command
    /// strings clean in Display impls and doc comments.
    #[test]
    fn codegen_dedup_preserves_vanilla_literals() {
        let dir = tempfile::tempdir().unwrap();
        let reports = dir.path().join("reports");
        std::fs::create_dir_all(&reports).unwrap();

        // "foo-bar" and "foo_bar" normalize identically through heck, forcing a collision.
        let fixture = serde_json::json!({
            "type": "root",
            "children": {
                "foo-bar": {
                    "type": "literal",
                    "executable": true
                },
                "foo_bar": {
                    "type": "literal",
                    "executable": true
                }
            }
        });

        std::fs::write(reports.join("commands.json"), fixture.to_string()).unwrap();

        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();
        generate(&reports, &out, "test-version").unwrap();

        let generated = std::fs::read_to_string(out.join("commands.rs")).unwrap();

        // Both vanilla command strings must appear in the output unchanged.
        assert!(
            generated.contains("foo-bar") || generated.contains("foo_bar"),
            "at least one vanilla literal must appear in generated output"
        );

        // The suffix `_2` must not appear inside any command string context
        // (doc comment or Display write! call). Rust identifier `FooBar2` is fine,
        // but `foo-bar_2` or `foo_bar_2` in a string literal is the bug.
        assert!(
            !generated.contains("\"foo-bar_2\"") && !generated.contains("\"foo_bar_2\""),
            "dedup suffix leaked into a command string literal"
        );
        assert!(
            !generated.contains("// /foo-bar_2") && !generated.contains("// /foo_bar_2"),
            "dedup suffix leaked into a command doc comment"
        );
    }
}
