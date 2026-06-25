use std::fmt::Write as FmtWrite;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use serde::Deserialize;
use sha1::{Digest, Sha1};

use crate::{
    cache::{ensure_dir, version_dir},
    error::{Error, Result},
};

#[derive(Deserialize)]
struct VersionPackage {
    downloads: Downloads,
}

#[derive(Deserialize)]
struct Downloads {
    server: ServerDownload,
}

#[derive(Deserialize)]
struct ServerDownload {
    sha1: String,
    size: Option<u64>,
    url: String,
}

/// Returns the path to the server jar for `version_id`, downloading it if
/// the cached copy is missing or its SHA1 does not match.
pub fn ensure_server_jar(version_id: &str, version_json_url: &str) -> Result<PathBuf> {
    let dir = version_dir(version_id)?;
    ensure_dir(&dir)?;

    let jar_path = dir.join("server.jar");

    // Fetch version-specific package JSON to get server jar URL and expected SHA1.
    let pkg: VersionPackage = reqwest::blocking::get(version_json_url)?.json()?;
    let expected_sha1 = pkg.downloads.server.sha1.to_lowercase();
    let server_url = pkg.downloads.server.url;
    let known_size = pkg.downloads.server.size;

    // Check if the cached jar already matches.
    if jar_path.exists() {
        let actual = sha1_of_file(&jar_path)?;
        if actual == expected_sha1 {
            return Ok(jar_path);
        }
        // SHA1 mismatch — re-download.
    }

    let tmp_path = jar_path.with_extension("jar.tmp");
    let actual = download_with_progress(&server_url, version_id, known_size, &tmp_path)?;
    if actual != expected_sha1 {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(Error::ChecksumMismatch {
            path: server_url,
            expected: expected_sha1,
            actual,
        });
    }

    if jar_path.exists() {
        std::fs::remove_file(&jar_path)?;
    }
    std::fs::rename(&tmp_path, &jar_path)?;
    Ok(jar_path)
}

fn download_with_progress(
    url: &str,
    version_id: &str,
    known_size: Option<u64>,
    dest: &Path,
) -> Result<String> {
    let response = reqwest::blocking::get(url)?;
    let total = known_size
        .or_else(|| response.content_length())
        .unwrap_or(0);

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template("  {msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .with_key("eta", |state: &ProgressState, w: &mut dyn FmtWrite| {
                let _ = write!(w, "{:.1}s", state.eta().as_secs_f64());
            })
            .progress_chars("█▓░"),
    );
    pb.set_message(format!("Downloading server.jar (Minecraft {version_id})"));

    let mut reader = response;
    let mut writer = BufWriter::new(std::fs::File::create(dest)?);
    let mut hasher = Sha1::new();
    let mut chunk = [0u8; 65536]; // 64 KiB chunks
    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        hasher.update(&chunk[..n]);
        writer.write_all(&chunk[..n])?;
        pb.inc(n as u64);
    }
    writer.flush()?;

    pb.finish_and_clear();
    Ok(hex::encode(hasher.finalize()))
}

/// Compute the SHA1 hex digest of a file on disk.
pub fn sha1_of_file(path: &Path) -> Result<String> {
    let mut reader = BufReader::new(std::fs::File::open(path)?);
    let mut hasher = Sha1::new();
    let mut chunk = [0u8; 65536];
    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        hasher.update(&chunk[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn sha1_of_known_bytes() {
        // SHA1("") = da39a3ee5e6b4b0d3255bfef95601890afd80709
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"").unwrap();
        let hash = sha1_of_file(f.path()).unwrap();
        assert_eq!(hash, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn sha1_of_hello() {
        // SHA1("hello") = aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"hello").unwrap();
        let hash = sha1_of_file(f.path()).unwrap();
        assert_eq!(hash, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
    }

    #[test]
    fn streaming_sha1_matches_whole_buffer_digest() {
        let mut f = NamedTempFile::new().unwrap();
        let data = (0..=255).cycle().take(1024 * 1024 + 13).collect::<Vec<_>>();
        f.write_all(&data).unwrap();

        let streaming = sha1_of_file(f.path()).unwrap();
        let buffered = hex::encode(Sha1::digest(&data));

        assert_eq!(streaming, buffered);
    }
}
