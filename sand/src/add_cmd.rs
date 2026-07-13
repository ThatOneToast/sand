//! `sand add` — add features to an existing Sand project.

use anyhow::{Context, Result, bail};
use colored::Colorize;

use crate::config::SandConfig;
use crate::scaffold::{WORKSPACE_ROOT, build_handlebars, write_rendered};
use sand_resourcepack::resource_pack_format_for;

const SAND_RESOURCE_EXPORT_RS_HBS: &str =
    include_str!("templates/default/sand_resource_export_rs.hbs");

// ── `sand add resourcepack` ───────────────────────────────────────────────────

/// Add resource pack support to an existing Sand project.
///
/// Idempotent — re-running on a project that already has resource pack support
/// prints a notice and exits without modifying any files.
pub fn run_resourcepack() -> Result<()> {
    // 1. Verify we're inside a Sand project.
    let config = load_config()?;
    let namespace = config.pack.namespace.as_str();
    let description = config
        .resourcepack
        .as_ref()
        .and_then(|r| r.description.as_deref())
        .unwrap_or(&config.pack.description)
        .to_string();

    println!(
        "{} resource pack support to {}...",
        "Adding".cyan().bold(),
        namespace.white().bold()
    );

    // 2. Guard: already configured?
    let cargo_toml_src = std::fs::read_to_string("Cargo.toml")
        .context("failed to read Cargo.toml — run this from your project root")?;
    if cargo_toml_src.contains("sand-resourcepack") {
        println!(
            "{} resource pack support is already present in this project.",
            "Note:".dimmed()
        );
        return Ok(());
    }

    // 3. Modify Cargo.toml.
    patch_cargo_toml(&cargo_toml_src, namespace)?;
    println!("  {} Cargo.toml", "updated".green());

    // 4. Modify sand.toml.
    let mc_version = &config.pack.mc_version;
    let rp_format = resource_pack_format_for(mc_version);
    patch_sand_toml(&description, rp_format)?;
    println!("  {} sand.toml", "updated".green());

    // 5. Create src/bin/sand_resource_export.rs (if not already present).
    let bin_path = std::path::PathBuf::from("src/bin/sand_resource_export.rs");
    if !bin_path.exists() {
        std::fs::create_dir_all("src/bin").context("failed to create src/bin")?;
        let hbs = build_handlebars();
        let ctx = serde_json::json!({
            "name_snake": namespace,
            "namespace":  namespace,
        });
        write_rendered(
            &hbs,
            "sand_resource_export_rs",
            SAND_RESOURCE_EXPORT_RS_HBS,
            &ctx,
            &bin_path,
        )?;
        println!("  {} src/bin/sand_resource_export.rs", "created".green());
    } else {
        println!(
            "  {} src/bin/sand_resource_export.rs already exists",
            "skipped".dimmed()
        );
    }

    // 6. Append __sand_resource_export to src/lib.rs (if not already present).
    patch_lib_rs(namespace)?;
    println!("  {} src/lib.rs", "updated".green());

    // 7. Create src/assets/ placeholder.
    if !std::path::Path::new("src/assets").exists() {
        std::fs::create_dir_all("src/assets").context("failed to create src/assets")?;
        println!("  {} src/assets/", "created".green());
    }

    println!();
    println!("{}", "Done! Next steps:".green().bold());
    println!(
        "  1. Add HUD elements and textures to {} using the macros:",
        "src/lib.rs".white().bold()
    );
    println!("       hud_bar!(name: \"health\", texture: \"src/assets/health_bar.png\", ...)");
    println!("       hud_element!(name: \"icon\", texture: \"src/assets/icon.png\", ...)");
    println!(
        "       texture!(id: \"{ns}:item/my_item\", path: \"src/assets/my_item.png\")",
        ns = namespace
    );
    println!();
    println!(
        "  2. Put your PNG assets in {}",
        "src/assets/".white().bold()
    );
    println!();
    println!(
        "  3. Run {} to build both packs",
        "`sand build --resourcepack`".white().bold()
    );

    Ok(())
}

// ── Patching helpers ──────────────────────────────────────────────────────────

