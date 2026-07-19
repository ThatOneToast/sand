# 16. Debugging And Validation

Sand pushes as many mistakes as possible into `cargo build` and `cargo
test`, before a datapack ever reaches a Minecraft server. This chapter
covers how Trailforge validates itself and what to check when something
doesn't compile or doesn't behave as expected in-game.

## Unit-testing generated commands

Trailforge's `#[cfg(test)] mod tests` block doesn't spin up a Minecraft
server — it calls the same functions `sand build` calls, and asserts on the
`Vec<String>` of `.mcfunction` command lines they return:

```rust,ignore
#[test]
fn tick_regenerates_and_warns() {
    let cmds = tick();
    assert!(
        cmds.iter().any(|c| c.contains("Grapple ready")),
        "readiness actionbar: {cmds:?}"
    );
    assert!(
        cmds.iter().any(|c| c.contains("Catch your breath")),
        "damage warning: {cmds:?}"
    );
}
```

Because every `#[component]`/`#[function]`-annotated function is an
ordinary Rust function that *returns* its generated commands, this works
with nothing beyond `cargo test` — no server, no world, no network. This is
the fastest feedback loop available: a broken condition, a wrong selector,
or a typo'd literal shows up as a failing assertion in milliseconds, not as
"nothing happened" when you test in-game later. Write these tests the same
way Trailforge does — assert on *substrings that would only be present if
the logic is right* (a specific command fragment, a specific message),
rather than asserting exact full output, so tests survive incidental
formatting changes upstream.

## `GrappleCore::PREDICATE`, tested directly

Chapter 5's generated predicate gets its own test:

```rust,ignore
#[test]
fn grapple_core_predicate_matches_custom_data() {
    assert!(GrappleCore::PREDICATE.contains("grapple_core"));
    assert_eq!(GrappleCore::BASE, "minecraft:heart_of_the_sea");
}
```

Any pack that generates `#[item]` predicates and then relies on them in
`InventoryChangedTrigger`s or `execute if items` checks (as
`ObtainedGrappleCoreEvent` does) should test the predicate's shape the same
way — it's cheap insurance against a custom-data key typo silently
breaking the item/predicate link.

## Version-aware validation at export time

`__sand_export` (chapter 17) resolves the target Minecraft version before
rendering anything:

```rust,ignore
let resolved = match sand::version::resolve_export_caps(mc_version) {
    Ok(resolved) => resolved,
    Err(e) => {
        eprintln!("sand export failed: {e}");
        std::process::exit(1);
    }
};
```

`resolve_export_caps` looks up the capability profile for the pinned
`mc_version` (`26.2` for Trailforge) and gates every component's rendering
against what that version actually supports — a component using a feature
that doesn't exist on the target version fails export with a clear error
instead of producing JSON the target server rejects (or worse, silently
misinterprets) at datapack load time. This is why Sand's canonical fixtures
and this book both target Minecraft Java 26.2 specifically (see
`docs/architecture/adr-001-crate-boundaries.md`): version-awareness is only
useful if the book's own examples are validated against a real, current
profile rather than an assumed-compatible one.

## Codegen failures

`build.rs`'s `sand_build::generate("26.2")` call downloads and caches
version-specific Minecraft data on first build. If that fails (no network,
firewalled CI), the default behavior is to warn and continue from cached
data rather than hard-failing — set `SAND_STRICT_CODEGEN=1` to make codegen
failures fatal, which is the right setting for CI pipelines that should
never silently build against stale cached data.

## In-game validation: `/reload` and function tags

Once a pack is running (chapter 17), the fastest debug loop for gameplay
logic itself (as opposed to compile-time logic) is editing `src/lib.rs`,
re-running `sand build`, and issuing `/reload` in-game — Sand's `load`
component re-runs automatically as part of vanilla's own reload behavior
(chapter 3), so state definitions and storage seeding re-apply without a
full server restart. Use `/function trail:menu` (or any other function
directly) to invoke a specific piece of logic in isolation while
debugging, bypassing whatever event or tick condition would normally
trigger it.
