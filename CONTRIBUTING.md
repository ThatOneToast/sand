# Contributing

Use small, green commits. For code or public docs changes, run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p sand-macros
cargo doc --workspace --all-features --no-deps
```

Or use the combined check script:

```sh
scripts/check.sh
```

## Optional: sccache for faster rebuilds (issue #347)

Sand's workspace is large enough (several crates, proc macros, build
scripts) that a shared-compilation-object cache noticeably speeds up clean
builds, builds after branch switches, and builds in additional Git
worktrees (including agent-created ones). This is entirely optional — Sand
builds and CI both work correctly with no compiler cache at all, and a
missing/broken cache only costs time, never correctness.

Install:

```sh
# macOS
brew install sccache

# Linux
cargo install sccache --locked
```

Enable it for your local Cargo builds by adding to `~/.cargo/config.toml`
(do **not** add this to the repo's own Cargo config — it must stay a
per-developer opt-in):

```toml
[build]
rustc-wrapper = "sccache"
```

Check it's active and see hit-rate stats with:

```sh
sccache --show-stats
```

### Worktree reuse

By default, sccache's cache keys include the absolute path to the crate
being compiled, which normally prevents cache hits across two different
Git worktrees of the same repo (e.g. `sand/` and an agent's temporary
worktree under `sand/.claude/worktrees/...`) even when the source is
identical. Setting `SCCACHE_BASEDIRS` (from sccache's `local` cache
backend path-normalization support) to a common ancestor directory of your
worktrees lets sccache normalize paths under that root before hashing, so
equivalent compilations in different worktrees can share cache entries:

```sh
export SCCACHE_BASEDIRS="$HOME/Desktop/Projects/sand"
```

Verify this is actually helping (rather than assuming it does) by checking
`sccache --show-stats` before and after building a second worktree — the
cache hit count should increase noticeably compared to a build without
`SCCACHE_BASEDIRS` set. Sand does not depend on this working for
correctness; it's purely a speed optimization.

### Incremental compilation

Sand does not disable Rust's incremental compilation to increase sccache
hit rates. In local benchmarking, active edit-compile-test loops benefit
far more from incremental compilation than from sccache (which mainly pays
off on cold/clean builds, branch switches, and new worktrees), so both stay
enabled together — `sccache` still gets used for the initial/cold portion
of an incremental build.

## Toolchain policy

`rust-toolchain.toml` is the Rust toolchain authority for local development and
CI. Keep the workspace `rust-version` in `Cargo.toml` aligned with that pinned
channel so Cargo metadata, rustup, and GitHub Actions agree.

## Workspace lint policy

Sand uses a staged strictness policy so public crates can tighten guarantees
without large API-adjacent documentation rewrites.

Currently enforced:

- `#![forbid(unsafe_code)]` in public library crates that do not require unsafe
  internals today: `sand-build`, `sand-commands`, `sand-macros`, and
  `sand-resourcepack`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` for
  default review and CI checks.
- `cargo doc --workspace --all-features --no-deps` to keep public rustdoc
  healthy.

Staged goals:

- Extend `#![forbid(unsafe_code)]` to `sand-core` and `sand-components` after
  their callback registries no longer require unsafe function-pointer erasure.
- Enable `missing_docs` crate by crate once each public crate has complete
  public-item documentation. Do not enable it globally until every public crate
  passes cleanly.

## Authoring guidance

- Teach `#[function]`, `#[datapack_component(Load)]`, and `#[datapack_component(Tick)]` first.
- Use `sand_core::prelude::*` in beginner examples.
- Keep raw commands behind `cmd::raw(...)`.
- Use `mcfunction!` only for advanced command grouping, migration, or interop.
- Add exact output tests when changing command builders or macro expansion.
- Generated command builders live in `sand-core/src/cmd/_generated` (from
  `commands.json`). Hand-written builtins live in `sand-commands/src/builtins.rs`.

## Testing

- `cargo test --workspace --all-features` — workspace tests
- `cargo test -p sand-macros` — trybuild compile tests
- `cargo doc --workspace --all-features --no-deps` — rustdoc validation
- `mdbook build` — book validation (if mdbook is installed)

Do not require network-heavy Minecraft data regeneration in default CI unless
the change is specifically about generated data.

Registry drift tests use checked-in Mojang report fixtures and require no
network or Java. Sand's canonical component inventory is
`sand-components/src/registry_coverage.rs` (`REGISTRY_COVERAGE` for
data-driven registries, `TAG_COVERAGE` for tag-only directories). The
checked-in fixtures under `sand-components/fixtures/registry-coverage/`
come from Mojang's server data generator report
(`generated/reports/datapack.json`); normal tests parse these small files
offline and report missing/stale rows, directory mismatches, and invalid
version gates. Refresh a fixture explicitly when a new Minecraft version
needs coverage:

```sh
cargo run -p sand-build --bin refresh-registry-coverage -- \
  26.2 sand-components/fixtures/registry-coverage/26.2.json
```

This is a maintenance command, not part of normal CI — it needs a cold-cache
network fetch and the Java runtime required by that Minecraft server. Review
the resulting fixture diff together with any intentional coverage-table/
version-gate updates.

## Local codegen contract

`sand-core/build.rs` runs `sand-build` codegen at build time to generate
`commands.rs`, `registries.rs`, and `block_states.rs`. The default target is
`sand_version::DEFAULT_CODEGEN_VERSION`, which tracks `LATEST_KNOWN`
(currently `26.2`), used when `SAND_MC_VERSION` is unset. `sand_version::
CI_STABLE_CODEGEN_VERSION` (`1.21.4`) is kept only as an explicit
compatibility-boundary target that CI also verifies codegens cleanly.

A clean `cargo test -p sand-core --lib` works without environment variables
when the default target is codegen-available (cached jar or network). If
codegen fails, the build fails immediately with an actionable message. Set
`SAND_ALLOW_PLACEHOLDER_CODEGEN=1` to compile with empty placeholder APIs
(tests will fail).
