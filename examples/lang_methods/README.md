# Blinkstep

A Sand language pack demonstrating nested struct values and State methods.
Players receive a Blinkstep wand on their first join. Using it moves the player
six blocks east, spends 15 energy, and starts a three-second cooldown. Death
restores energy.

`position.sand` encapsulates coordinates behind public methods. `east(mut self)`
returns a changed copy; `apply(self)` teleports the executor. `hero.sand` keeps a
Position in player State and uses `&self` / `&mut self` methods to read and change
persistent fields. Cooldown seconds are an ordinary runtime function argument.

From the repository root:

```sh
cargo run -p sand-cli -- lang check examples/lang_methods/pack.sand
cargo run -p sand-cli -- lang build examples/lang_methods/pack.sand --output dist/blinkstep
cargo run -p sand-cli -- lang run examples/lang_methods/pack.sand
```

Targets Minecraft Java 26.2. See the [language guide](../../book/src/language.md)
for value-copy, visibility, receiver, and executor rules.
