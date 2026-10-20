# Local watch/build/reload foundation

Run `sand dev --offline --ram 2G` in a Sand project (omit `--offline` for
normal account authentication). The initial build and server preparation use
`sand run`'s profile, jar, world-reset policy, console classifier, and sync
pipeline. Both Minecraft and RCON bind `127.0.0.1`; this command is for local
clients. Existing server-port settings are preserved.

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
