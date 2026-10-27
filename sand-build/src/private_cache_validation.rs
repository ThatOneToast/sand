//! Explicit maintenance-only validation. Never consults the user's cache root.
//! Modes require approved real inputs; normal test runs never execute this test.
use std::cell::RefCell;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::Result;
use crate::cache::{VersionCache, invalid};
use crate::download::{Http, Response, Transport};
use crate::fingerprint::hash_bytes;
use crate::manifest::{VersionEntry, VersionManifest};
use serde::Deserialize;
use sha1::{Digest, Sha1};

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const MODES: &[&str] = &[
    "cold_replay",
    "warm_panic",
    "warm_http_denied",
    "legacy_missing",
    "legacy_import",
];
const REPORTS: &[&str] = &[
    "registries.json",
    "blocks.json",
    "commands.json",
    "datapack.json",
];

struct Config {
    root: PathBuf,
    inputs: PathBuf,
    java: PathBuf,
    mode: String,
}

fn explicit_dir(name: &str) -> Result<PathBuf> {
    let path =
        PathBuf::from(std::env::var_os(name).ok_or_else(|| invalid(format!("missing {name}")))?);
    if !path.is_absolute() {
        return Err(invalid(format!("{name} must be absolute")));
    }
    let canonical = path.canonicalize()?;
    if canonical != path || !canonical.is_dir() {
        return Err(invalid(format!(
            "{name} must be an existing canonical directory"
        )));
    }
    Ok(canonical)
}

impl Config {
    fn read() -> Result<Self> {
        // Validate every parameter before reading trusted inputs or writing files.
        let mode = std::env::var("SAND_CACHE_VALIDATION_MODE")
            .map_err(|_| invalid("missing SAND_CACHE_VALIDATION_MODE"))?;
        if !MODES.contains(&mode.as_str()) {
            return Err(invalid("unknown validation mode"));
        }
        let root = explicit_dir("SAND_CACHE_VALIDATION_ROOT")?;
        if root.parent() != Some(Path::new("/private/tmp"))
            || !root.file_name().and_then(|s| s.to_str()).is_some_and(|s| {
                s.starts_with("sand-pr495-validation-")
                    && s.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            })
        {
            return Err(invalid(
                "private root must be /private/tmp/sand-pr495-validation-<unique>",
            ));
        }
        let inputs = explicit_dir("SAND_CACHE_VALIDATION_INPUTS")?;
        if inputs != root.join("inputs") {
            return Err(invalid(
                "inputs must be the private root's canonical inputs directory",
            ));
        }
        let java = PathBuf::from(
            std::env::var_os("SAND_CACHE_VALIDATION_JAVA")
                .ok_or_else(|| invalid("missing SAND_CACHE_VALIDATION_JAVA"))?,
        );
        if !java.is_absolute() || java.canonicalize()? != java || !java.is_file() {
            return Err(invalid(
                "Java must be an explicit canonical absolute executable path",
            ));
        }
        Ok(Self {
            root,
            inputs,
            java,
            mode,
        })
    }
}

#[derive(Deserialize)]
struct Package {
    id: String,
    downloads: Downloads,
}
#[derive(Deserialize)]
struct Downloads {
    server: Server,
}
#[derive(Deserialize)]
struct Server {
    url: String,
    sha1: String,
    size: u64,
}

struct Inputs {
    entry: VersionEntry,
    server: Server,
    manifest: PathBuf,
    package: PathBuf,
    jar: PathBuf,
}

fn file_in(dir: &Path, name: &str) -> Result<PathBuf> {
    let file = dir.join(name);
    if file.canonicalize()? != file || !file.is_file() {
        return Err(invalid(format!(
            "input must be a regular non-symlink file: {}",
            file.display()
        )));
    }
    Ok(file)
}

fn file_sha1(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha1::new();
    let mut bytes = [0; 65536];
    loop {
        let n = file.read(&mut bytes)?;
        if n == 0 {
            break;
        }
        hash.update(&bytes[..n]);
    }
    Ok(hex::encode(hash.finalize()))
}

