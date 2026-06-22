# Version Support

Sand targets modern Minecraft Java datapacks, including 1.19 through 1.21.x and
the emerging 26.x series.

Use `VersionProfile` for capability checks:

```rust
let profile = VersionProfile::resolve(&MinecraftVersion::parse("1.21.5").unwrap()).unwrap();
assert!(profile.supports_feature("dialogs"));
```

Unknown future versions use conservative fallback capabilities until confirmed.
