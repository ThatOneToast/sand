use parser::object::parse_object;
use sand_commands::prelude::*;
pub mod structure;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use structure::Object;

pub mod parser;

// Define the parser using the grammar file
#[derive(Debug, Parser)]
#[grammar = "grammar/parser.pest"] // Path to the grammar file
pub struct SandParser;

pub fn parse(input: &str) -> Result<SandTree, pest::error::Error<Rule>> {
    let mut tree = SandTree::new();

    let pairs = SandParser::parse(Rule::file, input)?;

    for pair in pairs {
        match pair.as_rule() {
            Rule::object => {
                tree.add_object(parse_object(pair));
            }
            _ => {}
        }
    }

    Ok(tree)
}

#[cfg(test)]
#[test]
fn test() {
    let input = r#"@ItemStack SuperPickaxe(name: String) {
            // This is a comment
            @.TYPE = "minecraft:diamond_pickaxe"
            @.name = name
        }

        


    "#
    .trim();

    match parse(input) {
        Ok(tree) => {
            println!("{:#?}", tree);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}

#[derive(Debug)]
pub struct SandTree {
    objects: Vec<structure::Object>,
}

impl SandTree {
    pub fn new() -> Self {
        SandTree {
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }
}
