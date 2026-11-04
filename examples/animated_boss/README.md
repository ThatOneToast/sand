# Animated boss encounter

A custom-health boss combines `Combat`, `Encounter`, `Sequence`, `Scene`, `Job`
and canonical State. Gameplay contains no handwritten Minecraft commands.

Build with `sand build`. In a test world, create an actor near a player:

```mcfunction
summon minecraft:marker ~ ~ ~ {Tags:["boss_demo"]}
execute as @e[tag=boss_demo,sort=nearest,limit=1] at @s run function animated_boss:start
```

The boss starts with 100 custom health, 20 shield, 20% resistance and a five-tick
invulnerability window. Call `animated_boss:hit` as a nearby player, or bind that
function to your item/input event. Each accepted ten-point attack resolves through
the shared combat policy. Boss health is State, independent of native Health NBT.

The normal attack charges an animated amethyst telegraph for forty loaded ticks,
then deals four native magic damage to the nearest player within six blocks.
Below half health it alternates with a burst dealing three native magic damage
to up to eight nearest players within ten blocks. Both attacks attribute native
damage to the boss actor. Each child timeline includes recovery; the controller
waits for children to settle before selecting another attack.

Defeat, cancellation, a five-minute deadline, or no player within 24 blocks ends
the encounter and cleans up all child work. `animated_boss:cancel`, executed as
an owner, cancels only that owner's encounter. Calling `animated_boss:start`
resets its health, shield and attack progression. Start multiple actors to
exercise independent ownership.

Sphere queries test player origins. Displays have no gameplay hitboxes. This
example does not implement pathfinding, threat tables, native melee interception,
or custom player health. Job/query budgets bound selected work, not server MSPT.
Unloaded tasks pause and clean up when observable again. The default target is
26.3; tests also export 26.1 and 26.2.
