use tlogger::prelude::*;
use tree_sitter::{Language, Node, Parser, Tree};

use crate::datapack::{enchantments::Enchantment, ExecuteSubcommand};

pub mod structure;
pub mod handles;

// Enum to represent different node types in your language
#[derive(Debug)]
pub enum NodeType<'a> {
    Function {
        name: &'a str,
        body: Vec<Statement>,
    },
    FunctionCall {
        name: &'a str,
        arguments: Vec<&'a str>,
    },
    StringLiteral(&'a str),
    Unknown,
}

pub type Selector = String;
pub type Effect = String;
pub type Duration = String;
pub type Amplifier = String;

#[derive(Debug, Clone, PartialEq)]
pub enum ClearType {
    Inventory,
    Effect,
}

impl ToString for ClearType {
    fn to_string(&self) -> String {
        match self {
            ClearType::Inventory => "inventory".to_string(),
            ClearType::Effect => "effect".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GamemodeType {
    Survival,
    Creative,
    Spectator,
    Adventure,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EXPType {
    Levels,
    Points,
}

impl EXPType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "levels" => Ok(Self::Levels),
            "points" => Ok(Self::Points),
            _ => Err(format!("Invalid EXPType: {}", s)),
        }
    }
}

impl ToString for EXPType {
    fn to_string(&self) -> String {
        match self {
            Self::Levels => "levels".to_string(),
            Self::Points => "points".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EXPCType {
    Add {
        amount: u32,
        selector: String,
        etype: EXPType,
    },
    Set {
        amount: u32,
        selector: String,
        etype: EXPType,
    },
    Query {
        selector: String,
        etype: EXPType,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl ToString for Location {
    fn to_string(&self) -> String {
        format!("{} {} {}", self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Say(String),
    TimeSet(String),
    TimeQuery(String),
    Tellraw(String),
    Effect(Selector, Effect, Duration, Amplifier),
    Clear(ClearType, Selector),
    Gamemode(GamemodeType, Selector),
    XP(EXPCType),
    Enchant(Enchantment, Selector, u32),
    Execute(Vec<ExecuteSubcommand>),
}

impl ToString for Statement {
    fn to_string(&self) -> String {
        match self {
            Statement::Say(text) => format!("say {}", text),
            Statement::TimeSet(value) => format!("time set {}", value),
            Statement::TimeQuery(query_type) => format!("time query {}", query_type),
            Statement::Tellraw(text) => format!("tellraw @s \"{}\"", text),
            Statement::Effect(selector, effect, duration, amplifier) => {
                format!(
                    "effect give {} {} {} {}",
                    selector, effect, duration, amplifier
                )
            }
            Statement::Clear(clear_type, selector) => match clear_type {
                ClearType::Inventory => format!("clear {}", selector),
                ClearType::Effect => format!("effect clear {}", selector),
            },
            Statement::Gamemode(gtype, selector) => match gtype {
                GamemodeType::Survival => format!("gamemode survival {}", selector),
                GamemodeType::Creative => format!("gamemode creative {}", selector),
                GamemodeType::Spectator => format!("gamemode spectator {}", selector),
                GamemodeType::Adventure => format!("gamemode adventure {}", selector),
            },
            Statement::XP(xptype) => match xptype {
                EXPCType::Add {
                    amount,
                    selector,
                    etype,
                } => format!("xp add {} {} {}", selector, amount, etype.to_string()),
                EXPCType::Set {
                    amount,
                    selector,
                    etype,
                } => format!("xp set {} {} {}", selector, amount, etype.to_string()),
                EXPCType::Query { selector, etype } => {
                    format!("xp query {} {}", selector, etype.to_string())
                }
            },
            Statement::Enchant(enchantment, selector, level) => {
                format!("enchant {} {} {}", selector, enchantment.as_str(), level)
            }
            Statement::Execute(commands) => {
                let subcommands = commands
                    .iter()
                    .map(|cmd| cmd.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("execute {}", subcommands)
            }
        }
    }
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

        for command in block_node.named_children(&mut block_node.walk()) {
            debug!("Processing command of type:", "{}", command.kind());

            match command.kind() {
                "say_command" => {
                    let mut cmd_cursor = command.walk();
                    for child in command.children(&mut cmd_cursor) {
                        if child.kind() == "text" {
                            if let Ok(text) = child.utf8_text(self.source.as_bytes()) {
                                let text = text.trim();
                                debug!("Found say command text:", "{}", text);
                                body_statements.push(Statement::Say(text.to_string()));
                            }
                        }
                    }
                }
                "enchant_command" => {
                    let mut cmd_cursor = command.walk();
                    let children: Vec<Node> = command.children(&mut cmd_cursor).collect();

                    if children.len() >= 3 {
                        if let Ok(selector) = children[1].utf8_text(self.source.as_bytes()) {
                            if let Ok(enchantment) = children[2].utf8_text(self.source.as_bytes()) {
                                if let Ok(level) = children[3].utf8_text(self.source.as_bytes()) {
                                    body_statements.push(Statement::Enchant(
                                        Enchantment::from_str(enchantment).unwrap(),
                                        selector.to_string(),
                                        level.parse::<u32>().unwrap(),
                                    ));
                                }
                            }
                        }
                    }
                }
                "xp_add_command" => {
                    let mut cmd_cursor = command.walk();
                    let children: Vec<Node> = command.children(&mut cmd_cursor).collect();

                    if children.len() >= 3 {
                        if let Ok(amount) = children[1].utf8_text(self.source.as_bytes()) {
                            if let Ok(selector) = children[2].utf8_text(self.source.as_bytes()) {
                                if let Ok(etype) = children[3].utf8_text(self.source.as_bytes()) {
                                    body_statements.push(Statement::XP(EXPCType::Add {
                                        amount: amount.parse::<u32>().unwrap(),
                                        selector: selector.to_string(),
                                        etype: EXPType::from_str(etype).unwrap(),
                                    }));
                                }
                            }
                        }
                    }
                }

                "xp_set_command" => {
                    let mut cmd_cursor = command.walk();
                    let children: Vec<Node> = command.children(&mut cmd_cursor).collect();

                    if children.len() >= 3 {
                        if let Ok(amount) = children[1].utf8_text(self.source.as_bytes()) {
                            if let Ok(selector) = children[2].utf8_text(self.source.as_bytes()) {
                                if let Ok(etype) = children[3].utf8_text(self.source.as_bytes()) {
                                    body_statements.push(Statement::XP(EXPCType::Set {
                                        amount: amount.parse::<u32>().unwrap(),
                                        selector: selector.to_string(),
                                        etype: EXPType::from_str(etype).unwrap(),
                                    }));
                                }
                            }
                        }
                    }
                }

                "xp_query_command" => {
                    let mut cmd_cursor = command.walk();
                    let children: Vec<Node> = command.children(&mut cmd_cursor).collect();

                    if children.len() >= 2 {
                        if let Ok(selector) = children[1].utf8_text(self.source.as_bytes()) {
                            if let Ok(etype) = children[2].utf8_text(self.source.as_bytes()) {
                                body_statements.push(Statement::XP(EXPCType::Query {
                                    selector: selector.to_string(),
                                    etype: EXPType::from_str(etype).unwrap(),
                                }));
                            }
                        }
                    }
                }

                "gm_creative_command" => {
                    let mut cmd_cursor = command.walk();
                    let children: Vec<Node> = command.children(&mut cmd_cursor).collect();

                    if children.len() >= 2 {
                        if let Ok(selector) = children[1].utf8_text(self.source.as_bytes()) {
                            body_statements.push(Statement::Gamemode(
                                GamemodeType::Creative,
                                selector.to_string(),
                            ));
                        }
                    }
                }
                "gm_spectator_command" => {
                    let mut cmd_cursor = command.walk();
                    let children: Vec<Node> = command.children(&mut cmd_cursor).collect();

                    if children.len() >= 2 {
                        if let Ok(selector) = children[1].utf8_text(self.source.as_bytes()) {
                            body_statements.push(Statement::Gamemode(
                                GamemodeType::Spectator,
                                selector.to_string(),
                            ));
                        }
                    }
                }
                "gm_survival_command" => {
                    let mut cmd_cursor = command.walk();
                    let children: Vec<Node> = command.children(&mut cmd_cursor).collect();

                    if children.len() >= 2 {
                        if let Ok(selector) = children[1].utf8_text(self.source.as_bytes()) {
                            body_statements.push(Statement::Gamemode(
                                GamemodeType::Survival,
                                selector.to_string(),
                            ));
                        }
                    }
                }
                "gm_adventure_command" => {
                    let mut cmd_cursor = command.walk();
                    let children: Vec<Node> = command.children(&mut cmd_cursor).collect();

                    if children.len() >= 2 {
                        if let Ok(selector) = children[1].utf8_text(self.source.as_bytes()) {
                            body_statements.push(Statement::Gamemode(
                                GamemodeType::Adventure,
                                selector.to_string(),
                            ));
                        }
                    }
                }
                "inv_clear_command" => {
                    let mut cmd_cursor = command.walk();
                    let children: Vec<Node> = command.children(&mut cmd_cursor).collect();

                    if children.len() >= 2 {
                        if let Ok(selector) = children[1].utf8_text(self.source.as_bytes()) {
                            body_statements
                                .push(Statement::Clear(ClearType::Inventory, selector.to_string()));
                        }
                    }
                }
                "effect_clear_command" => {
                    let mut cmd_cursor = command.walk();
                    let children: Vec<Node> = command.children(&mut cmd_cursor).collect();

                    if children.len() >= 2 {
                        if let Ok(selector) = children[1].utf8_text(self.source.as_bytes()) {
                            body_statements
                                .push(Statement::Clear(ClearType::Effect, selector.to_string()));
                        }
                    }
                }
                "tellraw_command" => {
                    let mut cmd_cursor = command.walk();
                    for child in command.children(&mut cmd_cursor) {
                        if child.kind() == "text" {
                            if let Ok(text) = child.utf8_text(self.source.as_bytes()) {
                                let text = text.trim();
                                debug!("Found tellraw command text:", "{}", text);
                                body_statements.push(Statement::Tellraw(text.to_string()));
                            }
                        }
                    }
                }
                "effect_command" => {
                    let mut cmd_cursor = command.walk();
                    let children: Vec<Node> = command.children(&mut cmd_cursor).collect();

                    debug!("Effect Command Children:", "{:?}", children);
                    if children.len() >= 5 {
                        // Get the target selector
                        if let Ok(selector) = children[1].utf8_text(self.source.as_bytes()) {
                            // Get the effect name
                            if let Ok(effect) = children[2].utf8_text(self.source.as_bytes()) {
                                // Get duration and amplifier
                                if let (Ok(duration), Ok(amplifier)) = (
                                    children[3].utf8_text(self.source.as_bytes()),
                                    children[4].utf8_text(self.source.as_bytes()),
                                ) {
                                    body_statements.push(Statement::Effect(
                                        selector.to_string(),
                                        effect.to_string(),
                                        duration.to_string(),
                                        amplifier.to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }
                "time_command" => {
                    let mut cursor = command.walk();
                    let children: Vec<Node> = command.children(&mut cursor).collect();

                    debug!("Time Command Children:", "{:?}", children);
                    if children.len() >= 2 {
                        match children[1].kind() {
                            "set" => {
                                if let Some(value_node) = children.get(2) {
                                    if let Ok(value_text) =
                                        value_node.utf8_text(self.source.as_bytes())
                                    {
                                        debug!("Found time set command:", "{}", value_text);
                                        body_statements
                                            .push(Statement::TimeSet(value_text.to_string()));
                                    }
                                }
                            }
                            "query" => {
                                if let Some(query_node) = children.get(2) {
                                    if let Ok(query_text) =
                                        query_node.utf8_text(self.source.as_bytes())
                                    {
                                        debug!("Found time query command:", "{}", query_text);
                                        body_statements
                                            .push(Statement::TimeQuery(query_text.to_string()));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "execute_command" => {
                    handles::execute_h::handle_execute_command(command, self.source, &mut body_statements);
                }

                other => {
                    debug!("Unhandled command type:", "{}", other);
                }
            }
        }

        debug!("Final statements", "for function {}:", name);
        for (i, stmt) in body_statements.iter().enumerate() {
            debug!(format!("Statement {}", i), ": {:?}", stmt);
        }

        Some(NodeType::Function {
            name,
            body: body_statements,
        })
    }
}

