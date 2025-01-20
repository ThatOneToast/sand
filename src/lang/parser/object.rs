use pest::iterators::Pair;

use crate::lang::{
    structure::{Object, Property, SuperConstructor},
    Rule,
};

use super::{parse_field_list, parse_property};

pub fn parse_object(pair: Pair<Rule>) -> Object {
    let mut constructor = None;
    let mut name = String::new();
    let mut fields = Vec::new();
    let mut properties = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::super_constructor => {
                constructor = Some(match inner_pair.as_str() {
                    "ItemStack" => SuperConstructor::ItemStack,
                    "Entities" => SuperConstructor::Entities,
                    "LootTables" => SuperConstructor::LootTables,
                    _ => unreachable!(),
                });
            }
            Rule::identifier => {
                name = inner_pair.as_str().to_string();
            }
            Rule::field_list => {
                fields = parse_field_list(inner_pair);
            }
            Rule::object_block => {
                properties = parse_object_block(inner_pair);
            }
            _ => {}
        }
    }

    Object {
        constructor: constructor.unwrap(),
        name,
        fields,
        properties,
    }
}

fn parse_object_block(pair: Pair<Rule>) -> Vec<Property> {
    let mut properties = Vec::new();

    for inner_pair in pair.into_inner() {
        if let Rule::property_assignment = inner_pair.as_rule() {
            properties.push(parse_property(inner_pair));
        }
    }

    properties
}
