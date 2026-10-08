//! Filesystem boundary for the flat manifest format. Nothing here invokes Cargo.
use sand::advanced::compiler::{Compiler, Diagnostic, Program};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
};
const DOCUMENT_BYTES: u64 = 1024 * 1024;
const INPUT_BYTES: usize = 8 * 1024 * 1024;

pub(super) fn error(code: &str, module: &str, message: impl std::fmt::Display) -> Diagnostic {
    Diagnostic {
        code: code.into(),
        severity: "error".into(),
        module: module.into(),
        pointer: String::new(),
        origin: None,
        message: message.to_string(),
    }
}
fn read_file(path: &Path) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let result = (|| -> anyhow::Result<Vec<u8>> {
        crate::build::output_manifest::reject_symlink_ancestors(path)?;
        anyhow::ensure!(path.is_file(), "module document is not a regular file");
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(DOCUMENT_BYTES + 1)
            .read_to_end(&mut bytes)?;
        anyhow::ensure!(
            bytes.len() <= DOCUMENT_BYTES as usize,
            "document exceeds 1 MiB"
        );
        Ok(bytes)
    })();
    result.map_err(|e| vec![error("SAND_PROGRAM_INPUT", &path.display().to_string(), e)])
}
pub(super) fn read(input: &str, root: Option<&Path>) -> Result<Program, Vec<Diagnostic>> {
    let (bytes, root) = if input == "-" {
        let mut bytes = Vec::new();
        std::io::stdin()
            .take(DOCUMENT_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| vec![error("SAND_PROGRAM_INPUT", "stdin", e)])?;
        (bytes, None)
    } else {
        let path = PathBuf::from(input);
        (
            read_file(&path)?,
            Some(
                root.map(Path::to_path_buf)
                    .unwrap_or_else(|| path.parent().unwrap_or(Path::new(".")).to_path_buf()),
            ),
        )
    };
    let paths = Compiler::module_paths(&bytes)?;
    if root.is_none() && !paths.is_empty() {
        return Err(vec![error(
            "SAND_PROGRAM_MODULE",
            "stdin",
            "stdin must be self-contained",
        )]);
    }
    let mut modules = BTreeMap::new();
    let mut total = bytes.len();
    for path in paths {
        let document = read_file(&root.as_ref().expect("file input has root").join(&path))?;
        total += document.len();
        if total > INPUT_BYTES {
            return Err(vec![error(
                "SAND_PROGRAM_LIMIT",
                &path,
                "program exceeds 8 MiB",
            )]);
        }
        modules.insert(path, document);
    }
    Compiler::decode(&bytes, &modules)
}
