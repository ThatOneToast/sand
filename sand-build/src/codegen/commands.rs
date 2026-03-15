use std::fmt::Write as FmtWrite;
use std::path::Path;

use heck::{ToPascalCase, ToSnakeCase};
use serde_json::Value;

use crate::error::Result;

/// Top-level commands to skip entirely (they use redirects or are aliases).
const SKIP_COMMANDS: &[&str] = &["execute", "tell", "tm", "tp", "w", "xp"];

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

fn emit_variant(code: &mut String, variant: &CommandVariant) {
    let literals = variant.literal_segments();
    let required = variant.required_args();
    let sname = struct_name(&literals);
    let fname = fn_name(&literals);
    // Command string for docs: literals only (args shown separately below).
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
    writeln!(code, "/// `{usage}`").unwrap();

    if !has_required {
        writeln!(code, "#[derive(Debug, Clone, Default)]").unwrap();
    } else {
        writeln!(code, "#[derive(Debug, Clone)]").unwrap();
    }

    writeln!(code, "pub struct {sname} {{").unwrap();
    for arg in &required {
        let (_param_ty, stored_ty, _needs_into) = map_parser(&arg.parser);
        writeln!(code, "    {}: {stored_ty},", arg.name).unwrap();
    }
    for arg in &variant.optional_args {
        let (_param_ty, stored_ty, _needs_into) = map_parser(&arg.parser);
        writeln!(code, "    {}: Option<{stored_ty}>,", arg.name).unwrap();
    }
    writeln!(code, "}}").unwrap();
    writeln!(code).unwrap();

    writeln!(code, "impl {sname} {{").unwrap();

    if has_required {
        let mut params = Vec::new();
        let mut body_lines = Vec::new();
        for arg in &required {
            let (param_ty, _stored_ty, needs_into) = map_parser(&arg.parser);
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
        let (param_ty, _stored_ty, needs_into) = map_parser(&arg.parser);
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

    writeln!(code, "/// Build a `{cmd_str}` command.").unwrap();

    if has_required {
        let mut params = Vec::new();
        let mut call_args = Vec::new();

        for arg in &required {
            let (param_ty, _stored_ty, _needs_into) = map_parser(&arg.parser);
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

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Parse `commands.json` and write `commands.rs` to `out_dir`.
pub fn generate(reports_dir: &Path, out_dir: &Path) -> Result<()> {
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

    // Deduplicate by struct name and fn name using indexed suffixes.
    let mut seen_structs: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut seen_fns: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for variant in &mut all_variants {
        let literals = variant.literal_segments();
        let sn = struct_name(&literals);
        let fn_n = fn_name(&literals);

        let struct_idx = seen_structs.entry(sn).or_insert(0);
        let fn_idx = seen_fns.entry(fn_n).or_insert(0);

        let idx = (*struct_idx).max(*fn_idx);
        if idx > 0 {
            // Append suffix to the last literal segment in full_path.
            for seg in variant.full_path.iter_mut().rev() {
                if let PathSegment::Literal(s) = seg {
                    s.push_str(&format!("_{}", idx + 1));
                    break;
                }
            }
        }
        *struct_idx += 1;
        *fn_idx += 1;
    }

    // Generate code.
    let mut code = String::new();
    writeln!(code, "// Generated by sand-build. Do not edit manually.").unwrap();
    writeln!(code).unwrap();

    for variant in &all_variants {
        emit_variant(&mut code, variant);
    }

    let out_path = out_dir.join("commands.rs");
    std::fs::write(out_path, code)?;
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
        generate(&reports, &out).unwrap();

        let generated = std::fs::read_to_string(out.join("commands.rs")).unwrap();
        assert!(generated.contains("pub struct Kill"), "missing Kill struct");
        assert!(generated.contains("pub struct Say"), "missing Say struct");
        assert!(generated.contains("pub fn say("), "missing say fn");
        assert!(generated.contains("pub fn kill("), "missing kill fn");
        assert!(
            generated.contains("impl Command for Say"),
            "missing Command impl"
        );
    }
}
