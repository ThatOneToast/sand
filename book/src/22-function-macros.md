# 22. Parameterized Functions

Minecraft 1.20.2 added function macros: a `.mcfunction` line beginning with
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
    let values = Nbt::storage("trailforge:runtime").path("greeting");

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
function references as `cmd::call`, plus a typed `NbtRef`; it validates the
function ID, NBT location, and NBT path.

The older `cmd::macro_var`, `cmd::macro_line`, and `cmd::function_with`
helpers remain the explicit unchecked escape hatch for custom or future
syntax. Their output is still version-gated at export: any function macro
line is rejected for Minecraft 1.20.1 and older, and for conservative fallback
profiles whose support Sand cannot prove.
