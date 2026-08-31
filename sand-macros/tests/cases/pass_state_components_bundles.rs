use sand::prelude::*;

#[derive(State)]
#[state(namespace = "rpg", scope = living, version = 2, migrate(from = 1, to = 2))]
struct Attack {
    #[state(default = 1, min = 0, max = 100, display_name = "Damage")]
    damage: Score,
    #[state(auto_tick)]
    cooldown: Cooldown,
}

#[derive(State)]
#[state(namespace = "rpg", scope = living)]
struct Defense {
    #[state(default = false)]
    blocking: Flag,
}

#[derive(State)]
#[state(namespace = "rpg", scope = living)]
struct Boss;

#[derive(State)]
#[state(namespace = "rpg", scope = living)]
struct Dead;

#[derive(StateBundle)]
struct Combat {
    attack: Attack,
    defense: Defense,
}

#[derive(StateBundle)]
struct BossCombat {
    combat: Combat,
    boss: Boss,
}

#[derive(StateQuery)]
#[query(scope = living)]
struct Combatants {
    #[require]
    combat: Combat,
    #[optional]
    boss: Boss,
    #[without]
    dead: Dead,
}

struct CombatSystems;

#[system]
impl CombatSystems {
    #[tick(every = 5)]
    fn recharge(query: Combatants) {
        query.each(|combatant| combatant.combat.attack.damage.add(1));
    }
}

fn main() {
    let entity = EntityContext::<ZombieKind>::default();
    let bundle = BossCombat::on(entity);
    let _: Vec<String> = bundle.combat.attack.damage.add(1);
    let _: Vec<String> = BossCombat::attach(entity);
    let _: Vec<String> = BossCombat::detach(entity);
    let _: Vec<String> = Boss::attach(entity);
    let _ = Boss::is_attached(entity);
    let _: Vec<String> = Combatants::each(|combatant| {
        let mut commands = combatant.combat.attack.damage.add(1);
        commands.extend(combatant.boss(|_| vec!["say boss".into()]));
        commands
    });
}
