use sand_core::{EventDescriptor, EventDispatch, TrackedSource, TrackedTransition, TransitionKind};

fn empty_handler() -> Vec<String> {
    Vec::new()
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "boolean_handler",
        id_override: None,
        make: empty_handler,
        dispatch: EventDispatch::Tracked(TrackedTransition::new(
            "conflicting_tracker",
            TrackedSource::BooleanCondition {
                description: "boolean source",
                condition: "entity @s[tag=test]",
            },
            TransitionKind::BecameTrue,
        )),
    }
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "score_handler",
        id_override: None,
        make: empty_handler,
        dispatch: EventDispatch::Tracked(TrackedTransition::new(
            "conflicting_tracker",
            TrackedSource::Score {
                description: "score source",
                objective: "points",
                criterion: "dummy",
            },
            TransitionKind::ScoreChanged,
        )),
    }
}

#[test]
fn conflicting_tracker_declarations_fail_the_normal_export_path() {
    let error = sand_core::try_export_components_json("conflictpack").unwrap_err();
    let message = error.to_string();
    assert!(message.contains("tracked_transition"));
    assert!(message.contains("conflicting transition tracker `conflicting_tracker`"));
    assert!(message.contains("boolean source"));
    assert!(message.contains("score source"));
}