/// Extract the `(git_url, branch)` pair from any `sand-* = { git = "...",
/// branch = "...", ... }` line in a `Cargo.toml` string, so a newly-added
/// dep tracks the same repo/branch as the project's existing sand-* deps.
fn extract_sand_git(cargo_toml: &str) -> Option<(String, String)> {
    for line in cargo_toml.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("sand-") || !trimmed.contains("git =") {
            continue;
        }
        let git_url = extract_quoted_value(trimmed, "git = \"")?;
        let branch =
            extract_quoted_value(trimmed, "branch = \"").unwrap_or_else(|| "main".to_string());
        return Some((git_url, branch));
    }
    None
}

/// Extract the version string from any legacy `sand-* = "X.Y.Z"` or
/// `sand-* = { version = "X.Y.Z", ... }` line in a `Cargo.toml` string.
/// Only present in projects scaffolded before Sand switched its default
/// scaffold to git dependencies (Sand has never been published to
/// crates.io, so a versioned dep here means an older, already-broken
/// project — `sand add` matches its existing style rather than silently
/// rewriting it).
fn extract_sand_version(cargo_toml: &str) -> Option<String> {
    for line in cargo_toml.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("sand-") || trimmed.contains("git =") || trimmed.contains("path =")
        {
            continue;
        }
        // Simple form: sand-foo = "1.2.3"
        if let Some(eq) = trimmed.find('=') {
            let rhs = trimmed[eq + 1..].trim();
            if rhs.starts_with('"') && rhs.ends_with('"') {
                return Some(rhs[1..rhs.len() - 1].to_string());
            }
            // Inline table form: sand-foo = { version = "1.2.3", ... }
            if let Some(version) = extract_quoted_value(rhs, "version = \"") {
                return Some(version);
            }
        }
    }
    None
}

/// Find `prefix` in `s` and return the quoted value that follows it.
fn extract_quoted_value(s: &str, prefix: &str) -> Option<String> {
    let start = s.find(prefix)? + prefix.len();
    let after = &s[start..];
    let end = after.find('"')?;
    Some(after[..end].to_string())
}

/// Add `sand-resourcepack` dep, `features = ["resourcepack"]` on sand-macros,
/// and a `[[bin]] sand_resource_export` target to `Cargo.toml`.
fn patch_cargo_toml(original: &str, namespace: &str) -> Result<()> {
    let _ = namespace; // reserved for future namespace-aware path derivation

    // Match the dependency style already used by this project's sand-* deps:
    // path (contributor workspace), git (the default since Sand has no
    // crates.io release), or a legacy version string (pre-existing projects
    // scaffolded before the git-dep default). Falls back to a git dep
    // against this CLI's own repo/main if no sand-* dep line is found.
    let uses_path_deps = original
        .lines()
        .any(|l| l.trim_start().starts_with("sand-") && l.contains("path ="));

    let sand_resourcepack_dep = if uses_path_deps {
        let path = format!("{}/sand-resourcepack", WORKSPACE_ROOT);
        format!("sand-resourcepack = {{ path = \"{path}\" }}")
    } else if let Some((git_url, branch)) = extract_sand_git(original) {
        format!("sand-resourcepack = {{ git = \"{git_url}\", branch = \"{branch}\" }}")
    } else if let Some(version) = extract_sand_version(original) {
        // Legacy versioned dep found — match it rather than mixing styles.
        format!("sand-resourcepack = \"{version}\"")
    } else {
        let git_url = env!("CARGO_PKG_REPOSITORY");
        format!("sand-resourcepack = {{ git = \"{git_url}\", branch = \"main\" }}")
    };

    let mut lines: Vec<String> = original.lines().map(String::from).collect();

    // 1. Modify the sand-macros line to add `features = ["resourcepack"]`.
    //    Handles both dep forms:
    //      sand-macros = { path = "..." }        → append before closing }
    //      sand-macros = "0.1.0"                 → expand to inline table form
    let mut modified_macros = false;
    for line in &mut lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("sand-macros") && !line.contains("resourcepack") {
            if let Some(idx) = line.rfind('}') {
                // Inline table form: insert before closing `}`
                line.insert_str(idx, ", features = [\"resourcepack\"]");
            } else if let Some(eq) = line.find('=') {
                // Simple string form: `sand-macros = "0.1.0"` → expand
                let rhs = line[eq + 1..].trim();
                if rhs.starts_with('"') && rhs.ends_with('"') {
                    let ver = &rhs[1..rhs.len() - 1];
                    let lhs = &line[..eq];
                    *line =
                        format!("{lhs}= {{ version = \"{ver}\", features = [\"resourcepack\"] }}");
                }
            }
            modified_macros = true;
            break;
        }
    }
    if !modified_macros {
        // sand-macros not found — unusual, but don't abort. Warn instead.
        eprintln!(
            "sand: warning: could not find `sand-macros` in Cargo.toml; \
             add `features = [\"resourcepack\"]` manually."
        );
    }

    // 2. Append sand-resourcepack after the sand-macros line (or at end of deps).
    //    Find the [dependencies] section and append there.
    let dep_line = sand_resourcepack_dep;
    let mut inserted_dep = false;
    let mut result: Vec<String> = Vec::with_capacity(lines.len() + 2);
    let mut in_deps = false;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed == "[dependencies]" {
            in_deps = true;
        } else if trimmed.starts_with('[') && trimmed != "[dependencies]" {
            // Entering a new section — insert dep now if we were in [dependencies].
            if in_deps && !inserted_dep {
                result.push(dep_line.clone());
                inserted_dep = true;
            }
            in_deps = false;
        }
        result.push(line.clone());
    }
    // If [dependencies] was the last section.
    if in_deps && !inserted_dep {
        result.push(dep_line.clone());
    }

    // 3. Append [[bin]] section for sand_resource_export at the end.
    result.push(String::new());
    result.push("[[bin]]".to_string());
    result.push("name = \"sand_resource_export\"".to_string());
    result.push("path = \"src/bin/sand_resource_export.rs\"".to_string());

    let new_content = result.join("\n") + "\n";
    std::fs::write("Cargo.toml", new_content).context("failed to write Cargo.toml")?;
    Ok(())
}

