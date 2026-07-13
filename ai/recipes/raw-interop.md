---
id: raw-interop
capabilities:
  - raw-commands
  - raw-json-snbt
minecraft:
  minimum: "1.18.0"
  maximum_verified: "26.2.0"
cargo_features: []
verification:
  compiles: true
  golden_output: false
  vanilla_reload: false
---

# Raw interop

## Intent

Isolate the specific gap where Sand has no typed coverage — calling another
datapack's documented function, and supplying a loot-condition shape the
typed `LootCondition` enum doesn't model — without dropping the rest of the
pack to raw commands.

## Required crates and features

`sand-core`, `sand-macros`. No optional Cargo features required.

## Code

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

#[function]
pub fn call_other_pack_api() {
    mcfunction! {
        // Escape hatch: this is another datapack's documented public contract,
        // not something Sand can model — it belongs to code Sand doesn't own.
        cmd::raw("function other_pack:api/do_special_thing");
    }
}

#[component]
pub fn custom_predicate_via_raw_json() -> Predicate {
    Predicate::new(
        ResourceLocation::new("my_pack", "unmodeled_condition").unwrap(),
        LootCondition::Custom {
            condition: "mymod:unmodeled_condition".to_string(),
            data: RawJson::new(serde_json::json!({ "value": 1 })),
        },
    )
}
```

## Expected generated resources

- `data/my_pack/function/call_other_pack_api.mcfunction` containing the
  literal line `function other_pack:api/do_special_thing`.
- `data/my_pack/predicate/unmodeled_condition.json` containing
  `{"condition": "mymod:unmodeled_condition", "value": 1}` — the
  `LootCondition::Custom` variant serializes `condition` as the JSON
  `condition` key and splices `data`'s JSON in.

## Sand limitations

This recipe exists *because* of a Sand limitation by definition — the
`raw_escape_hatch` fields in `ai/capability-manifest.yaml` name exactly
which typed API each raw path stands in for (`raw-commands`,
`raw-json-snbt`). Keep the raw usage scoped to the single command/field
that needs it; don't let it spread to parts of the pack a typed API already
covers.

## Vanilla limitations

None specific to this pattern — raw command/JSON interop is a Sand
authoring choice, not a vanilla constraint.

## Validation steps

1. `cargo build`.
2. `cargo run -p sand -- build`; read the generated `.mcfunction`/`.json` and confirm the raw content is exactly what was intended (raw strings get no compile-time validation).
3. Not vanilla-reload-verified in this review. Raw content is especially
   worth reload-testing since Sand cannot catch a malformed raw command or
   raw JSON shape at compile time.

## Common incorrect approaches

- Using `cmd::raw(...)` for a command that already has a typed builder in
  `sand_core::prelude::cmd` — check `sand-commands/src` first;
  `ai/authoring-guide.md` calls this out as an anti-hallucination rule.
- Using the `mcfunction!` macro as the default way to write a whole
  function — it's positioned as advanced tooling for raw-command
  collections (`LIM-EXP-002`), not the beginner path; prefer `#[function]`
  bodies made of typed calls, reserving `mcfunction! { cmd::raw(...) }` for
  the specific raw lines.
- Reaching for `RawComponent`/`RawJson` on an entire component when only one
  field is unsupported — as in the predicate example above, keep the typed
  wrapper (`Predicate::new`, `LootCondition::Custom { condition, data }`)
  and scope the raw JSON to just the unmodeled `data` payload.
