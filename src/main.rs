use clap::Parser;
use datapack::builder::Datapack;
use sand_commands::{
    prelude::*,
    types::{ItemState, TargetSelector, ToolProperties},
};
use std::path::PathBuf;

#[macro_use]
extern crate sand_commands;

pub mod datapack;
pub mod grammar;
pub mod lang;
pub mod macros;

pub mod tests;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    compile: bool,
}

fn main() {
    let mut datapack = Datapack::new(
        "ToasterTest",
        "An Example datapack written with the sand lang",
        "1.21.4",
        &PathBuf::from("/Users/austinaleshire/Projects/sand/output"),
    );

    let give_function = dp_func!(
        "superpick",
        Give {
            item: "minecraft:diamond_pickaxe".to_string(),
            target: TargetSelector::Current(None),
            block_states: Some(ItemState {
                tool: Some(ToolProperties {
                    default_mining_speed: 30.0,
                    damage_per_block: 10,
                    rules: vec![]
                }),
                damage: Some(50),
                ..Default::default()
            }),
            count: Some(1),
        },
        Title {
            action: sand_commands::types::TitleAction::Title("Granted!".to_string()),
            target: TargetSelector::Current(None),
        }
    );

    let if_score_10_send_message = execute!(
        as TargetSelector::All(None) =>
        if ExecuteConditionCategory::SCORE(
            ExecuteScoreCommand::MATCHES {
                target: TargetSelector::Current(None),
                objective: "message".to_string(),
                range: 20.to_string()
            }
        ) => run Title {
            action: ct::TitleAction::Title("You have been given a toast!".to_string()),
            target: TargetSelector::All(None),
        }
    );

    let if_score_10_set_score_to_0 = execute!(
        as TargetSelector::All(None) =>
        if ExecuteConditionCategory::SCORE(
            ExecuteScoreCommand::MATCHES {
                target: TargetSelector::Current(None),
                objective: "message".to_string(),
                range: 20.to_string()
            }
        ) => run Scoreboard::new(
            ct::ScoreboardAction::Players(
                ct::PlayerAction::Set {
                    target: TargetSelector::Current(None),
                    objective: "message".to_string(),
                    score: 0
                }
            )
        )
    );

    let add_one_to_score = execute!(
        as TargetSelector::All(None) =>
        run Scoreboard::new(
            ct::ScoreboardAction::Players(
                ct::PlayerAction::Add {
                        target: TargetSelector::Current(None),
                        objective: "message".to_string(),
                        score: 1,
                    }
            )
        )
    );

    let tick_send_message = dp_func!(
        "hello",
        if_score_10_send_message,
        if_score_10_set_score_to_0,
        add_one_to_score
    );
    let load_function = dp_func!(
        "load",
        Title {
            action: ct::TitleAction::Title("Loaded!".to_string()),
            target: TargetSelector::All(None),
        }
    );

    datapack.add_load_function(load_function);
    datapack.add_function(give_function);
    datapack.add_tick_function(tick_send_message);
    datapack.build().expect("Failed to build datapack");
    println!("Done!")
}

// fn main() {
//     let cli = Cli::parse();

//     if !cli.compile {
//         println!("Please use --compile flag to run compilation");
//         return;
//     }

//     extern "C" {
//         fn tree_sitter_sand() -> Language;
//     }
//     let language = unsafe { tree_sitter_sand() };

//     let sand_structure = SandStructure::new(PathBuf::from(env::current_dir().unwrap())).unwrap();
//     let grains = sand_structure.sand_to_grains();

//     let ast_builder = AstBuilder::new(grains.as_str());
//     let cwd = env::current_dir().unwrap();
//     let mut datapack = Datapack::new(
//         "Toaster",
//         "A test datapack",
//         "1.21.4",
//         &PathBuf::from(format!("{}/output", cwd.display())),
//     );

//     match ast_builder.parse_tree(&language) {
//         Ok(tree) => {
//             let nodes = ast_builder.traverse_tree(&tree);
//             println!("{:?}", nodes);
//             for node in nodes {
//                 match node {
//                     NodeType::Function { name, body } => {
//                         datapack.add_function(name, body);
//                     }
//                     NodeType::FunctionCall {
//                         name: _,
//                         arguments: _,
//                     } => {}
//                     NodeType::StringLiteral(_text) => {
//                         // is a comment do nothing.
//                     }
//                     NodeType::Unknown => {}
//                 }
//             }
//         }
//         Err(e) => eprintln!("Parsing error: {}", e),
//     }

//     datapack.set_namespace("toast");
//     datapack.build().unwrap();
// }
