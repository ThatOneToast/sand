# Custom health drill

Build with `sand lang build examples/lang_state_combat` for Minecraft 26.2.
Run `/function scout_combat:take_hit` as a player. Resistance reduces the hit
and the shield absorbs damage before State health changes. Run `heal` to
recover up to the configured maximum. `reset_drill` restores the training
values after defeat. The drill uses its own State health. Vanilla health,
armor and damage events are separate gameplay.
