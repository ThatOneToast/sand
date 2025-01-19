use sand_commands::prelude::*;
use tree_sitter::{Language, Node, Parser, Tree};
pub mod structure;

// Enum to represent different node types in your language
#[derive(Debug)]
pub enum NodeType<'a> {
    Function {
        name: &'a str,
        body: Vec<Box<dyn MinecraftCommand>>,
    },
    FunctionCall {
        name: &'a str,
        arguments: Vec<&'a str>,
    },
    StringLiteral(&'a str),
    Unknown,
}

pub struct AstBuilder<'a> {
    source: &'a str,
}

impl<'a> AstBuilder<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn parse_tree(&self, language: &Language) -> Result<Tree, &'static str> {
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .map_err(|_| "Failed to set language")?;
        parser
            .parse(self.source, None)
            .ok_or("Failed to parse source")
    }

    pub fn traverse_tree(&self, tree: &Tree) -> Vec<NodeType<'a>> {
        let mut nodes = Vec::new();
        let root = tree.root_node();

        // Directly iterate through source_file's children
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.kind() == "function_definition" {
                if let Some(func_node) = self.process_function(child) {
                    nodes.push(func_node);
                }
            }
        }

        nodes
    }

    pub fn process_function(&self, node: Node) -> Option<NodeType<'a>> {
        let name_node = node
            .children(&mut node.walk())
            .find(|child| child.kind() == "identifier")?;
        let name = name_node.utf8_text(self.source.as_bytes()).ok()?;

        let mut body_statements = Vec::new();
        let block_node = node
            .children(&mut node.walk())
            .find(|child| child.kind() == "block")?;

        for command in block_node.named_children(&mut block_node.walk()) {}

        Some(NodeType::Function {
            name,
            body: body_statements,
        })
    }
}
