//! Sand-owned player-state predicate emission for the export pipeline.
//!
//! Built-in player-state events reference generated entity predicates rather
//! than selector NBT; this module recognizes those Sand-owned condition
//! fragments and renders the internal predicate JSON.

/// Returns the output path and entity-predicate flag for a Sand-owned player
/// state predicate. Custom `TickCondition`s are deliberately left untouched.
pub(crate) fn sand_player_state_predicate(condition: &str) -> Option<(&'static str, &'static str)> {
    match condition {
        "predicate __sand_local:__sand/player_sneaking" => {
            Some(("__sand/player_sneaking", "is_sneaking"))
        }
        "predicate __sand_local:__sand/player_sprinting" => {
            Some(("__sand/player_sprinting", "is_sprinting"))
        }
        "predicate __sand_local:__sand/player_swimming" => {
            Some(("__sand/player_swimming", "is_swimming"))
        }
        "predicate __sand_local:__sand/player_on_fire" => {
            Some(("__sand/player_on_fire", "is_on_fire"))
        }
        _ => None,
    }
}

pub(crate) fn collect_sand_player_state_predicates(
    condition: &crate::condition::Condition,
    predicates: &mut std::collections::BTreeMap<&'static str, &'static str>,
) {
    match condition {
        crate::condition::Condition::Predicate(path) => {
            if let Some((predicate_path, flag)) =
                sand_player_state_predicate(&format!("predicate {path}"))
            {
                predicates.insert(predicate_path, flag);
            }
        }
        crate::condition::Condition::Raw(fragment) => {
            if let Some((predicate_path, flag)) = sand_player_state_predicate(fragment) {
                predicates.insert(predicate_path, flag);
            }
        }
        crate::condition::Condition::Not(inner) => {
            collect_sand_player_state_predicates(inner, predicates);
        }
        crate::condition::Condition::All(conditions)
        | crate::condition::Condition::Any(conditions) => {
            for condition in conditions {
                collect_sand_player_state_predicates(condition, predicates);
            }
        }
        crate::condition::Condition::Score { .. }
        | crate::condition::Condition::ScoreCompare { .. }
        | crate::condition::Condition::Flag { .. }
        | crate::condition::Condition::Entity(_)
        | crate::condition::Condition::StorageExists { .. } => {}
    }
}

pub(crate) fn player_state_predicate_json(flag: &str) -> serde_json::Value {
    serde_json::json!({
        "condition": "minecraft:entity_properties",
        "entity": "this",
        "predicate": { "flags": { flag: true } },
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        collect_sand_player_state_predicates, player_state_predicate_json,
        sand_player_state_predicate,
    };
    use crate::events::{PlayerSwimmingEvent, SandEvent};

    #[test]
    fn player_state_events_use_predicate_flags() {
        let dispatch: crate::events::SandEventDispatch = PlayerSwimmingEvent::dispatch();
        let condition = match dispatch {
            crate::events::SandEventDispatch::TickCondition(condition) => condition,
            _ => panic!("player swimming must be tick-polled"),
        };
        assert_eq!(
            sand_player_state_predicate(&condition),
            Some(("__sand/player_swimming", "is_swimming"))
        );
        assert!(!condition.contains("nbt={"));
        assert_eq!(
            player_state_predicate_json("is_swimming"),
            json!({
                "condition": "minecraft:entity_properties",
                "entity": "this",
                "predicate": { "flags": { "is_swimming": true } },
            })
        );
        assert_eq!(
            format!(
                "execute as @a if {} at @s run function audit:while_swimming",
                condition.replace("__sand_local:", "audit:")
            ),
            "execute as @a if predicate audit:__sand/player_swimming at @s run function audit:while_swimming"
        );
    }

    #[test]
    fn nested_persistent_conditions_collect_sand_owned_predicates() {
        let condition = crate::condition::Condition::all([
            crate::condition::Condition::entity("@s[tag=ready]"),
            !crate::condition::Condition::any([
                crate::condition::Condition::predicate("__sand_local:__sand/player_sneaking"),
                crate::condition::Condition::predicate("__sand_local:__sand/player_sprinting"),
            ]),
        ]);
        let mut predicates = std::collections::BTreeMap::new();
        collect_sand_player_state_predicates(&condition, &mut predicates);
        assert_eq!(
            predicates,
            std::collections::BTreeMap::from([
                ("__sand/player_sneaking", "is_sneaking"),
                ("__sand/player_sprinting", "is_sprinting"),
            ])
        );
    }

    #[test]
    fn all_owned_state_predicates_have_expected_flags() {
        assert_eq!(
            sand_player_state_predicate("predicate __sand_local:__sand/player_sneaking"),
            Some(("__sand/player_sneaking", "is_sneaking"))
        );
        assert_eq!(
            sand_player_state_predicate("predicate __sand_local:__sand/player_sprinting"),
            Some(("__sand/player_sprinting", "is_sprinting"))
        );
        assert_eq!(
            sand_player_state_predicate("predicate __sand_local:__sand/player_on_fire"),
            Some(("__sand/player_on_fire", "is_on_fire"))
        );
    }
}
