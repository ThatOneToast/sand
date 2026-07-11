# Version Support

Sand targets modern Minecraft Java datapacks, including the 1.19 through 1.21.x
and the emerging 26.x series.

Use `VersionProfile` for capability checks:

```rust
let profile = VersionProfile::resolve(&MinecraftVersion::parse("1.21.6").unwrap()).unwrap();
assert!(profile.supports_feature("dialogs"));
```

The latest known version is `26.2` (`data_fmt=107`, `res_fmt=88`). Known 26.x
profiles currently include `26.1`, `26.1.2`, and `26.2`; the two-part forms are
convenience inputs for verified table entries, not broad "any patch" ranges.
Unknown future 26.x entries and future 1.x minors use conservative fallback
capabilities until confirmed.

For `mc_version = "latest"`, Sand uses the bundled latest-known version so pack
metadata and version-sensitive feature flags stay aligned. Pinned versions
still resolve through Mojang's manifest when Sand needs server metadata for
codegen.

## Default codegen target

The default codegen target (`sand_version::DEFAULT_CODEGEN_VERSION`, currently
`1.21.11`) is a separate concern from `VersionProfile`: it answers "which
Minecraft jar generates our typed Rust APIs?" while `VersionProfile` answers
"which version do exported packs and feature flags target?". Override it with
`SAND_MC_VERSION=<version>`. See `docs/version-support.md` for the full codegen
contract including strict and placeholder fallback behavior.