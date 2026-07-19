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

If `mdbook` is unavailable locally, install it with `cargo install mdbook`.

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

Minecraft Java 26.2 is the canonical export/profile target
(`sand_version::LATEST_KNOWN`); 1.21.4 is retained as an explicit
oldest-profile/compatibility boundary (`sand_version::CI_STABLE_CODEGEN_VERSION`).
Unknown/future versions fall back to conservative capabilities via
`VersionProfile::resolve()`.

## Publishing

Sand is not yet published to crates.io. Build the CLI from the workspace:

```sh
cargo install --path sand-cli
```

## Post-release

- Update ROADMAP.md with next priorities
- Tag the release in git
- Announce in project channels
