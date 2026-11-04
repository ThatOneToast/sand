# Journal and checkpoint evidence

Reviewed 2026-11-04. This increment provides explicit literal journal refresh and
integer-only canonical State checkpoint projections. It also consolidates dynamic
EntityText materialization for journals and native entity names, removing author
text from macro bodies. Scene and Journal also share the internal
`runtime::component_nbt` serializer for canonical component payloads.

## Portable export and validation

`sand-core/tests/gameplay_journal_checkpoint.rs` passes deterministic export on
26.1, 26.2 and 26.3. The fixture checks State and bounds guards before field-copy
helpers, UUID owner reacquisition after preparation, committed results before
continuation, pack-shared checkpoint locks across resource namespaces, native
book delivery/verification calls and numeric-only text macro bodies. Private
unit regressions validate bounded journal pages/segments, the largest escaped
flag label, single-line encoding of component strings, and canonical checkpoint
registration/field-count requirements.

`examples/quest_rounds` demonstrates an event-driven saved quest, separately
resettable round, fixed-slot journal and explicit saved round projection. Quest
claim indices and points remain outside round rewind. The saved projection is a
normal separately-derived State schema; no private runtime state is copied.

## Native 26.2 and 26.3 proof

Reproduce from the repository root with the official cached server jar:

```sh
python3 scripts/validate_journal_checkpoint.py \
  --jar /path/to/26.3/server.jar \
  --version 26.3 \
  --output /tmp/sand-journal-checkpoint-proof
```

The 2026-11-04 proof passed separately on vanilla 26.2 and 26.3 with two real
passive clients per server (protocols 776 and 777), including an actual reconnect.
Use `--version 26.2` and its corresponding jar to reproduce the older target.
Results and both jar digests are recorded in
[journal-checkpoint-evidence.json](journal-checkpoint-evidence.json).

- Books contain literal numeric, enum and flag snapshots. Updating State leaves
  existing content unchanged until explicit refresh.
- Quotes, backslashes, newlines and literal `$(missing)` survive delivery and
  refresh; those author strings never enter macro function bodies.
- Unrelated slot contents are preserved. A changed snapshot triggers a real native
  inventory listener: synchronous journal reentry is blocked, and a listener's
  replacement item is preserved while delivery rejects.
- Each player receives their own content and saved State projection.
- Invalid saved/source bounds reject before any field copies or preparation.
- Preparation invalidation and live-State detachment are revalidated; rejected
  restoration does not recreate detached scores or invoke continuation.
- Same-definition and alternate-namespace save/restore/clear attempts in the
  preparation callback cannot bypass the checkpoint lock.
- Restoring cancels prior work and explicitly starts a fresh Session. Callback
  returns cannot leave a transaction lock or operation scratch behind.
- A canonical saved-State migration explicitly invalidates an old snapshot.
- Reload/reconnect preserve canonical saved State without implicitly restoring
  live State; an explicit restore succeeds afterward.
- Native dynamic entity names use the same safe materializer and retain literal
  macro tokens, numeric text and quoted flag labels.

Assertions compare real score values, native item component data and exact NBT
StringTag values using data-copy success semantics. The component codec may
normalize plain literal text to StringTags, so the proof reads both canonical
native forms rather than assuming JSON-shaped inventory NBT. A check requires an
actual emitted server marker line; diagnostic echoes cannot count as passes.
After the PR #505 serializer review fixes, both native targets were revalidated
with freshly exported fixtures; neither final run used `--reuse-export`. Each
invocation first passed all three focused integration tests, including export
checks on 26.1, 26.2 and 26.3, then passed the complete native scenario. These
runs confirm that sharing the control-safe component serializer preserves journal
quoting, literal controls, native listener behavior and checkpoint semantics.
The evidence records the serializer and fixture source digests for this rerun.

## Boundaries

Native proof covers 26.2 and 26.3; 26.1 has export evidence. Checkpoints
are player-only explicit integer projections, with no position, inventory, world,
entity or private-task continuation capture. Saved schema migrations and validity
are ordinary author State. Task recreation is explicit admission of fresh work,
with no rollback of arbitrary preparation or native listener effects. Journal
markers identify definitions, not secret claims or original players; books are
snapshots with no automatic tracking, refresh or forced opening. Full matchmaking,
participant/team assignment and offline execution remain outside this increment.
