# Event / Dialog / Function-Ref Repair Audit

Audit run: 2026-06-22  
Branch: main  
Starting commit: 3fab2c7

---

## Classification key

| Code | Meaning |
|---|---|
| A | Real API bug |
| B | Docs-only drift |
| C | Backward-compatible legacy API |
| D | Escape hatch that should remain |
| E | Missing namespace inference |
| F | Missing typed builder / ergonomic API |
| G | Missing golden test |

---

## Issues found

### Event<E> is overloaded as both builder and handler context

**Files:** `sand-core/src/event/mod.rs`  
**Class:** A  

`Event<E>` carries `advancement_id: String` and `handler_function: String` — it is the
advancement builder. But the intended UX makes `Event<E>` the zero-cost handler context
inside `#[event]` functions:

```rust
// Current (wrong user-facing API — docs show this):
let event = Event::<DrankHoney>::new("my_pack:drank_honey", "my_pack:handler_drank_honey");
let advancement = event.into_advancement();

// Target (handler context only):
#[event]
pub fn drank_honey(event: Event<DrankHoney>) {
    MANA.add(event.player(), 25);
}
```

**Fix:** Add a `PhantomData`-only `Event<E>` context type alongside or replacing the
builder. Move the advancement-building responsibility to an internal type
`EventAdvancement<E>` used only by the macro-generated code.

---

### EventHandle is stringly typed

**Files:** `sand-core/src/event/handle.rs`  
**Class:** A  

`EventHandle::new("my_pack:on_kill")` requires a full `namespace:path` string at
definition site. The `reset(advancement_id, selector)` method requires repeating the
advancement ID string. Neither is type-safe.

```rust
// Current:
static MY_EVENT: EventHandle = EventHandle::new("my_pack:on_kill");
MY_EVENT.reset("my_pack:on_kill", "@s");

// Target:
EventHandle::<AteGoldenApple>::reset(event.player());
// or:
static MY_EVENT: EventHandle<AteGoldenApple> = EventHandle::new();
MY_EVENT.reset("@s");
```

**Fix:** Make `EventHandle<E: AdvancementEvent>` generic; infer the advancement ID from
the event type at export time.

---

### Function pointer refs emit bare paths without namespace resolution

**Files:** `sand-core/src/function.rs`, `sand-macros/src/lib.rs`  
**Class:** A + E  

`FunctionPointerEntry::path` stores the bare path (or full `ns:path` if the user wrote
it explicitly). `IntoFunctionRef for fn() -> Vec<String>` emits `function {path}` —
which is `function show_mana_gain` (no namespace) for a bare `#[function]`.

```rust
// Current — bare path ends up in generated function call:
#[function]
pub fn show_mana_gain() { ... }
// → registers path = "show_mana_gain"
// → cmd::call(show_mana_gain) → "function show_mana_gain"  ← INVALID Minecraft command
```

**Fix:** At export time (or via placeholder), resolve bare paths through the pack
namespace from `sand.toml`. Introduce a placeholder token like `__sand_ns__:path` that
is substituted to `<namespace>:path` during the export pass.

---

### #[function("arcane:cast_dash")] requires explicit namespace

**Files:** `examples/arcane_starter.rs`, user code  
**Class:** E  

Users are currently expected to write `#[function("arcane:cast_dash")]` for nested
paths. The namespace should come from `sand.toml` `[pack].namespace`. Only the path
component should be user-provided:

```rust
// Current:
#[function("arcane:cast_dash")]
pub fn cast_dash() { ... }

// Target:
#[function("cast_dash")]   // or #[function("magic/cast_dash")] for nested
pub fn cast_dash() { ... }

// External override (explicitly opt-in):
#[function(external = "other_pack:api/do_thing")]
pub fn external_bridge() { ... }
```

---

### ConsumeItemTrigger / PlayerKilledEntityTrigger accept raw serde_json::Value

**Files:** `sand-core/src/event/trigger.rs`  
**Class:** A + F  

All trigger builders' `.item()`, `.entity()`, `.killing_blow()` etc. methods accept
`impl Into<serde_json::Value>`. This forces users to write raw JSON predicates:

```rust
// Current:
ConsumeItemTrigger::new().item(serde_json::json!({"items": "minecraft:golden_apple"}))

// Target:
ConsumeItemTrigger::new().item(ItemPredicate::id(Item::GoldenApple))
// Escape hatch:
ConsumeItemTrigger::new().item(ItemPredicate::raw_json(json!({...})))
```

**Fix:** Add `ItemPredicate`, `EntityPredicate`, `DamagePredicate` typed builder structs.
Keep `raw_json(Value)` escape hatches. The typed structs should impl `Into<Value>`.

---

### Condition lacks ergonomic chaining methods

**Files:** `sand-core/src/condition.rs`  
**Class:** F  

`Condition::all([a, b, c])` works but is not fluent. There is no `.and()`, `.or()`,
`.and_not()`, `.or_not()` on `Condition` values.

