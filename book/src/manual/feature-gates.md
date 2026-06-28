# Feature Gates

Optional systems are Cargo features: `systems-damage`, `systems-cooldowns`, `systems-lifecycle`, `systems-player-data`, `systems-movement`, `systems-inventory`, `systems-entities`, and `systems-all`.

```toml
sand-core = { version = "…", features = ["systems-inventory", "systems-movement"] }
```

`systems-player-data` implies lifecycle, but it does not auto-register schemas
or wire generated load/join functions today. Use `PlayerSchema::define_all()`
and `PlayerSchema::init_player(selector)` manually; automatic lifecycle wiring
is future #47/#68 work. Macros such as `SandStorage`, `function`, `component`,
and `event` belong to `sand-macros`, not a feature gate.
