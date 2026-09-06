use sand::prelude::*;

#[derive(State)]
#[state(namespace = "system_generics", scope = entity)]
struct Health;

#[system]
fn generic_free<Q>(query: Q) {
    query.each(|_item| Vec::new());
}

struct GenericSystems<T>(std::marker::PhantomData<T>);

#[system]
impl<T> GenericSystems<T> {
    #[tick]
    fn update(query: Health) {
        query.each(|_health| Vec::new());
    }
}

fn main() {}
