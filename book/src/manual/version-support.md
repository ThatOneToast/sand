# Version Support

Minecraft commands and data formats change independently of Rust APIs. Resolve a `VersionProfile` for explicit version-sensitive components and test the generated pack on the target game version.

```rust
let version = MinecraftVersion::parse("1.21.6").unwrap();
let profile = VersionProfile::resolve(&version).unwrap();
```

Interaction entities, dialogs, item components, and advancement fields are especially version-sensitive. See [Vanilla Limitations](../reference/vanilla-limitations.md).
