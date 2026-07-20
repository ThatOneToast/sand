# Sand Roadmap

Sand is pre-1.0 and evolving; this file tracks genuinely future direction
only. For what's already shipped, see the [book](book/src/introduction.md)
and `CHANGELOG.md`.

## Target versions

Minecraft Java 26.2 is the canonical export/profile target; 1.21.4 is
retained as an explicit oldest-profile/compatibility boundary. See
`sand-version/src/lib.rs` and `docs/architecture/adr-001-crate-boundaries.md`.

## Not yet stable

- **Event system.** `SandEvent` composition (same-cycle chained dispatch,
  multi-parent `after_any`/`after_all`, persistent `while_<E>()` conditions,
  bounded `.within(...)` correlation, advancement-backed graph parents) is
  implemented but not macro-transparent: authors must call
  `EventSetup::with_participants(...)` themselves, nothing auto-merges
  participant capabilities into graph propagation, and there is no typed
  `Event<T>` handler-context accessor yet. Victim, interacted-entity, and
  projectile-owner participant recovery are unimplemented.
- **Resource pack generation** — functional but requires manual asset setup.
- **crates.io publishing** — not yet available; install from the workspace
  (`cargo install --path sand-cli`).

## Next work

- Expand golden export tests to full datapack directory fixtures.
- Add a typed item stack builder with component API.
- Harden dialog actions with typed function references.
- Complete resource pack example crates.
