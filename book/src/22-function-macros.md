# 22. Parameterized Functions

Minecraft 26.1 added function macros: a `.mcfunction` line beginning with
`$` can substitute values from an NBT compound supplied by the caller. Sand
models the argument declaration and the call separately so placeholder
spelling is checked before the datapack is written.

```rust,ignore
use sand::prelude::*;

#[function("greet")]
fn greet() -> Vec<String> {
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
fn run_greeting() -> Vec<String> {
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
