# Verified Minecraft artifact cache

`sand_build::ensure_server_jar(version)` returns the authoritative JAR path.
Capture it rather than constructing `~/.sand/cache/<version>/server.jar`:

```sh
jar="$(cargo run --quiet -p sand-build --bin ensure-server-jar -- 26.2)"
python3 scripts/validate_player_joins.py --jar "$jar"
```

This applies to current reruns of the commands in `docs/testing/*-evidence.md`,
`docs/architecture/numeric-constant-results.md`, and
`docs/xp-level-detector-validation.md`. Their recorded commands, JAR paths and
hashes are historical evidence and remain unchanged. It also applies to direct
`validate-vanilla-reload.sh` calls: pass `--jar "$jar"` to skip its helper
invocation, or omit `--jar` to let it resolve and verify the JAR. The old
`SAND_JAR_CACHE` filename convention is removed; explicit `--jar` supports
externally prepared artifacts.

## Identity and offline readiness

Resolution preserves the complete Mojang `VersionEntry` (ID, package URL,
package SHA-1). `latest` means Sand's bundled `sand_version::LATEST_KNOWN`, not
Mojang's moving release. A cached pinned entry is preferred; missing or invalid
entries require manifest retrieval. A manifest selected for an invocation is
not re-resolved after waiting for the artifact lock.

The package cache key contains a format salt, concrete version, exact requested
URL and normalized package SHA-1. The record also stores and compares that full
identity. Raw package bytes are hashed before JSON parsing and package-ID
validation. A URL change requires its own verified package record, even when
its digest is unchanged. A changed expected digest cannot reuse old bytes.
Redirects do not change the requested identity or expected digest.

JAR bytes are hashed on every call, including cache hits, against the server
SHA-1 from verified package metadata. Declared sizes are checked when present.
HTTP errors, interrupted bodies and mismatched sizes/checksums do not publish
artifacts. New downloads use unique same-filesystem temporary files and publish
only after successful verification. A failed replacement preserves prior bytes.
No retry or placeholder bypass is part of the cache contract.

A warm manifest, package and JAR require zero HTTP. A legacy JAR alone is not
proof of the expected checksum: the first call may need to fetch package
metadata, then copy and verify the old JAR without downloading it again. The
copy is verified after copying to handle concurrent legacy writers. Missing or
corrupt metadata during an outage is an error, even if a JAR exists.

Returned paths are `~/.sand/cache/<version>/artifacts/<jar-sha1>/server.jar`.
Different selected JAR digests cannot overwrite one another. Reports are
staged privately and published beside their JAR under `reports-v1`; provenance
records its digest and required report hashes. Legacy version-only reports are
never reused. This can require one Java report-generation run after migration.
The generated-Rust fingerprint cache continues to use report bytes and generator
inputs. No global migration or garbage collection is performed.

## Locking and rollout

Rust std OS locks are held through open file handles and release on process
exit, including a killed process. Waiting for a live owner is bounded to five
minutes. Stable lock files are never unlinked. Manifest resolution holds
`.sand-manifest-v1.lock`, releases it, then callers acquire the per-version
`.sand-artifacts-v1.lock`. All download callers share that discipline; generation
and registry-fixture refresh retain the version guard through report use.
There is no recursive acquisition or reverse lock order.

**The new protocol does not coordinate with legacy presence locks.** It never
unlinks, opens as an OS lock, or writes `.sand-codegen.lock`. The new manifest
`download-manifest-v1.json` imports the legacy `version_manifest_v2.json` only
when absent, then becomes independent. New code never writes legacy manifest,
JAR, or report paths. Old writers can continue using them. New artifact/report
paths are disjoint. Existing generated-Rust cache entries still use their
pre-existing unique staging and atomic publication protocol.

Actual shared-cache validation must be coordinated after legacy validation jobs
are released; focused tests use disposable roots only. No manual stale-lock
removal is needed for the new protocol. Do not delete a new lock file while any
process might hold it: that would create a second lock inode.

A pinned version string is not a project lockfile. Replacing the new manifest,
cache eviction or resolving a previously missing version can select changed
metadata on later calls. The current invocation retains its snapshot. A
changed selected JAR uses different reports; failed acquisition never silently
falls back to a different identity. Cached manifest bytes and Mojang HTTPS are
the trust root. SHA-1 preserves Mojang's checksum contract against corruption,
not authenticity against a hostile cache owner or deliberate SHA-1 collisions.
The API returns a path, not a tamper-proof open handle. Atomic publication does
not promise durability across power failure.

Related prior work: #347/#348/#349 (closed incremental/codegen work). This change
does not close or reopen those issues.
