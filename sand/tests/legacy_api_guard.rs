use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("sand crate has a workspace parent")
        .to_path_buf()
}

fn rust_and_active_docs(root: &Path) -> Vec<PathBuf> {
    let output = std::process::Command::new("rg")
        .args(["--files", "-g", "*.rs", "-g", "*.md"])
        .current_dir(root)
        .output()
        .expect("rg must be available for repository guards");
    assert!(output.status.success(), "rg --files failed");
    String::from_utf8(output.stdout)
        .expect("paths are UTF-8")
        .lines()
        .map(|path| root.join(path))
        .filter(|path| !path.ends_with("CHANGELOG.md"))
        .filter(|path| !path.to_string_lossy().contains("/target/"))
        .collect()
}

#[test]
fn removed_declaration_macros_do_not_return() {
    let root = workspace_root();
    let removed = [
        ["sand", "_state", "!"].concat(),
        ["temp", "_score", "!"].concat(),
    ];
    let mut violations = Vec::new();

    for path in rust_and_active_docs(&root) {
        if path.ends_with("legacy_api_guard.rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for name in &removed {
            if source.contains(name) {
                violations.push(format!("{} contains `{name}`", path.display()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "removed declaration macro names are forbidden outside historical changelog entries:\n{}",
        violations.join("\n")
    );
}

#[test]
fn removed_entity_state_derive_does_not_return() {
    let root = workspace_root();
    let old_derive = ["derive(", "Entity", "State", ")"].concat();
    let old_attribute = ["entity", "_state("].concat();
    let mut violations = Vec::new();

    for path in rust_and_active_docs(&root) {
        if path.ends_with("legacy_api_guard.rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        if source.contains(&old_derive) || source.contains(&old_attribute) {
            violations.push(path.display().to_string());
        }
    }

    assert!(
        violations.is_empty(),
        "removed EntityState derive syntax remains in:\n{}",
        violations.join("\n")
    );
}
