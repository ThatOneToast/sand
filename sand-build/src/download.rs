use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::cache::{Sha1Digest, VersionCache, ensure_dir, invalid, read_optional, write_atomic};
use crate::error::{Error, Result};
use crate::fingerprint::{combine, hash_bytes};
use crate::manifest::VersionEntry;

/// Private transport seam: tests supply synthetic streams, never a server/socket.
pub(crate) trait Transport {
    fn get(&self, url: &str) -> Result<Response>;
}

pub(crate) struct Response {
    pub(crate) status: u16,
    pub(crate) size: Option<u64>,
    pub(crate) body: Box<dyn Read>,
}

impl Response {
    pub(crate) fn checked(self, url: &str) -> Result<Self> {
        if !(200..300).contains(&self.status) {
            return Err(invalid(format!("HTTP status {} for {url}", self.status)));
        }
        Ok(self)
    }

    pub(crate) fn bytes(mut self, url: &str) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        self.body.read_to_end(&mut bytes)?;
        check_size(url, self.size, bytes.len() as u64)?;
        Ok(bytes)
    }
}

pub(crate) struct Http;
impl Transport for Http {
    fn get(&self, url: &str) -> Result<Response> {
        Self::get_with(url, |url| {
            let response = reqwest::blocking::Client::builder()
                .https_only(true)
                .redirect(reqwest::redirect::Policy::none())
                .build()?
                .get(url)
                .send()?;
            Ok(Response {
                status: response.status().as_u16(),
                size: response.content_length(),
                body: Box::new(response),
            })
        })
    }
}

impl Http {
    /// Validate before dispatch, and reject redirects without contacting Location.
    /// Keep this in the real adapter: synthetic transports do not need a host override.
    fn get_with(
        url: &str,
        dispatch: impl FnOnce(reqwest::Url) -> Result<Response>,
    ) -> Result<Response> {
        let parsed =
            reqwest::Url::parse(url).map_err(|e| invalid(format!("invalid Mojang URL: {e}")))?;
        // Url normalizes empty userinfo away; reject its explicit spelling too.
        let authority = url
            .split_once("://")
            .and_then(|(_, rest)| rest.split(['/', '?', '#']).next())
            .filter(|authority| !authority.is_empty())
            .ok_or_else(|| invalid("download URL requires an explicit authority"))?;
        let explicit_userinfo = authority.contains('@');
        if parsed.scheme() != "https"
            || !matches!(
                parsed.host_str(),
                Some("piston-meta.mojang.com" | "piston-data.mojang.com")
            )
            || parsed.port_or_known_default() != Some(443)
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || explicit_userinfo
            || parsed.fragment().is_some()
        {
            return Err(invalid("download requires an approved Mojang HTTPS origin"));
        }
        // error_for_status does not reject 3xx. Check every non-2xx explicitly.
        dispatch(parsed)?.checked(url)
    }
}

