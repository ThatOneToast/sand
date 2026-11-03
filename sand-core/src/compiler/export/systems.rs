//! Stable dependency ordering for tick systems. Dependencies are barriers over
//! complete system scans, rather than per-entity callback ordering.
use std::collections::{BTreeMap, BTreeSet};

use crate::function::StateSystemDescriptor;

pub(super) fn order_systems(
    systems: Vec<&StateSystemDescriptor>,
) -> Result<Vec<&StateSystemDescriptor>, String> {
    let mut declarations = BTreeMap::<_, Vec<_>>::new();
    let mut constraints = BTreeMap::new();
    for system in systems {
        let id = (system.id)();
        let (before, after) = (system.order)();
        let order = (
            before.into_iter().collect::<BTreeSet<_>>(),
            after.into_iter().collect::<BTreeSet<_>>(),
        );
        if let Some(previous) = constraints.insert(id, order.clone())
            && previous != order
        {
            return Err(format!(
                "State system `{id}` has conflicting ordering declarations"
            ));
        }
        declarations.entry(id).or_default().push(system);
    }
    let mut outgoing: BTreeMap<_, BTreeSet<_>> = declarations
        .keys()
        .map(|id| (*id, BTreeSet::new()))
        .collect();
    let mut incoming: BTreeMap<_, usize> = declarations.keys().map(|id| (*id, 0)).collect();
    for (&id, (before, after)) in &constraints {
        for (&other, is_before) in before
            .iter()
            .map(|other| (other, true))
            .chain(after.iter().map(|other| (other, false)))
        {
            if !declarations.contains_key(other) {
                return Err(format!(
                    "State system `{id}` references unregistered tick system `{other}`; dependencies must name registered tick systems"
                ));
            }
            let (from, to) = if is_before { (id, other) } else { (other, id) };
            if outgoing.get_mut(from).unwrap().insert(to) {
                *incoming.get_mut(to).unwrap() += 1;
            }
        }
    }
    let mut ready: BTreeSet<_> = incoming
        .iter()
        .filter_map(|(&id, &count)| (count == 0).then_some(id))
        .collect();
    let mut ordered = Vec::new();
    let mut visited = 0;
    while let Some(id) = ready.pop_first() {
        visited += 1;
        ordered.extend(declarations[&id].iter().copied());
        for next in &outgoing[&id] {
            let count = incoming.get_mut(next).unwrap();
            *count -= 1;
            if *count == 0 {
                ready.insert(*next);
            }
        }
    }
    if visited != declarations.len() {
        let blocked = incoming
            .iter()
            .filter_map(|(&id, &count)| (count > 0).then_some(id))
            .collect::<Vec<_>>();
        return Err(format!(
            "State system dependency cycle; blocked systems: {}",
            blocked.join(", ")
        ));
    }
    Ok(ordered)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn system(
        id: fn() -> &'static str,
        order: fn() -> (Vec<&'static str>, Vec<&'static str>),
    ) -> StateSystemDescriptor {
        StateSystemDescriptor {
            id,
            order,
            every: 1,
            make: crate::ir::Actions::default,
        }
    }
    #[test]
    fn dependencies_override_names_and_input_order() {
        let damage = system(|| "z_damage", || (vec!["a_death"], vec![]));
        let death = system(|| "a_death", || (vec![], vec![]));
        for input in [vec![&death, &damage], vec![&damage, &death]] {
            assert_eq!(
                order_systems(input)
                    .unwrap()
                    .iter()
                    .map(|system| (system.id)())
                    .collect::<Vec<_>>(),
                ["z_damage", "a_death"]
            );
        }
    }
    #[test]
    fn missing_references_and_cycles_fail_before_lowering() {
        let missing = system(|| "a", || (vec!["absent"], vec![]));
        assert!(
            order_systems(vec![&missing])
                .err()
                .unwrap()
                .contains("unregistered tick system `absent`")
        );
        let first = system(|| "a", || (vec!["b"], vec![]));
        let second = system(|| "b", || (vec!["a"], vec![]));
        let error = order_systems(vec![&second, &first]).err().unwrap();
        assert!(error.contains("cycle"));
        assert!(error.contains("a, b"));
    }
}
