# Feature Gates

Optional systems are Cargo features: `systems-damage`, `systems-cooldowns`, `systems-lifecycle`, `systems-player-data`, `systems-movement`, `systems-inventory`, `systems-entities`, and `systems-all`.

```toml
sand-core = { version = "…", features = ["systems-inventory", "systems-movement"] }
```

`systems-player-data` implies lifecycle. Macros such as `SandStorage`, `function`, `component`, and `event` belong to `sand-macros`, not a feature gate.
