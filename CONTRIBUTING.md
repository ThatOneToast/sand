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

- Teach `#[function]`, `#[component(Load)]`, and `#[component(Tick)]` first.
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
network or Java. See [registry coverage drift fixtures](docs/registry-coverage.md)
for provenance and the explicit maintenance refresh command.

## Local codegen contract

`sand-core/build.rs` runs `sand-build` codegen at build time to generate
`commands.rs`, `registries.rs`, and `block_states.rs`. The default target is
`sand_version::DEFAULT_CODEGEN_VERSION` (currently `1.21.11`), used when
`SAND_MC_VERSION` is unset. This is the *codegen target*, deliberately separate
from `sand_version::LATEST_KNOWN` (`26.2`), the *export/profile anchor*.

A clean `cargo test -p sand-core --lib` works without environment variables
when the default target is codegen-available (cached jar or network). If
codegen fails, the build fails immediately with an actionable message. Set
`SAND_ALLOW_PLACEHOLDER_CODEGEN=1` to compile with empty placeholder APIs
(tests will fail). See `docs/version-support.md` for the full contract.
