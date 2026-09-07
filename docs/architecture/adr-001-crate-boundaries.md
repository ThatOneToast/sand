# ADR 001 — Public façade and crate boundaries

Status: accepted

Datapack authors depend on the `sand` façade. It exposes curated authoring
modules, the prelude, procedural macros, and a documented `advanced` export
surface. `sand::__private` is reserved for macro and compiler wiring.

The workspace implementation is split by responsibility:

- `sand-core` owns framework state, ECS/archetypes, events, compiler IR, and
  datapack export;
- `sand-commands` owns typed command construction and rendering;
- `sand-components` owns typed datapack JSON resources;
- `sand-version` owns the Minecraft 26.x+ version model;
- `sand-build` owns code generation and Minecraft data acquisition;
- `sand-macros` owns procedural macros;
- `sand-cli` owns project scaffolding, builds, and local-server workflows.

Dependencies flow from the façade and CLI into these implementation crates.
Macros expand through `sand::__private`, so author projects do not depend on
internal crates directly. Compiler descriptors and registry drains are not
part of the ordinary authoring surface.

Sand has no stable public release. Obsolete APIs are replaced or deleted
instead of retained as aliases or compatibility layers. The canonical test,
example, documentation, and generated-data target is Minecraft Java 26.2;
known 26.x profiles remain exact and pre-26 versions are unsupported.