impl Inputs {
    fn verify(config: &Config) -> Result<Self> {
        let manifest = file_in(&config.inputs, "manifest.json")?;
        let parsed: VersionManifest = serde_json::from_slice(&fs::read(&manifest)?)?;
        let entry = parsed.resolve("26.2")?;
        let package = file_in(&config.inputs, "package.json")?;
        let bytes = fs::read(&package)?;
        if hash_bytes(&bytes) != entry.sha1 {
            return Err(invalid("raw package SHA-1 differs from trusted manifest"));
        }
        let parsed: Package = serde_json::from_slice(&bytes)?;
        if parsed.id != entry.id {
            return Err(invalid("package ID differs from selected entry"));
        }
        let mut server = parsed.downloads.server;
        server.sha1 = crate::download::digest(&server.sha1)?;
        let jar = file_in(&config.inputs, "server.jar")?;
        if file_sha1(&jar)? != server.sha1 || fs::metadata(&jar)?.len() != server.size {
            return Err(invalid("trusted JAR size/SHA-1 differs from package"));
        }
        Ok(Self {
            entry,
            server,
            manifest,
            package,
            jar,
        })
    }
}

/// Replays exact approved bytes, retaining URL identity and streaming the JAR.
/// Unknown requests, including a JAR request during legacy import, fail closed.
struct Replay<'a> {
    inputs: &'a Inputs,
    jar_allowed: bool,
    calls: RefCell<Vec<String>>,
}
impl Transport for Replay<'_> {
    fn get(&self, url: &str) -> Result<Response> {
        self.calls.borrow_mut().push(url.to_owned());
        let path = if url == MANIFEST_URL {
            &self.inputs.manifest
        } else if url == self.inputs.entry.url {
            &self.inputs.package
        } else if url == self.inputs.server.url && self.jar_allowed {
            &self.inputs.jar
        } else {
            return Err(invalid(format!("unapproved replay request: {url}")));
        };
        Ok(Response {
            status: 200,
            size: Some(fs::metadata(path)?.len()),
            body: Box::new(File::open(path)?),
        })
    }
}
struct NoRequests;
impl Transport for NoRequests {
    fn get(&self, url: &str) -> Result<Response> {
        panic!("unexpected validation HTTP request: {url}")
    }
}
struct MissingMetadata;
impl Transport for MissingMetadata {
    fn get(&self, _: &str) -> Result<Response> {
        Err(invalid("validation metadata unavailable"))
    }
}
struct CountedHttp {
    calls: RefCell<Vec<String>>,
}
impl Transport for CountedHttp {
    fn get(&self, url: &str) -> Result<Response> {
        self.calls.borrow_mut().push(url.to_owned());
        Http.get(url)
    }
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}

/// Caller supplies PATH; do not mutate global process environment inside tests.
fn check_launcher(config: &Config) -> Result<()> {
    let bin = config.root.join("bin");
    let first = std::env::var_os("PATH").and_then(|p| std::env::split_paths(&p).next());
    if first.as_deref() != Some(bin.as_path()) {
        return Err(invalid("PATH must begin with private root/bin"));
    }
    let launcher = file_in(&bin, "java")?;
    let expected = format!(
        "#!/bin/sh\nprintf 'BEGIN\\0' >> {log}\nprintf '%s\\0' \"$@\" >> {log}\nexec {java} \"$@\"\n",
        log = shell_quote(&config.root.join("java-argv.nul")),
        java = shell_quote(&config.java)
    );
    if fs::read_to_string(launcher)? != expected {
        return Err(invalid(
            "Java recording launcher does not match explicit executable",
        ));
    }
    Ok(())
}

fn read_log(root: &Path) -> Result<Vec<u8>> {
    match fs::read(root.join("java-argv.nul")) {
        Ok(bytes) => Ok(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e.into()),
    }
}

