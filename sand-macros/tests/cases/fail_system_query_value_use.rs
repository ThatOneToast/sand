use sand::prelude::*;

#[derive(State)]
#[state(namespace = "query_value", scope = entity)]
struct Mana;

#[system]
fn unsupported_value_use(query: Mana) {
    let _marker = &query;
    query.each(|_mana| Vec::new());
}

fn main() {}
