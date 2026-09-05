#![deny(deprecated)]

use sand::prelude::*;

#[deprecated]
fn legacy_commands() -> Vec<String> {
    Vec::new()
}

macro_rules! ident_only {
    ($value:ident) => {
        stringify!($value)
    };
}

#[derive(State)]
#[state(namespace = "direct", scope = entity)]
struct EntityHealth {
    #[state(default = 20)]
    health: Score,
}

#[derive(State)]
#[state(namespace = "direct", scope = living)]
struct LivingHealth {
    #[state(default = 20)]
    health: Score,
}

#[derive(State)]
#[state(namespace = "direct", scope = player)]
struct PlayerHealth {
    #[state(default = 20)]
    health: Score,
}

#[derive(State)]
#[state(namespace = "direct", scope = entity)]
struct Dead;

impl Dead {
    fn each(&self) {}
}

#[derive(StateBundle)]
struct EntityCombat {
    health: EntityHealth,
    dead: Dead,
}

#[derive(StateBundle)]
struct NestedEntityCombat {
    combat: EntityCombat,
}

#[system(tick, every = 10)]
#[allow(deprecated)]
fn free_tick(query: EntityHealth) {
    legacy_commands();
    query.each(|health| health.health.add(1));
    query.each(|health| health.health.add(2));
}

#[system(tick, every = 10)]
fn bundle_tick(query: NestedEntityCombat) {
    query.each(|nested| nested.combat.health.health.add(1));
}

#[system]
fn inherent_name_collision(query: Dead) {
    query.each(|_dead| vec![cmd::say("canonical query operation").to_string()]);
}

#[system]
fn statement_cfg_is_preserved(query: Dead) {
    #[cfg(any())]
    query.each(|_dead| {
        compile_error!("cfg-disabled query call was emitted");
        Vec::new()
    });
}

#[system]
#[cfg_attr(debug_assertions, inline)]
fn nongating_cfg_attr_stays_on_endpoint(query: Dead) {
    query.each(|_dead| Vec::new());
}

#[system]
#[cfg_attr(debug_assertions, cfg(any()), inline)]
fn mixed_cfg_attr_keeps_only_gate(query: MissingCfgAttrQuery) {
    query.each(|_| Vec::new());
}

#[system]
#[cfg(any())]
fn cfg_disabled_free_system(query: MissingFreeQuery) {
    query.each(|_| Vec::new());
}

struct DirectSystems;

mod events {
    pub struct QualifiedPulse;
}

use events::QualifiedPulse;

trait SystemTypes {
    type Query;
    type Event;
}

impl SystemTypes for DirectSystems {
    type Query = LivingHealth;
    type Event = DirectPulse;
}

#[system]
#[allow(deprecated)]
impl DirectSystems {
    const MESSAGE: &'static str = "grouped Self context";

    fn commands() -> Vec<String> {
        vec![cmd::say(Self::MESSAGE).to_string()]
    }

    #[tick(every = 10)]
    fn grouped_tick(query: LivingHealth) {
        struct Helper;
        impl Helper {
            const VALUE: i32 = 1;

            fn value() -> i32 {
                Self::VALUE
            }
        }
        let _ = Helper::value();
        let _ = format!("{}", Self::MESSAGE);
        let _ = ident_only!(Self);
        legacy_commands();
        query.each(|health| {
            let mut commands = health.health.add(1);
            commands.extend(Self::commands());
            commands
        });
        query.each(|health| health.health.add(2));
    }

    #[tick]
    fn grouped_tick_with_self_query(query: <Self as SystemTypes>::Query) {
        query.each(|health| health.health.add(1));
    }

    #[event(DirectPulse)]
    fn current(pulse: DirectPulse, query: PlayerHealth) {
        let _ = pulse;
        let _ = vec![Self::MESSAGE];
        let _ = ident_only!(Self);
        legacy_commands();
        query.current(|health| {
            let mut commands = health.health.add(1);
            commands.extend(Self::commands());
            commands
        });
        query.each(|health| health.health.add(2));
    }

