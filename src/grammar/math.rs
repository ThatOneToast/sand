use std::{collections::HashMap, str::FromStr};

use pest::{iterators::Pair, Parser};

use crate::lang::{Rule, SandParser};

#[derive(Debug, Clone)]
pub enum MathExpression {
    Add(Box<MathExpression>, Box<MathExpression>),
    Sub(Box<MathExpression>, Box<MathExpression>),
    Mul(Box<MathExpression>, Box<MathExpression>),
    Div(Box<MathExpression>, Box<MathExpression>),
    Pow(Box<MathExpression>, Box<MathExpression>),
    Number(f64),
    Variable(String),
}

impl FromStr for MathExpression {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        println!("Attempting to parse math expression: '{}'", s);
        match SandParser::parse(Rule::math_expression, s.trim()) {
            Ok(pairs) => {
                println!("Successfully parsed math expression");
                let expr = Self::from_pest(pairs.peek().unwrap());
                println!("Created expression: {:?}", expr);
                Ok(expr)
            }
            Err(e) => {
                println!("Failed to parse math expression: {}", e);
                Err(e.to_string())
            }
        }
    }
}
impl MathExpression {
    pub fn from_pest(pair: Pair<Rule>) -> Self {
        println!("Parsing math expression from pest: {:?}", pair);
        Self::parse_expression(pair.into_inner().collect::<Vec<_>>())
    }

    fn parse_expression(tokens: Vec<Pair<Rule>>) -> Self {
        // First pass: Handle multiplication and division
        let mut terms = Vec::new();
        let mut operators: Vec<Pair<Rule>> = Vec::new();

        let mut i = 0;
        while i < tokens.len() {
            match tokens[i].as_rule() {
                Rule::term => {
                    let term = Self::parse_term(tokens[i].clone());
                    if !operators.is_empty()
                        && (operators.last().unwrap().as_str() == "*"
                            || operators.last().unwrap().as_str() == "/")
                    {
                        let op = operators.pop().unwrap();
                        let left = terms.pop().unwrap();
                        let combined = match op.as_str() {
                            "*" => MathExpression::Mul(Box::new(left), Box::new(term)),
                            "/" => MathExpression::Div(Box::new(left), Box::new(term)),
                            _ => unreachable!(),
                        };
                        terms.push(combined);
                    } else {
                        terms.push(term);
                    }
                }
                Rule::math_operator => {
                    operators.push(tokens[i].clone());
                }
                _ => panic!("Unexpected token: {:?}", tokens[i].as_rule()),
            }
            i += 1;
        }

        // Second pass: Handle addition and subtraction
        let mut result = terms[0].clone();
        let mut term_index = 1;
        for op in operators.iter() {
            if op.as_str() == "+" || op.as_str() == "-" {
                result = match op.as_str() {
                    "+" => {
                        MathExpression::Add(Box::new(result), Box::new(terms[term_index].clone()))
                    }
                    "-" => {
                        MathExpression::Sub(Box::new(result), Box::new(terms[term_index].clone()))
                    }
                    _ => unreachable!(),
                };
                term_index += 1;
            }
        }

        result
    }

    fn parse_term(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::term => {
                let inner = pair.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::number_type => MathExpression::Number(inner.as_str().parse().unwrap()),
                    Rule::identifier => MathExpression::Variable(inner.as_str().to_string()),
                    _ => panic!("Unexpected inner term rule: {:?}", inner.as_rule()),
                }
            }
            _ => panic!("Unexpected term rule: {:?}", pair.as_rule()),
        }
    }

    pub fn evaluate(&self, variables: &HashMap<String, f64>) -> Result<f64, String> {
        println!("Evaluating expression: {:?}", self);
        let result = match self {
            MathExpression::Number(n) => Ok(*n),
            MathExpression::Variable(name) => variables
                .get(name)
                .copied()
                .ok_or_else(|| format!("Variable '{}' not found", name)),
            MathExpression::Add(left, right) => {
                let l = left.evaluate(variables)?;
                let r = right.evaluate(variables)?;
                println!("Adding {} + {}", l, r);
                Ok(l + r)
            }
            MathExpression::Sub(left, right) => {
                let l = left.evaluate(variables)?;
                let r = right.evaluate(variables)?;
                println!("Subtracting {} - {}", l, r);
                Ok(l - r)
            }
            MathExpression::Mul(left, right) => {
                let l = left.evaluate(variables)?;
                let r = right.evaluate(variables)?;
                println!("Multiplying {} * {}", l, r);
                Ok(l * r)
            }
            MathExpression::Div(left, right) => {
                let l = left.evaluate(variables)?;
                let r = right.evaluate(variables)?;
                println!("Dividing {} / {}", l, r);
                if r == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(l / r)
                }
            }
            MathExpression::Pow(left, right) => {
                let l = left.evaluate(variables)?;
                let r = right.evaluate(variables)?;
                println!("Power {} ^ {}", l, r);
                Ok(l.powf(r))
            }
        };
        println!("Evaluation result: {:?}", result);
        result
    }
}
