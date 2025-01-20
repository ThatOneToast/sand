use pest::iterators::Pair;

use super::{
    structure::{Field, FieldType, Property, PropertyValue},
    Rule,
};

pub mod function;
pub mod object;

pub fn parse_field_list(pair: Pair<Rule>) -> Vec<Field> {
    let mut fields = Vec::new();

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::field {
            fields.push(parse_field(inner_pair));
        }
    }

    fields
}

fn parse_property(pair: Pair<Rule>) -> Property {
    let mut name = String::new();
    let mut value = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => {
                name = inner_pair.as_str().to_string();
            }
            Rule::value => {
                value = Some(parse_value(inner_pair));
            }
            _ => {}
        }
    }

    Property {
        name,
        value: value.unwrap(),
    }
}

fn parse_value(pair: Pair<Rule>) -> PropertyValue {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::string_literal => {
            let s = inner.as_str();
            PropertyValue::String(s[1..s.len() - 1].to_string()) // Remove quotes
        }
        Rule::number => PropertyValue::Number(inner.as_str().parse().unwrap()),
        Rule::boolean => PropertyValue::Boolean(inner.as_str() == "true"),
        _ => unreachable!(),
    }
}

pub fn parse_field(pair: Pair<Rule>) -> Field {
    let mut name = String::new();
    let mut field_type = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => {
                name = inner_pair.as_str().to_string();
            }
            Rule::r#type => {
                field_type = Some(match inner_pair.as_str() {
                    "String" => FieldType::String,
                    "Int" => FieldType::Int,
                    "Float" => FieldType::Float,
                    "Boolean" => FieldType::Boolean,
                    _ => unreachable!(),
                });
            }
            _ => {}
        }
    }

    Field {
        name,
        field_type: field_type.unwrap(),
    }
}
