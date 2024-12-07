use clap::Parser;
use datapack::builder::Datapack;
use lang::{structure::SandStructure, AstBuilder, NodeType};
use std::{env, path::PathBuf};
use tlogger::prelude::*;
use tree_sitter::Language;

pub mod lang;
pub mod datapack;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    compile: bool,
}
fn main() {
    let cli = Cli::parse();

    if !cli.compile {
        println!("Please use --compile flag to run compilation");
        return;
    }

    extern "C" {
        fn tree_sitter_sand() -> Language;
    }
    let language = unsafe { tree_sitter_sand() };

    let sand_structure = SandStructure::new(PathBuf::from(env::current_dir().unwrap())).unwrap();
    let grains = sand_structure.sand_to_grains();

    let ast_builder = AstBuilder::new(grains.as_str());
    let cwd = env::current_dir().unwrap();
    let mut datapack = Datapack::new("Toaster", "A test datapack", "1.21.4", &PathBuf::from(format!("{}/output", cwd.display())));

    #[cfg(not(debug_assertions))]
    set_debug(false);

    match ast_builder.parse_tree(&language) {
        Ok(tree) => {
            let nodes = ast_builder.traverse_tree(&tree);
            println!("{:?}", nodes);
            for node in nodes {
                match node {
                    NodeType::Function { name, body } => {
                        datapack.add_function(name, body);
                    }
                    NodeType::FunctionCall { name: _, arguments: _ } => {

                    }
                    NodeType::StringLiteral(_text) => {
                        // is a comment do nothing.
                    }
                    NodeType::Unknown => {
                        warn!("Unknown Node", "Got an unknown Node");
                    }
                }
            }
        }
        Err(e) => eprintln!("Parsing error: {}", e),
    }

    datapack.set_namespace("toast");
    datapack.build().unwrap();
}