fn verify_reports(reports: &Path, expected_jar: &str) -> Result<serde_json::Value> {
    let provenance: serde_json::Value =
        serde_json::from_slice(&fs::read(reports.join(".sand-provenance.json"))?)?;
    if provenance["schema"] != 1 || provenance["jar_sha1"] != expected_jar {
        return Err(invalid("wrong report provenance"));
    }
    for name in REPORTS {
        let bytes = fs::read(reports.join(name))?;
        if !serde_json::from_slice::<serde_json::Value>(&bytes)?.is_object()
            || provenance["files"][name] != hash_bytes(&bytes)
        {
            return Err(invalid(format!("invalid report {name}")));
        }
    }
    Ok(provenance)
}

fn pipeline(
    config: &Config,
    inputs: &Inputs,
    cache_root: &Path,
    output: &Path,
    http: &impl Transport,
    warm: bool,
) -> Result<serde_json::Value> {
    let before = read_log(&config.root)?;
    let entry = crate::resolve_version(cache_root, "26.2", http)?;
    if entry.id != inputs.entry.id
        || entry.url != inputs.entry.url
        || entry.sha1 != inputs.entry.sha1
    {
        return Err(invalid("resolved entry differs from approved inputs"));
    }
    let cache = VersionCache::acquire(cache_root, &entry.id)?;
    let jar = crate::download::ensure_server_jar(&cache, &entry, http)?;
    let expected = cache_root
        .join("26.2/artifacts")
        .join(&inputs.server.sha1)
        .join("server.jar");
    if jar != expected || file_sha1(&jar)? != inputs.server.sha1 {
        return Err(invalid("wrong immutable JAR path/bytes"));
    }
    let reports = crate::report::ensure_reports(&cache, &jar)?;
    let provenance = verify_reports(&reports, &inputs.server.sha1)?;
    fs::create_dir(output)?;
    // Explicit root is essential: generate_all_cached would use the real HOME.
    crate::codegen::generate_all_cached_with_root(cache_root, &reports, output, &entry.id)?;
    for file in ["registries.rs", "block_states.rs", "commands.rs"] {
        let text = fs::read_to_string(output.join(file))?;
        if text.trim().is_empty() || text.contains("Generation failed") {
            return Err(invalid("placeholder/empty generated code"));
        }
    }
    let after = read_log(&config.root)?;
    if warm {
        if before != after {
            return Err(invalid("warm report hit launched Java"));
        }
    } else {
        let delta = after
            .strip_prefix(before.as_slice())
            .ok_or_else(|| invalid("Java log changed unexpectedly"))?;
        let args: Vec<_> = delta.split(|b| *b == 0).collect();
        let jar_bytes = jar.to_string_lossy();
        if !args
            .windows(2)
            .any(|pair| pair == [b"-jar".as_slice(), jar_bytes.as_bytes()])
            || !args.contains(&b"--reports".as_slice())
        {
            return Err(invalid(
                "real report invocation did not record selected immutable JAR",
            ));
        }
    }
    drop(cache); // Never recursively acquire the guard.
    if sand_version::LATEST_KNOWN != "26.2" {
        return Err(invalid(
            "bundled anchor changed; revise this 26.2 experiment",
        ));
    }
    if crate::ensure_server_jar_in(cache_root, "latest", &NoRequests)? != jar {
        return Err(invalid("latest changed artifact identity"));
    }
    Ok(
        serde_json::json!({"jar": jar, "jar_sha1": inputs.server.sha1, "reports": reports, "provenance": provenance, "warm": warm}),
    )
}

