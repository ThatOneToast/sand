use std::fs::{File, OpenOptions, TryLockError};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::error::{Error, Result};

pub(crate) fn cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or(Error::NoHomeDir)?;
    Ok(home.join(".sand").join("cache"))
}

pub(crate) fn ensure_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

pub(crate) fn invalid(message: impl Into<String>) -> Error {
    std::io::Error::new(ErrorKind::InvalidData, message.into()).into()
}

pub(crate) fn validate_version(id: &str) -> Result<()> {
    if id.is_empty()
        || id == "."
        || id == ".."
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err(invalid(format!("invalid Minecraft version ID {id:?}")));
    }
    Ok(())
}

/// Missing cache data is a miss; permission and other storage errors are not.
pub(crate) fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Publish a complete file without unlinking its previous destination first.
/// NamedTempFile uses unique create-new names on the destination filesystem.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    write_atomic_with(path, |file| {
        file.write_all(bytes)?;
        Ok(())
    })
}

fn write_atomic_with(path: &Path, write: impl FnOnce(&mut File) -> Result<()>) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid("cache path has no parent"))?;
    ensure_dir(parent)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    write(temp.as_file_mut())?;
    temp.flush()?;
    temp.persist(path).map_err(|e| e.error)?;
    Ok(())
}

/// OS-owned lock: the stable inode is never unlinked, including on Drop.
/// These v1 locks are deliberately distinct from legacy .sand-codegen.lock.
pub(crate) struct CacheLock {
    _file: File,
}

impl CacheLock {
    pub(crate) fn acquire(path: &Path) -> Result<Self> {
        Self::acquire_for(path, Duration::from_secs(300))
    }

    fn acquire_for(path: &Path, timeout: Duration) -> Result<Self> {
        ensure_dir(
            path.parent()
                .ok_or_else(|| invalid("lock path has no parent"))?,
        )?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        let start = Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Self { _file: file }),
                Err(TryLockError::WouldBlock) if start.elapsed() < timeout => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(TryLockError::WouldBlock) => {
                    return Err(std::io::Error::new(
                        ErrorKind::TimedOut,
                        format!("timed out waiting for cache lock '{}'", path.display()),
                    )
                    .into());
                }
                Err(TryLockError::Error(error)) => return Err(error.into()),
            }
        }
    }
}

/// One guard spans package/JAR verification and any report consumption.
/// Its directory is derived from the same validated identity used by the caller.
pub(crate) struct VersionCache {
    pub(crate) dir: PathBuf,
    pub(crate) id: String,
    _lock: CacheLock,
}

