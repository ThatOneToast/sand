//! Final command validation phase of the export pipeline.
//!
//! Validates every collected `.mcfunction` line at the string boundary before
//! any record is accepted, using the version-resolved command profile.
#![allow(clippy::result_large_err)]

use super::records::{ComponentRecord, ExportResult};
use crate::component::ComponentExportError;

pub(crate) fn validate_function_records(
    records: &mut [ComponentRecord],
    command_profile: &sand_commands::CommandProfile,
) -> ExportResult<()> {
    for record in records
        .iter_mut()
        .filter(|record| record.ext == "mcfunction")
    {
        let location =
            sand_components::ResourceLocation::new(record.namespace.clone(), record.path.clone())?;
        let mut validated = Vec::new();
        for (index, line) in record.content.lines().enumerate() {
            let line = sand_commands::render::validate_collected_line(line, command_profile)
                .map_err(|error| ComponentExportError::ComponentValidation {
                    location: location.clone(),
                    kind: "function".to_string(),
                    field: format!("commands[{index}].{}", error.field),
                    message: format!(
                        "{} (Minecraft profile {})",
                        error,
                        command_profile.requested_version()
                    ),
                })?;
            validated.push(line);
        }
        record.content = validated.join("\n");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn typed_execute_version_error_reports_function_and_capability() {
        let line = sand_commands::Execute::new()
            .if_items(
                sand_commands::Selector::self_(),
                sand_commands::ItemSlot::MainHand,
                "minecraft:diamond",
            )
            .run_raw("say found");
        let mut records = vec![super::ComponentRecord {
            namespace: "vanilla_plus".to_string(),
            dir: "function".to_string(),
            path: "detect_target".to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: line,
        }];
        let error = super::validate_function_records(
            &mut records,
            &sand_commands::CommandProfile::new("1.20.4", false),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("vanilla_plus:detect_target"), "{error}");
        assert!(error.contains("SAND-COMMAND-VERSION"), "{error}");
        assert!(error.contains("ExecuteItemCondition"), "{error}");
        assert!(
            error.contains("if items entity @s weapon.mainhand minecraft:diamond"),
            "{error}"
        );
        assert!(error.contains("Minecraft profile 1.20.4"), "{error}");
        assert!(error.contains("Minecraft 1.20.5+"), "{error}");
    }

    #[test]
    fn function_validation_fails_before_records_are_accepted_with_owner_context() {
        let mut records = vec![super::ComponentRecord {
            namespace: "audit".to_string(),
            dir: "function".to_string(),
            path: "invalid_selector".to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: "say valid\nkill @e[limit=-1]".to_string(),
        }];
        let profile = sand_commands::CommandProfile::new("1.21.11", false);
        let error = super::validate_function_records(&mut records, &profile)
            .expect_err("malformed typed output must fail before export")
            .to_string();
        assert!(error.contains("audit:invalid_selector"), "{error}");
        assert!(error.contains("commands[1].limit"), "{error}");
        assert!(error.contains("1.21.11"), "{error}");
        assert_eq!(records[0].content, "say valid\nkill @e[limit=-1]");
    }

    #[test]
    fn explicit_raw_function_lines_preserve_literal_and_unmodelled_syntax() {
        let content = [
            r#"tellraw @a {"text":"example @e[limit=-1]"}"#,
            r#"tellraw @a {"text":"function not_a_resource_location"}"#,
            r#"data modify storage pack:test message set value "function not_a_resource_location""#,
            r#"data modify storage pack:test value set value {message:"@e[limit=-1]"}"#,
            "say function plain-text",
            "say scoreboard players operation @s a = @a b",
            r#"custom_command "@e[limit=-1]""#,
            "modded command syntax",
            r#"tellraw @a {"text":"escaped \"@e[limit=-1]\"","extra":[{"text":"function invalid"}]}"#,
            r#"$data modify storage pack:test value set value "$(payload)""#,
        ]
        .join("\n");
        let mut records = vec![super::ComponentRecord {
            namespace: "audit".to_string(),
            dir: "function".to_string(),
            path: "raw".to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: content.clone(),
        }];
        super::validate_function_records(
            &mut records,
            &sand_commands::CommandProfile::unprofiled(),
        )
        .unwrap();
        assert_eq!(records[0].content, content);
    }

    #[test]
    fn typed_text_events_survive_the_command_export_validation_boundary() {
        let entity_type = sand_components::EntityTypeId::minecraft("zombie").unwrap();
        let entity_id =
            sand_commands::EntityHoverId::parse("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let text = sand_commands::Text::new("Inspect")
            .gold()
            .click_change_page(0)
            .hover_entity_with_id(
                entity_type,
                entity_id,
                sand_commands::Text::new("Undead").red(),
            );
        let command = format!("tellraw @s {text}");
        let mut records = vec![super::ComponentRecord {
            namespace: "audit".to_string(),
            dir: "function".to_string(),
            path: "typed_text_events".to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: command.clone(),
        }];

        super::validate_function_records(
            &mut records,
            &sand_commands::CommandProfile::new("1.21.11", false),
        )
        .unwrap();

        assert_eq!(records[0].content, command);
        let json = records[0]
            .content
            .strip_prefix("tellraw @s ")
            .expect("validated tellraw prefix must be preserved");
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(
            value["clickEvent"],
            serde_json::json!({"action": "change_page", "value": 0})
        );
        assert_eq!(
            value["hoverEvent"],
            serde_json::json!({
                "action": "show_entity",
                "type": "minecraft:zombie",
                "id": "123e4567-e89b-12d3-a456-426614174000",
                "name": {"text": "Undead", "color": "red"},
            })
        );
    }

    #[test]
    fn recognized_invalid_function_retains_export_context() {
        let mut records = vec![super::ComponentRecord {
            namespace: "audit".to_string(),
            dir: "function".to_string(),
            path: "invalid_function".to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: "say valid\nfunction not_a_resource_location".to_string(),
        }];
        let profile = sand_commands::CommandProfile::new("1.21.11", false);
        let error = super::validate_function_records(&mut records, &profile)
            .expect_err("recognized malformed function command must fail before export")
            .to_string();
        assert!(error.contains("audit:invalid_function"), "{error}");
        assert!(error.contains("commands[1].id"), "{error}");
        assert!(error.contains("not_a_resource_location"), "{error}");
        assert!(error.contains("1.21.11"), "{error}");
    }

    #[test]
    fn function_validation_preserves_data_shaped_like_commands() {
        let content = concat!(
            "tellraw @a {\"text\":\"example @e[limit=-1]\"}\n",
            "data modify storage audit:messages message set value \"function not_a_resource_location\"\n",
            "say function plain-text\n",
            "modded command {payload:\"scoreboard players operation @s a = @a b\"}",
        );
        let mut records = vec![super::ComponentRecord {
            namespace: "audit".to_string(),
            dir: "function".to_string(),
            path: "raw_data".to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: content.to_string(),
        }];
        super::validate_function_records(
            &mut records,
            &sand_commands::CommandProfile::unprofiled(),
        )
        .unwrap();
        assert_eq!(records[0].content, content);
    }
}
