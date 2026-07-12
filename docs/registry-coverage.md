# Registry coverage drift fixtures

Sand's canonical component inventory is
`sand-components/src/registry_coverage.rs`. Real data-driven registries live in
`REGISTRY_COVERAGE`; tag-only directories live separately in `TAG_COVERAGE`
with the registry that their values reference. Strings such as
`minecraft:block (tags)` are not valid registry identifiers and are rejected
by the semantic tests.

## Fixture source and normalization

The checked-in fixtures under
`sand-components/fixtures/registry-coverage/` come from Mojang's server data
generator report `generated/reports/datapack.json`. Sand retains entries where
`registries.<id>.elements` is true and adds the report's `others.function`
component as `minecraft:function`. Registry IDs are sorted and their datapack
directory is the path after `minecraft:`. The fixture records its concrete
Minecraft version and provenance but omits registry contents, server jars, and
the full Mojang report.

Normal tests parse these small files entirely offline. For each fixture
version they apply `RegistryCoverage::version_gate` (introduced in) and the
exclusive `REGISTRY_REMOVED_IN` gates, then report sorted missing rows, stale
rows, directory mismatches, duplicate or invalid IDs, and invalid version
gates. A `RawOnly`, `PartiallyImplemented`, or
`IntentionallyUnsupported` row is valid coverage; drift detection does not
pretend a registry is absent merely because Sand lacks a typed builder.
Custom or modded registries are intentionally not added to this vanilla table;
they remain accessible through Sand's raw component/resource-location paths.

The initial fixtures cover the established CI baseline (`1.21.4`) and the
version named by `sand_version::LATEST_KNOWN`. A test prevents the latest
fixture from drifting away from that Rust constant.

## Refreshing fixtures

Refresh one fixture explicitly from the repository root:

```sh
cargo run -p sand-build --bin refresh-registry-coverage -- \
  26.2 sand-components/fixtures/registry-coverage/26.2.json
```

The command accepts only the requested version and output file. It uses
Sand's existing Mojang manifest, SHA-verified server-jar cache, and report
generator. A cold cache requires network access and the Java runtime required
by that Minecraft server (Java 25 for `26.2`). It is a maintenance command,
not part of normal CI. Review the resulting fixture diff together with any
intentional coverage-table/version-gate updates.
