# Version Support

Sand targets modern Minecraft Java datapacks, including 1.19 through 1.21.x and
the emerging 26.x series.

Use `VersionProfile` for capability checks:

```rust
let profile = VersionProfile::resolve(&MinecraftVersion::parse("1.21.6").unwrap()).unwrap();
assert!(profile.supports_feature("dialogs"));
```

The latest known version is `26.2` (`data_fmt=107`, `res_fmt=88`). Known 26.x
profiles currently include `26.1.x` and `26.2.x`; unknown future 26.x entries
and future 1.x minors use conservative fallback capabilities until confirmed.
