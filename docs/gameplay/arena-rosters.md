# Bounded arena rosters

`Roster<Role>` supplies explicit online membership for a registered arena. It
composes with `Session`, `Lifetime` and ordinary State rather than owning another
save format. Use one definition per arena; this is not dynamic matchmaking.

```rust,ignore
#[derive(Clone, Copy, Debug, PartialEq, Eq, EntityStateEnum)]
enum Role { Spectator = 0, Red = 1, Blue = 2 }

#[derive(State)]
#[state(namespace = "arena", scope = player)]
struct Member {
    arena: Score,
    role: EntityEnum<Role>,
}

#[datapack_component]
fn roster() -> Roster<Role> {
    Roster::new("arena:match".parse().unwrap(), Member::arena, Member::role, 8)
}
```

Import the normal prelude and `sand::systems::Roster`. Register the definition
before calling its actions. `open()` begins a fresh generation; `close()`
invalidates it. Reload closes all rosters. The generation allocator persists and
refuses to wrap. Membership must be a default-zero, non-ticking player Score with
bounds covering every positive i32 generation. Role must be a canonical player
enum. Capacity is 1–128 online members, including spectators.

`join(Role::Red)` runs as the player and checks attached State, the current arena
and capacity before writing. Reusing the membership field across arenas gives
exclusive membership: successful admission transfers the player; failed admission
preserves their previous membership and role. An existing valid member can change
role when full. `leave()` affects only this arena. Death and dimension changes do
not implicitly leave; author that game policy using events and conditions.

Use `contains()` or `is(Role::Red)` for authorization. A stored nonzero membership
score alone is not authoritative. Disconnect changes the native leave-game
statistic, immediately invalidating the saved connection snapshot. Reconnect
requires fresh admission, so an old member cannot revive an over-capacity roster.
Offline seats are not reserved. Invalidated scores may remain until an explicit
State reset or later admission; their presence cannot authorize gameplay.

`has_members(Some(Role::Red))` works from server or entity context. It stops after
finding the first valid member. `each(Some(Role::Red), |player| ...)` visits current
online members at their own positions. Both scan online players on demand; the
capacity bounds successful membership, not the cost of scanning a large server.

During `each`, roster mutation and nested iteration sharing that membership field
are rejected across imported definition namespaces. The lock is export-pack
scoped. Callback returns cannot strand it. This makes team damage/reward iteration
stable, but it means a defeat callback cannot synchronously change roster roles
inside `each`. Commit custom health first and transition defeated players in a
subsequent player system/event. Do not bypass this contract by writing the fields
directly inside callbacks.

Rosters do not cancel gameplay, teleport players, assign native scoreboard teams,
or change gamemodes. Stop owner work explicitly on leave/transfer and make owned
work validate current membership. Likewise, cancel round work before reopening a
roster. A reset is not an uninstall: unloaded task cleanup follows Lifetime's
existing documented contract.

See [the cooperative arena example](../../examples/cooperative_arena/README.md).
Export regressions cover 26.1/26.2/26.3. The native roster harness targets 26.3:

```sh
python3 scripts/validate_roster.py --jar /path/to/26.3/server.jar --output /tmp/roster-proof
```