/// Append a `[resourcepack]` section to `sand.toml` if one isn't already there.
fn patch_sand_toml(description: &str, rp_format: u32) -> Result<()> {
    let original = std::fs::read_to_string("sand.toml").context("failed to read sand.toml")?;

    if original.contains("[resourcepack]") {
        return Ok(());
    }

    let addition = format!(
        "\n[resourcepack]\ndescription = \"{description}\"\n\
         # namespace defaults to [pack].namespace; uncomment to override:\n\
         # namespace = \"\"\n\
         # resource_pack_format is derived automatically; uncomment to override:\n\
         # resource_pack_format = {rp_format}\n"
    );

    let new_content = original.trim_end().to_string() + "\n" + &addition;
    std::fs::write("sand.toml", new_content).context("failed to write sand.toml")?;
    Ok(())
}

/// Append the `__sand_resource_export` function to `src/lib.rs` if it isn't
/// already present.
fn patch_lib_rs(namespace: &str) -> Result<()> {
    let _ = namespace;
    let lib_path = "src/lib.rs";
    let original = std::fs::read_to_string(lib_path).context("failed to read src/lib.rs")?;

    if original.contains("__sand_resource_export") {
        // Already present (either active or commented). Try to uncomment if
        // the function body is commented out.
        if original.contains("pub fn __sand_resource_export") {
            // Already active — nothing to do.
            return Ok(());
        }
        // Commented — uncomment the block.
        let uncommented = uncomment_resource_export_hook(&original);
        std::fs::write(lib_path, uncommented).context("failed to write src/lib.rs")?;
        return Ok(());
    }

    // Not present at all — append the active function.
    let addition = concat!(
        "\n",
        "// ── Resource pack export hook ─────────────────────────────────────────────\n",
        "\n",
        "#[doc(hidden)]\n",
        "pub fn __sand_resource_export(namespace: &str) {\n",
        "    println!(\"{}\", sand_resourcepack::export_resourcepack_json(namespace));\n",
        "}\n",
    );

    // Also prepend the macro imports to the use statement if we can find it.
    let patched = add_rp_imports(&original) + addition;
    std::fs::write(lib_path, patched).context("failed to write src/lib.rs")?;
    Ok(())
}

/// Attempt to add `hud_bar, hud_element, texture` to an existing
/// `use sand_macros::{...}` statement. Returns the original string unchanged
/// if the pattern is not found or the imports are already present.
fn add_rp_imports(src: &str) -> String {
    // Find `use sand_macros::{...};` and add the RP macros if not present.
    if src.contains("hud_bar") {
        return src.to_string();
    }
    if let Some(idx) = src.find("use sand_macros::{")
        && let Some(end) = src[idx..].find("};")
    {
        let insert_at = idx + end;
        let mut result = src.to_string();
        result.insert_str(insert_at, ", hud_bar, hud_element, texture");
        return result;
    }
    src.to_string()
}

