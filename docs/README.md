# Sand Documentation

This directory holds Sand's internal architecture, compiler, and testing
notes — for maintainers and contributors, not the authoring guide.

The [mdBook guide](../book/src/introduction.md) is the single canonical
user-facing documentation tree: tutorials, the manual, and reference pages
for typed state, commands, conditions, events, items, and more.

## Contents

- [Architecture](architecture.md) and [architecture/](architecture/) — crate
  boundaries ([ADR 001](architecture/adr-001-crate-boundaries.md)), the
  compiler/export pipeline, and the event dependency graph.
- [testing/vanilla-limitations.md](testing/vanilla-limitations.md) —
  evidence-linked reference for constraints of vanilla Minecraft itself that
  no amount of Sand typing can remove.

For contributor workflow (checks, toolchain policy, testing commands), see
[`CONTRIBUTING.md`](../CONTRIBUTING.md). For release process, see
[`RELEASE.md`](../RELEASE.md).
