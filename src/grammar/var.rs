
use crate::lang::Rule;

use super::math::MathExpression;

#[derive(Debug, Clone)]
pub enum Scope {
    Global,
    Function(String),
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub identifier: String,
    pub value: Type,
    pub scope: Option<Scope>,
}

impl Variable {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        println!("Parsing variable: {}", pair.as_str());
        let mut inner = pair.into_inner();
        let identifier = inner
            .next()
            .expect("Failed retrieving variable identifier")
            .as_str()
            .to_string();
        let value = Type::from_pest(inner.next().expect("Failed retrieving variable value"));

        println!("Created variable: {} = {:?}", identifier, value);
        Variable {
            identifier,
            value,
            scope: None,
        }
    }

    pub fn from_pest_scoped(pair: pest::iterators::Pair<Rule>, scope: Scope) -> Self {
        let mut var = Variable::from_pest(pair);
        println!("Applying Scope to variable {}", var.identifier);
        var.scope = Some(scope);
        var
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    String(String),
    Number(f64),
    Boolean(bool),
    Math(MathExpression),
}

impl Type {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        // Add debug print to see what we're parsing
        println!(
            "Parsing type from: {:?} - {}",
            pair.as_rule(),
            pair.as_str()
        );

        // If it's already a value type, use it directly
        let pair = match pair.as_rule() {
            Rule::value => pair.into_inner().next().unwrap(),
            _ => pair,
        };

        match pair.as_rule() {
            Rule::string_type => {
                let value = pair
                    .as_str()
                    .trim_matches('\'')
                    .trim_matches('`')
                    .to_string();
                Type::String(value)
            }
            Rule::number_type => {
                Type::Number(pair.as_str().parse().expect("Failed parsing number"))
            }
            Rule::boolean_type => {
                Type::Boolean(pair.as_str().parse().expect("Failed parsing boolean"))
            }
            Rule::back_tick_mexpr => {
                let math_expr = pair.into_inner().next().expect("Expected math expression");
                println!("Processing math expression: {}", math_expr.as_str());
                Type::Math(MathExpression::from_pest(math_expr))
            }
            rule => {
                println!("Unexpected rule: {:?}", rule);
                panic!("Unexpected type rule: {:?}", rule)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeNotation {
    String,
    Number,
    Boolean,
}

impl TypeNotation {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        let type_str = pair.as_str();
        match type_str {
            "String" => TypeNotation::String,
            "Number" => TypeNotation::Number,
            "Boolean" => TypeNotation::Boolean,
            _ => unreachable!("Unknown type notation: {}", type_str),
        }
    }
}
