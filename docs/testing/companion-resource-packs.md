# Companion resource-pack packaging evidence

The companion asset pipeline is optional. It reuses the canonical CLI
configuration, resource identifiers, output ownership manifests, pack publication
and zip implementation; no facade API or second gameplay item model is added.

## Metadata provenance

`sand-cli/src/build/resource_pack_versions.json` retains the `version.json`
objects extracted from the nested server jar in Mojang's cached distribution,
paired with the outer server artifact's SHA-1. The native metadata explicitly
separates `resource_major`/`resource_minor` from `data_major`/`data_minor`:

| Verified server | Resource format | Datapack format |
| --- | --- | --- |
| 26.1 | 84.0 | 101.1 |
| 26.1.2 | 84.0 | 101.1 |
| 26.2 | 88.0 | 107.1 |
| 26.3 | 97.1 | 121.0 |

The resource table has no conservative future-version fallback. The explicit
`[resource_pack].pack_format` override accepts a positive signed-32-bit major
number or `[major, minor]` with a nonnegative signed-32-bit minor. Exact minor
bounds survive in generated `pack.mcmeta`.

## Regression checks

```sh
cargo test -p sand-cli --test resource_pack_pipeline --test cli_new_user_journey
cargo test -p sand-cli --lib build::publication
```

Tests cover optional configuration, source/scaffold idempotence, byte-preserved
assets, independent metadata, deterministic archive bytes, unchanged-file
rebuilds, stale owned-file pruning, preserved unrelated files, malformed JSON,
invalid source roots, symlink source/assets/archive rejection and archive failure
preserving the old zip. Grouped publication tests inject a failure at the
second installation and verify both prior pack contents, manifests and user
files remain intact. Existing single-pack ownership, read-only metadata,
file/directory transition and rollback recovery regressions also remain active.

## End-to-end and native metadata checks

The [recorded result](companion-resource-packs-evidence.json) captures the local
2026-11-03 run; temporary artifact paths are reproduction context, not bundled files.
A fresh 26.3 scaffold with one authored item-model JSON passed
`sand add resourcepack` and `sand build --release --format json`. The companion
result reported `dist/sand_resourcepack_proof-resources`, exact format `[97, 1]`,
one asset and a separate release zip. A repeated build reported zero writes and
two unchanged companion files; SHA-256 snapshots of both folders and archives
were identical. An invalid authored JSON failed before changing any of those
bytes. Removing the opt-in table produced a successful datapack-only JSON result
with no `resource_pack` field. Local outputs are retained under
`/tmp/sand_resourcepack_proof` and `/tmp/sand-resourcepack-build*.json`.

The unmodified 26.3 vanilla `PackMetadataSection.CLIENT_TYPE` codec also decoded
the generated metadata and retained the exact `[97.1, 97.1]` range. Reproduce
with the extracted server/libraries from the existing vanilla harness:

```sh
PROBE_CLASSPATH="$(rg --files "$SERVER_DIRECTORY/libraries" "$SERVER_DIRECTORY/versions" -g '*.jar' | paste -sd: -)"
java -cp "$PROBE_CLASSPATH" sand-cli/tests/fixtures/resource_pack/MetadataProbe.java \
  "$PACK_DIRECTORY/pack.mcmeta" 97 1
```

This exercises native metadata decoding; it does not load or render asset files.

## Boundaries

Packaging validates paths and JSON syntax, and checks a `pack.png` signature;
it does not validate all native asset schemas, cross-file references, image/audio
codecs or client rendering. No assets are downloaded, installed into a client or
required by a server through this feature. Generated asset outputs are separate
from `data/` and are excluded from gameplay runtime cost analysis.

The two output folders share staged publication and ordinary-error rollback.
Release archives are each written through a temporary file after folder
publication; folders and archives are separate transactions. Concurrent writers,
process crashes and failed filesystem rollback are not described as atomic.
