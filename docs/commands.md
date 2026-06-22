# Command Lowering

Sand command builders encode vanilla command shape where practical.

- `/damage` accepts one entity. `cmd::damage(...)` requires a `SingleEntity`.
- `Damage::new().to(EntityTargets...)` accepts many targets and lowers to
  `execute as <targets> run damage @s ...`.
- `tellraw` and effect commands continue to support many targets directly where
  vanilla supports them.
- Generated helper functions are registered under `sand/...` paths and dynamic
  branch helpers with identical bodies are deduplicated when context semantics
  are the same.

The command model is intentionally incremental. Builders that still accept raw
`Selector` should be audited before they are considered fully arity-typed.
Use `cmd::raw(...)` only for commands outside Sand's typed coverage.
