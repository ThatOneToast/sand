use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub(crate) fn zip_dir(dist: &Path, name: &str) -> Result<PathBuf> {
    let zip_path = dist.parent().unwrap().join(format!("{name}.zip"));
    let file = std::fs::File::create(&zip_path)
        .with_context(|| format!("failed to create zip '{}'", zip_path.display()))?;
    let mut zip = zip::ZipWriter::new(BufWriter::new(file));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in walkdir::WalkDir::new(dist).sort_by_file_name() {
        let entry = entry?;
        let abs = entry.path();
        if abs.is_file() {
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
    zip.finish()?;
    Ok(zip_path)
}
