# Quest rounds

A saved two-objective personal quest and a separate two-phase, 60-second round.
Both use ordinary player State. Build from this directory with `sand build`.
Default target is 26.3; export tests also cover 26.1 and 26.2.

Run `/function quest_rounds:new_round`, then start sneaking three times. Each
sneak event updates canonical State and checks the saved quest. The session
checks one phase on each loaded tick and cleans up when its last phase completes.
The quest awards 5 then 20 canonical points once, independently of new rounds.

- `/function quest_rounds:cancel` ends owned work and preserves both indices.
- `/function quest_rounds:resume` starts from the saved phase after cancellation
  or reload; completed rounds reject this action.
- `/function quest_rounds:new_round` explicitly resets the round phase and sneak
  counter, without replaying quest rewards.
- `/function quest_rounds:replay_quest` explicitly permits quest rewards again.
- `/function quest_rounds:journal` reports completed quests in chat.

Join and death handlers cancel ephemeral controllers. Reload invalidates tasks;
canonical player State retains its usual persistence. Neither API restores
coroutines, promises offline ticking, or records an arbitrary callback outcome.
Reordering objective indices requires an explicit State schema migration.
For shared matches, attach entity State to a dedicated host per session and
manage participant/team/spectator references explicitly; this example does not
supply matchmaking or team assignment.

`/function quest_rounds:refresh_journal` writes a literal snapshot of quest and
round progress into the last hotbar slot. It accepts an empty slot or this same
journal identity; other items are preserved. Refresh again after State changes.
Moving the book elsewhere does not create a persistent tracking record or stop
you from issuing another snapshot. The book never opens automatically.

`/function quest_rounds:save_round` saves only the round phase and sneak counter
into the separately-derived `RoundSave` State. `restore_round` cancels current
round work, revalidates the saved fields, copies them back and explicitly starts
a fresh round controller. `forget_round` disables snapshot validity. The quest
claim index and earned points remain outside rewind, so restoring a round does
not replay quest rewards. A future change to the saved layout must version and
migrate `RoundSave`, or invalidate its valid flag. No positions, inventories,
entities or private task continuations are captured.
