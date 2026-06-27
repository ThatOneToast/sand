# API Tiers

Most packs should import the prelude and the proc macros:

```rust
use sand_core::prelude::*;
use sand_macros::{component, event, function};
```

The prelude is Sand's default authoring surface. It includes typed commands,
selectors, execute conditions, state, storage, events, text, resource
references, common datapack component builders, custom items, and deliberate raw
escape hatches.

Use `sand_core::advanced` when you are building custom tooling or framework
extensions that need export registries, dynamic function registration, lifecycle
drains, or raw records. These APIs are supported, but they sit closer to Sand's
generated-output machinery than normal pack code.

Use `sand_core::compat` only for older code that still names compatibility
exports. New examples should not introduce compat imports.

Items hidden from rustdoc are for proc-macro expansion and internal wiring. Do
not depend on them in user packs.

## Migration Policy

Compatibility-preserving API cleanup should happen in small steps:

- Add the replacement to `prelude` or `advanced`.
- Update docs and examples to use the replacement.
- Keep the old export compiling through `compat` or the crate root.
- Add a deprecation note before any future removal.

Raw commands, raw JSON, and raw SNBT remain intentional escape hatches. Prefer
typed builders when Sand models the vanilla feature, and use raw forms for
interop or version-specific fields that are not modeled yet.
