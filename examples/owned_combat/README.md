# Owned combat

A complete player ability using `sand::systems::{Ability, Lifetime, Projectile,
StatusEffect}` and canonical State. The default target is 26.3; export tests also
cover 26.1 and 26.2.

From this directory, run `sand build` and install the resulting pack. With two
players present, face the other player and start sneaking to launch a fireball.
The spell costs 20 mana, deals 6 native magic damage on an entity hit and has a
three-second cooldown. Mana recovers by one each second. The 1.75-block sampling
radius lets an eye-height cast reach the target player position used by the
proximity check.

Player commands:

- `/function owned_combat:cast` activates the same ability directly.
- `/function owned_combat:regenerate` applies five seconds of mana regeneration;
  reapplying refreshes the effect.
- `/function owned_combat:cancel` removes your active projectiles and regeneration.
- `/function owned_combat:diagnose` explains failed ability requirements and capacity.

Each projectile owns its lifetime. Missing/dead owners, State detachment,
explicit cancellation, expiry and reload use framework cleanup. Death and join
event handlers also invalidate unloaded work across respawn or reconnect.
Unloaded instances clean up when they next load. Collision is bounded quarter-block
sampling; it is not exact swept collision. The example selects player targets
and uses vanilla damage rules, including armor and immunity.

The library's registration functions and event/system handlers are the project
entry points. No application object or mandatory shutdown callback is required.
See [owned gameplay](../../book/src/23d-owned-gameplay.md) for lifecycle policies,
system dependencies, numeric lowering and native gameplay tests.
