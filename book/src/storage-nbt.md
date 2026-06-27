# Storage And NBT

Prefer a [derived schema](manual/derive-sand-storage.md) for a stable structured path. For a small manual schema, use `StorageSchema` and its typed fields. `NbtPath`, `StorageVar`, `StorageField`, `EntityNbt`, and `BlockNbt` model static data targets. Use `RawSnbt` only when typed values cannot express the desired NBT; it is the explicit raw escape hatch. `data storage` is global, so static typed paths are not player records; see [Player Data Schemas](manual/player-data.md).

Use `StorageSchema<T>` for structured datapack storage.

```rust
#[derive(Debug)]
struct Settings;

static SETTINGS: StorageSchema<Settings> =
    StorageSchema::new("example:data", "settings");
static MANA: StorageField<Settings, i32> = SETTINGS.field("mana");
static SCHOOL: StorageField<Settings, String> = SETTINGS.field("school");

#[component(Load)]
pub fn load_storage() {
    SETTINGS.set(SnbtCompound::new().field("mana", 100).field("school", "unbound"));
    MANA.set(100);
    SCHOOL.set("pyromancy");
}
```

Storage fields can be used as execute conditions:

```rust
TypedExecute::as_players()
    .when(MANA.exists())
    .run(Actionbar::show(Selector::self_(), Text::new("Storage ready").green()));
```

Build paths with segments instead of dotted strings:

```rust
let path = NbtPath::root("settings").field("magic").field("mana");
```

Use `StorageVar<T>` for simple legacy variables. Use `RawSnbt` only when no
typed `SnbtValue` or `SnbtCompound` builder covers the NBT shape.
