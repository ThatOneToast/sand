use sand::prelude::*;

#[derive(State)]
#[state(namespace = "diagnostics", scope = entity)]
struct Health;

#[system]
fn invalid_return(query: Health) -> Vec<String> {
    query.each(|_| Vec::new())
}

fn main() {}