    #[event(DirectPulse)]
    fn event_with_self_query(
        _pulse: DirectPulse,
        query: <Self as SystemTypes>::Query,
    ) {
        query.current(|health| health.health.add(1));
    }

    #[event(DirectPulse)]
    fn event_without_query(_pulse: DirectPulse) {
        Self::commands();
    }

    #[event(events::QualifiedPulse)]
    fn qualified_event_type(_pulse: QualifiedPulse) {}

    #[event(DirectPulse)]
    fn self_event_type(_pulse: <Self as SystemTypes>::Event) {}

    #[tick]
    #[cfg_attr(debug_assertions, inline)]
    fn nongating_cfg_attr_method(query: Dead) {
        query.each(|_dead| Vec::new());
    }
}

struct OtherDirectSystems;

#[system]
impl OtherDirectSystems {
    #[tick]
    fn grouped_tick(query: LivingHealth) {
        query.each(|health| health.health.add(1));
    }

    #[event(DirectPulse)]
    fn current(_pulse: DirectPulse, query: PlayerHealth) {
        query.current(|health| health.health.add(1));
    }
}

#[system]
impl DirectSystems {
    #[tick]
    #[cfg(any())]
    fn cfg_disabled_tick_method(query: MissingTickQuery) {
        query.each(|_| Vec::new());
    }

    #[event(MissingEvent)]
    #[cfg(any())]
    fn cfg_disabled_event_method(_event: MissingEvent, query: MissingEventQuery) {
        query.each(|_| Vec::new());
    }
}

#[system]
#[cfg(any())]
impl MissingImplSystems {
    #[tick]
    fn cfg_disabled_impl(query: MissingImplQuery) {
        query.each(|_| Vec::new());
    }
}

struct DirectPulse;

impl SandEvent for DirectPulse {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

impl SandEvent for QualifiedPulse {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
    }
}

fn main() {
    let _: fn(EntityHealth) = free_tick;
    let _: fn(NestedEntityCombat) = bundle_tick;
    let _: fn(Dead) = inherent_name_collision;
    let _: fn(Dead) = statement_cfg_is_preserved;
    let _: fn(Dead) = nongating_cfg_attr_stays_on_endpoint;
    let _: fn(LivingHealth) = DirectSystems::grouped_tick;
    let _: fn(LivingHealth) = DirectSystems::grouped_tick_with_self_query;
    let _: fn(DirectPulse, PlayerHealth) = DirectSystems::current;
    let _: fn(DirectPulse, LivingHealth) = DirectSystems::event_with_self_query;
    let _: fn(DirectPulse) = DirectSystems::event_without_query;
    let _: fn(QualifiedPulse) = DirectSystems::qualified_event_type;
    let _: fn(DirectPulse) = DirectSystems::self_event_type;
    let _: fn(LivingHealth) = OtherDirectSystems::grouped_tick;
    let _: fn(DirectPulse, PlayerHealth) = OtherDirectSystems::current;
    let _: Vec<String> = <EntityCombat as sand::__private::StateQuerySpec>::each(|combat| {
        combat.health.health.add(1)
    });
    let _: Vec<String> =
        <NestedEntityCombat as sand::__private::StateQuerySpec>::current(|nested| {
            nested.combat.health.health.add(1)
    });
}

#[allow(dead_code)]
fn source_level_methods(
    entity: EntityHealth,
    living: LivingHealth,
    player: PlayerHealth,
    marker: Dead,
) {
    let _: Vec<String> = entity.each(|health| health.health.add(1));
    let _: Vec<String> = living.current(|health| health.health.add(1));
    let _: Vec<String> = player.each(|health| health.health.add(1));
    let _: Vec<String> = StateQueryOperations::each(&marker, |_dead| vec!["say dead".into()]);
}

#[allow(dead_code)]
fn source_level_bundle_methods(bundle: NestedEntityCombat) {
    let _: Vec<String> = bundle.each(|nested| nested.combat.health.health.add(1));
}
