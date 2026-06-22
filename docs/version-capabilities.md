# Version Capabilities

Use `VersionProfile` as the source of truth for feature support.

```rust
use sand_core::prelude::*;

let version = MinecraftVersion::parse("1.21.5").unwrap();
let profile = VersionProfile::resolve(&version).unwrap();

assert!(profile.supports_feature("dialogs"));
assert!(profile.supports_dialogs());
```

Capabilities currently include pack formats, dialogs, function macros, item and
data components, resource-pack overlays, trim assets, jukebox songs, damage
types, chat types, enchantments, and 26.x fallback tracking.