impl VersionCache {
    pub(crate) fn acquire(root: &Path, id: &str) -> Result<Self> {
        validate_version(id)?;
        let dir = root.join(id);
        let lock = CacheLock::acquire(&dir.join(".sand-artifacts-v1.lock"))?;
        Ok(Self {
            dir,
            id: id.to_owned(),
            _lock: lock,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    fn child(root: &Path, mode: &str) -> std::process::Child {
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "cache::tests::cache_child",
                "--ignored",
                "--nocapture",
            ])
            .env("SAND_TEST_CACHE_CHILD_ROOT", root)
            .env("SAND_TEST_CACHE_CHILD_MODE", mode)
            .stdout(Stdio::null())
            .spawn()
            .unwrap()
    }
    fn wait_for(path: &Path) {
        let start = Instant::now();
        while !path.exists() {
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "child readiness timeout"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    #[test]
    #[ignore = "subprocess fixture; requires a disposable root supplied by parent test"]
    fn cache_child() {
        let root =
            std::env::var_os("SAND_TEST_CACHE_CHILD_ROOT").expect("parent supplies isolated root");
        let root = Path::new(&root);
        let mode = std::env::var("SAND_TEST_CACHE_CHILD_MODE").unwrap();
        if mode == "owner" {
            let _lock = CacheLock::acquire(&root.join("lock")).unwrap();
            write_atomic_with(&root.join("published"), |file| {
                file.write_all(b"partial bytes")?;
                file.flush()?;
                std::fs::write(root.join("ready"), b"ready")?;
                std::thread::sleep(Duration::from_secs(30));
                Ok(())
            })
            .unwrap();
        } else {
            std::fs::write(root.join(format!("ready-{}", std::process::id())), b"ready").unwrap();
            let cache = VersionCache::acquire(root, "26.2").unwrap();
            let (entry, pkg) = crate::download::tests::fixture(b"concurrent jar");
            let http = crate::download::tests::Fake::new();
            http.push(&entry.url, &pkg);
            http.push("https://test.invalid/server", b"concurrent jar");
            let jar = crate::download::ensure_server_jar(&cache, &entry, &http).unwrap();
            assert_eq!(std::fs::read(jar).unwrap(), b"concurrent jar");
            let mut requests = OpenOptions::new()
                .create(true)
                .append(true)
                .open(root.join("requests"))
                .unwrap();
            for call in http.calls.borrow().iter() {
                writeln!(requests, "{call}").unwrap();
            }
        }
    }
    #[test]
    fn killed_owner_releases_os_lock_and_live_owner_wait_is_bounded() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("published"), b"prior complete bytes").unwrap();
        let mut owner = child(root.path(), "owner");
        wait_for(&root.path().join("ready"));
        let start = Instant::now();
        let result = CacheLock::acquire_for(&root.path().join("lock"), Duration::from_millis(60));
        owner.kill().unwrap();
        owner.wait().unwrap();
        assert!(matches!(result, Err(Error::Io(e)) if e.kind() == ErrorKind::TimedOut));
        assert!(start.elapsed() < Duration::from_secs(3));
        let _next =
            CacheLock::acquire_for(&root.path().join("lock"), Duration::from_secs(1)).unwrap();
        assert!(root.path().join("lock").exists());
        assert_eq!(
            std::fs::read(root.path().join("published")).unwrap(),
            b"prior complete bytes"
        );
        write_atomic(&root.path().join("published"), b"next complete bytes").unwrap();
        assert_eq!(
            std::fs::read(root.path().join("published")).unwrap(),
            b"next complete bytes"
        );
    }
    #[test]
    fn concurrent_processes_publish_one_complete_artifact() {
        let root = tempfile::tempdir().unwrap();
        let gate = VersionCache::acquire(root.path(), "26.2").unwrap();
        let mut first = child(root.path(), "publish");
        let mut second = child(root.path(), "publish");
        wait_for(&root.path().join(format!("ready-{}", first.id())));
        wait_for(&root.path().join(format!("ready-{}", second.id())));
        drop(gate);
        assert!(first.wait().unwrap().success());
        assert!(second.wait().unwrap().success());
        assert_eq!(
            std::fs::read_to_string(root.path().join("requests"))
                .unwrap()
                .lines()
                .count(),
            2
        );
        let cache = VersionCache::acquire(root.path(), "26.2").unwrap();
        let (entry, _) = crate::download::tests::fixture(b"concurrent jar");
        let jar = crate::download::ensure_server_jar(
            &cache,
            &entry,
            &crate::download::tests::Fake::new(),
        )
        .unwrap();
        assert_eq!(std::fs::read_dir(jar.parent().unwrap()).unwrap().count(), 1);
    }
    #[test]
    fn failed_publication_preserves_destination_and_cleans_private_temp() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("destination");
        std::fs::create_dir(&destination).unwrap();
        std::fs::write(destination.join("sentinel"), b"old").unwrap();
        assert!(write_atomic(&destination, b"new").is_err());
        assert_eq!(std::fs::read(destination.join("sentinel")).unwrap(), b"old");
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    }
    #[test]
    fn failed_writer_preserves_previous_complete_file() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("published");
        write_atomic(&path, b"old").unwrap();
        assert!(
            write_atomic_with(&path, |file| {
                file.write_all(b"partial")?;
                Err(std::io::Error::other("injected disk write failure").into())
            })
            .is_err()
        );
        assert_eq!(std::fs::read(path).unwrap(), b"old");
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    }
}
