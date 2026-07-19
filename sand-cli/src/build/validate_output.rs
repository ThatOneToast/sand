use std::path::Path;

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

use super::validate::validate_function_tag;

/// Validates the on-disk layout and content of a generated Sand datapack directory.
///
/// Checks:
/// - `pack.mcmeta` is present and has the required shape (`pack.pack_format`).
/// - The `data/` directory exists and contains at least one file.
/// - Every `.json` file under `data/` parses as valid JSON.
/// - Function tag files under `data/<ns>/tags/function/` have a valid `values` array.
/// - Every `.mcfunction` file contains no null bytes and no `/`-prefixed commands.
///
/// Each error message identifies the failing file path so failures are easy to pinpoint.
pub fn validate_output_dir(pack_dir: &Path) -> Result<()> {
    validate_pack_mcmeta(pack_dir)?;
    validate_data_dir(pack_dir)?;
    Ok(())
}

fn validate_pack_mcmeta(pack_dir: &Path) -> Result<()> {
    let mcmeta_path = pack_dir.join("pack.mcmeta");
    let content = std::fs::read_to_string(&mcmeta_path).with_context(|| {
        format!(
            "pack.mcmeta missing or unreadable: '{}'",
            mcmeta_path.display()
        )
    })?;

    let v: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("pack.mcmeta is not valid JSON: '{}'", mcmeta_path.display()))?;

    let pack = v.get("pack").ok_or_else(|| {
        anyhow::anyhow!(
            "pack.mcmeta at '{}' is missing the required 'pack' key",
            mcmeta_path.display()
        )
    })?;

    if pack.get("pack_format").and_then(|v| v.as_u64()).is_none() {
        bail!(
            "pack.mcmeta at '{}' is missing or has an invalid 'pack.pack_format'",
            mcmeta_path.display()
        );
    }

    Ok(())
}

fn validate_data_dir(pack_dir: &Path) -> Result<()> {
    let data_dir = pack_dir.join("data");
    if !data_dir.is_dir() {
        bail!(
            "generated datapack at '{}' is missing the 'data/' directory",
            pack_dir.display()
        );
    }

    let mut file_count = 0usize;

    for entry in WalkDir::new(&data_dir) {
        let entry = entry.with_context(|| format!("error walking '{}'", data_dir.display()))?;

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "json" => {
                let content = std::fs::read_to_string(path)
                    .with_context(|| format!("failed to read '{}'", path.display()))?;
                if is_function_tag_path(&data_dir, path) {
                    let tag_name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("<unknown>");
                    validate_function_tag(tag_name, &content)
                        .with_context(|| format!("invalid function tag at '{}'", path.display()))?;
                } else {
                    serde_json::from_str::<serde_json::Value>(&content)
                        .with_context(|| format!("invalid JSON in '{}'", path.display()))?;
                }
            }
            "mcfunction" => validate_mcfunction_file(path)?,
            _ => {}
        }

        file_count += 1;
    }

    if file_count == 0 {
        bail!(
            "generated datapack at '{}' has no files under data/",
            pack_dir.display()
        );
    }

    Ok(())
}

fn validate_mcfunction_file(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read '{}'", path.display()))?;

    if content.contains('\0') {
        bail!(
            "'{}': contains a null byte — not a valid .mcfunction file",
            path.display()
        );
    }

    for (line_no, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Modern .mcfunction files must not use a leading '/' — that syntax is
        // from chat commands, not function files. A bare '/' is also invalid.
        if trimmed == "/" || trimmed.starts_with("/ ") || trimmed.starts_with('/') {
            bail!(
                "'{}':{}: command starts with '/' — .mcfunction files must not use the '/' prefix",
                path.display(),
                line_no + 1
            );
        }
    }

    Ok(())
}

