# Command Lowering

Sand command builders should hide vanilla parser rules from authors.

- `cmd::damage(...)` requires one entity.
- `Damage::new().to(EntityTargets...)` lowers many-target damage through
  `execute as <targets> run damage @s ...`.
- Commands that support many targets directly, such as `tellraw`, keep direct
  many-target output.
- Generated branch helpers are deduplicated when equivalent bodies are safe to
  reuse.

Raw commands remain available with `cmd::raw(...)`, but normal gameplay should
prefer typed builders.
