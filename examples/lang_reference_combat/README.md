# Marked combat

A Sand language pack demonstrating borrowed struct methods, retained attackers,
entity-State attachment, runtime timer arguments, recipes and vanilla kill credit.

Use the marking wand near zombies. Each mark stores its caster's `EntityRef` on
the zombie, then deals six half-hearts after twenty ticks. A later cast replaces
an existing mark's owner and timer. The caster receives vanilla kill credit when
the damage is lethal, and the kill event updates their personal score. If the
caster is offline, the attack is skipped and the mark is cleared. Entity timers
advance while their entities are loaded.

`Spell.empower(&mut self)` updates its mutable local receiver. `Mark.owner`
retains the caster after the casting function returns and across pack reloads.
This uses the same pack-owned identity allocator as Sand's Rust gameplay runtime.

From the repository root:

```sh
cargo run -p sand-cli -- lang check examples/lang_reference_combat/pack.sand
cargo run -p sand-cli -- lang build examples/lang_reference_combat/pack.sand --output dist/marked_combat
cargo run -p sand-cli -- lang run examples/lang_reference_combat/pack.sand
```

Targets Minecraft Java 26.2. See the [language guide](../../book/src/language.md).
