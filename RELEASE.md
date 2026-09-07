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
- [ ] Escape hatches are documented and beginner docs remain attribute-first

## Stability levels

- **Stable**: `#[function]`, `#[datapack_component]`, typed state, typed conditions,
  typed text, typed execute, generated command builders, scaffold
- **Alpha**: Event system and dialog components
- **Experimental**: `mcfunction!` macro (advanced tooling), generated registries
  for future Minecraft versions

## Supported Minecraft versions

Sand targets Minecraft Java 26.x and newer. Minecraft 26.2 is the latest
verified export/profile target (`sand_version::LATEST_KNOWN`), while 26.1 keeps
its exact known profile. Earlier versions are rejected. Unknown future calendar
versions fall back to conservative capabilities via `VersionProfile::resolve()`.

## Publishing

Sand is not yet published to crates.io. Build the CLI from the workspace:

```sh
cargo install --path sand-cli
```

## Post-release

- Tag the release in git
- Announce in project channels
