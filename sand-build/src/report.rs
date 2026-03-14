use std::path::PathBuf;
use std::process::Command;

use crate::{
    cache::version_dir,
    error::{Error, Result},
};

/// Ensures the data generator reports exist for `version_id`.
///
/// If `generated/reports/registries.json` is already present inside the
/// version cache directory, the generator is skipped. Otherwise, `java` is
/// invoked to produce the reports.
///
/// Returns the path to the `generated/reports/` directory.
pub fn ensure_reports(version_id: &str, jar_path: &PathBuf) -> Result<PathBuf> {
    let version_dir = version_dir(version_id)?;
    let reports_dir = version_dir.join("generated").join("reports");
    let sentinel = reports_dir.join("registries.json");

    if sentinel.exists() {
        return Ok(reports_dir);
    }

    run_generator(&version_dir, jar_path)?;

    if !sentinel.exists() {
        return Err(Error::DataGeneratorFailed {
            code: 0,
            stderr: format!(
                "Generator ran but '{}' was not produced.",
                sentinel.display()
            ),
        });
    }

    Ok(reports_dir)
}

fn run_generator(working_dir: &PathBuf, jar_path: &PathBuf) -> Result<()> {
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
