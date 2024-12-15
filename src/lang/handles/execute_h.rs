use tlogger::prelude::*;
use tree_sitter::Node;

use crate::{
    datapack::{Condition, DataDestination, ExecuteSubcommand, ScoreCondition, StoreType},
    lang::{Location, Statement},
};

pub fn fix_selector(selector: &str) -> String {
    if !selector.starts_with('@') {
        format!("@{}", selector)
    } else {
        selector.to_string()
    }
}

pub fn handle_execute_command(command: Node, source: &str, body_statements: &mut Vec<Statement>) {
    let mut cmd_cursor = command.walk();
    let mut subcommands = Vec::new();

    for child in command.children(&mut cmd_cursor) {
        if child.kind() == "execute_subcommand" {
            if let Some(subcommand) = child.named_children(&mut child.walk()).next() {
                match subcommand.kind() {
                    "execute_as" => {
                        if let Some(selector) = subcommand.child_by_field_name("target") {
                            if let Ok(selector_text) = selector.utf8_text(source.as_bytes()) {
                                subcommands
                                    .push(ExecuteSubcommand::As(fix_selector(selector_text)));
                            }
                        }
                    }
                    "execute_at" => {
                        if let Some(selector) = subcommand.child_by_field_name("target") {
                            if let Ok(selector_text) = selector.utf8_text(source.as_bytes()) {
                                subcommands
                                    .push(ExecuteSubcommand::At(fix_selector(selector_text)));
                            }
                        }
                    }
                    "execute_align" => {
                        if let Some(axes) = subcommand.child_by_field_name("axes") {
                            if let Ok(axes_text) = axes.utf8_text(source.as_bytes()) {
                                subcommands.push(ExecuteSubcommand::Align(axes_text.to_string()));
                            }
                        }
                    }
                    "execute_anchored" => {
                        if let Some(anchor) = subcommand.child_by_field_name("anchor") {
                            if let Ok(anchor_text) = anchor.utf8_text(source.as_bytes()) {
                                subcommands
                                    .push(ExecuteSubcommand::Anchored(anchor_text.to_string()));
                            }
                        }
                    }
                    "execute_facing" => {
                        if let Some(pos) = subcommand.child_by_field_name("pos") {
                            if let Some(location) = process_position(pos, source) {
                                subcommands.push(ExecuteSubcommand::Facing(location));
                            }
                        }
                    }
                    "execute_facing_entity" => {
                        if let Some(selector) = subcommand.child_by_field_name("target") {
                            if let Ok(selector_text) = selector.utf8_text(source.as_bytes()) {
                                subcommands.push(ExecuteSubcommand::FacingEntity(fix_selector(
                                    selector_text,
                                )));
                            }
                        }
                    }
                    "execute_in" => {
                        if let Some(dimension) = subcommand.child_by_field_name("dimension") {
                            if let Ok(dimension_text) = dimension.utf8_text(source.as_bytes()) {
                                subcommands.push(ExecuteSubcommand::In(dimension_text.to_string()));
                            }
                        }
                    }
                    "execute_positioned" => {
                        if let Some(pos) = subcommand.child_by_field_name("pos") {
                            if let Some(location) = process_position(pos, source) {
                                subcommands.push(ExecuteSubcommand::Positioned(location));
                            }
                        }
                    }
                    "execute_positioned_as" => {
                        if let Some(selector) = subcommand.child_by_field_name("target") {
                            if let Ok(selector_text) = selector.utf8_text(source.as_bytes()) {
                                subcommands.push(ExecuteSubcommand::PositionedAs(fix_selector(
                                    selector_text,
                                )));
                            }
                        }
                    }
                    "execute_rotated" => {
                        if let Some(rot) = subcommand.child_by_field_name("rot") {
                            if let Some(location) = process_rotation(rot, source) {
                                subcommands.push(ExecuteSubcommand::Rotated(location));
                            }
                        }
                    }
                    "execute_rotated_as" => {
                        if let Some(selector) = subcommand.child_by_field_name("target") {
                            if let Ok(selector_text) = selector.utf8_text(source.as_bytes()) {
                                subcommands.push(ExecuteSubcommand::RotatedAs(fix_selector(
                                    selector_text,
                                )));
                            }
                        }
                    }
                    "execute_if" | "execute_unless" => {
                        if let Some(condition) = subcommand.child_by_field_name("condition") {
                            if let Some(parsed_condition) = process_condition(condition, source) {
                                if subcommand.kind() == "execute_if" {
                                    subcommands.push(ExecuteSubcommand::If(parsed_condition));
                                } else {
                                    subcommands.push(ExecuteSubcommand::Unless(parsed_condition));
                                }
                            }
                        }
                    }
                    "execute_store" => {
                        if let Some(store_type) = process_store(subcommand, source) {
                            subcommands.push(ExecuteSubcommand::Store(store_type));
                        }
                    }
                    "execute_run" => {
                        if let Some(command_node) = subcommand.child_by_field_name("command") {
                            if let Some(run_cmd) =
                                command_node.named_children(&mut command_node.walk()).next()
                            {
                                match run_cmd.kind() {
                                    "say_command" | "text" => {
                                        if let Ok(text) = run_cmd.utf8_text(source.as_bytes()) {
                                            subcommands.push(ExecuteSubcommand::Run(Box::new(
                                                Statement::Say(text.trim().to_string()),
                                            )));
                                        }
                                    }
                                    "target_selector" => {
                                        // Handle target selector
                                        if let Ok(selector) = run_cmd.utf8_text(source.as_bytes()) {
                                            debug!("Processing target", "selector: {}", selector);
                                            // Handle based on context
                                        }
                                    }
                                    "execute_command" => {
                                        let mut nested_statements = Vec::new();
                                        handle_execute_command(
                                            run_cmd,
                                            source,
                                            &mut nested_statements,
                                        );
                                        if let Some(Statement::Execute(nested_subcommands)) =
                                            nested_statements.last()
                                        {
                                            subcommands.push(ExecuteSubcommand::Run(Box::new(
                                                Statement::Execute(nested_subcommands.clone()),
                                            )));
                                        }
                                    }
                                    "effect_command" => {
                                        if let Some(statement) =
                                            process_effect_command(run_cmd, source)
                                        {
                                            subcommands
                                                .push(ExecuteSubcommand::Run(Box::new(statement)));
                                        }
                                    }
                                    _ => {
                                        debug!("Unhandled run command type:", "{}", run_cmd.kind());
                                    }
                                }
                            }
                        }
                    }
                    _ => debug!("Unhandled execute subcommand:", "{}", subcommand.kind()),
                }
            }
        }
    }

    if !subcommands.is_empty() {
        body_statements.push(Statement::Execute(subcommands));
    }
}

