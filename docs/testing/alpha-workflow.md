# Reproduce the bounded multiplayer alpha scenarios

From a Sand checkout with the pinned Rust toolchain, Java 25+ and a cached
Minecraft 26.3 server jar:

```sh
python3 scripts/validate_alpha.py \
  --jar /path/to/26.3/server.jar \
  --output /tmp/sand-alpha-proof
```

The output directory must not exist. The runner builds the maintained fixtures,
starts disposable headless servers in sequence, connects passive players and
runs the roster and cooperative-arena assertions. It does not launch Prism or
interact with an existing client/world. The harnesses accept the Minecraft EULA
for their disposable test servers; use them only if you accept those terms.
You can obtain the jar using the existing cache tool:

```sh
cargo run --quiet -p sand-build --bin ensure-server-jar -- 26.3
```

The command prints the cached jar path. Pass that path to `--jar`. Artifact
retrieval may need network access. It does not prove the game itself works.

## Results and failures

- `summary.json` records each command, exit status, evidence path and error.
- `roster.log` and `arena.log` preserve build and harness output.
- Each scenario directory retains its pack, runtime logs and `evidence.json`.
- A scenario passes only with exit status zero and passing 26.3 evidence.
- A failed scenario does not prevent the other from running. The command exits
  nonzero if either fails. `complete: false` means the aggregate is unfinished.
- Existing directories are refused so a rerun cannot overwrite previous evidence.

Use `--scenario roster` or `--scenario arena` for a focused reproduction in a
new output directory. Attach the summary and relevant logs when reporting a
failure; retain the source revision and local diff with any evidence you publish.
The runner inherits Cargo configuration and defaults debug information and
incremental compilation off to limit disk use. Shared Cargo locks can delay a run;
the wait is not evidence of a gameplay failure.

## What these scenarios establish

The roster scenario exercises bounded admission, roles, exclusive transfer,
rejection preservation, disconnect/reconnect invalidation, iteration locking and
generation exhaustion. The arena scenario exercises two independently active
arenas with four clients, cooperative rewards, cross-arena isolation, replay,
defeat, repeated reset and reload persistence.

The arena invokes test-only functions that call the shared typed gameplay actions.
It does not drive menu clicks or sneaking over the client protocol. Four clients
do not prove the full eight-member configured capacity under load. The scenario
runner is a repository workflow, not a general `sand test` implementation.

## Remaining alpha acceptance

Use the [reference game's instructions](../../examples/cooperative_arena/README.md)
for a separate ordinary-player playthrough. Record menu selection, sneak attacks,
HUD rendering, results and replay, plus integrated disconnect/death/dimension
policies. Record actual client/version details separately from passive-client
results. Do not infer these outcomes from a passing headless test.

Measure idle, normal, saturated and repeated-reset workloads before publishing an
operating limit. Record hardware/JVM settings, source/pack fingerprints, timing
variance and command-chain behavior. Static runtime advice and admission capacity
are not MSPT guarantees.

Round reset is deliberately narrower than pack cleanup: the example cancels loaded
controllers and online HUDs, preserves saved wins and platforms, and does not
restore a round on reload. Pack-wide cleanup/upgrade tooling must still inventory
owned resources and specify offline-player and unloaded-entity treatment. Do not
remove broad `sand.*` tags, objectives or storage by guesswork: multiple definitions
and packs can coexist, and deleting a datapack does not run cleanup code.

See the [alpha contract](../architecture/alpha-game-contract.md) and
[capability matrix](../architecture/capability-matrix.md) for release scope.
