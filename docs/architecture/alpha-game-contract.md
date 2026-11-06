# Alpha game milestone

The alpha milestone is a reproducible, bounded multiplayer game on 26.3, authored
through Sand's canonical public APIs. It is not exhaustive vanilla coverage.
The initial reference game is `examples/cooperative_arena`: two explicitly
registered arenas, at most four online members each, a cooperative custom-health
encounter, private health HUD, saved win counters, and repeatable rounds.

The first roster policy admits or transfers online players, counts spectators
against capacity, invalidates membership on disconnect and reload, and requires
explicit readmission. Roles are an author-defined State enum. Death, dimensions,
game-mode changes and owned-task cancellation are game policy, not implicit roster
behavior. Dynamic session allocation and offline seat reservation are excluded.

## Release gates

- Complete a game and replay it using ordinary survival-player inputs; an operator
  may perform documented initial arena setup in a disposable world.
- Run both arenas independently. Prove membership, damage, rewards, HUD and cleanup
  isolation under capacity rejection, disconnect, reconnect and replayed input.
- Define results, reset, reload and upgrade behavior. Saved wins survive round
  reset; old controllers and membership must never silently resume.
- Validate every critical path on 26.3. Export-only coverage of older targets must
  not imply native parity. Record an actual-client input/rendering smoke test
  separately from passive-client and server-command proofs.
- Measure idle, normal, saturated and reset workloads at the declared population,
  retaining hardware/JVM settings and source fingerprints. Four members per arena
  is an admission bound, not yet a measured server performance guarantee.
- Provide an author-facing scenario run/failure-artifact workflow and complete
  the full repository gate without loosening API or architecture checks.

## Remaining release work

The roster increment and reference-game source do not by themselves finish the
alpha milestone. Pack-wide reset/uninstall needs an owned-resource inventory and
explicit treatment of offline players/unloaded entities. The [repository scenario runner](../testing/alpha-workflow.md) now collects
headless roster/arena failures and evidence. General author-defined scenarios,
default-target coverage expansion, actual game-client evidence and measured
operating limits remain separate required work. Advanced forms,
container workflows, animation blending and general navigation can wait unless a
reference-game requirement demonstrates a need.