/// Returns `true` if `path` is under `data/<namespace>/tags/function/`.
fn is_function_tag_path(data_dir: &Path, path: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(data_dir) else {
        return false;
    };
    // rel components: [<namespace>, "tags", "function", ..., name.json]
    // Require at least 4 components so there is a filename after "function/".
    let mut components = rel.components();
    let _namespace = components.next(); // <namespace>
    let tags = components
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("");
    let function = components
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("");
    let has_filename = components.next().is_some();
    tags == "tags" && function == "function" && has_filename
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_minimal_pack(dir: &Path) {
        let data = dir.join("data/golden/function");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(
            dir.join("pack.mcmeta"),
            r#"{"pack":{"pack_format":71,"description":"test"}}"#,
        )
        .unwrap();
        std::fs::write(data.join("load.mcfunction"), "say loaded\n").unwrap();
    }

    // ── pack.mcmeta ───────────────────────────────────────────────────────────

    #[test]
    fn rejects_missing_pack_mcmeta() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        std::fs::create_dir_all(pack.join("data/ns/function")).unwrap();
        std::fs::write(pack.join("data/ns/function/load.mcfunction"), "say hi").unwrap();
        let err = validate_output_dir(&pack).unwrap_err();
        assert!(
            err.to_string().contains("pack.mcmeta"),
            "error should mention pack.mcmeta: {err}"
        );
    }

    #[test]
    fn rejects_pack_mcmeta_invalid_json() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        std::fs::create_dir_all(pack.join("data/ns/function")).unwrap();
        std::fs::write(pack.join("pack.mcmeta"), "{bad json").unwrap();
        std::fs::write(pack.join("data/ns/function/load.mcfunction"), "say hi").unwrap();
        let err = validate_output_dir(&pack).unwrap_err();
        assert!(
            err.to_string().contains("pack.mcmeta"),
            "error should mention pack.mcmeta: {err}"
        );
    }

    #[test]
    fn rejects_pack_mcmeta_missing_pack_key() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        std::fs::create_dir_all(pack.join("data/ns/function")).unwrap();
        std::fs::write(pack.join("pack.mcmeta"), r#"{"not_pack":{}}"#).unwrap();
        std::fs::write(pack.join("data/ns/function/load.mcfunction"), "say hi").unwrap();
        let err = validate_output_dir(&pack).unwrap_err();
        assert!(
            err.to_string().contains("'pack'"),
            "error should mention 'pack' key: {err}"
        );
    }

    #[test]
    fn rejects_pack_mcmeta_missing_pack_format() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        std::fs::create_dir_all(pack.join("data/ns/function")).unwrap();
        std::fs::write(
            pack.join("pack.mcmeta"),
            r#"{"pack":{"description":"no format here"}}"#,
        )
        .unwrap();
        std::fs::write(pack.join("data/ns/function/load.mcfunction"), "say hi").unwrap();
        let err = validate_output_dir(&pack).unwrap_err();
        assert!(
            err.to_string().contains("pack_format"),
            "error should mention pack_format: {err}"
        );
    }

    #[test]
    fn accepts_valid_pack_mcmeta() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        setup_minimal_pack(&pack);
        assert!(
            validate_output_dir(&pack).is_ok(),
            "valid minimal pack should pass"
        );
    }

    // ── data/ directory ───────────────────────────────────────────────────────

    #[test]
    fn rejects_missing_data_dir() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        std::fs::create_dir_all(&pack).unwrap();
        std::fs::write(
            pack.join("pack.mcmeta"),
            r#"{"pack":{"pack_format":71,"description":"test"}}"#,
        )
        .unwrap();
        let err = validate_output_dir(&pack).unwrap_err();
        assert!(
            err.to_string().contains("data/"),
            "error should mention data/: {err}"
        );
    }

    #[test]
    fn rejects_empty_data_dir() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        std::fs::create_dir_all(pack.join("data")).unwrap();
        std::fs::write(
            pack.join("pack.mcmeta"),
            r#"{"pack":{"pack_format":71,"description":"test"}}"#,
        )
        .unwrap();
        let err = validate_output_dir(&pack).unwrap_err();
        assert!(
            err.to_string().contains("no files"),
            "error should mention empty output: {err}"
        );
    }

    // ── JSON file validation ──────────────────────────────────────────────────

    #[test]
    fn rejects_invalid_json_file_with_path_in_error() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        let recipe_dir = pack.join("data/mypack/recipe");
        std::fs::create_dir_all(&recipe_dir).unwrap();
        std::fs::write(
            pack.join("pack.mcmeta"),
            r#"{"pack":{"pack_format":71,"description":"test"}}"#,
        )
        .unwrap();
        std::fs::write(recipe_dir.join("broken.json"), "{not valid json").unwrap();

        let err = validate_output_dir(&pack).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("broken.json"),
            "error must identify the failing file: {msg}"
        );
    }

    #[test]
    fn accepts_valid_json_files() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        setup_minimal_pack(&pack);
        let recipe_dir = pack.join("data/golden/recipe");
        std::fs::create_dir_all(&recipe_dir).unwrap();
        std::fs::write(
            recipe_dir.join("crafting.json"),
            r#"{"type":"minecraft:crafting_shaped"}"#,
        )
        .unwrap();
        assert!(validate_output_dir(&pack).is_ok());
    }

    // ── Function tag validation ───────────────────────────────────────────────

    #[test]
    fn validates_function_tags_under_minecraft_namespace() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        let fn_dir = pack.join("data/golden/function");
        let tag_dir = pack.join("data/minecraft/tags/function");
        std::fs::create_dir_all(&fn_dir).unwrap();
        std::fs::create_dir_all(&tag_dir).unwrap();
        std::fs::write(
            pack.join("pack.mcmeta"),
            r#"{"pack":{"pack_format":71,"description":"test"}}"#,
        )
        .unwrap();
        std::fs::write(fn_dir.join("load.mcfunction"), "say loaded").unwrap();
        std::fs::write(tag_dir.join("load.json"), r#"{"values":["golden:load"]}"#).unwrap();
        assert!(validate_output_dir(&pack).is_ok());
    }

    #[test]
    fn rejects_malformed_function_tag_with_path_in_error() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        let fn_dir = pack.join("data/golden/function");
        let tag_dir = pack.join("data/minecraft/tags/function");
        std::fs::create_dir_all(&fn_dir).unwrap();
        std::fs::create_dir_all(&tag_dir).unwrap();
        std::fs::write(
            pack.join("pack.mcmeta"),
            r#"{"pack":{"pack_format":71,"description":"test"}}"#,
        )
        .unwrap();
        std::fs::write(fn_dir.join("load.mcfunction"), "say loaded").unwrap();
        // Missing "values" key — structurally invalid tag
        std::fs::write(tag_dir.join("load.json"), r#"{"entries":[]}"#).unwrap();

        let err = validate_output_dir(&pack).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("load.json") || msg.contains("tags/function"),
            "error must identify the failing function tag: {msg}"
        );
    }

    #[test]
    fn rejects_function_tag_with_invalid_resource_location() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        let fn_dir = pack.join("data/golden/function");
        let tag_dir = pack.join("data/minecraft/tags/function");
        std::fs::create_dir_all(&fn_dir).unwrap();
        std::fs::create_dir_all(&tag_dir).unwrap();
        std::fs::write(
            pack.join("pack.mcmeta"),
            r#"{"pack":{"pack_format":71,"description":"test"}}"#,
        )
        .unwrap();
        std::fs::write(fn_dir.join("load.mcfunction"), "say loaded").unwrap();
        // Uppercase namespace is not a valid resource location
        std::fs::write(
            tag_dir.join("load.json"),
            r#"{"values":["BadNamespace:load"]}"#,
        )
        .unwrap();

        assert!(validate_output_dir(&pack).is_err());
    }

    // ── .mcfunction file validation ───────────────────────────────────────────

    #[test]
    fn rejects_mcfunction_with_null_byte() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        setup_minimal_pack(&pack);
        let fn_dir = pack.join("data/golden/function");
        std::fs::write(fn_dir.join("bad.mcfunction"), "say hi\0").unwrap();

        let err = validate_output_dir(&pack).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("bad.mcfunction"),
            "error must identify the failing file: {msg}"
        );
        assert!(
            msg.contains("null byte"),
            "error should mention null byte: {msg}"
        );
    }

    #[test]
    fn rejects_mcfunction_with_slash_prefix() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        setup_minimal_pack(&pack);
        let fn_dir = pack.join("data/golden/function");
        std::fs::write(fn_dir.join("slash.mcfunction"), "/say hi\n").unwrap();

        let err = validate_output_dir(&pack).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("slash.mcfunction"),
            "error must identify the failing file: {msg}"
        );
        assert!(
            msg.contains('/'),
            "error should mention the '/' prefix: {msg}"
        );
    }

    #[test]
    fn accepts_mcfunction_with_comments_and_blank_lines() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        setup_minimal_pack(&pack);
        let fn_dir = pack.join("data/golden/function");
        std::fs::write(
            fn_dir.join("clean.mcfunction"),
            "# This is a comment\n\nsay hello\nexecute as @a run say hi\n",
        )
        .unwrap();
        assert!(validate_output_dir(&pack).is_ok());
    }

    // ── Object-form tag ref regression ───────────────────────────────────────

    #[test]
    fn accepts_object_form_tag_ref_in_function_tag() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        let fn_dir = pack.join("data/golden/function");
        let tag_dir = pack.join("data/minecraft/tags/function");
        std::fs::create_dir_all(&fn_dir).unwrap();
        std::fs::create_dir_all(&tag_dir).unwrap();
        std::fs::write(
            pack.join("pack.mcmeta"),
            r#"{"pack":{"pack_format":71,"description":"test"}}"#,
        )
        .unwrap();
        std::fs::write(fn_dir.join("load.mcfunction"), "say loaded").unwrap();
        // Object-form optional tag ref — must be accepted
        std::fs::write(
            tag_dir.join("load.json"),
            r##"{"values":[{"id":"#other_pack:startup","required":false},{"id":"golden:load","required":true}]}"##,
        )
        .unwrap();
        assert!(
            validate_output_dir(&pack).is_ok(),
            "object-form tag ref must be accepted"
        );
    }

    // ── Golden end-to-end fixture ─────────────────────────────────────────────

    #[test]
    fn golden_fixture_full_pack_validates_cleanly() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("myfirstpack");

        let fn_dir = pack.join("data/myfirstpack/function");
        let tag_dir = pack.join("data/minecraft/tags/function");
        let recipe_dir = pack.join("data/myfirstpack/recipe");

        for d in [&fn_dir, &tag_dir, &recipe_dir] {
            std::fs::create_dir_all(d).unwrap();
        }

        std::fs::write(
            pack.join("pack.mcmeta"),
            r#"{"pack":{"pack_format":71,"description":"My First Pack"}}"#,
        )
        .unwrap();
        std::fs::write(fn_dir.join("load.mcfunction"), "say loaded\n").unwrap();
        std::fs::write(
            fn_dir.join("tick.mcfunction"),
            "# runs every tick\nsay tick\n",
        )
        .unwrap();
        std::fs::write(
            tag_dir.join("load.json"),
            r#"{"values":["myfirstpack:load"]}"#,
        )
        .unwrap();
        std::fs::write(
            tag_dir.join("tick.json"),
            r#"{"values":["myfirstpack:tick"]}"#,
        )
        .unwrap();
        std::fs::write(
            recipe_dir.join("my_item.json"),
            r#"{"type":"minecraft:crafting_shaped"}"#,
        )
        .unwrap();

        assert!(
            validate_output_dir(&pack).is_ok(),
            "complete golden fixture must validate cleanly"
        );
    }
}
