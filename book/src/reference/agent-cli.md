# CLI For Coding Agents

Sand's CLI exposes the project identity, API contracts, and validation loop as
stable JSON. These commands are local and deterministic after Cargo and the
selected Minecraft profile are available in the local cache.

Start every automated authoring session from the project directory:

```sh
sand context --format json
```

Check `compatibility.compatible` before trusting API results. Sand compares the
project's resolved `sand` dependency source, Git revision or path, enabled
features, and Minecraft target with the catalog compiled into the CLI. A false
value means the CLI can describe a different API than the project actually
uses. Install the CLI from the project's pinned Sand checkout rather than
guessing across the mismatch.

Use narrow contract queries instead of loading the entire catalog:

```sh
sand api search damage player --all-terms --format json
sand api search score --field path --field summary --kind method --format json
sand api show sand::command::Target::nearby --format json
sand api module sand::entity --format json
sand api module sand::systems::Progression --format json
sand api alternatives "damage entity" --format json
```

Use `module` with a type path to list its constructors, methods, variants and
fields; registered aliases resolve to the canonical type. `show` points to this
member query and prioritizes the queried type’s own members in related APIs.

Search remains deterministic keyword matching at word/identifier starts,
including snake_case and CamelCase boundaries. For example, `quest` matches
quest descriptions but not the interior of `requested_version`. JSON results contain stable
rank and score values, the field/snippet that matched, canonical paths,
availability, result counts, and truncation state. `alternatives` separates
typed authoring APIs from advanced/raw escape hatches and refuses to claim an
alternative when the project/catalog compatibility check fails.

Close the loop with structured build and Sand-specific validation:

```sh
cargo check
sand build --format json
sand check --agent --format json
```

Diagnostics have stable codes and sources. Agent validation fails for missing
configuration, catalog revision/profile mismatch, partial contract coverage,
invalid pack setup, and missing or stale generated output. It also
warns about literal raw-command calls only when the compatible API contracts
produce a likely typed alternative. Compiler diagnostics remain authoritative
for Rust type errors; Sand's check coordinates the additional project and
Minecraft-specific checks.

The same command sequence works in Zed tasks, Claude Code hooks, Codex scripts,
and local-agent tool definitions. Consume JSON fields, not colored terminal
prose, and treat a nonzero exit status as a failed authoring step.

## Discovery limits and a reproducible authoring audit

Generated names from your own State struct cannot be finite installed catalog
entries. `sand api show sand::State` teaches declaration, bound accessors,
associated field constants, initialization and typed queries. For owned systems,
register the complete definition with `#[datapack_component]` before calling
its actions. `sand api show sand::systems::Scene` includes that full workflow.
Export rejects missing static private gameplay helpers, while external author
function references remain an interoperability boundary.

The [fresh-agent experiment](../../../benchmarks/evidence/api-authoring-2026-11-05/README.md)
records a CLI-only authoring attempt, its original independent ratings and
subsequent repairs. A passing `sand check` is not a substitute for native parsing:
the original broad pack exposed both an invalid dialog action and a missing
26.3 root-advancement background during the parent's server smoke test.
