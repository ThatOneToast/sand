# Scoreboard And Storage Commands

## Scoreboards

```rust
static MANA: ScoreVar<i32> = ScoreVar::new("arcane_mana");
MANA.define();
MANA.set("@s", 100);
MANA.add("@s", 5);
MANA.remove("@s", 20);
MANA.reset("@s");
```

These generate `scoreboard objectives add`, `scoreboard players set/add/remove/reset`. Use `MANA.of("@s").gte(20)` as an execute condition rather than hand-writing score ranges.

## Storage/data modify

```rust
static NOTE: StorageVar<String> = StorageVar::new("arcane:data", "runtime.note");
NOTE.set_string("hello");
NOTE.get();
NOTE.remove();
```

This generates `data modify/get/remove storage` commands. It is global storage; see [Global Storage](../../manual/data-model/storage.md).
