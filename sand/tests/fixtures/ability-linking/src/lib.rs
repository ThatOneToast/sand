use sand::prelude::*;
#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "linking", scope = player)]
pub struct Reservoir {
    #[state(default = 32, min = 0, max = 32)]
    pub value: Score,
    #[state(auto_tick)]
    pub cooldown: Cooldown,
    #[state(auto_tick)]
    pub timer: Timer,
}

pub fn export() {
    println!(
        "{}",
        sand::advanced::try_export_pack_json("linking", "26.2").unwrap()
    );
}