```rust
// Current:
Condition::all([
    MANA.of("@s").gte(25),
    DASH.ready("@s"),
    !CASTING.of("@s").is_true(),
])

// Target:
MANA.of("@s").gte(25)
    .and(DASH.ready("@s"))
    .and_not(CASTING.of("@s").is_true())
```

---

### Dialog uses raw strings for IDs, titles, labels, and actions

**Files:** `sand-components/src/dialog/mod.rs`  
**Class:** A + F  

```rust
// Current (user API in docs/examples):
Dialog::notice("example:welcome")
    .title("Welcome")
    .button(DialogButton::new("Start")
        .action(DialogAction::run_command("/function example:start")));

// Target:
#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::notice()
        .title(Text::new("Welcome").gold())
        .button(
            DialogButton::new(Text::new("Start").green())
                .action(DialogAction::run_function(start_tutorial))
        )
}
```

Issues:
- `Dialog::notice("example:welcome")` — ID should be inferred from function name
- `.title("Welcome")` — should accept `Text` builder
- `DialogButton::new("Start")` — should accept `Text`
- `DialogAction::run_command("/function example:start")` — should use `run_function(fn_ptr)`

---

### Two parallel event systems: sand_core::events vs sand_core::event

**Files:** `sand-core/src/events/mod.rs`, `sand-core/src/event/mod.rs`  
**Class:** A  

`sand_core::events` contains `SandEvent`, `OnJoinEvent`, `ItemConsumeEvent`, etc.  
`sand_core::event` contains `AdvancementEvent`, `Event<E>`, `EventHandle`, trigger builders.

Users must know which module to import from. Event types have verbose `Event` suffix
(`OnJoinEvent`, `EntityKillEvent`) inconsistent with the target API (`OnJoin`, `EntityKill`).

**Fix:**  
- Preferred module: `sand_core::event::vanilla::{OnJoin, FirstJoin, OnDeath, ...}`  
- Old names: `sand_core::events::{OnJoinEvent, ...}` stay as `#[deprecated]` re-exports  
- `prelude` exports new names

---

### EventHandle book/docs teach stringly API

**Files:** `book/src/events.md`  
**Class:** B  

```markdown
static GOLDEN_APPLE_HANDLE: EventHandle = EventHandle::new("my_pack:on_ate_golden_apple");
#[event(dispatch = "advancement")]
```

The book still shows the stringly `EventHandle::new("my_pack:…")` pattern and the
`dispatch = "advancement"` attribute form instead of the typed `Event<E>` form.

---

### README teaches Dialog::run_command("/function …")

**Files:** `README.md`  
**Class:** B  

```rust
.action(DialogAction::run_command(cmd::function(
    ResourceLocation::new("example", "start").unwrap(),
)))
```

Raw `ResourceLocation::new` + `cmd::function` should not be the taught path for local
function references from dialog buttons.

---

### IntoFunctionRef for &str / String allow bypassing namespace resolution

**Files:** `sand-core/src/function.rs`  
**Class:** C  

`impl IntoFunctionRef for &str` lets `cmd::call("example:my_fn")` compile. This bypasses
namespace inference and teaches the wrong pattern. Should be deprecated with a note pointing
to `cmd::call(local_fn_pointer)` or `cmd::function(FunctionRef::external("…"))`.

---

### Missing golden tests for namespace inference

**Files:** (none yet)  
**Class:** G  

No test verifies that `cmd::call(local_fn_pointer)` emits `function <namespace>:fn_name`
(with namespace from export config) rather than a bare `function fn_name`.

---

## Summary table

| # | Issue | Class | Status |
|---|---|---|---|
| 1 | `Event<E>` is builder not handler context | A | Open |
| 2 | `EventHandle` is stringly typed | A | Open |
| 3 | Function pointers emit bare paths | A+E | Open |
| 4 | `#[function]` requires explicit namespace | E | Open |
| 5 | Trigger builders accept raw `Value` | A+F | Open |
| 6 | No condition chaining methods | F | Open |
| 7 | Dialog IDs/labels/actions use strings | A+F | Open |
| 8 | Two parallel event module hierarchies | A | Open |
| 9 | Book teaches stringly `EventHandle` | B | Open |
| 10 | README teaches `Dialog::run_command("/function")` | B | Open |
| 11 | `IntoFunctionRef for &str/String` bypass namespace | C | Open |
| 12 | No golden test for namespace inference | G | Open |
| 13 | `WhenBuilder::and_then` ignored condition | A | **Fixed (3fab2c7)** |

## Planned escape hatches to keep

| API | Reason |
|---|---|
| `FunctionRef::external("other:path")` | Cross-datapack calls need explicit namespace |
| `DialogRef::external("other:dialog")` | Same |
| `ItemPredicate::raw_json(json!({...}))` | Unlisted future predicates |
| `Trigger::custom("minecraft:trigger").raw_conditions(json!(...))` | Unlisted triggers |
| `cmd::raw("...")` | True escape hatch, already well-named |
| `Condition::predicate("my_pack:can_cast")` | Predicate refs by string are valid |
