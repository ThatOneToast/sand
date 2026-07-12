# Datapack Tags

Use `TypedTag<T>` when the registry is known. Its type controls both accepted
entries and the output directory, so item values cannot be added to block tags.

```rust,ignore
use sand_core::{ItemId, TagId, TypedTag};

let tools = TypedTag::new(TagId::<ItemId>::minecraft("tools")?)
    .entry(ItemId::minecraft("diamond_pickaxe")?)
    .optional_entry(ItemId::minecraft("netherite_pickaxe")?)
    .tag_ref(TagId::minecraft("axes")?)
    .optional_tag_ref(TagId::minecraft("modded_tools")?);
```

Required entries serialize as strings. Optional entries serialize as
`{"id":"namespace:path","required":false}`. Tag references always contain
exactly one leading `#`. Item, block, entity-type, and function tags are
supported initially and export under their registry-specific `tags/...`
directory.

Typed tags reject empty value lists by default. Use `.allow_empty(true)` when
an empty tag is deliberate. Entries retain insertion order and duplicates are
not removed, matching Minecraft's file representation and the legacy builder.

`raw_entry` and `optional_raw_entry` are validated escape hatches for modded
IDs or tag references. The existing `Tag` builder remains source-compatible
for dynamic registry layouts; because it is intentionally untyped, callers are
responsible for selecting the correct output path and value registry.
