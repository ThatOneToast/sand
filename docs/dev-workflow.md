# Local watch/build/reload foundation

Run `sand dev --offline --ram 2G` in a Sand project (omit `--offline` for
normal account authentication). The initial build and server preparation use
`sand run`'s profile, jar, world-reset policy, console classifier, and sync
pipeline. Both Minecraft and RCON bind `127.0.0.1`; this command is for local
clients. Existing server-port settings are preserved. The dev session disables empty-server
pausing so load functions run even without a connected player, and restores the
previous pause setting during cleanup.

Inputs are captured before the initial build (and before each restart build),
so edits during compilation queue another build after startup.
After the ready banner, content polling checks project files every 250 ms and
waits for 500 ms without changes before building. Creation, deletion, rename,
and equal-mtime edits count. Only root `dist`, `target`, and `.git` directories
are excluded: `src/target` and nested resource `dist` directories are watched.
Symlink targets and dependencies outside the project root are not watched.
Polling is portable Rust filesystem code; this slice has local macOS evidence,
not a claim of completed Windows/Linux acceptance testing. Large projects may
need a narrower/configurable watcher in follow-up work.

Each rebuild invokes the existing `sand build --format json --profile …`
pipeline. A failed build leaves the installed datapack untouched. A successful
build stages the existing incremental mirror, swaps it into the managed pack
path, and sends RCON `reload`. It does not touch neighboring packs. Copy failures
preserve the live pack; interrupted swaps retain a `.sand-dev-backup` directory
for manual recovery and block further installation. The server console stays
responsive while compilation runs. Quit reaps the CLI build worker; Cargo
subprocesses may finish cache writes, but cannot install or reload a pack.

Enter `r` followed by Enter to rebuild/reload, or `q` to stop. Other lines pass
through to the server. `R` stops and reaps the JVM, rebuilds, reapplies the
configured world-reset policy, and starts a new JVM with fresh credentials.
The same stdin reader and Ctrl-C handler survive repeated restarts.
EOF and Ctrl-C request a graceful stop,
with a 30-second bound. On normal/error cleanup, RCON is disabled and its
password removed. Forced process termination can leave the local credential in
`dist/server/server.properties`; the next dev session replaces it. Unix file
permissions are 0600. Windows uses the current directory's inherited ACL.
Credentials come from 256 bits of OS entropy, are never printed, and are never
placed inside the pack or release zip. The selected ephemeral RCON port can be
claimed between selection and server launch; connection/authentication failure
is reported, never treated as reload success.

The status includes build elapsed time, existing publisher file counts and
sync counts. `dist/.sand-dev-build.json` retains the latest build diagnostics
and #465 runtime report. The console says whether the report changed relative
to the preceding installed rebuild (the first rebuild establishes a baseline).
Those are static explicit evaluation bounds, **not** player/entity scan counts,
whole-tick totals, command-budget units, or MSPT. Unknown ranges stay unknown.
RCON acknowledges dispatch before Minecraft finishes resource loading. A unique
load-tag sentinel is added only to the installed development pack, after its
load entries. Each reload remains pending until that sentinel is observed or
30 seconds elapse. The existing console diagnostic model separates load failures
from warnings/nonfatal command diagnostics. A sentinel with no classified load
failures reports complete; errors report failed, and missing evidence times out.
Later unrelated logs do not affect the closed reload. This establishes that the
load tag ran, not that every gameplay invariant holds. Other packs can replace
the load tag; missing sentinels time out safely. There is no automatic rollback
of Minecraft's in-memory resources. The development sentinel adds one load-only
command excluded from the source build's runtime report and release package.

World/registry data, generated world-init commands, pack metadata, `sand.toml`,
and host configuration changes block installation until a new session. Functions,
function tags and common reloadable data can update live. This is conservative;
a restart still cannot regenerate existing chunks or apply a new seed to an
existing world. Resource sources are watched and rebuilt, but resource-pack
client installation/reload is not automated.

Current reload semantics are intentionally preserved:

- #438's namespace-safe, reconnect-aware `OnJoinEvent` still runs for online
  players after reload. `FirstJoinEvent` is the once-ever alternative. See
  [the player join evidence](testing/player-join-evidence.md).
- `__sand_world_init` remains a Minecraft load function. Unchanged world setup
  can therefore reapply spawn, fills, time and gamerules on every reload.
- #407 remains open. This CLI does not delete old compiler-owned objectives or
  claim unchanged-pack reload is universally idempotent for user State.

This is an independently mergeable CLI foundation for #443, not completion of
that umbrella. Numerical cost deltas (rather than changed/unchanged), framework
load semantics/cleanup, additional platform evidence, and live State
inspection/debug maps remain separate work.

## Repeatable validation

After `cargo build -p sand-cli --bin sand`, run:

```sh
python3 scripts/validate_dev_workflow.py --output /tmp/sand-dev-evidence
```

The output directory must not already contain a `project` subdirectory. Java 25
must be on PATH. The script uses Minecraft 26.2, a disposable project, loopback
ports and the normal Sand jar cache. It proves a watched nested source edit,
failed-build retention, live function behavior, native parser failure and
subsequent reload recovery, startup-resource guarding, repeated `R`, `r`, `q`,
EOF, credential rotation/cleanup and release exclusion. Logs and a commit-labeled
evidence file remain in the output directory; secrets are never printed.
Validation subprocesses have dedicated process groups and bounded shutdown; a
failed run kills its descendants. The release build has a five-minute deadline.
The fixture uses `<output>/target` for Cargo artifacts, isolating its `sand_export`
executable from workspace builds while sharing Cargo's registry/download cache.

Restart preparation checks queued `q`, EOF and Ctrl-C before preparing the next
run, after the synchronous build and jar check, and before world reset and JVM
launch. Ordinary queued console commands remain available for the new JVM.
Cancellation cannot interrupt an in-progress build, download, or file operation;
it takes effect at the next checkpoint. No fixed cancellation latency is claimed
for those synchronous operations. Cancellation before reset preserves the world,
including with `AlwaysReset`; it cannot undo a reset that has already begun.

The 30-second reload budget starts after installation and successful RCON
dispatch, so copying a large pack does not consume Minecraft's loading window.
For `sand dev`, a confirmed successful sentinel clears failures from earlier
managed reloads: error, fix, reload, then `q` can exit successfully without `R`.
An unrecovered failure, timeout, unconfirmed dispatch, or quit during a pending
reload remains unsuccessful. Startup and unrelated fatal diagnostics stay sticky;
a ready banner alone cannot recover health. Ordinary `sand run` retains its
historical, monotonic health contract. The native harness verifies both recovered
and unrecovered quit outcomes and queued restart cancellation with `AlwaysReset`.

Across `R` restarts, console log events are scoped to their originating JVM;
late output cannot mark the next JVM ready or carry errors into it. A namespace
change installs the new pack before removing the previous pack managed by that
dev invocation, only within the same world. Other datapacks are preserved.
Cancellation before the install commit or failed installation retains the old
pack. The original `server-ip` setting is restored on cleanup; loopback binding
is temporary, and the RCON port is chosen distinct from the configured game port.
