//! Schema-aware final lowering for resources also emitted by compiler-owned systems.
//!
//! Only known resource fields are visited. Item components, custom data, display
//! text and other opaque payloads must never be recursively rewritten by key name.
use super::{ComponentRecord, ExportResult};
use serde_json::{Value, json};

pub(super) fn lower(records: &mut [ComponentRecord]) -> ExportResult<()> {
    for record in records {
        if record.ext != "json" || record.content_type != "text" {
            continue;
        }
        let transform: fn(&mut Value) = match record.dir.as_str() {
            "predicate" => predicate,
            "advancement" => advancement,
            "item_modifier" => modifier_root,
            "loot_table" => loot_table,
            "villager_trade" => trade,
            "worldgen/biome" => biome,
            _ => continue,
        };
        let mut value: Value =
            serde_json::from_str(&record.content).map_err(|error| error_for(record, error))?;
        transform(&mut value);
        record.content =
            serde_json::to_string_pretty(&value).map_err(|error| error_for(record, error))?;
    }
    Ok(())
}

fn error_for(
    record: &ComponentRecord,
    error: impl std::fmt::Display,
) -> sand_components::error::SandError {
    sand_components::error::SandError::ComponentValidation {
        location: sand_components::ResourceLocation::new(&record.namespace, &record.path)
            .expect("validated resource identity"),
        kind: record.dir.clone(),
        field: "<26.3 schema>".into(),
        message: error.to_string(),
    }
}

fn rename(value: &mut Value, old: &str, new: &str) {
    if let Some(map) = value.as_object_mut()
        && let Some(value) = map.remove(old)
    {
        map.insert(new.into(), value);
    }
}

fn each(value: &mut Value, apply: fn(&mut Value)) {
    if let Some(values) = value.as_array_mut() {
        for value in values {
            apply(value);
        }
    } else {
        apply(value);
    }
}

fn predicate(value: &mut Value) {
    if value.is_array() {
        let mut terms = value.take();
        each(&mut terms, predicate);
        *value = json!({"type":"minecraft:all_of", "terms":terms});
        return;
    }
    rename(value, "condition", "type");
    match value.get("type").and_then(Value::as_str) {
        Some("minecraft:reference") => {
            if let Some(name) = value.get("name").cloned() {
                *value = name;
            }
        }
        Some("minecraft:all_of" | "minecraft:any_of") => {
            if let Some(terms) = value.get_mut("terms") {
                each(terms, predicate);
            }
        }
        Some("minecraft:block_state_property") => {
            value["type"] = json!("minecraft:match_block");
            rename(value, "block", "blocks");
            rename(value, "properties", "state");
        }
        Some("minecraft:inverted") => {
            if let Some(term) = value.get_mut("term") {
                predicate(term);
            }
        }
        _ => {}
    }
}

fn contextual_predicate(value: &mut Value) {
    // `type` is also a legacy entity predicate field (an entity ID, list or
    // tag), so its mere presence cannot identify a native loot predicate.
    let native_predicate = value
        .get("type")
        .and_then(Value::as_str)
        .is_some_and(|kind| {
            matches!(
                kind,
                "minecraft:all_of"
                    | "minecraft:any_of"
                    | "minecraft:damage_source_properties"
                    | "minecraft:enchantment_active_check"
                    | "minecraft:entity_properties"
                    | "minecraft:entity_scores"
                    | "minecraft:environment_attribute_check"
                    | "minecraft:float_value_check"
                    | "minecraft:int_value_check"
                    | "minecraft:inverted"
                    | "minecraft:killed_by_player"
                    | "minecraft:location_check"
                    | "minecraft:match_block"
                    | "minecraft:match_tool"
                    | "minecraft:random_chance"
                    | "minecraft:random_chance_with_enchanted_bonus"
                    | "minecraft:survives_explosion"
                    | "minecraft:table_bonus"
                    | "minecraft:time_check"
                    | "minecraft:weather_check"
            )
        });
    if value.is_object() && !native_predicate && value.get("condition").is_none() {
        *value = json!({"type":"minecraft:entity_properties", "entity":"this", "predicate":value.take()});
    } else {
        predicate(value);
    }
}

