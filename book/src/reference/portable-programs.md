# Portable programs and embedding

Sand can compile a small owned gameplay program without generating Rust or
running Cargo for each input. Install or build the `sand` binary once, then use
`program check` and `program compile` from any directory:

```sh
sand program capabilities --format json
sand program schema --format json
sand program check --input counter.sand.json --format json
sand program compile --input counter.sand.json --output dist/demo --format json
```

The response is one versioned JSON object. Success includes sorted file paths
and byte counts. Add `--include-files` to include UTF-8 file bodies. Errors have
stable codes, a module/document identity, a JSON field pointer, and optional
source provenance; failures exit nonzero. Logs go to stderr.

## A small shared model

Protocol v1 supports player State with revision 1 and zero-default integer
fields, named server/player functions, score assignment and addition, inclusive
integer comparisons, branches, player iteration, calls, ordered load/tick work,
and function-tag membership. Every declaration is checked, including unused
functions. Events, participants, storage/entity State, migrations, timers,
scheduling and native bindings are outside v1.

A program envelope supplies `format: "sand.program"`, `format_version: 1`, an
explicit `target.minecraft`, pack namespace/description, and modules. The target
must be an exact verified 26.x+ release; `latest` and unverified releases fail.
Protocol revision, State revision, compiler package version and target version
are independent. Capability discovery describes supported operations and limits.

A score operation is gameplay data rather than a manually assembled command:

```json
{
  "action": {
    "op": "score_add",
    "score": {"state": "demo:counter", "field": "value"},
    "value": 1
  },
  "origin": {"node": "increment-counter", "port": "value"}
}
```

The complete counter fixture is `tests/fixtures/program/counter.sand.json` in
the repository. Its tick function iterates players and calls a player-context
increment function. Canonical State initialization preserves existing values
and keeps players independent. Player functions initialize State on direct
entry too. Player iteration uses `as @a`, preserving positional context.
Branches evaluate their condition once, even if the selected body changes the
compared score.

Function references explicitly distinguish internal declarations from external
functions. Internal references resolve across the complete program; external
references must not alias locally declared functions. They declare a context
requirement, but Sand cannot prove their existence
or behavior. A server body cannot access player self or call a player function
until iteration establishes player context.

## Modules and schema-guided producers

Small programs embed module objects. Larger programs list flat local paths:

```json
{
  "format": "sand.program",
  "format_version": 1,
  "target": {"minecraft": "26.2"},
  "pack": {"namespace": "demo", "description": "Player counter"},
  "modules": ["state.sand.json", "tick.sand.json"]
}
```

The CLI defaults the module root to the envelope's parent; `--root` sets an
explicit root. Absolute module paths, traversal, empty segments, duplicate paths
and symlinks fail. Modules cannot import other modules. `--input -` reads a
self-contained program from stdin and never discovers files in the current
directory. Module IDs, function IDs and State IDs are explicit semantic names;
filenames and editor layout never determine generated identities.

`schemas/program/bundled.schema.json` contains a self-contained generated
schema. `program.schema.json` and the domain files provide reusable offline
references. `module.schema.json` validates separate module documents. The Rust
models own these schemas; regenerate them with
`scripts/generate-program-schemas.py --binary /absolute/path/to/sand`.
`scripts/check-program.sh` checks for drift.

The Python example in `scripts/portable_program/producer.py` builds dictionaries
and validates them against Sand's exported schema. Its tiny blueprint adapter
accepts tick → players → increment, preserves node/port origins, rejects cycles
and ambiguous edges, and ignores layout. Language/DSL frontends can follow the
same pattern. Generate and compile the Python counter directly:

```sh
python3 scripts/portable_program/producer.py | sand program compile --input - --output dist/demo
```

AI-generated documents should use the exported schema, then pass through
`program check`; schema acceptance does not replace semantic validation.

## Rust embedding

Use `sand::advanced::compiler::{Compiler, Program}`. Construct the owned model
or call `Compiler::decode(envelope_bytes, &module_contents)`, where module
contents are an explicit `BTreeMap<String, Vec<u8>>`. `Compiler::module_paths`
lets a host implement its own constrained resolver. `Compiler::check` validates
the complete program and output limits; `Compiler::compile` returns sorted
relative paths and final bytes including pack metadata.

The compiler reads no inventory, environment, filesystem or network and invokes
no author callbacks. Calls can repeat or run concurrently. Publication and ZIP
packaging belong to the host. The CLI validates everything before staging a
complete pack; it preserves unrelated files, rejects unmanaged/modified-file
conflicts and restores the previous pack on ordinary installation failures.
Directory publication is not crash-atomic or a cross-process transaction.

Existing Rust authoring keeps its collection path. The supported State slice,
lifecycle assembly, tags, entry initialization and pack metadata share backend
code with portable programs. Declare a Rust player entry with
`#[function("demo:increment", context = player)]`; callers must establish a
player executor. This annotation does not statically analyze arbitrary Rust
command factories for context correctness.

## Bounds and escape hatches

Each UTF-8 document is limited to 1 MiB, complete input to 8 MiB and modules to
64. Maximum JSON nesting is 64, operation nesting 32, declarations 10,000 and
operations 100,000. Generated packs allow 10,000 resources, 1 MiB per resource
and 32 MiB total. All limits apply together. Constructed programs obey semantic,
depth/count and generated-output limits too.
Generated resource paths are limited to 768 UTF-8 bytes relative to the pack
root, with at most 255 bytes per segment including file extensions. Capability
discovery reports both bounds. Publication adapters must still account for the
length of their chosen destination root.

Raw operations require `"requires": ["raw_commands"]`. They receive line safety
and supported syntax checks; Sand cannot prove their State, context or reference
semantics. The acceptance counter uses only typed operations.

Run `scripts/check-program.sh` for schema and cross-frontend tests. Real-server
validation is separately runnable with
`scripts/mc_validation/run_portable_program_audit.py`; its evidence explicitly
states whether automatic ticks or explicit invocations were exercised.
