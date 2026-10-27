use crate::cache::{VersionCache, invalid, read_optional};
use crate::error::{Error, Result};
use crate::fingerprint::hash_bytes;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

const REQUIRED: &[&str] = &[
    "registries.json",
    "blocks.json",
    "commands.json",
    "datapack.json",
];

#[derive(Serialize, Deserialize)]
struct Provenance {
    schema: u32,
    jar_sha1: String,
    files: std::collections::BTreeMap<String, String>,
}

/// Only called while the artifact guard is held. Reports live with their JAR,
/// not at the legacy version-only sentinel. Staging is private until complete.
pub(crate) fn ensure_reports(cache: &VersionCache, jar: &Path) -> Result<PathBuf> {
    ensure_reports_with(cache, jar, run_generator)
}

fn ensure_reports_with(
    cache: &VersionCache,
    jar: &Path,
    generate: impl FnOnce(&Path, &Path) -> Result<()>,
) -> Result<PathBuf> {
    let digest = cache.digest_for_jar(jar)?;
    let artifact = cache.artifact_dir(digest);
    let sha1 = digest.hex();
    let published = cache.reports_dir(digest);
    if valid_reports(&published, &sha1)? {
        return Ok(published);
    }
    let stage = tempfile::Builder::new()
        .prefix(".reports-")
        .tempdir_in(artifact)?;
    generate(stage.path(), jar)?;
    let reports = stage.path().join("generated/reports");
    let files = report_files(&reports)?
        .ok_or_else(|| invalid("generator did not produce complete JSON reports"))?;
    let provenance = Provenance {
        schema: 1,
        jar_sha1: sha1,
        files,
    };
    std::fs::write(
        reports.join(".sand-provenance.json"),
        serde_json::to_vec(&provenance)?,
    )?;

    // All readers hold this version lock. Quarantine an invalid old directory
    // only after a complete replacement exists; restore it if publication fails.
    let backup = stage.path().join("previous");
    let had_previous = match std::fs::rename(&published, &backup) {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(e) => return Err(e.into()),
    };
    if let Err(error) = std::fs::rename(&reports, &published) {
        if had_previous {
            std::fs::rename(&backup, &published)?;
        }
        return Err(error.into());
    }
    Ok(published)
}

fn report_files(dir: &Path) -> Result<Option<std::collections::BTreeMap<String, String>>> {
    let mut files = std::collections::BTreeMap::new();
    for name in REQUIRED {
        let Some(bytes) = read_optional(&dir.join(name))? else {
            return Ok(None);
        };
        if !serde_json::from_slice::<serde_json::Value>(&bytes).is_ok_and(|v| v.is_object()) {
            return Ok(None);
        }
        files.insert((*name).to_owned(), hash_bytes(&bytes));
    }
    Ok(Some(files))
}

fn valid_reports(dir: &Path, sha1: &str) -> Result<bool> {
    let Some(bytes) = read_optional(&dir.join(".sand-provenance.json"))? else {
        return Ok(false);
    };
    let Ok(provenance) = serde_json::from_slice::<Provenance>(&bytes) else {
        return Ok(false);
    };
    Ok(provenance.schema == 1
        && provenance.jar_sha1 == sha1
        && report_files(dir)?.is_some_and(|files| files == provenance.files))
}

fn run_generator(working_dir: &Path, jar_path: &Path) -> Result<()> {
    // Probe for java on PATH first so we can give a clear error.
    let java = which_java()?;

    let output = Command::new(&java)
        .arg("-DbundlerMainClass=net.minecraft.data.Main")
        .arg("-jar")
        .arg(jar_path)
        .arg("--reports")
        .current_dir(working_dir)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::JavaNotFound
            } else {
                Error::Io(e)
            }
        })?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(Error::DataGeneratorFailed { code, stderr });
    }

    Ok(())
}

/// Resolve the `java` binary path, returning `Error::JavaNotFound` if absent.
fn which_java() -> Result<String> {
    // `java -version` exits 0 when java is present.
    let result = Command::new("java").arg("-version").output();
    match result {
        Ok(out) if out.status.success() || !out.stderr.is_empty() => Ok("java".to_string()),
        Ok(_) => Err(Error::JavaNotFound),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(Error::JavaNotFound),
        Err(e) => Err(Error::Io(e)),
    }
}

#[cfg(test)]
mod tests;
