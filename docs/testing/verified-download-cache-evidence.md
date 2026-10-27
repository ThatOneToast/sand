# Verified download cache: bounded validation evidence

PR #495 fixes the warm-cache package fetch failure by preserving the selected
version ID, package URL and expected raw SHA-1, caching verified metadata, and
returning an immutable checksum-addressed JAR path. No public signatures changed.

## Attribution

- `6459c93abf810d28eee09ad489cdd57c9bf0b965`: production fix and synthetic focused
  tests; independent source review approved. Original focused evidence belongs
  to this source, not to a later integrated candidate.
- `44f75ba288108ae68e1d3894be0989ebef34e5ae`: three duplicate workflow arguments
  removed; no cache semantic change.
- `6bc8da82e9cda85e6850a3b98135f14b2fd81991`: test-only ignored private harness.
  All five real-input experiments below used its frozen binary, Java launcher,
  sandbox profile and externally supervised commands.
- `77fdb52a96b457c7b6774e2591ea58aeec3cf38b`: merge of exact main
  `269a4ab1f3fd7c83c525bbe3ccfeee9f946823f2`. Exactly seven StateFlow files changed
  from 6bc8; cache implementation, consumers and harness are byte-identical.
  These experiments were not rerun after integration.

## Trusted inputs and evidence

Inputs were copied read-only into `/private/tmp/sand-pr495-validation-knb7_cnu/inputs`.
The manifest originated at the existing `~/.sand/cache/version_manifest_v2.json`;
its SHA-256 is `1e9f0052c6f385576f0effac072f2b32278346ce647d681e396dcf7b6f7d738a`.
This is the experiment's trusted manifest snapshot, not a signature or freshness
claim. Its complete 26.2 entry selected:

- Package: `https://piston-meta.mojang.com/v1/packages/bdfd9856166b7171857c491f15479762f3abda99/26.2.json`.
  One separately authorized HTTPS acquisition; raw SHA-1
  `bdfd9856166b7171857c491f15479762f3abda99` verified before parse/ID validation.
  SHA-256: `aafa877bbf83b0fad7db06b251808adc107c008c236e4f49bc6bbc6cc3ddb852`.
- JAR: read-only copy of existing `~/.sand/cache/26.2/server.jar`, no download.
  Package-declared size 60,894,273 and SHA-1
  `823e2250d24b3ddac457a60c92a6a941943fcd6a` matched the copy. SHA-256:
  `cdacdfb25898de5e4b4b0e5ddcc2722f77067e46605709c2d886c000ebb63ec5`.
  Origin declared by package: `https://piston-data.mojang.com/v1/objects/823e2250d24b3ddac457a60c92a6a941943fcd6a/server.jar`.

Private checkpoints, retained outside the repository:

| File under private root | SHA-256 |
| --- | --- |
| `supervised-release-supplement.json` (36 frozen hashes) | `2c5dca87679b3349d2bb3ea4f0f63e012a071769d78c5b20d51f6fb54d5d5d87` |
| `cold-replay-checkpoint.json` | `c0d731fc7b3ce42aa99ceb7a29ce0779ba0e18ff4a2ad93f7808e0b2c95a43a9` |
| `remaining-modes-checkpoint.json` (59 artifact hashes) | `dee9cc0126f1b925afdb97bff07213c85a39c4659eac9266934326d0721a6b17` |

The root reviewer independently recomputed the supplement and artifact hashes.
No raw properties or credentials are included here. Temporary artifacts are
host-local evidence; preserve them separately if the host is retired.

## Five modes and reproduction boundaries

Each command used the frozen `supervise.py --mode MODE` in that private root,
with `/opt/homebrew/opt/python@3.14/bin/python3.14`. These are completed runs,
not commands to repeat in the used root. A failure preserves the entire root;
never remove a marker or retry in place. A new reproduction requires a fresh
explicit canonical private root, frozen launcher/profile/inputs and reviewed
supervisor configuration; no HOME override or public cache-root API.

Build the ignored test with `cargo test -p sand-build --lib --no-run --offline
--message-format=json`, jobs 2 and an exclusive absolute CARGO_TARGET_DIR.
Normal suites leave `private_cache_validation::trusted_26_2` ignored. The harness
requires explicit absolute SAND_CACHE_VALIDATION_ROOT, INPUTS and JAVA variables
(with their full `SAND_CACHE_VALIDATION_` prefixes), MODE, and private launcher
first in PATH. Do not invoke the test binary directly for real Java validation.
The supervisor supplies these and runs sandbox-exec: network denied, all access
to shared `.sand` denied, writes allowed only inside the private experiment
and the frozen device exceptions. No sandbox permissions were broadened.

| Mode, sequential order | Deadline / actual seconds | Established result |
| --- | --- | --- |
| cold_replay | 600 / 15.93 | Local trusted-byte replay; actual Java 25 report generation from selected immutable JAR; provenance and nonplaceholder generated Rust |
| warm_panic | 120 / 9.58 | Panic-on-request seam untouched; no Java launch; verified warm JAR/reports |
| warm_http_denied | 120 / 9.64 | Counted transport recorded zero calls; no Java launch; denied-egress sandbox |
| legacy_missing | 60 / 4.00 | JAR-only and JAR+manifest without package both error; no artifact published or Java launch |
| legacy_import | 600 / 25.13 | Exactly package replay, no JAR request; verified legacy copy; legacy report sentinel ignored and unchanged; new actual Java reports; subsequent warm hit |

