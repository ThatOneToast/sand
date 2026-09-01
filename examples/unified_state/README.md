# A small State-powered RPG

This is the kitchen-sink example, but it is still meant to be readable. It has
player progression, separate attack/defense/status components, nested bundles,
presence-aware queries, tick and attack-event systems, migrations, an
archetype, and one global resource with typed data.

Run it from this directory:

```sh
cargo check
cargo run --quiet --bin sand_export | jq
```

That second command prints JSON component records; it does not install anything
into a world. Use the normal Sand CLI build flow when you want a server-ready
pack.