fn advancement(value: &mut Value) {
    let Some(criteria) = value.get_mut("criteria").and_then(Value::as_object_mut) else {
        return;
    };
    for criterion in criteria.values_mut() {
        let trigger = criterion
            .get("trigger")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let Some(conditions) = criterion.get_mut("conditions") else {
            continue;
        };
        for key in [
            "player",
            "entity",
            "child",
            "parent",
            "partner",
            "villager",
            "zombie",
            "source",
            "cause",
            "bystander",
            "lightning",
            "projectile",
        ] {
            if let Some(value) = conditions.get_mut(key) {
                contextual_predicate(value);
            }
        }
        if let Some(victims) = conditions.get_mut("victims").and_then(Value::as_array_mut) {
            for victim in victims {
                contextual_predicate(victim);
            }
        }
        if matches!(
            trigger.as_str(),
            "minecraft:any_block_use"
                | "minecraft:default_block_use"
                | "minecraft:placed_block"
                | "minecraft:item_used_on_block"
                | "minecraft:allay_drop_item_on_block"
        ) && let Some(location) = conditions.get_mut("location")
        {
            predicate(location);
        }
        match trigger.as_str() {
            "minecraft:recipe_crafted" | "minecraft:crafter_recipe_crafted" => {
                rename(conditions, "recipe_id", "recipes")
            }
            "minecraft:recipe_unlocked" => rename(conditions, "recipe", "recipes"),
            "minecraft:player_generates_container_loot" => {
                rename(conditions, "loot_table", "loot_tables")
            }
            "minecraft:enter_block"
            | "minecraft:slide_down_block"
            | "minecraft:bee_nest_destroyed" => rename(conditions, "block", "blocks"),
            _ => {}
        }
    }
}

fn conditions(value: &mut Value) {
    rename(value, "conditions", "condition");
    if let Some(condition) = value.get_mut("condition") {
        predicate(condition);
    }
}

fn modifier_root(value: &mut Value) {
    if value.is_array() {
        let mut functions = value.take();
        each(&mut functions, modifier);
        *value = json!({"type":"minecraft:sequence", "functions":functions});
    } else {
        modifier(value);
    }
}

// Lower the four NumberProvider forms exposed by Sand's typed loot model.
// Context integer and float providers are separate registries in 26.3.
fn number(value: &mut Value, integer: bool) {
    if integer && let Some(constant) = value.as_f64() {
        *value = json!(constant.floor() as i32);
        return;
    }
    match value.get("type").and_then(Value::as_str) {
        Some("minecraft:uniform") => {
            for key in ["min", "max"] {
                if let Some(bound) = value.get_mut(key) {
                    number(bound, integer);
                }
            }
        }
        Some("minecraft:score") => {
            let scale = value.as_object_mut().unwrap().remove("scale");
            if !integer || scale.is_some() {
                let mut result = json!({"type":"minecraft:from_int", "input":value.take()});
                if let Some(scale) = scale {
                    result = json!({"type":"minecraft:mul", "inputs":[result, scale]});
                }
                *value = if integer {
                    json!({"type":"minecraft:from_float", "input":{"type":"minecraft:floor", "input":result}})
                } else {
                    result
                };
            }
        }
        Some("minecraft:binomial") if !integer => {
            *value = json!({"type":"minecraft:from_int", "input":value.take()});
        }
        _ => {}
    }
}

fn modifier(value: &mut Value) {
    if value.is_array() {
        each(value, modifier);
        return;
    }
    // Legacy set_loot_table's `type` describes a loot context, not a modifier.
    if value.get("function").and_then(Value::as_str) == Some("minecraft:set_loot_table") {
        value.as_object_mut().unwrap().remove("type");
        rename(value, "name", "loot_table_id");
    }
    rename(value, "function", "type");
    conditions(value);
    let number_field = match value.get("type").and_then(Value::as_str) {
        Some("minecraft:set_count") => Some(("count", true)),
        Some("minecraft:enchant_with_levels") => Some(("levels", true)),
        Some("minecraft:set_damage") => Some(("damage", false)),
        Some("minecraft:enchanted_count_increase" | "minecraft:looting_enchant") => {
            Some(("count", false))
        }
        _ => None,
    };
    if let Some((key, integer)) = number_field
        && let Some(provider) = value.get_mut(key)
    {
        number(provider, integer);
    }
    match value.get("type").and_then(Value::as_str) {
        Some("minecraft:reference") => {
            if let Some(name) = value.get("name").cloned() {
                *value = match value.get("condition").cloned() {
                    Some(condition) => {
                        json!({"type":"minecraft:sequence", "condition":condition, "functions":[name]})
                    }
                    None => name,
                };
            }
        }
        Some("minecraft:sequence") => {
            if let Some(functions) = value.get_mut("functions") {
                each(functions, modifier);
            }
        }
        Some("minecraft:modify_contents" | "minecraft:filtered") => {
            if let Some(modifier_value) = value.get_mut("modifier") {
                each(modifier_value, modifier);
            }
        }
        _ => {}
    }
}

