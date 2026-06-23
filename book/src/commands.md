# Command Lowering

Sand command builders should hide vanilla parser rules from authors.

- `cmd::damage(...)` requires one entity.
- `Damage::new().to(EntityTargets...)` lowers many-target damage through
  `execute as <targets> run damage @s ...`.
- Commands that support many targets directly, such as `tellraw`, keep direct
  many-target output.
- Status effect commands use `EffectId` instead of raw effect strings:
  `cmd::effect_give(Selector::self_(), EffectId::Speed).seconds(10)`,
  `cmd::effect_clear(Selector::self_())`, and
  `cmd::effect_clear_effect(Selector::self_(), EffectId::Regeneration)`.
- Generated branch helpers are deduplicated when equivalent bodies are safe to
  reuse.

Raw commands remain available with `cmd::raw(...)`, but normal gameplay should
prefer typed builders.
