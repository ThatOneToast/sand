//! Sneak through a personal two-phase round and a separately saved quest.
use sand::entity::EntityText;
use sand::prelude::*;
use sand::systems::{Checkpoint, Journal, Lifetime, Progression, Session};

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "quest_rounds", scope = player)]
struct Journey {
    #[state(min = 0, max = 2)]
    quest: Score,
    #[state(min = 0, max = 2)]
    phase: Score,
    sneaks: Score,
    points: Score,
}

// Checkpoint data is an ordinary separately-versioned State projection. The
// quest claim index and accumulated points deliberately stay outside rewind.
#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "quest_rounds", scope = player)]
struct RoundSave {
    #[state(min = 0, max = 2)]
    phase: Score,
    sneaks: Score,
    valid: Flag,
}

#[datapack_component]
fn checkpoint() -> Checkpoint {
    Checkpoint::new("quest_rounds:round_save".parse().unwrap(), RoundSave::valid)
        .field(Journey::phase, RoundSave::phase)
        .field(Journey::sneaks, RoundSave::sneaks)
        .before_restore(|_| round().cancel())
        .after_restore(|_| round().start())
}

#[datapack_component]
fn journal() -> Journal {
    Journal::new(
        "quest_rounds:journal".parse().unwrap(),
        "Journey journal",
        "Sand",
        InventoryIndex::new(8).unwrap(),
    )
    .page(
        EntityText::new()
            .literal("Quest objectives: ")
            .score(Journey::quest)
            .literal(" / 2\nPoints: ")
            .score(Journey::points)
            .literal("\nRound phase: ")
            .score(Journey::phase),
    )
    .replace_owned()
    .on_rejected(|| {
        cmd::tellraw(
            Target::self_(),
            Text::new("Your last hotbar slot needs to be empty or contain this journal."),
        )
    })
}

#[datapack_component]
fn quest() -> Progression {
    Progression::new("quest_rounds:journey".parse().unwrap(), Journey::quest)
        .objective(
            |owner| owner.state(Journey::sneaks).matches(1..).unwrap(),
            |owner| owner.state(Journey::points).add(5),
        )
        .objective(
            |owner| owner.state(Journey::sneaks).matches(3..).unwrap(),
            |owner| owner.state(Journey::points).add(20),
        )
}

#[datapack_component]
fn round() -> Session {
    Session::new(
        Lifetime::new("quest_rounds:round".parse().unwrap(), Ticks::seconds(60))
            .requires::<Journey>()
            .on_start(|ctx| {
                ctx.owner(|_| {
                    cmd::tellraw(
                        Target::self_(),
                        Text::new("Sneak three times to finish this round."),
                    )
                })
            })
            .on_stop(|ctx| {
                ctx.owner(|_| {
                    cmd::tellraw(
                        Target::self_(),
                        Text::new("Round ended. Your quest progress is saved."),
                    )
                })
            }),
        Progression::new("quest_rounds:round_phases".parse().unwrap(), Journey::phase)
            .objective(
                |owner| owner.state(Journey::sneaks).matches(1..).unwrap(),
                |_| cmd::tellraw(Target::self_(), Text::new("First phase complete.")),
            )
            .objective(
                |owner| owner.state(Journey::sneaks).matches(3..).unwrap(),
                |_| cmd::tellraw(Target::self_(), Text::new("Round complete!")),
            ),
    )
}

#[on_event]
fn sneak(_: Event<sand::events::PlayerStartSneakingEvent>) {
    Journey::sneaks.bind().add(1);
    quest().advance();
}

// Reconnection discards owned work; saved quest/phase fields stay canonical State.
#[on_event]
fn reconnect(_: Event<sand::events::OnJoinEvent>) {
    round().cancel();
}
#[on_event]
fn death(_: Event<sand::events::OnDeathEvent>) {
    round().cancel();
}

#[function("quest_rounds:new_round", context = player)]
fn new_round() {
    Journey::sneaks.bind().set(0);
    round().restart();
}
#[function("quest_rounds:resume", context = player)]
fn resume() {
    round().start();
}
#[function("quest_rounds:cancel", context = player)]
fn cancel() {
    round().cancel();
}
#[function("quest_rounds:replay_quest", context = player)]
fn replay_quest() {
    // Explicitly enable quest rewards again. Starting a round never does this.
    quest().reset();
    Journey::sneaks.bind().set(0);
}
#[function("quest_rounds:journal", context = player)]
fn journal_status() {
    when(quest().completed()).then(cmd::tellraw(
        Target::self_(),
        Text::new("Journey complete."),
    ));
}

#[function("quest_rounds:refresh_journal", context = player)]
fn refresh_journal() {
    journal().refresh();
}
#[function("quest_rounds:save_round", context = player)]
fn save_round() {
    checkpoint().save();
}
#[function("quest_rounds:restore_round", context = player)]
fn restore_round() {
    checkpoint().restore();
}
#[function("quest_rounds:forget_round", context = player)]
fn forget_round() {
    checkpoint().clear();
}

#[doc(hidden)]
pub fn __sand_export(namespace: &str, version: &str) {
    println!(
        "{}",
        sand::advanced::try_export_pack_json(namespace, version).unwrap()
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn all_supported_targets_export_deterministically() {
        for version in ["26.1", "26.2", "26.3"] {
            let first = sand::advanced::try_export_pack_json("quest_rounds", version).unwrap();
            assert_eq!(
                first,
                sand::advanced::try_export_pack_json("quest_rounds", version).unwrap()
            );
            assert!(first.contains("progression_advance"));
            assert!(first.contains("session_restart"));
            assert!(first.contains("journal_refresh"));
            assert!(first.contains("checkpoint_restore_commit"));
        }
    }
}