fn loot_entry(value: &mut Value) {
    conditions(value);
    rename(value, "functions", "modifier");
    if let Some(modifier_value) = value.get_mut("modifier") {
        each(modifier_value, modifier);
    }
    if let Some(children) = value.get_mut("children") {
        each(children, loot_entry);
    }
    if value.get("type").and_then(Value::as_str) == Some("minecraft:tag") {
        if let Some(name) = value.get_mut("name")
            && let Some(id) = name.as_str()
        {
            *name = Value::String(format!("#{id}"));
        }
        rename(value, "name", "items");
    }
}

fn loot_table(value: &mut Value) {
    if let Some(functions) = value.get_mut("functions") {
        each(functions, modifier);
    }
    if let Some(pools) = value.get_mut("pools").and_then(Value::as_array_mut) {
        for pool in pools {
            if let Some(rolls) = pool.get_mut("rolls") {
                number(rolls, true);
            }
            if let Some(rolls) = pool.get_mut("bonus_rolls") {
                number(rolls, false);
            }
            // Pool conditions/functions retain their list fields.
            if let Some(predicates) = pool.get_mut("conditions") {
                each(predicates, predicate);
            }
            if let Some(functions) = pool.get_mut("functions") {
                each(functions, modifier);
            }
            if let Some(entries) = pool.get_mut("entries") {
                each(entries, loot_entry);
            }
        }
    }
}

fn trade(value: &mut Value) {
    if let Some(modifiers) = value
        .as_object_mut()
        .and_then(|map| map.remove("given_item_modifiers"))
    {
        let mut modifier_value = modifiers;
        modifier_root(&mut modifier_value);
        value["given_item_modifier"] = modifier_value;
    }
    if let Some(condition) = value.get_mut("merchant_predicate") {
        predicate(condition);
    }
}

