# 4. Resource IDs And Namespaces

Everything Sand exports — functions, items, recipes, dialogs, advancements —
lives at a `namespace:path` **resource location**, exactly like vanilla
Minecraft's own IDs (`minecraft:diamond_sword`, `minecraft:tick`). Sand
validates these at construction time rather than letting a malformed ID
reach the generated datapack.

## The pack namespace

Trailforge's `sand.toml` sets the pack's namespace once:

```toml
[pack]
namespace   = "trail"
description = "Trailforge — upgradeable equipment and traversal, built with Sand"
mc_version  = "26.2"
```

Every `#[function]`, `#[component]`, and `#[item]` in `src/lib.rs` is
exported under this namespace unless you give it an explicit ID string, as
`trail:grapple` does:

```rust,ignore
#[function("trail:grapple")]
pub fn grapple() { /* ... */ }
```

## `ResourceLocation`

When Trailforge needs to reference a function or resource programmatically
— not just at the attribute-macro definition site — it constructs a
`ResourceLocation` directly:

```rust,ignore
cmd::function(ResourceLocation::new("trail", "grapple/execute").unwrap())
```

`ResourceLocation::new` validates both segments against Minecraft's
namespace/path character rules (lowercase ASCII, digits, underscore,
hyphen, dot, and `/` in the path) and returns `Err` for anything else — so
a typo'd or illegal namespace fails at pack-build time in Rust, not as a
silent no-op `/function` call in-game. Trailforge's nested path
(`grapple/execute`) is exactly what you'd write by hand in a vanilla
datapack's function tree; Sand just makes the reference typed.

## `ItemId` and storage namespaces follow the same rule

The same validation applies to item IDs and storage locations:

```rust,ignore
CustomItem::new(ItemId::minecraft("leather_boots").unwrap())
```

```rust,ignore
static GRAPPLE_RANGE: StorageVar<i32> = StorageVar::new("trail:data", "config.grapple_range");
```

`ItemId::minecraft(...)` is shorthand for `minecraft:leather_boots` —
Trailforge upgrades a *vanilla* base item rather than inventing a brand-new
one, since Trail Striders need to occupy the boots equipment slot and carry
vanilla armor rendering. `StorageVar::new("trail:data", "config.grapple_range")`
targets a command-storage NBT location at `trail:data`, an entirely separate
namespace from function/item resource locations but validated the same way.

## Why this matters for a growing pack

As Trailforge (or your own pack) grows past a handful of functions, keeping
every resource under one namespace is what lets `/function trail:menu`,
loot tables, and advancement triggers cross-reference each other reliably.
Sand's compile-time validation catches the class of bug where a hand-typed
`"trail:grapple/exceute"` (a typo) would otherwise only surface as "nothing
happened" when a player runs the dash in-game.
