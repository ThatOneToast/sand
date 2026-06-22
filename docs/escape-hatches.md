# Escape Hatches

Raw command strings are allowed, but they are explicit escape hatches. Prefer
typed APIs everywhere Sand has coverage.

Use raw commands for:

- Interoperability with another datapack's public command/function contract
- Modded commands
- Snapshot-only syntax not modeled yet
- Unknown future Minecraft features
- Debugging generated output

```rust
use sand_core::prelude::*;

#[function]
pub fn interop() {
    // Escape hatch: this calls another datapack's documented API.
    cmd::raw("function other_pack:api/do_special_thing");
}
```

Keep raw examples out of beginner docs and examples unless the point of the
example is interop or debugging.
