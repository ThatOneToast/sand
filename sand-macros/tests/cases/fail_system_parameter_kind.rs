use sand::prelude::*;

#[system]
fn invalid_query(query: EntityContext<AnyEntity>) {
    query.each(|_| Vec::new());
}

fn main() {}
