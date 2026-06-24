# Typed Execute

Use `TypedExecute` for entity-context execute chains. Use `when` / `unless` / `if_`
for in-function conditional branches.

## TypedExecute (entity context chaining)

```rust
#[function]
pub fn show_ready() {
    TypedExecute::as_players_at_self()
        .when(all![MANA.of("@s").gte(25), DASH.ready("@s")])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Dash ready").aqua(),
        ));
}
```

When an `any!` condition expands to multiple execute plans, Sand emits one
command per plan.

## when / unless / if_ (in-function conditional branches)

These emit conditional logic directly inside a function body:

```rust
#[function("powers:grant_enhanced_cells")]
pub fn grant_enhanced_cells() {
    cmd::say("Enhanced cells was called");

    if_(HAS_ENHANCED_CELLS.of("@s").is_true())
        .then_all(mcfunction![
            cmd::tellraw(Selector::self_(), Text::new("You already have enhanced cells").red());
            cmd::return_fail();
        ])
        .else_all(mcfunction![
            cmd::attribute_base_set(Selector::self_(), "minecraft:max_health", 40.0);
            cmd::tellraw(Selector::self_(), Text::new("Granted enhanced cells!").green());
            HAS_ENHANCED_CELLS.enable("@s");
            cmd::return_cmd(0);
        ]);
}
```

Generated parent function (`grant_enhanced_cells.mcfunction`):
```mcfunction
say Enhanced cells was called
execute if score @s … matches 1 run function powers:sand/branches/0
execute unless score @s … matches 1 run function powers:sand/branches/1
```

Branch 0 (`sand/branches/0.mcfunction`):
```mcfunction
tellraw @s {"text":"You already have enhanced cells","color":"red"}
return fail
```

Branch 1 (`sand/branches/1.mcfunction`):
```mcfunction
attribute @s minecraft:max_health base set 40.0
tellraw @s {"text":"Granted enhanced cells!","color":"green"}
scoreboard players set @s … 1
return 0
```

The flag mutation (`enable("@s")`) inside the else branch does **not** prevent
`return 0` from running, because all commands are inside one helper function
evaluated under one condition check.

## Score expressions

For compound integer math, use `expr()`. Sand writes the required scoreboard
temporary operations immediately before the branch check; Minecraft cannot
evaluate arithmetic inside `execute if score` directly.

```rust,ignore
when(MANA.of("@s").expr().minus(COST.of("@s")).gte(0))
    .then_all([MANA.of("@s").sub_score(COST.of("@s")), cmd::call(cast_spell)]);
```

This lowers to a copy into Sand's `__sand_tmp` objective, a `-=` operation, and
then `execute if score @s __sand_tmp matches 0..`. The temporary objective is
created automatically in `__sand_score_init` during `minecraft:load`.

## Return behavior

| Context | `cmd::return_fail()` / `cmd::return_cmd(n)` |
|---|---|
| Inside `then_all` / `else_all` branch | Stops the **branch function** and returns to parent |
| Inside `then_one` (direct execute) | Inline `return` in the condition chain |

The parent function continues after the execute-branch line. If you need the
parent to exit after a branch matches, gate subsequent logic on the same
condition or use another branch.
