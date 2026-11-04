# Progression and session evidence

Reviewed 2026-11-04. `Progression` is an ordered objective track over canonical
owner State; `Session` adds one bounded Lifetime controller per owner. These are
bounded gameplay increments, not complete matchmaking or persistence systems.

## Export and validation

`sand-core/tests/gameplay_progression.rs` exports deterministically for 26.1,
26.2 and 26.3. It checks guarded single-objective dispatch, terminal membership,
commit-before-reward ordering, isolated callback returns, callback lock release,
explicit session control/reset functions, captured instance continuation and
owner/instance-bound Lifetime cleanup. Unit tests in `progression.rs` reject
unregistered/mismatched fields, unsuitable defaults, insufficient bounds,
automatic ticking, global progress and incompatible player ownership. The
maintained `examples/quest_rounds` exports all three targets and shows canonical
State updates from a native sneak event, saved quest rewards, separate round
reset and explicit reconnect/death cancellation.

## Native 26.3 proof

Run from the repository root with an official cached 26.3 server jar:

```sh
python3 scripts/validate_progression.py \
  --jar /path/to/server.jar \
  --output /tmp/sand-progression-proof
```

The proof exports the Rust fixture, then runs a disposable vanilla server. It
creates two loaded marker hosts with attached entity State. Assertions inspect
Minecraft scoreboard values, entity presence and tracked score-holder counts.
The 2026-11-04 run passed every scenario in
[progression-sessions-evidence.json](progression-sessions-evidence.json):

- One objective per invocation and one phase per loaded tick.
- Reward index commits before callbacks; terminal rewards run once per reset.
- Reward reentry cannot advance/reset again or restart/reset its session.
- Stop callbacks cannot recursively restart/reset a session.
- Two simultaneous hosts retain independent progress, rewards and controllers.
- Cancellation/reset reclaim parents, descendants and their score holders.
- Missing progress State ends work on the next loaded tick; explicit reset
  immediately cancels work even after State detachment.
- Reload invalidates ephemeral controllers and descendants while canonical phase
  State remains; explicit start resumes and completes that saved phase.

The final proof reran the complete export fixture and native scenario after adding
an alternate-namespace progression over the same canonical field. Reward callbacks
attempt both same-definition and cross-namespace advance/reset; neither can bypass
the pack-shared field lock. Native assertions require actual emitted marker lines,
so a diagnostic echo containing an assertion token cannot count as a pass. Reload
crosses a scheduled barrier before resuming exact-tick checks, because load hooks
reset the proof counter on the first post-reload tick.

## Limits

Native evidence covers two loaded entity hosts on 26.3. It does not establish
real-player reconnect transport, offline execution, forced chunk loading,
participant/team/spectator assignment, matchmaking, arbitrary task restoration,
or a save/checkpoint file format. Exports on older targets are not native proofs
for those targets. Objective indices persist only through ordinary State schema
semantics; changes to saved index meaning require explicit author migrations.
Rewards retain ordinary action failure behavior and do not promise rollback.
