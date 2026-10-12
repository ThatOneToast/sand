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
