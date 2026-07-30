use std::ffi::OsStr;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("sand crate has a workspace parent")
        .to_path_buf()
}

fn rust_and_active_docs(root: &Path) -> Vec<PathBuf> {
    fn visit(directory: &Path, files: &mut Vec<PathBuf>) {
        let mut entries: Vec<_> = std::fs::read_dir(directory)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to read guard directory {}: {error}",
                    directory.display()
                )
            })
            .map(|entry| {
                entry.unwrap_or_else(|error| {
                    panic!(
                        "failed to read an entry under {}: {error}",
                        directory.display()
                    )
                })
            })
            .collect();
        entries.sort_by_key(std::fs::DirEntry::file_name);

        for entry in entries {
            let path = entry.path();
            let file_type = entry.file_type().unwrap_or_else(|error| {
                panic!("failed to inspect guard path {}: {error}", path.display())
            });
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                if matches!(
                    entry.file_name().to_str(),
                    Some(".git" | ".claude" | "target" | "dist" | "generated" | "node_modules")
                ) {
                    continue;
                }
                if path.ends_with("book/book") {
                    continue;
                }
                visit(&path, files);
                continue;
            }
            if path.file_name() == Some(OsStr::new("CHANGELOG.md")) {
                continue;
            }
            if matches!(path.extension().and_then(OsStr::to_str), Some("rs" | "md")) {
                files.push(path);
            }
        }
    }

    let mut files = Vec::new();
    visit(root, &mut files);
    files.sort();
    files
}

fn relative_path<'a>(root: &Path, path: &'a Path) -> &'a Path {
    path.strip_prefix(root).unwrap_or(path)
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
                violations.push(format!(
                    "{} contains `{name}`",
                    relative_path(&root, &path).display()
                ));
            }
        }
    }
    violations.sort();

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
            violations.push(relative_path(&root, &path).display().to_string());
        }
    }
    violations.sort();

    assert!(
        violations.is_empty(),
        "removed EntityState derive syntax remains in:\n{}",
        violations.join("\n")
    );
}
