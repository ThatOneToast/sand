# Version Support

Sand targets modern Minecraft Java datapacks, including the 1.19 through 1.21.x
series and the emerging 26.x series.

Configure projects through `sand.toml`:

```toml
[pack]
namespace = "example"
mc_version = "1.21.6"
```

Known pack formats and feature flags are resolved through `VersionProfile`.
The latest known version is `26.2` (`data_fmt=107`, `res_fmt=88`).
When `mc_version = "latest"`, Sand uses this bundled latest-known version so
pack metadata and version-sensitive feature flags all target the same verified
profile. Pinned versions such as `"1.21.6"` or `"26.2"` still resolve through
Mojang's manifest when Sand needs server metadata for codegen.

Known 26.x profiles currently include `26.1`, `26.1.2`, and `26.2`; the
two-part forms are convenience inputs for verified table entries, not broad
"any patch" ranges. Unknown future 26.x entries, including unverified patches
such as `26.1.99` or `26.2.99`, and future 1.x minors use a conservative
fallback: Sand keeps the latest known pack formats for structurally valid
`pack.mcmeta` output, but marks the profile as fallback and disables
version-sensitive capability flags until the version is verified.

## Default codegen target

The Minecraft version used to run `sand-build` codegen for local `sand-core`
builds/tests is a *separate* concern from `VersionProfile` and the `latest`
export/profile anchor above. The codegen target answers "which Minecraft jar
generates our typed Rust APIs?"; `VersionProfile` answers "which version do
exported packs and feature flags target?". Do not conflate the two.

- The default codegen target is `sand_version::DEFAULT_CODEGEN_VERSION`, currently
  `1.21.11`. `sand-core/build.rs` uses it when `SAND_MC_VERSION` is unset.
- It MUST be a verified, codegen-available version: `sand-build` downloads and
  caches the matching server jar (`~/.sand/cache/<version>/`) and runs the
  Minecraft data generator to produce `commands.rs`, `registries.rs`, and
  `block_states.rs`.
- It does NOT need to equal `latest`/`26.2`.

Override the codegen target for a single build with:

```sh
SAND_MC_VERSION=1.21.4 cargo test -p sand-core --lib
SAND_MC_VERSION=26.2 SAND_STRICT_CODEGEN=1 cargo test -p sand-core generated_api_health --lib
```

### Codegen fallback contract

- **Default** (no env set): codegen runs. If it fails (no cache + no network),
  the build **fails immediately** with an actionable message naming the target,
  the cache path, and the override. No placeholder files are written.
- **`SAND_STRICT_CODEGEN=1`** (CI): same immediate hard failure; kept for CI
  backward compatibility and explicit intent.
- **`SAND_ALLOW_PLACEHOLDER_CODEGEN=1`** (explicit opt-in): if codegen fails,
  writes `// Generation failed` placeholder files so `include!` macros compile,
  and emits a `cargo:warning`. The `generated_api_health` tests then fail on
  those placeholders — empty generated APIs can never silently pass a default
  `cargo test -p sand-core --lib`.

A clean `cargo test -p sand-core --lib` therefore works without requiring
contributors to know to set `SAND_MC_VERSION`, as long as the default codegen
target is codegen-available (cached jar or network).