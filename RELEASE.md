# Release Process

## Pre-release validation

Run the canonical validation entry point and whitespace check:

```sh
scripts/check.sh
git diff --check
```

This includes Python harness regressions, strict Clippy, workspace/trybuild tests,
standalone example builds and runtime baselines, external consumers, portable
frontends, Rustdoc, mdBook and local documentation links. Do not substitute a
shorter hand-maintained command list. See [Contributing](CONTRIBUTING.md) for
focused iteration and intentional baseline updates.

For runtime changes, also run the relevant native server proof. Export tests do
not prove in-game behavior. The vanilla workflow retains version-specific lanes;
26.2 player proofs and 26.3 compute/load proofs have different coverage.

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

- **Established (still pre-1.0)**: `#[function]`, `#[datapack_component]`, typed state, typed conditions,
  typed text, typed execute, generated command builders, scaffold
- **Alpha**: Events, dialogs, owned gameplay systems, combat and interactions
- **Experimental**: `mcfunction!` macro (advanced tooling), generated registries
  for future Minecraft versions

## Supported Minecraft versions

Sand targets Minecraft Java 26.x and newer. Minecraft 26.3 is the latest
verified export/profile target (`sand_version::LATEST_KNOWN`), while 26.1 and 26.2 keep
their exact known profiles. Earlier versions are rejected. Unknown future calendar
versions fall back to conservative capabilities via `VersionProfile::resolve()`.

## Publishing

Sand is not yet published to crates.io. Build the CLI from the workspace:

```sh
cargo install --path sand-cli
```

## Post-release

- Tag the release in git
- Announce in project channels
