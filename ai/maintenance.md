# Maintaining `ai/` resources

These files are handwritten and reviewed alongside source, not
code-generated (see "What should be generated instead" below for the
exception path). They go stale exactly like any other doc — the difference
is that agents will act on stale claims automatically, so treat drift here
as a bug, not a docs nice-to-have.

## When to update, by change type

| Change | Update |
|---|---|
| New or changed public API (type, method, macro) | `capability-manifest.yaml`: add/update the capability's `status`, `preferred_api`, `evidence`. Add a new capability entry if it's a new category of behavior. |
| New/changed Cargo feature gate | `capability-manifest.yaml` (`cargo_features` field on affected capabilities) and `AGENTS.md`'s systems-feature list if the set of `systems-*` flags changed. |
| New Minecraft version support, pack_format mapping, or version gate | `ai/project-status.yaml` (`minecraft_support`), affected capability `minecraft:` ranges in `capability-manifest.yaml`, and `LIM-VER-*` entries in `known-limitations.md` if behavior around unknown versions changed. |
| CLI command added, removed, or its status changed (stub → implemented, etc.) | `ai/project-status.yaml` (`cli.commands`) and, if it's a new command entirely, a new `capability-manifest.yaml` entry under `category: cli`. |
| Capability moved between experimental/partial/implemented | `capability-manifest.yaml` `status`/`stability`, and remove or add the matching `known-limitations.md` entry. |
| New raw-only escape hatch, or an existing gap gets a typed API | `capability-manifest.yaml` (`raw_escape_hatch` field, `status`) and `known-limitations.md` (remove the `LIM-API-*` entry once closed, don't leave it dangling). |
| Vanilla-reload validation coverage changes (new harness, CI gate added) | `ai/project-status.yaml` (`test_and_validation_status.vanilla_server_reload_validation`) and `LIM-VAL-001`/`LIM-VAL-002` in `known-limitations.md`. |
| New example or recipe added under `examples/` or `book/src/recipes/` | Consider adding/updating a matching file in `ai/recipes/` if it demonstrates a capability agents will commonly need. |
| `registry_coverage.rs` / `trigger_coverage.rs` rows change | Re-derive affected `capability-manifest.yaml` entries from the table (counts, `Missing`→`PartiallyImplemented` transitions, etc.) — don't let the manifest drift the way `docs/research/datapack-parity-audit.md` did (`LIM-DOC-002`). |

## PR checklist

```text
[ ] Capability status updated (ai/capability-manifest.yaml)
[ ] Version range / version_gate updated where relevant
[ ] Limitation documented or removed (ai/known-limitations.md)
[ ] Recipe updated or added (ai/recipes/) if the change affects a common task
[ ] Tests referenced in capability evidence still exist and pass
[ ] Generated manifests regenerated (none yet — see below; keep this line
    for when a generator exists)
[ ] llms.txt still points to canonical resources (paths unchanged)
```

## Validating the resources themselves

Run `scripts/check-ai-resources.py` (added alongside this system) before
committing changes to `ai/`. It checks:

- Required files exist (`AGENTS.md`, `llms.txt`, `ai/README.md`,
  `ai/project-status.yaml`, `ai/capability-manifest.yaml`,
  `ai/authoring-guide.md`, `ai/known-limitations.md`, `ai/maintenance.md`,
  `ai/recipes/README.md`).
- Both YAML files parse.
- Capability IDs in `capability-manifest.yaml` are unique.
- Every `evidence`/local-file-shaped path referenced in the YAML files
  resolves relative to the repo root.
- Every `implemented` capability has a non-empty `evidence` list.
- `status` and `stability` values are drawn from the documented enums.
- Recipe front matter parses as YAML and every `capabilities:` entry it
  lists exists in `capability-manifest.yaml`.
- Every `known-limitations.md` `Affects:` capability reference exists in
  `capability-manifest.yaml`.

`scripts/check-docs.py` (existing) validates Markdown links across the repo,
including `ai/**/*.md`, `AGENTS.md`, and `llms.txt` — run it too; both are
wired into `scripts/check.sh`.

```sh
python3 scripts/check-ai-resources.py
python3 scripts/check-docs.py
scripts/check.sh   # full pre-release set, includes both of the above
```

## What should eventually be generated from Rust instead of handwritten

These are recommended follow-ups, not yet implemented:

- `capability-manifest.yaml`'s per-registry and per-trigger detail
  (`sand_limitations`, `status`, `minecraft.minimum`) for datapack-component
  capabilities could be generated directly from `REGISTRY_COVERAGE` and
  `TRIGGER_COVERAGE`, eliminating the class of drift documented in
  `LIM-DOC-002`. A generator binary (parallel to
  `sand-build/src/bin/refresh-registry-coverage.rs`) emitting a partial YAML
  fragment, checked by a CI "regeneration produces no diff" step, would
  close this gap.
- `ai/project-status.yaml`'s `cli.commands` table could be generated from
  the clap command definitions in `sand/src/main.rs` to catch new/removed
  subcommands automatically.
- Cargo feature lists in this directory could be generated from
  `sand-core/Cargo.toml` `[features]` directly rather than copied by hand.

If any of the above is implemented, add the generator to the repository
under `scripts/` or a `*-build` crate `bin/`, add a `<!-- generated by
scripts/... — do not edit by hand -->` header to the output, and add a CI
check that regenerating produces no diff (mirroring the existing
`registry-coverage` fixture drift tests in
`sand-components/src/registry_coverage.rs`).
