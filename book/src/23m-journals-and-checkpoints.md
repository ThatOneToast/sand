# Journal snapshots and explicit checkpoints

`Journal` publishes a player's canonical quest State as a literal written book.
`Checkpoint` copies explicitly selected integer State fields into another
canonical State schema. Both require ordinary registration and explicit calls;
neither introduces automatic persistence or a second gameplay State model.
The [quest rounds example](../../examples/quest_rounds/) combines both APIs.

## Explicit journal delivery

```rust,ignore
use sand::entity::EntityText;
use sand::systems::Journal;

#[datapack_component]
fn journal() -> Journal {
    Journal::new(
        "journey:journal".parse().unwrap(), "Journey", "Sand",
        InventoryIndex::new(8).unwrap(),
    )
    .page(EntityText::new().literal("Objectives: ").score(Journey::objective))
    .replace_owned()
}
```

`refresh()` renders State at the call boundary and writes one native written
book in the configured numbered player slot. Later State changes leave that
book unchanged until another successful refresh. EntityText literals, numeric
scores, enum labels and caller-supplied flag labels share the canonical text
materializer used for entity names. Numeric segments show their stored integer
value; this increment does not add decimal formatting. Field handles must match
registered current State schemas. Missing fields, obsolete membership and invalid
enum/flag encodings reject delivery. Global field segments use their canonical
singleton holder; personal field segments use the executing player.

Titles have the WrittenBook 32 UTF-16-unit limit. Journals support one to 100
pages and at most 64 segments per page, with WrittenBook page validation applied
to the largest possible rendered content. Static styled TextComponent payloads
retain their normal validation. Unresolved score, NBT and selector components
cannot be used as apparent refresh-time snapshots; use typed EntityText field
segments instead. Arbitrary author text never enters a function macro body:
only numeric scalars and complete compiler-staged component compounds cross
macro arguments.

Delivery requires an empty slot by default. `replace_owned()` explicitly admits
a single written book with this definition's custom-data marker. Other items,
different journal definitions and stacks with another count are preserved.
The marker identifies a definition, not a player or secret claim; a recipient
can refresh a transferred book using their own State. No book locations or
copies are tracked persistently. There is no forced opening or automatic
inventory search, drop-on-full behavior, periodic refresh or merge policy.

The pack inventory lock is shared with Purchase and InventoryTransfer. A native
inventory listener cannot synchronously enter another Sand inventory mutation.
After placement, the native component codec must confirm the staged book
content and identity, and the slot must still have count one. Listener-added
unrelated components may remain. A listener that replaces/removes the journal or
changes its required content causes rejection; its replacement is preserved.
There is no arbitrary listener rollback. `on_success` and `on_rejected` callbacks
run after scratch cleanup and lock release. Non-player/locked calls are silent
no-ops and do not recurse into rejection callbacks.

## Explicit canonical saved State

```rust,ignore
#[derive(State)]
#[state(namespace = "journey", scope = player)]
struct SavedRound {
    #[state(min = 0, max = 2)]
    phase: Score,
    discoveries: Score,
    valid: Flag,
}

#[datapack_component]
fn checkpoint() -> Checkpoint {
    Checkpoint::new("journey:round_save".parse().unwrap(), SavedRound::valid)
        .field(Journey::phase, SavedRound::phase)
        .field(Journey::discoveries, SavedRound::discoveries)
        .before_restore(|_| round().cancel())
        .after_restore(|_| round().start())
}
```

Every saved field and the initially-disabled valid flag must belong to one
separately-derived saved State component. Live fields belong to other canonical
owner-scoped components. The first increment is player-only and supports one to
64 distinct pairs of ordinary integer Scores. Global, fixed-point,
criterion-driven and automatically ticking fields are rejected. Pair sources and
destinations must be distinct and unique. This is an explicit projection of
selected gameplay facts, not an arbitrary copy of an entity or storage tree.

`save()` validates current membership, field existence and every source's own
and destination bounds before the first saved-field write. It copies the whole
selected projection and commits valid last. Invalid values preserve the prior
snapshot. `clear()` disables validity without changing either projection; live
State may be absent. Saved State is reclaimed by its ordinary detach lifecycle.

`restore()` first requires a valid snapshot and validates every field. It then
runs `before_restore`, resolves the original online player by captured UUID, and
revalidates all membership, validity and bounds before writing any live field.
If preparation changed or detached State, restoration may reject after those
preparation effects occurred. There is no arbitrary preparation rollback. The
restored projection uses values current after preparation. All live field writes
commit before `after_restore`. Callback returns cannot bypass scratch/lock
cleanup. A pack checkpoint lock rejects synchronous save/restore/clear reentry,
including attempts from preparation, continuation and session stop callbacks.

Task restoration is explicit: cancel ephemeral sessions/tasks in preparation,
then optionally start fresh registered definitions in continuation. Their new
identities, admission and cleanup use their normal contracts. No private task
cursor, lifetime marker, entity, position or inventory is captured or restored.
Callers must select which reward claims remain outside rewind; otherwise restoring
an older claim index may intentionally permit rewards again. The example leaves
saved quest claims and accumulated points outside round rewind.

## Schema migrations and reload/reconnect

The saved component's State schema version **is** the saved format. Reordering or
changing field meaning requires a normal State version increment and explicit
migration hook, or invalidating the saved-valid flag. A migration that rejects
old snapshots can return `SavedRound::valid.bind_to(ctx.holder(), false).disable()`
from the canonical `StateLifecycle::migrate` hook. Checkpoint never invents its
own parallel schema version or copies old private runtime state.

Saved fields and validity retain normal State persistence through reload and
player reconnect. Loading a pack or reconnecting does not call restore. Call
restore explicitly when your gameplay policy permits it. Removing a datapack
cannot run an invented unload/save callback. No cross-server save-file format,
automatic checkpoint schedule or offline task execution is provided.

Portable export regressions are in
`sand-core/tests/gameplay_journal_checkpoint.rs`; the two-player native proof is
`scripts/validate_journal_checkpoint.py`. Its version and evidence limits are
recorded in the corresponding testing evidence document.
