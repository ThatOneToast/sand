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
When `mc_version = "latest"`, Sand asks Mojang's manifest for the current
release when available and falls back to this bundled latest-known version when
offline or when the manifest cannot be refreshed.

Known 26.x profiles currently include `26.1`, `26.1.2`, and `26.2`; the
two-part forms are convenience inputs for verified table entries, not broad
"any patch" ranges. Unknown future 26.x entries, including unverified patches
such as `26.1.99` or `26.2.99`, and future 1.x minors use a conservative
fallback: Sand keeps the latest known pack formats for structurally valid
`pack.mcmeta` output, but marks the profile as fallback and disables
version-sensitive capability flags until the version is verified.
