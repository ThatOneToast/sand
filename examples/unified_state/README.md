# Unified State tutorial

This isolated example demonstrates player progression, independent attack,
defense, status, and marker components, nested `StateBundle` composition,
required/optional/forbidden `StateQuery` fields, tick and event systems,
component migration hooks, explicit archetype attachment, and a global
score-plus-typed-data resource.

Run it from this directory:

```sh
cargo check
cargo run --quiet --bin sand_export | jq
```

The export is JSON component records rather than an installed world. For a
server-ready pack, use Sand's normal CLI build flow from a generated project.