All supervisors exited 0, accepted complete ledgers, reaped their direct child,
and certified zero descendants. No timeout, signals or 20 GiB floor breach.
Dedicated process groups use bounded TERM/KILL cleanup; five fake-child
regressions cover success, nonzero, timeout, resistant descendants and leftover
children after leader success. This is not hostile process containment.

Java was Temurin 25.0.3; executable SHA-256
`0a1eea36b7899323b32caab6f1d0e416ad7208792b076391278062efab4b15d8`.
Frozen test binary SHA-256:
`af622415d02959c89c7a5f0fd9cf13fc76043af85a93aa1e0c5d7114d61d78cc`.
Actual generator argv was `-DbundlerMainClass=net.minecraft.data.Main -jar
<private-cache>/26.2/artifacts/823e2250d24b3ddac457a60c92a6a941943fcd6a/server.jar
--reports`, exit 0. The `-version` probe exit was not separately recorded.

## Limits and remaining gates

Zero transport calls are instrumented proof, not syscall tracing or live HTTP
failure recovery. Actual Java compatibility supplements synthetic fake-generator
proof. Only registries.json, blocks.json, commands.json and datapack.json hashes
are certified by report provenance. Generated Rust was checked nonplaceholder,
not compiled by this experiment. Missing-metadata assertions check errors, not
exact error identity. Public operational consumers were audited/tested separately
for returned-path plumbing; this is not dynamic execution of every public wrapper.
Synthetic subprocess concurrency covers same identity, not changed-manifest races.
Report publication crash can leave an invalid entry absent/regenerable. There is
no hostile-local-write guarantee or power-loss promise. No broad speed claim.

Full scripts/check.sh, shared-cache/public-helper execution, integrated Java
reruns and native gameplay remain held for explicit coordinated release. PR #494
full validation at frozen 3d268 passed, and PR #494 merged as exact main
`735de6547db857ae940a4c38435604a3a7e22bbd`; root reported no furnace survivors.
Read-only inventory and private backup are authorized only after independent
process/cwd checks establish quiescence. This is not release to execute full
validation or shared-cache writes. No CI polling or profile changes are implied.

## Shared-cache rollout plan (not executed)

For root `~/.sand/cache` and selected version V, new code may write:

- `.sand-manifest-v1.lock`, `download-manifest-v1.json`, unique adjacent file temps;
- `V/.sand-artifacts-v1.lock`, `V/packages-v1/<identity-key>.json` and unique temps;
- `V/artifacts/<jar-sha1>/server.jar`, unique JAR temps, `reports-v1/` including
  provenance, and `.reports-*` staging/quarantine contents;
- existing shared `V/rust-codegen/<fingerprint>/` generated files and manifest,
  `.tmp-*` publication and `.stale-*` repair directories. This namespace is not
  isolated from legacy codegen users. Caller OUT_DIR is also populated.

Protected legacy paths: `version_manifest_v2.json`, `V/server.jar`,
`V/generated/` (including old reports), and legacy `.sand-codegen.lock` wherever
present. New download/report code only reads manifest/JAR seeds, never rewrites
these or adopts/unlinks a presence lock. It ignores old version-only reports.
New OS-lock inodes remain stable and must never be manually removed. No mutable
server.jar alias, global GC or eager migration is introduced.

Before release, coordinate completion of furnace and all other legacy writers.
After independently verifying quiescence, inventory affected paths read-only (relative path, type, size,
mtime and SHA-256 for regular files; record symlinks without following them).
Include the shared rust-codegen namespace and pre-existing v1 entries. Copy a
verified backup into a unique private directory outside the cache, check copied
hashes against the quiescent inventory and retain sufficient disk headroom
(20 GiB minimum during validation). Do not call a live-writer copy consistent.
Do not dump raw properties or secrets into logs. Inventory/backup execution and hashes belong to the separate rollout checkpoint;
this document does not certify a live cache snapshot.

Freeze the exact candidate/commands and obtain coordinated full/shared release
before executing. Afterward compare protected legacy hashes and inventory new
writes against the list above; separately account for expected rust-codegen
changes. Preserve failures and all evidence; no automatic retry, deletion,
restore or stale-lock removal. Any rollback requires quiescence and review of
specific paths against the backup; do not replace an entire live cache.

## Integration after PR #494

Merge `934e60bc9b579758e9840b09b8873ed89f5511d6` integrates only exact main
`735de6547db857ae940a4c38435604a3a7e22bbd`. The sole conflict was the
`scripts/check.sh` Python check list: retain main's furnace checks and the
branch's returned-JAR consumer test. No generated reports were edited.
All sand-build source/private harness and operational JAR consumers remain
byte-identical to private-tested 6bc8. Inherited main changes and this check-list
union are integration evidence, not a rerun of the original private experiments.