fn process_effect_command(node: Node, source: &str) -> Option<Statement> {
    let target = node
        .child_by_field_name("target")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())?;

    let effect = node
        .child_by_field_name("effect_type")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())?;

    let duration = node
        .child_by_field_name("duration")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())?;

    let amplifier = node
        .child_by_field_name("amplifier")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())?;

    Some(Statement::Effect(
        fix_selector(target),
        effect.to_string(),
        duration.to_string(),
        amplifier.to_string(),
    ))
}

fn process_score_condition(node: Node, source: &str) -> Option<ScoreCondition> {
    let target = node
        .child_by_field_name("target")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())?;
    let objective = node
        .child_by_field_name("objective")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())?;

    if let Some(range) = node.child_by_field_name("range") {
        let range_text = range.utf8_text(source.as_bytes()).ok()?;
        Some(ScoreCondition::Matches(
            target.to_string(),
            objective.to_string(),
            range_text.to_string(),
        ))
    } else if let (Some(operator), Some(source_target), Some(source_objective)) = (
        node.child_by_field_name("operator"),
        node.child_by_field_name("source"),
        node.child_by_field_name("source_objective"),
    ) {
        Some(ScoreCondition::Compared(
            target.to_string(),
            objective.to_string(),
            operator.utf8_text(source.as_bytes()).ok()?.to_string(),
            source_target.utf8_text(source.as_bytes()).ok()?.to_string(),
            source_objective
                .utf8_text(source.as_bytes())
                .ok()?
                .to_string(),
        ))
    } else {
        None
    }
}

