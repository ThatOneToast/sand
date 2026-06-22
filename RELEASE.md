# Release Process

## Pre-release validation

Run the full validation set:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p sand-macros
cargo doc --workspace --all-features --no-deps
mdbook build
```

Or use the combined check script:

```sh
scripts/check.sh
```

## What to verify

- [ ] All tests pass (workspace, trybuild, golden)
- [ ] No clippy warnings
- [ ] No formatting issues
- [ ] Rustdoc builds without warnings
- [ ] mdBook builds
- [ ] Scaffold generates attribute-first typed code
- [ ] Examples compile and have golden tests
- [ ] ROADMAP.md reflects current state
- [ ] CHANGELOG.md has entry for this version
- [ ] Escape hatches are documented and beginner docs remain attribute-first

## Stability levels

- **Stable**: `#[function]`, `#[component]`, typed state, typed conditions,
  typed text, typed execute, generated command builders, scaffold
- **Alpha**: Event system, dialog components, resource pack generation
- **Experimental**: `mcfunction!` macro (advanced tooling), generated registries
  for future Minecraft versions

## Supported Minecraft versions

- 1.19 through 1.21.x (pack formats 9-61)
- 26.x series (emerging, fallback capabilities via `VersionProfile`)

## Publishing

Sand is not yet published to crates.io. Build from the workspace:

```sh
cargo build -p sand --release
```

## Post-release

- Update ROADMAP.md with next priorities
- Tag the release in git
- Announce in project channels
