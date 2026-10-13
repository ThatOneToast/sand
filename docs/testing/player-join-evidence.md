# Player join and reconnect evidence

`OnJoinEvent` observes presence on a Sand tick. It runs after first connection,
reconnection, and datapack load/reload. It is not a network-login callback;
a connection that ends before a Sand tick may never be observed.

## Real-server regression

Run the generated two-pack audit with Minecraft 26.2 and Java 25:

```sh
python3 scripts/validate_player_joins.py --jar "$HOME/.sand/cache/26.2/server.jar"
```

The script exports `sand-core/tests/join_lifecycle_export.rs` through Cargo,
writes each namespace as a separate datapack, and connects two passive clients
to a disposable loopback-only server with RCON disabled. It does not manually
invoke join dispatch or manipulate the private baseline to simulate reconnects.

Each pack has two handlers. The first calls its own generated join check again,
proving the baseline is committed before callbacks. Namespace-specific fake
score holders count each handler independently without sharing detector state.
The audit checks:

- Initial callbacks for both players and both namespaces, followed by unchanged
  counts across five observed game ticks.
- An absent initial leave-game score for Beta and a pre-seeded nonzero score for
  Alpha. A real disconnect increments Alpha's score, and reconnecting increments
  every callback count once while Beta stays connected.
- One pack's generated load lifecycle resetting only its baseline while Alpha is
  offline. Beta's callbacks in the other pack remain unchanged.
- Another real reconnect and a full `/reload`, followed by stable exact counts.
- A preexisting legacy `__sand_join` score neither suppressing callbacks nor being
  reset by the new implementation.

Client logs retain separate files for each reconnect. The harness waits for a
fresh join message for each client session, and reaps each session's processes.
The latest-version vanilla CI workflow runs this audit and retains failure logs.

## Export and lifecycle evidence

The join export fixtures cover deterministic repeated/concurrent exports,
disjoint private objectives between namespaces, baseline-before-callback output,
short-circuiting the first-observation check, deterministic handler order and
repositioning each callback at the player. Separate fixtures reject private
helper and objective collisions, including a same-criterion declaration, and
verify player State initialization precedes joins and owned observation sampling
follows them.

These ordering checks inspect the generated pack. The live fixture does not
exercise arbitrary user State schemas, callbacks that deliberately reset private
state, forced server crashes, or connections shorter than one Sand tick.

A focused vanilla probe also verified that a retained leave-game score advances
on a real disconnect and persists across reconnect. The exported two-pack audit
passes on Minecraft 26.2 after correcting the fixture's counter objective names;
those fixture parser errors were not a failure of the generated join detector.
