# Selector Arity

New typed command APIs distinguish selector arity:

- `SingleEntity`
- `EntityTargets`
- `SinglePlayer`
- `PlayerTargets`

```rust
let target = SingleEntity::self_();
let nearby = EntityTargets::nearby(5.0)
    .excluding_players()
    .excluding_self();
let nearest = EntityTargets::all().entity_type("minecraft:zombie").nearest();
```

Use typed wrappers when a command has vanilla target rules. `Selector` remains
available for older APIs and lower-level builders.

## Entity queries and execution-scoped contexts

`sand_core::entity` (re-exported from the prelude) builds cardinality-aware
queries on top of the typed selector wrappers above, and adds typed
relationship traversal so you don't have to hand-write `execute on <relation>`
chains or invent your own temporary tags to keep a reference to an entity
across traversal.

```rust
use sand_core::entity::{EntityQuery, EntityScope};
use sand_core::version::{MinecraftVersion, VersionProfile};

let profile = VersionProfile::resolve(&MinecraftVersion::parse("latest").unwrap()).unwrap();

let cmds = EntityQuery::entities()
    .entity_type("minecraft:arrow")
    .each(|arrow| {
        // `EntityScope::bind` preserves a reference to `arrow` across the
        // relationship traversal below, which reassigns `@s` to the owner.
        EntityScope::bind(arrow, |arrow_ref| {
            arrow_ref
                .owner()
                .if_player(&profile, |owner| vec![owner.add_tag("shot_by_owner")])
                .unwrap_or_default()
        })
    });
```

Key types:

- `EntityQuery<A>` / `PlayerQuery<A>` — filter while cardinality is `Many`;
  `.limit(n)` / `.nearest()` narrow to `One`. `.each(|ctx| ...)` lowers the
  query into `execute as <selector> at @s run function <generated>`.
- `EntityContext<K>` — the **execution-scoped** `@s` handle passed into
  `.each(...)`. It has no meaning outside the generated command chain that
  produced it — it is not a persistent entity reference.
- Typed relationship traversal — `.owner()`, `.leasher()`, `.target()`,
  `.vehicle()`, `.controller()`, `.attacker()`, `.origin()` (single-cardinality,
  call `.if_present(&profile, ...)` or `.if_player(&profile, ...)`), and
  `.passengers()` (many-cardinality, call `.each(&profile, ...)`). Each takes
  the active `VersionProfile` and returns `Err` with an actionable diagnostic
  if the relation predates that profile (e.g. `attacker`/`controller`/`origin`
  on older 1.x releases).
- `EntityScope::bind(&ctx, |scoped| ...)` — tags the currently bound entity
  with a unique, collision-safe temporary tag so `scoped` can still refer to
  it after nested relationship traversal changes `@s`; the tag is removed
  again at the end of the generated function.

This is framework infrastructure only — no generated-function name or tag
scheme here changes any existing datapack output until a feature is built on
top of it.