/// Uncomment the `__sand_resource_export` block that the scaffold template
/// emits as a comment in the non-resourcepack variant of src/lib.rs.
///
/// Looks for the pattern:
/// ```text
/// // #[doc(hidden)]
/// // pub fn __sand_resource_export(namespace: &str) {
/// //     println!("{}", sand_resourcepack::export_resourcepack_json(namespace));
/// // }
/// ```
/// and strips the leading `// ` prefix from each line.
fn uncomment_resource_export_hook(src: &str) -> String {
    let marker = "// pub fn __sand_resource_export";
    if !src.contains(marker) {
        return src.to_string();
    }

    let mut result = String::with_capacity(src.len());
    let mut lines = src.lines().peekable();

    while let Some(line) = lines.next() {
        // Find the comment line immediately before the function definition.
        if line.trim() == "// #[doc(hidden)]" {
            // Peek ahead: is the next line the function start?
            if lines
                .peek()
                .map(|l| l.trim().starts_with(marker))
                .unwrap_or(false)
            {
                // Uncomment this line and all subsequent comment lines until
                // we hit the closing `// }`.
                result.push_str(&uncomment_line(line));
                result.push('\n');
                for inner in lines.by_ref() {
                    result.push_str(&uncomment_line(inner));
                    result.push('\n');
                    // Stop after the closing brace line.
                    if inner.trim() == "// }" {
                        break;
                    }
                }
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    result
}

fn uncomment_line(line: &str) -> String {
    if let Some(rest) = line.strip_prefix("// ") {
        rest.to_string()
    } else if line.trim() == "//" {
        String::new()
    } else {
        line.to_string()
    }
}

/// Load and parse `sand.toml` from the current directory.
fn load_config() -> Result<SandConfig> {
    let path = "sand.toml";
    if !std::path::Path::new(path).exists() {
        bail!("sand.toml not found — run `sand add resourcepack` from your project root");
    }
    toml::from_str(&std::fs::read_to_string(path)?).context("failed to parse sand.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIT_DEP_CARGO_TOML: &str = concat!(
        "[package]\n",
        "name = \"my_pack\"\n",
        "version = \"0.1.0\"\n",
        "\n",
        "[dependencies]\n",
        "sand-core   = { git = \"https://github.com/ThatOneToast/sand\", branch = \"main\" }\n",
        "sand-macros = { git = \"https://github.com/ThatOneToast/sand\", branch = \"main\" }\n",
        "serde_json = \"1\"\n",
    );

    const PATH_DEP_CARGO_TOML: &str = concat!(
        "[package]\n",
        "name = \"my_pack\"\n",
        "version = \"0.1.0\"\n",
        "\n",
        "[dependencies]\n",
        "sand-core   = { path = \"/workspace/sand-core\" }\n",
        "sand-macros = { path = \"/workspace/sand-macros\" }\n",
    );

    const LEGACY_VERSION_CARGO_TOML: &str = concat!(
        "[package]\n",
        "name = \"my_pack\"\n",
        "version = \"0.1.0\"\n",
        "\n",
        "[dependencies]\n",
        "sand-core = \"0.1.0\"\n",
        "sand-macros = \"0.1.0\"\n",
    );

    #[test]
    fn extract_sand_git_reads_url_and_branch() {
        let (url, branch) = extract_sand_git(GIT_DEP_CARGO_TOML).unwrap();
        assert_eq!(url, "https://github.com/ThatOneToast/sand");
        assert_eq!(branch, "main");
    }

    #[test]
    fn extract_sand_git_returns_none_for_path_deps() {
        assert!(extract_sand_git(PATH_DEP_CARGO_TOML).is_none());
    }

    #[test]
    fn extract_sand_version_ignores_git_and_path_deps() {
        assert!(extract_sand_version(GIT_DEP_CARGO_TOML).is_none());
        assert!(extract_sand_version(PATH_DEP_CARGO_TOML).is_none());
        assert_eq!(
            extract_sand_version(LEGACY_VERSION_CARGO_TOML),
            Some("0.1.0".to_string())
        );
    }

    // patch_cargo_toml writes directly to "Cargo.toml" in the process's
    // current directory, so it isn't exercised here to avoid clobbering the
    // real file under test; the dependency-style selection it relies on is
    // covered by the extract_sand_git/extract_sand_version tests above.
}
