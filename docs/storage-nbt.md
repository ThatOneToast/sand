# Storage And NBT

Use scoreboards for compact integer conditions and `StorageSchema<T>` for
structured datapack storage.

```rust
use sand_core::prelude::*;

#[derive(Debug)]
struct PlayerMagic;

static MAGIC: StorageSchema<PlayerMagic> =
    StorageSchema::new("example:data", "players.self.magic");
static MANA: StorageField<PlayerMagic, i32> = MAGIC.field("mana");
static SCHOOL: StorageField<PlayerMagic, String> = MAGIC.field("school");

#[component(Load)]
pub fn load_storage() {
    MAGIC.set(SnbtCompound::new().field("mana", 100).field("school", "unbound"));
    MANA.set(100);
    SCHOOL.set("pyromancy");
}
```

Typed fields produce storage commands and conditions:

```rust
MANA.get();
MANA.get_scaled(1.0);
MANA.remove();

TypedExecute::as_players()
    .when(MANA.exists())
    .run(Actionbar::show(Selector::self_(), Text::new("Mana loaded").green()));
```

Build nested paths without handwriting dotted strings:

```rust
let path = NbtPath::root("players")
    .field("self")
    .field("magic")
    .field("mana");
```

Use `StorageVar<T>` for simple legacy variables. Use `RawSnbt` only as an
explicit escape hatch for unsupported, modded, or future NBT shapes:

```rust
MANA.set_raw_snbt(RawSnbt::new("{custom:1b}"));
```
