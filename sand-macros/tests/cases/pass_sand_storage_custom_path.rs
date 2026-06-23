use sand_macros::SandStorage;

#[derive(SandStorage)]
#[sand(storage = "mana:data", root = "school")]
pub struct MagicSchool {
    pub name: String,
    #[sand(path = "tier_level")]
    pub tier: i32,
}

fn main() {
    assert_eq!(MagicSchool::name().field_name(), "name");
    // Custom path override
    assert_eq!(MagicSchool::tier().field_name(), "tier_level");
}
