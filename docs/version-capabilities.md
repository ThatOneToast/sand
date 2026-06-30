# Version Capabilities

Use `VersionProfile` as the source of truth for feature support.

```rust
use sand_core::prelude::*;

let version = MinecraftVersion::parse("1.21.6").unwrap();
let profile = VersionProfile::resolve(&version).unwrap();

assert!(profile.supports_feature("dialogs"));
assert!(profile.supports_dialogs());
```

Capabilities currently include pack formats, dialogs, function macros, item and
data components, resource-pack overlays, trim assets, jukebox songs, damage
types, chat types, enchantments, and 26.x fallback tracking.

Cargo features are separate from `VersionProfile`: `systems-damage`, `systems-cooldowns`, `systems-lifecycle`, `systems-player-data`, `systems-movement`, `systems-inventory`, `systems-entities`, and `systems-all` opt into Sand systems. `systems-player-data` currently exposes manual `PlayerDataSchema` helpers (`PlayerSchema` remains as an alias); automatic lifecycle wiring remains future #47/#68 work. `SandStorage` and the authoring macros are macro-crate APIs, not features. See the [guide capability matrix](../book/src/version-capabilities.md).