fn biome(value: &mut Value) {
    let Some(map) = value.as_object_mut() else {
        return;
    };
    let mut spawns = map.remove("spawners");
    let costs = map.remove("spawn_costs");
    let probability = map.remove("creature_spawn_probability");
    if let Some(categories) = spawns.as_mut().and_then(Value::as_object_mut) {
        for category in categories.values_mut() {
            if let Some(entries) = category.as_array_mut() {
                for entry in entries {
                    if let Some(entry) = entry.as_object_mut()
                        && let (Some(min), Some(max)) =
                            (entry.remove("minCount"), entry.remove("maxCount"))
                    {
                        let count = if min == max {
                            min
                        } else {
                            json!({"type":"minecraft:uniform", "min_inclusive":min, "max_inclusive":max})
                        };
                        entry.insert("count".into(), count);
                    }
                }
            }
        }
    }
    let attributes = map.entry("attributes").or_insert_with(|| json!({}));
    if let Some(attributes) = attributes.as_object_mut() {
        if spawns.is_some() || costs.is_some() {
            attributes.insert("minecraft:gameplay/natural_mob_spawns".into(), json!({"spawns_by_category":spawns.unwrap_or_else(||json!({})), "spawn_costs":costs.unwrap_or_else(||json!({}))}));
        }
        if let Some(probability) = probability {
            attributes.insert(
                "minecraft:gameplay/creature_world_gen_spawn_probability".into(),
                probability,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicate_lists_and_references_use_explicit_26_3_shapes() {
        let mut value = json!([{"condition":"minecraft:reference", "name":"demo:ready"}, {"condition":"minecraft:inverted", "term":{"condition":"minecraft:random_chance", "chance":0.5}}]);
        predicate(&mut value);
        assert_eq!(
            value,
            json!({"type":"minecraft:all_of", "terms":["demo:ready", {"type":"minecraft:inverted", "term":{"type":"minecraft:random_chance", "chance":0.5}}]})
        );
    }

    #[test]
    fn modifier_lowering_does_not_rewrite_custom_data() {
        let data = json!({"condition":"gameplay condition", "function":"gameplay function", "type":"my data"});
        let mut value = json!([{"function":"minecraft:set_custom_data", "tag":data}]);
        modifier_root(&mut value);
        assert_eq!(value["functions"][0]["tag"], data);
        assert_eq!(value["functions"][0]["type"], "minecraft:set_custom_data");
    }

    #[test]
    fn loot_number_providers_preserve_scaled_score_rounding_and_float_binomial() {
        let mut score =
            json!({"type":"minecraft:score", "target":"this", "score":"points", "scale":0.5});
        number(&mut score, true);
        assert_eq!(
            score,
            json!({"type":"minecraft:from_float", "input":{"type":"minecraft:floor", "input":{"type":"minecraft:mul", "inputs":[{"type":"minecraft:from_int", "input":{"type":"minecraft:score", "target":"this", "score":"points"}}, 0.5]}}})
        );
        let mut binomial = json!({"type":"minecraft:binomial", "n":3, "p":0.5});
        number(&mut binomial, false);
        assert_eq!(binomial["type"], "minecraft:from_int");
        let mut constant = json!(-1.5);
        number(&mut constant, true);
        assert_eq!(constant, -2);
    }

    #[test]
    fn conditional_modifier_reference_keeps_its_guard() {
        let mut value = json!({"function":"minecraft:reference", "name":"demo:reward", "conditions":[{"condition":"minecraft:random_chance", "chance":0.5}]});
        modifier(&mut value);
        assert_eq!(
            value,
            json!({"type":"minecraft:sequence", "functions":["demo:reward"], "condition":{"type":"minecraft:all_of", "terms":[{"type":"minecraft:random_chance", "chance":0.5}]}})
        );
    }

    #[test]
    fn biome_spawns_move_to_environment_attributes_without_losing_audio() {
        let mut value = json!({"attributes":{"minecraft:audio/ambient_sounds":{}}, "spawners":{"creature":[{"type":"minecraft:cow","weight":5,"minCount":2,"maxCount":4}]}, "spawn_costs":{"minecraft:cow":{"energy_budget":0.7,"charge":0.2}}});
        biome(&mut value);
        assert!(value.get("spawners").is_none());
        assert!(
            value["attributes"]
                .get("minecraft:audio/ambient_sounds")
                .is_some()
        );
        let spawn = &value["attributes"]["minecraft:gameplay/natural_mob_spawns"];
        assert_eq!(
            spawn["spawns_by_category"]["creature"][0]["count"],
            json!({"type":"minecraft:uniform","min_inclusive":2,"max_inclusive":4})
        );
        assert_eq!(spawn["spawn_costs"]["minecraft:cow"]["charge"], 0.2);
    }

    #[test]
    fn legacy_entity_type_filter_is_wrapped_without_confusing_native_predicates() {
        let mut value = json!({"type":"minecraft:pig"});
        contextual_predicate(&mut value);
        assert_eq!(
            value,
            json!({"type":"minecraft:entity_properties", "entity":"this", "predicate":{"type":"minecraft:pig"}})
        );
        let native = json!({"type":"minecraft:entity_properties", "entity":"this", "predicate":{"type":"minecraft:cow"}});
        let mut value = native.clone();
        contextual_predicate(&mut value);
        assert_eq!(value, native);
    }

    #[test]
    fn advancement_context_and_recipe_fields_are_targeted() {
        let mut value = json!({"criteria":{"recipe":{"trigger":"minecraft:recipe_crafted", "conditions":{"recipe_id":"demo:bread", "player":[{"condition":"minecraft:random_chance","chance":1.0}]}}}});
        advancement(&mut value);
        assert_eq!(
            value["criteria"]["recipe"]["conditions"]["recipes"],
            "demo:bread"
        );
        assert_eq!(
            value["criteria"]["recipe"]["conditions"]["player"]["type"],
            "minecraft:all_of"
        );
    }
}
