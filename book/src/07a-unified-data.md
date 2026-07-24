# Unified Data And Inventory

Sand models a data operation as one typed target plus one NBT path. The target
is storage, an entity, or a block entity; the same `NbtRef<T>` then supports
reads, writes, copies, collection edits, conditions, schema fields, and
inventory snapshots.

```rust,ignore
use sand::prelude::*;

let health = Nbt::entity(Selector::self_()).path("Health");
let first_item = Nbt::block(BlockPos::here()).path("Items[0]");
let cache = Nbt::storage("trail:cache").path("last_item");
```

`path(...)` creates an untyped reference for dynamic vanilla data.
`typed_path::<T>(...)` and schema fields retain an application value type.
This is useful Rust-side information, not a promise that Minecraft will
validate every runtime NBT shape.

## Paths and values

`NbtPath::new(...)` validates ordinary field, key, and list-index syntax when
the command is exported. Build paths structurally with `.field(...)`,
`.key(...)`, and `.index(...)` when convenient. `NbtPath::raw(...)` is the
explicit escape hatch for modded or newly added syntax: it renders unchanged,
cannot receive structural validation, and remains the author's responsibility.

Typed integers, floats, booleans, strings, lists, and `NbtCompound` values
render as SNBT. Raw SNBT remains explicit and opaque; Sand does not parse it.

```rust,ignore
let config = Nbt::storage("trail:config").path("max_level");
config.set(10);
config.get();
config.get_scaled(10.0);
config.remove();

let queue = Nbt::storage("trail:data").path("queue");
queue.append(1);
queue.prepend_from(&config);
queue.insert(2, NbtCompound::new().field("ready", true));
queue.merge(NbtCompound::new().field("owner", "trail"));
```

The typed data command IR represents `get`, `remove`, `merge`, and every
`modify` source/operation until the final renderer. Invalid resource
locations, empty or malformed paths, non-finite scales, invalid list indices,
multi-entity writes, and unsupported command/profile combinations become
structured build diagnostics instead of partial datapacks.

## Inventory locations

`ItemLocation` is the canonical live-item location model. Its entity and block
factories cover selected/main hand, offhand, armor, hotbar, main inventory,
ender chest, generic entity slots, and block-container slots.

```rust,ignore
let player = ItemLocation::entity(Selector::self_());
let selected = player.mainhand();
let helmet = player.helmet();
let hotbar_three = player.hotbar(3)?;

let input = ItemLocation::block(BlockPos::here()).slot(0)?;
```

An `ItemLocation` is live. Its `.nbt()` view is an item-stack snapshot source
for `/data`; copying from one live item location to another uses `/item
replace`, which is the safe vanilla mutation family.

```rust,ignore
let selected = ItemLocation::entity(Selector::self_()).mainhand();
let cache = Nbt::storage("trail:cache").path("last_item");
selected.copy_to(&cache);

ItemLocation::block(BlockPos::here())
    .slot(0)?
    .copy_to(&Nbt::storage("trail:cache").path("input"));

let offhand = ItemLocation::entity(Selector::self_()).offhand();
offhand.replace_from(&selected)?;
```

Arbitrary player/entity inventory NBT writes are rejected: vanilla does not
safely expose them through `/data modify entity`. Block-container NBT writes
are supported. Use `.replace_from(...)` for live inventory mutation,
`.copy_to(...)` for snapshots, and `.copy_from(...)` only where the location
supports NBT writes.

`.matches(...)` and `.is_empty()` lower through typed `execute if items`
conditions; `.exists()` lowers through typed `execute if data`.

## Schema-owned field handles

`PlayerDataSchema` groups initialization while field handles remain the single
source of truth for access. Ordinary Rust methods give fields semantic names
without runtime string lookup or macro attributes:

```rust,ignore
static MANA: ScoreField = ScoreField::new("trail_mana").default(100);
static HAS_WAND: FlagField = FlagField::new("trail_wand").default(false);
static CAST: CooldownField =
    CooldownField::new("trail_cast", Ticks::seconds(3));
static PHASE: GameStateField<BossPhase> =
    GameStateField::with_default_score("trail_phase", 0);

struct PlayerModel;
static PLAYER: PlayerModel = PlayerModel;

impl PlayerModel {
    fn schema(&self) -> PlayerDataSchema {
        PlayerDataSchema::new("trail")
            .score_field(&MANA)
            .flag_field(&HAS_WAND)
            .cooldown_field(&CAST)
            .game_state(&PHASE)
    }

    fn mana(&self) -> &'static ScoreField { &MANA }
    fn has_wand(&self) -> &'static FlagField { &HAS_WAND }
    fn cast_cooldown(&self) -> &'static CooldownField { &CAST }
    fn phase(&self) -> &'static GameStateField<BossPhase> { &PHASE }
}

PLAYER.mana().of("@s").gte(25);
PLAYER.has_wand().of("@s").is_true();
PLAYER.cast_cooldown().of("@s").ready();
```

`define_all()` deduplicates objectives and `init_player(...)` applies defaults
only to missing scores, so reloads and reconnects do not clobber state.
Timer and cooldown ticking still use their existing lifecycle operations.

Storage fields are deliberately named `GlobalStorageField`. Command storage is
pack-global; attaching one to a player schema does not make it per-player.
Sand does not invent an implicit UUID-keyed record format.
