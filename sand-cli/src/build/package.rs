use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};

pub fn zip_dir(dist: &Path, name: &str) -> Result<PathBuf> {
    super::output_manifest::reject_symlink_ancestors(dist)?;
    ensure!(dist.is_dir(), "pack to zip is not a directory");
    ensure!(
        !name.is_empty()
            && Path::new(name).components().count() == 1
            && !matches!(name, "." | "..")
            && !name.contains(['/', '\\']),
        "zip name must be a single filename stem"
    );
    let zip_path = dist.parent().unwrap().join(format!("{name}.zip"));
    super::output_manifest::reject_symlink_ancestors(&zip_path)?;
    let mut temporary = tempfile::NamedTempFile::new_in(dist.parent().unwrap())?;
    let mut zip = zip::ZipWriter::new(BufWriter::new(temporary.as_file_mut()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in walkdir::WalkDir::new(dist).sort_by_file_name() {
        let entry = entry?;
        let abs = entry.path();
        ensure!(
            entry.file_type().is_dir() || entry.file_type().is_file(),
            "pack contains symlink or special file: {}",
            abs.display()
        );
        if entry.file_type().is_file() {
            // Sand's own output manifest (issue #347 Phase 7) is bookkeeping
            // for incremental writes, not part of the datapack: the release
            // zip must contain exactly the canonical current output.
            if abs.file_name().and_then(|n| n.to_str())
                == Some(super::output_manifest::MANIFEST_FILE_NAME)
            {
                continue;
            }
            // Strip dist itself so pack.mcmeta and assets/ sit at the zip root,
            // which is what Minecraft requires.
            let rel = abs.strip_prefix(dist)?;
            zip.start_file(rel.to_str().context("non-UTF-8 path")?, options)?;
            let mut input = BufReader::new(
                std::fs::File::open(abs)
                    .with_context(|| format!("failed to open '{}'", abs.display()))?,
            );
            std::io::copy(&mut input, &mut zip)
                .with_context(|| format!("failed to zip '{}'", abs.display()))?;
        }
    }
    zip.finish()?.flush()?;
    temporary
        .persist(&zip_path)
        .with_context(|| format!("install zip '{}'", zip_path.display()))?;
    Ok(zip_path)
}
