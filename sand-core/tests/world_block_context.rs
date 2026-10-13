//! Runtime block actions preserve dimension scope and cached typed validation.
use sand::prelude::*;

fn body() -> Actions {
    static BODY: std::sync::OnceLock<Actions> = std::sync::OnceLock::new();
    BODY.get_or_init(|| {
        let world = WorldContext::current();
        let dimension: sand::registry::DimensionId = "minecraft:the_nether".parse().unwrap();
        mcfunction![
            world.in_dimension(dimension, |world| {
                let block = world.block(BlockPos::absolute(1, 64, 2));
                let chest = block.matches(vanilla::Block::Chest);
                let slot = block.inventory().slot(0).unwrap();
                let seed = block.data().typed_path::<i64>(NbtPath::root("LootTableSeed"));
                mcfunction![
                    block.replace(vanilla::Block::Chest);
                    when(chest.clone()).then_all(mcfunction![
                        slot.set(ItemStack::new(vanilla::Item::Diamond.into())).unwrap();
                        seed.get();
                    ]);
                    when(!chest).then_all([block.destroy()]);
                ]
            });
            world.block(BlockPos::here()).replace(vanilla::Block::Stone);
        ]
    })
    .clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor {
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "world", make: body,
    }
}

fn verify(namespace: &str, json: &str) {
    let records: Vec<serde_json::Value> = serde_json::from_str(json).unwrap();
    let functions: Vec<_> = records.iter().filter(|r| r["dir"] == "function").collect();
    let root = functions.iter().find(|r| r["path"] == "world").unwrap();
    let body = root["content"].as_str().unwrap();
    let dimension_prefix = format!("execute in minecraft:the_nether run function {namespace}:");
    let mut lines = body.lines();
    let scoped = lines
        .next()
        .unwrap()
        .strip_prefix(&dimension_prefix)
        .unwrap();
    assert_eq!(lines.next(), Some("setblock ~ ~ ~ minecraft:stone"));
    assert_eq!(
        lines.next(),
        None,
        "dimension change is scoped to one owned action"
    );
    let dimension_body = functions.iter().find(|r| r["path"] == scoped).unwrap()["content"]
        .as_str()
        .unwrap();
    assert!(dimension_body.starts_with("setblock 1 64 2 minecraft:chest\n"));
    let queries: Vec<_> = functions
        .iter()
        .filter(|r| {
            r["content"]
                .as_str()
                .unwrap()
                .contains("if block 1 64 2 minecraft:chest")
        })
        .collect();
    assert_eq!(
        queries.len(),
        1,
        "positive and negative guards share one typed query"
    );
    assert_eq!(
        queries[0]["content"],
        "execute if block 1 64 2 minecraft:chest run return 1\nreturn 0"
    );
    let query = queries[0]["path"].as_str().unwrap();
    assert!(dimension_body.contains(&format!("if function {namespace}:{query}")));
    assert!(dimension_body.contains(&format!("unless function {namespace}:{query}")));
    // Guard bodies are owned functions. Verify the call graph rooted under
    // execute-in rather than requiring writes to be flattened inline.
    let bodies: std::collections::BTreeMap<_, _> = functions
        .iter()
        .map(|r| (r["path"].as_str().unwrap(), r["content"].as_str().unwrap()))
        .collect();
    let namespace_prefix = format!("{namespace}:");
    let mut pending = vec![scoped];
    let mut visited = std::collections::BTreeSet::new();
    let mut scoped_body = String::new();
    while let Some(path) = pending.pop() {
        if !visited.insert(path) {
            continue;
        }
        let body = bodies[path];
        scoped_body.push_str(body);
        scoped_body.push('\n');
        let words: Vec<_> = body.split_whitespace().collect();
        for pair in words.windows(2) {
            if pair[0] == "function"
                && let Some(target) = pair[1].strip_prefix(&namespace_prefix)
            {
                pending.push(target);
            }
        }
    }
    assert!(
        !scoped_body.contains("execute in "),
        "child actions retain the selected dimension"
    );
    assert!(scoped_body.contains("item replace block 1 64 2 container.0 with minecraft:diamond"));
    assert!(scoped_body.contains("data get block 1 64 2 LootTableSeed"));
    assert!(scoped_body.contains("setblock 1 64 2 minecraft:air destroy"));
    assert!(!json.contains("__sand_local:"));
    assert!(!json.contains("forceload"));
}

#[test]
fn cached_block_gameplay_keeps_dimension_inventory_data_and_condition_scope() {
    let first = sand_core::try_export_components_json("world_context").unwrap();
    verify("world_context", &first);
    assert_eq!(
        first,
        sand_core::try_export_components_json("world_context").unwrap()
    );
    let second = sand_core::try_export_components_json("second_world").unwrap();
    verify("second_world", &second);
    assert!(!second.contains("world_context:"));
}

#[sand_macros::function]
fn dimension_macro_actions() {
    let args = FunctionMacroArgs::new(["block"]).unwrap();
    let dimension: sand::registry::DimensionId = "minecraft:the_nether".parse().unwrap();
    WorldContext::current().in_dimension(dimension, |world| {
        mcfunction![
            args.line("setblock ~ ~ ~ $(block)").unwrap();
            world.block(BlockPos::here()).replace(vanilla::Block::Stone);
        ]
    });
}

#[test]
fn dimension_macro_actions_keep_the_argument_bearing_frame() {
    let json = sand_core::try_export_components_json("world_macros").unwrap();
    let records: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    let function = records
        .iter()
        .find(|r| r["path"] == "dimension_macro_actions")
        .unwrap();
    assert_eq!(
        function["content"],
        "$execute in minecraft:the_nether run setblock ~ ~ ~ $(block)\nexecute in minecraft:the_nether run setblock ~ ~ ~ minecraft:stone"
    );
    assert_eq!(
        json,
        sand_core::try_export_components_json("world_macros").unwrap()
    );
}
