# Raw Commands

`cmd::raw` is an intentional interop boundary, not the normal authoring path.

```rust
// Minecraft has no typed builder for this external datapack API.
cmd::raw("function other_pack:api/do_special_thing");
```

Use it for another datapack/mod command, snapshot syntax, or an unsupported command field. Prefer typed builders for give, effects, teleport/summon, inventory, state, text, and normal execute. Comment why a raw string is necessary and keep it small.
