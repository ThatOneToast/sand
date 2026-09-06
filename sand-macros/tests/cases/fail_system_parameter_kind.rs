use sand::prelude::*;

#[system]
fn invalid_query(_query: EntityContext<AnyEntity>) {
    cmd::say("the parameter must still be validated");
}

fn main() {}
