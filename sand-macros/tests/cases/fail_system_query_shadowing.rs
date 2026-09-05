use sand::prelude::*;

#[derive(State)]
#[state(namespace = "diagnostics", scope = player)]
struct Mana;

#[system]
fn shadowed(query: Mana) {
    query.each(|_mana| {
        let query = Target::players();
        vec![cmd::kill(query).to_string()]
    });
}

#[system]
fn shadowed_by_const(query: Mana) {
    const query: Mana = Mana;
    query.each(|_mana| Vec::new());
}

#[system]
fn shadowed_by_static(query: Mana) {
    static query: Mana = Mana;
    query.each(|_mana| Vec::new());
}

#[system]
fn shadowed_by_function(query: Mana) {
    fn query() {}
    query.each(|_mana| Vec::new());
}

#[system]
fn shadowed_by_unit_struct(query: Mana) {
    struct query;
    query.each(|_mana| Vec::new());
}

#[system]
fn shadowed_by_import(query: Mana) {
    use imported::mana as query;
    query.each(|_mana| Vec::new());
}

mod imported {
    pub const mana: crate::Mana = crate::Mana;
}

fn main() {}
