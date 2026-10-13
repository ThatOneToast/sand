//! Assertions inspect emitted function resources instead of compiler containers.
use sand::registration::{ComponentContent, DatapackComponent};

pub fn emitted(actions: impl sand::component::IntoCommands) -> Vec<String> {
    let function = sand::component::McFunction::new("test:body".parse().unwrap())
        .commands(actions.into_commands());
    let ComponentContent::Text(content) = function.try_content().unwrap() else {
        panic!("a function must export text");
    };
    content.lines().map(str::to_owned).collect()
}

/// Follow local function calls in an actual export, retaining their exact commands.
#[allow(
    dead_code,
    reason = "shared fixture helper is used only by resource-level assertions"
)]
pub fn reachable(namespace: &str, entry: &str) -> Vec<String> {
    use std::collections::{BTreeMap, BTreeSet};
    let export = sand_core::try_export_components_json(namespace).expect("fixture exports");
    let records: Vec<sand_core::serde_json::Value> =
        sand_core::serde_json::from_str(&export).expect("export is JSON");
    let functions = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .map(|record| {
            (
                record["path"].as_str().unwrap(),
                record["content"].as_str().unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let prefix = format!("{namespace}:");
    let mut pending = vec![entry];
    let mut visited = BTreeSet::new();
    let mut commands = Vec::new();
    while let Some(path) = pending.pop() {
        if !visited.insert(path) {
            continue;
        }
        let body = functions
            .get(path)
            .unwrap_or_else(|| panic!("missing reachable function {path}"));
        for line in body.lines() {
            commands.push(line.to_owned());
            let mut words = line.split_whitespace();
            while let Some(word) = words.next() {
                if word == "function" {
                    if let Some(local) =
                        words.next().and_then(|target| target.strip_prefix(&prefix))
                    {
                        pending.push(local);
                    }
                }
            }
        }
    }
    commands
}