fn run(config: Config) -> Result<()> {
    let inputs = Inputs::verify(&config)?;
    check_launcher(&config)?;
    // A mode runs once per private root. No cache or output location is reused
    // by a different experiment, except the explicitly warm cold-cache pair.
    let evidence = config.root.join(format!("{}.json", config.mode));
    let mut record = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&evidence)?;
    let output = config.root.join(format!("output-{}", config.mode));
    let cache = config.root.join(if config.mode.starts_with("warm_") {
        "cache-cold_replay"
    } else {
        match config.mode.as_str() {
            "cold_replay" => "cache-cold_replay",
            "legacy_missing" => "cache-legacy_missing",
            _ => "cache-legacy_import",
        }
    });
    if config.mode.starts_with("warm_") {
        if cache.canonicalize()? != cache {
            return Err(invalid("warm cache must be canonical and pre-existing"));
        }
    } else {
        fs::create_dir(&cache)?;
    }
    let replay = Replay {
        inputs: &inputs,
        jar_allowed: config.mode == "cold_replay",
        calls: RefCell::new(Vec::new()),
    };
    let outcome = match config.mode.as_str() {
        "cold_replay" => pipeline(&config, &inputs, &cache, &output, &replay, false)?,
        "warm_panic" => pipeline(&config, &inputs, &cache, &output, &NoRequests, true)?,
        "warm_http_denied" => {
            let http = CountedHttp {
                calls: RefCell::new(Vec::new()),
            };
            let result = pipeline(&config, &inputs, &cache, &output, &http, true)?;
            if !http.calls.borrow().is_empty() {
                return Err(invalid("warm path attempted transport requests"));
            }
            result
        }
        "legacy_missing" | "legacy_import" => {
            let legacy_dir = cache.join("26.2");
            fs::create_dir(&legacy_dir)?;
            let legacy = legacy_dir.join("server.jar");
            fs::copy(&inputs.jar, &legacy)?;
            if config.mode == "legacy_missing" {
                if crate::ensure_server_jar_in(&cache, "26.2", &MissingMetadata).is_ok() {
                    return Err(invalid("JAR-only cache unexpectedly succeeded"));
                }
                fs::copy(&inputs.manifest, cache.join("version_manifest_v2.json"))?;
                if crate::ensure_server_jar_in(&cache, "26.2", &MissingMetadata).is_ok() {
                    return Err(invalid("missing package unexpectedly succeeded"));
                }
                if legacy_dir.join("artifacts").exists() {
                    return Err(invalid("missing metadata published artifacts"));
                }
                serde_json::json!({"metadata_required": true})
            } else {
                fs::copy(&inputs.manifest, cache.join("version_manifest_v2.json"))?;
                let legacy_reports = legacy_dir.join("generated/reports");
                fs::create_dir_all(&legacy_reports)?;
                fs::write(legacy_reports.join("registries.json"), b"legacy sentinel")?;
                let result = pipeline(&config, &inputs, &cache, &output, &replay, false)?;
                if replay.calls.borrow().as_slice() != [inputs.entry.url.as_str()] {
                    return Err(invalid(
                        "legacy import must replay exactly one package and no JAR",
                    ));
                }
                if fs::read(legacy_reports.join("registries.json"))? != b"legacy sentinel" {
                    return Err(invalid("legacy reports were modified"));
                }
                pipeline(
                    &config,
                    &inputs,
                    &cache,
                    &config.root.join("output-legacy-warm"),
                    &NoRequests,
                    true,
                )?;
                result
            }
        }
        _ => unreachable!(),
    };
    if config.mode.starts_with("legacy_")
        && file_sha1(&cache.join("26.2/server.jar"))? != inputs.server.sha1
    {
        return Err(invalid("legacy JAR was modified"));
    }
    let ledger = serde_json::json!({"mode": config.mode, "inputs": config.inputs, "java": config.java,
        "requests_replayed": replay.calls.borrow().clone(), "outcome": outcome,
        "limits": "Injected private orchestration; not all public wrappers. No syscall-attempt proof. Only four required report hashes. No changed-manifest concurrency proof."});
    record.write_all(&serde_json::to_vec_pretty(&ledger)?)?;
    record.write_all(b"\n")?;
    Ok(())
}

#[test]
#[ignore = "explicit private-root trusted-input Java validation; never part of the normal suite"]
fn trusted_26_2() -> Result<()> {
    run(Config::read()?)
}
