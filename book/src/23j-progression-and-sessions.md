# Quests and session phases

`Progression<K>` checks ordered objectives using one canonical State `Score`.
`Session<K>` adds bounded `Lifetime` ownership, loaded-tick checks and cleanup.
Both support 26.1, 26.2 and 26.3. The
[quest rounds example](../../examples/quest_rounds/) combines a saved quest,
event-driven progress and a separately resettable round.

## Ordered objectives

The progress score stores the zero-based index of the next objective. Zero is
the first objective; the objective count is terminal completion. Use a registered
owner-scoped integer Score with default zero, no automatic ticking, the dummy
criterion, and bounds covering zero through objective count. Global State is
rejected because it cannot represent independent owners. Player State requires
`Progression<PlayerKind>`; entity State can represent dedicated shared hosts.

```rust,ignore
#[derive(State)]
#[state(namespace = "journey", scope = player)]
struct Journey {
    #[state(min = 0, max = 2)]
    objective: Score,
    discoveries: Score,
    reward_points: Score,
}

#[datapack_component]
fn quest() -> Progression {
    Progression::new("journey:exploration".parse().unwrap(), Journey::objective)
        .objective(
            |owner| owner.state(Journey::discoveries).matches(1..).unwrap(),
            |owner| owner.state(Journey::reward_points).add(5),
        )
        .objective(
            |owner| owner.state(Journey::discoveries).matches(3..).unwrap(),
            |owner| owner.state(Journey::reward_points).add(20),
        )
}
```

After an event updates discoveries, call `quest().advance()` as that owner.
One invocation checks and consumes only the current objective. It commits the
next index before running isolated owner-context reward actions. Repeated calls
at terminal completion do nothing. Missing/current-version-incompatible State,
invalid indices and false predicates do nothing. `completed()` includes current
State membership. Definitions support one to 256 objectives.

The progress field is the sole completion record; do not also maintain a quest
flag or mutate the index in reward callbacks. Progressions sharing a field reject
synchronous `advance` and `reset` calls throughout the reward callback, including
calls for other owners. Commands execute synchronously, so independent owners
can progress normally afterward. An authored `return` cannot skip lock release.

Reward actions have their ordinary semantics. A failed inventory placement does
not roll back progress or other rewards. When delivery success must gate a quest,
perform the checked operation first and record its success in canonical State;
then make that State the objective predicate. No general transaction is implied.

`reset()` explicitly writes zero without running rewards and permits replay.
Only current attached State is changed. Changing objective order or inserting an
objective changes the meaning of saved indices: use the State schema's explicit
migration hooks to remap them. Reload/reconnect does not reset player progress.

## Bounded session lifecycle

Compose a `Lifetime<K>` and a `Progression<K>` with `Session::new`. Register only
the Session; its registration includes both definitions. Their IDs may differ.
The owner must have the progression State attached before admission. A dedicated
entity host with entity State represents a shared match; two hosts produce two
independent sessions. Explicit participant/team/spectator State and selection
remain the author's policy.

| Action | Prior owned work | Progress | New work |
| --- | --- | --- | --- |
| `start()` | Cancel this owner's controller and descendants | Preserve | Admit unless complete/missing State |
| `restart()` | Cancel this owner's controller and descendants | Set zero | Attempt admission |
| `reset()` | Cancel even if State is missing | Set zero when attached | None |
| `cancel()` | Cancel even if State is missing | Preserve | None |

A failed admission after restart leaves progress zero. The framework cannot undo
arbitrary start/stop callbacks. Reset additional game data through canonical State
in the Lifetime callbacks, using appropriate membership guards when State may
have been detached.

Each loaded tick preserves the Lifetime's validation/tick callbacks, then checks
one objective as the exact owner. Completion stops the controller and cancels its
descendants. Missing progression State also stops it on the next loaded tick.
Required/owned State, expiry, owner loss and cleanup retain the normal Lifetime
contract. Owned instance State is detached and task entities/scores are reclaimed;
progression State remains owned by its original player/entity.

Rewards reject session control reentry. Stop callbacks also reject synchronous
start/restart/reset/cancel, so a callback cannot recursively restart a round or
undo cleanup. Schedule the next round through a subsequent event or tick.
Instance identity is captured before callbacks and reacquired afterward, so
cancellation or dimension transfer cannot continue work on a detached instance.

## Persistence boundaries and evidence

Canonical State follows its own ownership and migration rules. Controllers are
ephemeral Lifetime tasks: reload invalidates them; explicit `start()` may resume
saved progress afterward. Unloaded instances do not run or force chunks, and
invalidated unloaded descendants clean up when they become available under the
Lifetime contract. There is no automatic checkpoint/task restoration, save-file
format, reconnect policy, team assignment or full matchmaking subsystem.

Export tests are in `sand-core/tests/gameplay_progression.rs`; native 26.3 proof
is `scripts/validate_progression.py`. The proof uses two independent marker hosts
and checks one-stage dispatch, once-per-reset rewards, callback reentry,
cancellation/reset, descendants and score cleanup, State detachment and reload
preservation followed by explicit resume. It does not establish real-player
reconnect transport or offline ticking.

## Forward quest branches

`Progression::branch(from, to, condition)` selects a conditional forward successor
after the current objective becomes eligible. The first matching branch wins;
otherwise the next sequential objective is used. Only the current objective's
reward runs. Skipped objectives receive no reward. `to == objective_count` means
completion. Forward-only indices keep rewards once per explicit reset; changing
branch meanings or objective ordering requires a saved State migration.

For example, `.branch(0, 2, |owner| owner.state(Quest::reputation).matches(10..).unwrap())`
skips objective 1 when objective 0 completes with sufficient reputation. Use the
ordinary objective condition for prerequisites. At most 256 branches are accepted;
cycles, invalid indices and out-of-range terminal indices fail export.

For explicit multiplayer arenas, compose Session with `Roster<Role>` over ordinary
player State. See [bounded arena rosters](../../docs/gameplay/arena-rosters.md) and
the [cooperative arena example](../../examples/cooperative_arena/README.md).
