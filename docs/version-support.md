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
Future 26.x entries use fallback capabilities until exact formats are confirmed.
