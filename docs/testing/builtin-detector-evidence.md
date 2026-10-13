# Built-in detector ownership

Death, respawn, XP level-up, and armor history belong to the exporting datapack
namespace. Installing another Sand pack with the same subscriptions must not
consume an event before this pack observes it.

Death and respawn use namespace-owned vanilla criteria and a per-player phase.
They share the player lifecycle with joins and owned observations. The respawn
check precedes new death observation, and the pending phase is consumed before
callbacks. XP retains current, previous, delta, and first-observation scores for
each namespace; the first sample establishes a silent baseline. XP decreases do
not dispatch. Existing XP baselines survive reload and reconnect.

The typed `PlayerLevelUpEvent` snapshot helpers accept the callback's player
target and return a `StatCurve`. Their logical detector identity survives cached
Rust declarations and resolves to physical storage only during export. These
snapshots compose with numeric conditions and State assignments; they are not
live experience queries outside the XP event lifecycle.

Armor watches retain exact slot/item/custom-data grouping. Namespace-qualified
identities use the shared deterministic collision allocator. Identical watches
within one pack share history; separate packs own independent history.

## Existing worlds

The old global `__sand_dc`, `__sand_rp`, `__sand_tsd`, `__sand_xp_*` objectives,
`__sand_just_died`, and unqualified armor history tags are left untouched. Sand
cannot establish ownership of those names in an existing world. New private
objectives begin their normal first-observation lifecycle; pending state is not
copied from ambiguous global storage. In particular, upgrading establishes a
fresh XP baseline and fresh armor history. Other packs may still use the old
state.

## Reproducible server audit

```sh
python3 scripts/validate_detector_ownership.py --jar "$HOME/.sand/cache/26.2/server.jar"
```

The script exports the real `builtin_detector_ownership` Cargo fixture twice,
writes both packs without changing their generated commands, and connects two
passive clients to a disposable Minecraft 26.2 server. A final vanilla tick-tag
pack supplies the generated tick entries in explicit order. A second disposable
server repeats the audit with the opposite order.

For each player, the audit changes XP levels, equips and removes a real helmet,
and kills the player. The passive client requests respawn in response to the
actual vanilla death packet. Namespace-specific counters must report exactly one
level-up, equip, unequip, death, and respawn in both packs. Other-player counters
must remain unchanged. Counts must remain stable across five further game ticks.
The audit also seeds old global scores and tags and verifies they remain intact;
the vanilla time-since-death statistic may naturally increase.

Export regressions check disjoint bounded objective names, deterministic
concurrent exports, generated callback ordering, real armor hash collisions,
private helper/objective collision rejection, and cached XP expressions reading
their exporting namespace's detector scores. The latest-version vanilla workflow
runs the server audit and uploads diagnostics on failure.

This audit covers tick-observed events with connected clients. It does not prove
callbacks for connections or complete death/respawn cycles shorter than a Sand
observation interval, nor callback code that deliberately modifies private state.

## Observed before and after

On the unfixed compiler, the export regression fails because the two namespaces'
objective sets overlap. The actual Minecraft 26.2 audit then gives Alpha five
levels: `detecta` receives its callback, while `detectb` never reaches one callback
and the assertion times out. This is a lost event, not a parser or client failure.

With owned detectors, the same audit passes both tick orders, all five event
kinds for both players, stable counts, and the legacy-state assertions. A separate
protocol probe also completed two actual deaths and client-requested respawns
while the other player remained alive; the protocol unit tests cover opt-in
respawn requests and teleport acknowledgment after respawn.
