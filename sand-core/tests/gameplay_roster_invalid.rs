use sand::prelude::*;
use sand::systems::Roster;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, EntityStateEnum)]
enum Role {
    Fighter = 1,
}
#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "roster_invalid", scope = player)]
struct Player {
    member: Score,
    #[state(min = 0, max = 16)]
    bounded: Score,
    #[state(default = 1)]
    nonzero: Score,
    #[state(default = "Role::Fighter")]
    role: EntityEnum<Role>,
}
#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "roster_invalid", scope = entity)]
struct Host {
    member: Score,
}
static CASE: AtomicUsize = AtomicUsize::new(0);
#[datapack_component]
fn invalid() -> Roster<Role> {
    let (member, capacity) = match CASE.load(Ordering::Relaxed) {
        0 => (Player::member, 0),
        1 => (Player::member, 129),
        2 => (Player::bounded, 8),
        3 => (Player::nonzero, 8),
        _ => (Host::member, 8),
    };
    Roster::new(
        "roster_invalid:arena".parse().unwrap(),
        member,
        Player::role,
        capacity,
    )
}
#[test]
fn invalid_rosters_fail_export_with_actionable_diagnostics() {
    for (case, expected) in [
        (0, "1..=128"),
        (1, "1..=128"),
        (2, "0..=i32::MAX"),
        (3, "default-zero"),
        (4, "player-scoped"),
    ] {
        CASE.store(case, Ordering::Relaxed);
        let error = sand::advanced::try_export_pack_json("roster_invalid", "26.3")
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "case {case}: {error}");
    }
}
