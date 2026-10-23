# 22. Parameterized Functions

Minecraft 26.1 added function macros: a `.mcfunction` line beginning with
`$` can substitute values from an NBT compound supplied by the caller. Sand
models the argument declaration and the call separately so placeholder
spelling is checked before the datapack is written.

```rust,ignore
use sand::prelude::*;

#[function("greet")]
fn greet() {
    let args = FunctionMacroArgs::new(["player", "count"]).unwrap();
    let player = args.variable("player").unwrap();
    let count = args.variable("count").unwrap();

    vec![
        args.line(format!("say Hello, {player}!")).unwrap(),
        args.line(format!("give {player} minecraft:diamond {count}"))
            .unwrap(),
    ]
}

#[function("run_greeting")]
fn run_greeting() {
    let args = FunctionMacroArgs::new(["player", "count"]).unwrap();
    let values = Nbt::storage(ResourceLocation::new("trailforge", "runtime").unwrap())
        .path("greeting");

    vec![args.call_with(greet, &values).unwrap()]
}
```

The generated function body and call are:

```mcfunction
$say Hello, $(player)!
$give $(player) minecraft:diamond $(count)
function trailforge:greet with storage trailforge:runtime greeting
```

`FunctionMacroArgs::new` rejects empty, malformed, and duplicate names.
`variable` rejects an undeclared name, and `line` scans the complete command
for undeclared, malformed, or unterminated `$(name)` placeholders.
`cmd::try_call_with` accepts the same registered function pointers and typed
function references as `cmd::function`, plus a typed `NbtRef`; it validates the
function ID, NBT location, and NBT path.

`cmd::macro_var`, `cmd::macro_line`, and the function-name argument to
`cmd::function_with` are explicit unchecked escape hatches for custom or future
syntax; `function_with` still requires a typed `NbtRef`. Function macros are part
of Sand's 26+ command baseline; there is no pre-26 lowering or rejection path.
Builds that require exact generated Minecraft data still reject unverified
future profiles.

## Player execution handles

A player-context function may receive one authoring-time parameter:

```rust,ignore
use sand::prelude::*;
use sand::events::OnJoinEvent;

#[function("refresh_supplies", context = player)]
fn refresh_supplies(player: EntityContext<PlayerKind>) {
    Supplies::on(player).diamonds.set(player.inventory().count(vanilla::Item::Diamond));
}

#[on_event]
fn joined(event: Event<OnJoinEvent>) {
    event.player().call(refresh_supplies);
}
```

`Supplies` is a player-scoped `State` with a numeric `diamonds` field. Events,
player queries, and injected function parameters all supply the same
`EntityContext<PlayerKind>`. The function macro still produces a zero-argument
Rust function item usable as a canonical `FunctionRef`; the parameter is
injected while building the datapack, never passed at Minecraft runtime.
Functions without the parameter remain supported. Server-context functions
cannot request a player parameter, and generic entity handles cannot use the
player `call` capability.

`player.call` returns `Actions` that retain the registered function identity
until export. Export rejects unknown/external IDs and reads the existing
`ExecutionContext` metadata. Player callees run behind a current-executor player
guard; server-compatible callees inherit the current executor too. Neither call
changes position, rotation, dimension, or anchor. Player entry initialization
still uses the canonical State lifecycle.

The handle means the current `@s` where its generated actions execute; it is not
a captured or persistent player identity. Use `EntityScope::bind` when retaining
an entity across relationship traversal. `EntityContext::default()` remains an
explicit assumption, not an unforgeable proof: it selects nothing and verifies
no kind. Likewise manually constructed event contexts do not establish an
executor. Direct external Minecraft invocation must establish the declared
context itself. Sand does not statically analyze arbitrary Rust factories or
raw commands; existing advanced `cmd::function` and raw calls retain their
separate documented boundary.

`Target` answers which entities to select; the execution handle exposes the
capabilities of the current entity; `State` owns its persistent gameplay data.
Use `Target::from(player)` when crossing from gameplay handles to typed command
builders. `EntityContext::target()` continues to mean relationship traversal.