pub fn process_position(node: Node, source: &str) -> Option<Location> {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 0.0;

    if let Some(x_node) = node.child_by_field_name("x") {
        if let Ok(x_text) = x_node.utf8_text(source.as_bytes()) {
            x = parse_coordinate(x_text);
        }
    }
    if let Some(y_node) = node.child_by_field_name("y") {
        if let Ok(y_text) = y_node.utf8_text(source.as_bytes()) {
            y = parse_coordinate(y_text);
        }
    }
    if let Some(z_node) = node.child_by_field_name("z") {
        if let Ok(z_text) = z_node.utf8_text(source.as_bytes()) {
            z = parse_coordinate(z_text);
        }
    }

    Some(Location { x, y, z })
}

pub fn process_rotation(node: Node, source: &str) -> Option<Location> {
    let mut yaw = 0.0;
    let mut pitch = 0.0;

    if let Some(yaw_node) = node.child_by_field_name("yaw") {
        if let Ok(yaw_text) = yaw_node.utf8_text(source.as_bytes()) {
            yaw = parse_coordinate(yaw_text);
        }
    }
    if let Some(pitch_node) = node.child_by_field_name("pitch") {
        if let Ok(pitch_text) = pitch_node.utf8_text(source.as_bytes()) {
            pitch = parse_coordinate(pitch_text);
        }
    }

    Some(Location {
        x: yaw,
        y: pitch,
        z: 0.0,
    })
}

pub fn parse_coordinate(text: &str) -> f32 {
    if text == "~" {
        0.0
    } else if text.starts_with('~') {
        text[1..].parse().unwrap_or(0.0)
    } else {
        text.parse().unwrap_or(0.0)
    }
}

pub fn process_condition(node: Node, source: &str) -> Option<Condition> {
    match node.kind() {
        "condition_block" => {
            let pos = node.child_by_field_name("pos")?;
            let block = node.child_by_field_name("block")?;
            if let (Some(location), Ok(block_text)) = (
                process_position(pos, source),
                block.utf8_text(source.as_bytes()),
            ) {
                Some(Condition::Block(location, block_text.to_string()))
            } else {
                None
            }
        }
        "condition_entity" => {
            let selector = node.child_by_field_name("target")?;
            if let Ok(selector_text) = selector.utf8_text(source.as_bytes()) {
                Some(Condition::Entity(selector_text.to_string()))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn process_store(node: Node, source: &str) -> Option<StoreType> {
    let mode = node.child_by_field_name("mode")?;
    let target = node.child_by_field_name("target")?;
    let path = node.child_by_field_name("path")?;

    match mode.kind() {
        "result" => {
            let type_node = node.child_by_field_name("type")?;
            let scale_node = node.child_by_field_name("scale")?;
            if let (Ok(path_text), Ok(type_text), Ok(scale_text)) = (
                path.utf8_text(source.as_bytes()),
                type_node.utf8_text(source.as_bytes()),
                scale_node.utf8_text(source.as_bytes()),
            ) {
                Some(StoreType::Result(
                    process_store_target(target, source)?,
                    path_text.to_string(),
                    type_text.to_string(),
                    scale_text.parse().unwrap_or(1.0),
                ))
            } else {
                None
            }
        }
        "success" => {
            if let Ok(path_text) = path.utf8_text(source.as_bytes()) {
                Some(StoreType::Success(
                    process_store_target(target, source)?,
                    path_text.to_string(),
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn process_store_target(node: Node, source: &str) -> Option<DataDestination> {
    match node.kind() {
        "block" => {
            let pos = node.child_by_field_name("pos")?;
            Some(DataDestination::Block(process_position(pos, source)?))
        }
        "entity" => {
            let selector = node.child_by_field_name("target")?;
            if let Ok(selector_text) = selector.utf8_text(source.as_bytes()) {
                Some(DataDestination::Entity(selector_text.to_string()))
            } else {
                None
            }
        }
        "storage" => {
            let source_node = node.child_by_field_name("source")?;
            if let Ok(source_text) = source_node.utf8_text(source.as_bytes()) {
                Some(DataDestination::Storage(source_text.to_string()))
            } else {
                None
            }
        }
        _ => None,
    }
}
