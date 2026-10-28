# Directly addressed keyed owners

The candidate for #442 replaces the legacy list filter with
`owners_v2."[I;<signed u0>,<signed u1>,<signed u2>,<signed u3>]"`. The key contains
all 128 UUID bits. Vanilla macro expansion of the complete UUID array supplies
this quoted NBT path segment; no identity allocator, scoreboard identity, hash,
truncation, or current/last-owner cache is involved. The owner record still
contains its UUID, `components`, and any unknown retained fields.

The one whole-UUID entity read is inherited from baseline `8bfbfde3` (#498).
Every access still resolves its own executor. An unchanged copy still invokes
the macro. Missing executors clear the UUID before the failed copy; macro
instantiation then fails without accessing a previous owner. Authored values
remain outside macro source, staged and cleaned up in an ordinary function.
A set creates an absent owner, matching the legacy filtered data-modify path.
Get, remove and presence checks do not create owners. State attachment also
creates a missing record before applying field defaults.

## Upgrade protocol

Each registered keyed storage has one shared migration, independent of how many
components use it. Fresh storage is provisioned with `__sand_keyed:{version:2}`.
A legacy `owners` container instead starts at `{version:1,cursor:0}`. Load does
not scan or copy owner rows. The lifecycle tick calls one bounded step per
storage until completion or an error. Migration does not select entities or
players: offline, unloaded, and never-returning owners are included.

Each step visits at most one source row. It snapshots that complete row into
`__sand_keyed.journal.row`, validates a four-element **int array** UUID and a
compound `components`, and computes an exact destination key. Only typed integer
arguments enter the validation macro; malformed UUID strings never enter the
owner-key macro. An intent is persisted before writing an absent destination.
An existing destination without that intent is a conflict, even when identical.
Duplicate legacy UUIDs therefore block instead of being silently merged.

The destination is copied from the snapshot and compared for whole-record
identity. Missing or differing destinations cannot be committed. A completed
journal is revalidated on the next step before its durable `next` cursor is
committed, then removed. Repeating an interrupted cursor commit reuses that
same `next`; it cannot skip the following owner. Copy interruption can retry an
absent destination, but a partial/different destination is preserved and blocks.

Only after every row has committed does a finishing marker precede removal of
the legacy container. The final metadata replacement is `{version:2}`. A reload
between those commands resumes finishing. The canonical map retains every
owner; retirement removes the redundant legacy representation, not owner data.
Steady reads and repeated attachment do not update migration metadata, receipts,
owner timestamps, or counters. Completed storage has no per-tick migration
mutations. Bounds are in rows and generated command work, **not payload bytes,
wall time, or Minecraft internal command-budget units**: a row can contain an
arbitrarily large authored value.

## Availability and conflicts

Keyed macros fail closed until their storage reaches version 2. The complete
component attachment/detachment sequence is guarded, including hooks and final
presence publication, so defaults cannot win over legacy values or publish a
partially initialized keyed component. Player initialization retries each tick.
Explicit entity attachments attempted during upgrade must be retried after
readiness. This is a storage upgrade barrier, not a transaction for arbitrary
authored gameplay surrounding an attachment. Perform upgrades during a
maintenance window if that gameplay cannot tolerate unavailable keyed state.

Errors retain both the complete legacy container and existing destinations.
`__sand_keyed.error`, `.cursor`, and `.journal` identify the blocked row and
write intent; a server message gives inspection instructions. Back up storage
before repair. Repair the current source or destination deliberately, preserving
unknown fields. Clear the error and uncommitted journal only once the ambiguity
has been resolved; do not reset the cursor or delete previously committed map
entries. Do not concurrently edit legacy rows or committed destinations behind
the cursor, forge compiler-owned journal/completion receipts, or run an older
pack writing the legacy layout during migration. Unknown schema versions and
wrong-type or out-of-range cursors stop. Completed receipts are trusted; this is
resumption of compiler-owned work, not recovery from arbitrary external edits.

## Reclamation extension contract

This change does not reclaim owners, empty components, or dormant data. Direct
field removal retains the owner. A later compiler-owned cleanup may prune empty
structures **after cleanup hooks and owned-field deletion**. It must resolve the
executor again after hooks, which can access another owner. Other components,
unknown component fields, and unknown owner fields prevent whole-owner removal.
The persistent key has no allocator state or secondary index requiring cleanup.
Retention policy for dead entities or offline players remains separate work.

## Validation and scope

The maintained `validate_keyed_literals.py` recognizes only the known legacy
and indexed layouts and rejects mixed or unknown paths. It executes exported
get/set/remove/presence operations as well as the historical observer probes.
Independent UUID/value readback checks signed boundaries, multi-component and
unknown data, dormant owners, duplicate/conflicting/malformed input, persistent
journal phases, and the actual load-tag reload sentinel.

Portable revision-one State currently supports scoreboard fields only; it
cannot declare keyed data. Its shared exporter now drains generated lifecycle
resources and resolves local helper references. Portable fixtures and shared
helper isolation/collision tests cover that boundary without inventing another
portable storage schema or widening the public API.

Context reuse, reclamation, and the remaining #442 acceptance criteria are not
claimed by this slice. Historical UUID-read reduction evidence remains under
[keyed owner resolution](keyed-owner-resolution.md).
