# Fresh-agent API authoring experiment

A fresh agent with no inherited conversation was asked to author **Vanilla
Expedition**, a broad Minecraft 26.3 exploration/adventure datapack. Its only Sand
reference was a frozen `sand api` binary. It could edit its own scaffold, compile,
inspect diagnostics and exported resources, and run build/check infrastructure.
It was instructed not to read Sand source, examples, docs or skills and not to
browse the internet. These were agent instructions, not an OS filesystem sandbox.

## Original assessment

- [Independent assessment and feature matrix](assessment.md)
- [Exact-query diary and repair cycles](diary.md)
- [Breadth-gap inventory](breadth-addendum.md)
- [Experiment identity and limits](experiment.json)
- `discovery-evidence.tar.gz`: frozen catalog, query output, build logs and original verifier.
- `author-source.tar.gz`: the original completed author source, with dependency paths
  adapted to this checkout and a standalone workspace declaration.
- `baseline-export.tar.gz`: its original 96-component export, **not a validated release**.

The subjective baseline scores were discoverability 5/10, composability 7/10,
diagnostics 7/10 and representative export breadth 8/10. These are one agent's
ratings, not a coverage percentage, benchmark or post-fix reassessment. The
exercise did not exhaust every vanilla resource, field, event or behavior.

## Parent verification and repairs

The original project compiled and passed artifact checks but failed native load:

1. `DialogAction::close()` emitted the nonexistent `minecraft:close` action.
   Sand now omits the button action, matching the optional native action codec.
2. The displayed root advancement lacked a background. The 26.3 native codec
   requires one and forbids child backgrounds. Sand now validates this target
   constraint; the retained `project` adds a stone background explicitly.

The agent independently caught an earlier successful export with a missing
Scene helper. Sand now rejects static references to missing private owned-gameplay
helpers and explains registration. The original broken-call evidence is retained
inside `discovery-evidence.tar.gz`.

Search word boundaries, type/member navigation, score-read type reachability,
raw-material classification, State usage and Scene registration examples were
also repaired. The original scores were not revised to rate those repairs.

`project/` is the replayable source, including the separately attributed
background repair. It is a test artifact, not a maintained production game. Its
README reflects the original author workflow. From the repository root:

```sh
CARGO_TARGET_DIR="$PWD/target" CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 \
  cargo build --manifest-path benchmarks/evidence/api-authoring-2026-11-05/project/Cargo.toml
```

For datapack export, install the matching CLI and run `sand build` in `project/`.
For code discovery use the current `sand api`; the archived catalog is retained
only to reproduce the original discovery assessment.

Native startup/reload acceptance is distinct from actual player input, rendered
UI, reward authorization, multiplayer gameplay and performance. No graphical
client was launched for this experiment or its parent smoke tests.

The repaired pack passed 26.3 native startup, reload and clean shutdown with zero
players. `native-final.log.gz` and `final-pack-sha256.json` retain that proof.
[vanilla_expedition.zip](vanilla_expedition.zip) is the matching pack artifact.
This establishes resource loading, not an ordinary-player playthrough.

The adapted retained project was rebuilt independently and its artifact verifier
passed. Every output file, including metadata, matches the native-tested pack
SHA-256 manifest byte for byte; see `retained-project-build.json.gz`.

The final repository gate (`scripts/check.sh`) and `git diff --check` passed.
`repository-check.log.gz` retains the complete gate output.