#[derive(Deserialize)]
struct VersionPackage {
    id: String,
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

/// Exact identity is stored alongside the original bytes. The key is an index,
/// not evidence: even a key collision must pass this full comparison and SHA-1.
#[derive(Deserialize, Serialize)]
struct PackageRecord {
    schema: u32,
    id: String,
    url: String,
    sha1: String,
    bytes: Vec<u8>,
}

pub(crate) fn digest(value: &str) -> Result<String> {
    Ok(Sha1Digest::parse(value)?.hex())
}

fn validate_url(url: &str) -> Result<()> {
    let parsed =
        reqwest::Url::parse(url).map_err(|e| invalid(format!("invalid download URL: {e}")))?;
    if parsed.scheme() != "https" || parsed.host_str().is_none() {
        return Err(invalid(format!("expected HTTPS download URL: {url}")));
    }
    Ok(())
}

fn package(bytes: &[u8], entry: &VersionEntry, expected: &str) -> Result<ServerDownload> {
    // Hash raw bytes BEFORE parsing. Reserializing JSON changes the checksum.
    let actual = hash_bytes(bytes);
    if actual != expected {
        return Err(Error::ChecksumMismatch {
            path: entry.url.clone(),
            expected: expected.to_owned(),
            actual,
        });
    }
    let mut pkg: VersionPackage = serde_json::from_slice(bytes)?;
    if pkg.id != entry.id {
        return Err(invalid(format!(
            "package ID {:?} does not match {:?}",
            pkg.id, entry.id
        )));
    }
    pkg.downloads.server.sha1 = digest(&pkg.downloads.server.sha1)?;
    validate_url(&pkg.downloads.server.url)?;
    Ok(pkg.downloads.server)
}

fn server_download(
    cache: &VersionCache,
    entry: &VersionEntry,
    http: &impl Transport,
) -> Result<ServerDownload> {
    if cache.id != entry.id {
        return Err(invalid("version cache guard does not match selected entry"));
    }
    let expected = digest(&entry.sha1)?;
    validate_url(&entry.url)?;
    let key = combine(["package-v1", &entry.id, &entry.url, &expected]);
    let path = cache.dir.join("packages-v1").join(format!("{key}.json"));
    if let Some(bytes) = read_optional(&path)?
        && let Ok(record) = serde_json::from_slice::<PackageRecord>(&bytes)
        && record.schema == 1
        && record.id == entry.id
        && record.url == entry.url
        && record.sha1 == expected
        && let Ok(server) = package(&record.bytes, entry, &expected)
    {
        return Ok(server);
    }
    let bytes = http
        .get(&entry.url)?
        .checked(&entry.url)?
        .bytes(&entry.url)?;
    let server = package(&bytes, entry, &expected)?;
    let record = PackageRecord {
        schema: 1,
        id: entry.id.clone(),
        url: entry.url.clone(),
        sha1: expected,
        bytes,
    };
    write_atomic(&path, &serde_json::to_vec(&record)?)?;
    Ok(server)
}

/// Requires the one version guard held by every public caller. Returned paths
/// never change identity when a manifest selects new bytes for the same version.
pub(crate) fn ensure_server_jar(
    cache: &VersionCache,
    entry: &VersionEntry,
    http: &impl Transport,
) -> Result<PathBuf> {
    let server = server_download(cache, entry, http)?;
    let sha1 = Sha1Digest::parse(&server.sha1)?;
    let dir = cache.artifact_dir(sha1);
    ensure_dir(&dir)?;
    let jar = cache.jar_path(sha1);
    if matches_file(&jar, &server)? {
        return Ok(jar);
    }

    // Legacy writers may still be running. Read their JAR only as a seed: copy
    // to our own temp file, then verify that copy. Never modify their paths/lock.
    let mut temp = tempfile::NamedTempFile::new_in(&dir)?;
    let legacy = cache.dir.join("server.jar");
    match std::fs::File::open(&legacy) {
        Ok(mut source) => {
            std::io::copy(&mut source, &mut temp)?;
            temp.flush()?;
            if matches_file(temp.path(), &server)? {
                temp.persist(&jar).map_err(|e| e.error)?;
                return Ok(jar);
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    // A fresh private file prevents an invalid legacy copy leaving a tail.
    let mut temp = tempfile::NamedTempFile::new_in(&dir)?;
    let mut response = http.get(&server.url)?.checked(&server.url)?;
    let progress = indicatif::ProgressBar::new(server.size.or(response.size).unwrap_or(0));
    progress.set_message(format!("Downloading server.jar (Minecraft {})", entry.id));
    let mut hasher = Sha1::new();
    let mut count = 0u64;
    let mut chunk = [0; 65536];
    loop {
        let n = response.body.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        temp.write_all(&chunk[..n])?;
        hasher.update(&chunk[..n]);
        count += n as u64;
        progress.inc(n as u64);
    }
    temp.flush()?;
    progress.finish_and_clear();
    check_size(&server.url, response.size, count)?;
    check_size(&server.url, server.size, count)?;
    let actual = hex::encode(hasher.finalize());
    if actual != server.sha1 {
        return Err(Error::ChecksumMismatch {
            path: server.url,
            expected: server.sha1,
            actual,
        });
    }
    temp.persist(&jar).map_err(|e| e.error)?;
    Ok(jar)
}

fn check_size(label: &str, expected: Option<u64>, actual: u64) -> Result<()> {
    if let Some(expected) = expected
        && expected != actual
    {
        return Err(invalid(format!(
            "size mismatch for {label}: expected {expected}, got {actual}"
        )));
    }
    Ok(())
}

fn matches_file(path: &Path, server: &ServerDownload) -> Result<bool> {
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.into()),
    };
    let mut hash = Sha1::new();
    let mut chunk = [0; 65536];
    let mut size = 0;
    loop {
        let n = file.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        hash.update(&chunk[..n]);
        size += n as u64;
    }
    Ok(hex::encode(hash.finalize()) == server.sha1
        && server.size.is_none_or(|expected| expected == size))
}

#[cfg(test)]
pub(crate) mod tests;
