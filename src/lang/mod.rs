pub mod structure;

use pest::Parser;
use pest_derive::Parser;
use structure::SandTree;

use crate::grammar;

// Define the parser using the grammar file
#[derive(Debug, Parser)]
#[grammar = "grammar/parser.pest"] // Path to the grammar file
pub struct SandParser;

pub fn parse(input: &str) -> Result<SandTree, pest::error::Error<Rule>> {
    let pairs = SandParser::parse(Rule::file, input).map_err(|e| {
        println!("Parsing error: {}", e);
        e
    })?;

    let mut tree = SandTree::new();

    let file = grammar::File::from_pest(pairs.peek().unwrap());
    tree.add_file(file);

    Ok(tree)
}
