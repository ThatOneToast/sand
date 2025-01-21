use crate::{grammar::var::Type, lang::Rule};

use super::{var::Variable, Statement};


#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl ComparisonOperator {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        let operator = pair.as_str();
        match operator {
            ">" => ComparisonOperator::GreaterThan,
            "<" => ComparisonOperator::LessThan,
            "==" => ComparisonOperator::Equals,
            "!=" => ComparisonOperator::NotEquals,
            ">=" => ComparisonOperator::GreaterThanOrEqual,
            "<=" => ComparisonOperator::LessThanOrEqual,
            _ => unreachable!("Unknown comparison operator: {}", operator),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Condition {
    Comparison {
        left: Box<ConditionValue>,
        operator: ComparisonOperator,
        right: Box<ConditionValue>,
    },
    Boolean(bool),
    Identifier(String),
}

impl Condition {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        println!(
            "Parsing condition: {} (Rule: {:?})",
            pair.as_str(),
            pair.as_rule()
        );

        if pair.as_rule() == Rule::comparison {
            println!("Direct comparison");
            let mut comp_inner = pair.into_inner();
            let left = comp_inner.next().unwrap();
            let operator = comp_inner.next().unwrap();
            let right = comp_inner.next().unwrap();

            println!("Comparison parts:");
            println!("  Left: {} (Rule: {:?})", left.as_str(), left.as_rule());
            println!(
                "  Operator: {} (Rule: {:?})",
                operator.as_str(),
                operator.as_rule()
            );
            println!("  Right: {} (Rule: {:?})", right.as_str(), right.as_rule());

            return Condition::Comparison {
                left: Box::new(ConditionValue::from_pest(left)),
                operator: ComparisonOperator::from_pest(operator),
                right: Box::new(ConditionValue::from_pest(right)),
            };
        }

        // For other cases (boolean and identifier)
        match pair.as_rule() {
            Rule::boolean_type => {
                println!("Parsing boolean");
                Condition::Boolean(pair.as_str().parse().unwrap())
            }
            Rule::identifier => {
                println!("Parsing identifier");
                Condition::Identifier(pair.as_str().to_string())
            }
            _ => {
                println!("Unexpected rule: {:?}", pair.as_rule());
                unreachable!("Unexpected condition rule: {:?}", pair.as_rule())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConditionValue {
    Literal(Type),
    Identifier(String),
}

impl ConditionValue {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        println!(
            "Parsing condition value: {} (Rule: {:?})",
            pair.as_str(),
            pair.as_rule()
        );

        match pair.as_rule() {
            Rule::identifier => {
                println!("Direct identifier");
                ConditionValue::Identifier(pair.as_str().to_string())
            }
            Rule::number_type => {
                println!("Direct number");
                ConditionValue::Literal(Type::Number(pair.as_str().parse().unwrap()))
            }
            _ => {
                println!("Trying inner value");
                // Clone the pair before getting inner to avoid the move
                let inner_pairs: Vec<_> = pair.clone().into_inner().collect();
                if let Some(inner) = inner_pairs.first() {
                    println!("Inner: {} (Rule: {:?})", inner.as_str(), inner.as_rule());
                    match inner.as_rule() {
                        Rule::number_type => {
                            ConditionValue::Literal(Type::Number(inner.as_str().parse().unwrap()))
                        }
                        Rule::identifier => ConditionValue::Identifier(inner.as_str().to_string()),
                        _ => unreachable!("Unexpected condition value rule: {:?}", inner.as_rule()),
                    }
                } else {
                    // If there's no inner value, use the original pair
                    match pair.as_rule() {
                        Rule::number_type => {
                            ConditionValue::Literal(Type::Number(pair.as_str().parse().unwrap()))
                        }
                        Rule::identifier => ConditionValue::Identifier(pair.as_str().to_string()),
                        _ => unreachable!("Unexpected condition value rule: {:?}", pair.as_rule()),
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Condition,
    pub block: Vec<Statement>,
}

impl IfStatement {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let condition = Condition::from_pest(inner.next().unwrap());

        let block_pair = inner.next().unwrap();
        let mut block = Vec::new();

        for statement in block_pair.into_inner() {
            match statement.as_rule() {
                Rule::variable => {
                    block.push(Statement::Variable(Variable::from_pest(statement)));
                }
                Rule::function_call => {
                    block.push(Statement::FunctionCall(
                        super::FunctionCall::from_pest(statement),
                    ));
                }
                _ => {}
            }
        }

        IfStatement { condition, block }
    }
}
