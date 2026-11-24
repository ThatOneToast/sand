//! Typed value sets: `Item::Diamond`, `Effect::Speed`, `Rarity::Epic`, ...
//!
//! Vanilla sets come from the shared offline, version-specific Mojang report
//! catalogs in `sand-version`. Their variant naming uses Rust's registry
//! generator. Checking and completion use the pack's target version. Sand's
//! own option sets, such as `Rarity`, mirror the portable compiler's enums.
//!
//! Every set follows one naming rule: a PascalCase singular noun, with
//! PascalCase values.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use sand::entity::Relation;

/// One typed registry, such as `Item`.
#[derive(Debug, Clone)]
pub struct Registry {
    /// Language and Rust name, such as `Item`.
    pub name: &'static str,
    /// Rust path, such as `sand::vanilla::Item`.
    pub rust_path: &'static str,
    /// What the registry names, for messages: "item".
    pub noun: &'static str,
    /// Whether the values are Minecraft registry entries (rather than one of
    /// Sand's own option sets).
    pub vanilla: bool,
    /// Variant name to namespaced identity, in name order.
    pub variants: BTreeMap<String, String>,
    /// Identity to variant name.
    pub by_id: BTreeMap<String, String>,
    /// Variant names in presentation order: alphabetical for vanilla sets,
    /// declaration order for Sand's option sets.
    pub order: Vec<String>,
}

impl Registry {
    /// The identity of a variant, such as `minecraft:diamond`.
    pub fn id(&self, variant: &str) -> Option<&str> {
        self.variants.get(variant).map(String::as_str)
    }

    /// The closest variant names to a misspelling, best first.
    pub fn suggest(&self, wrong: &str) -> Vec<&str> {
        let wrong = wrong.to_ascii_lowercase();
        let mut scored: Vec<(usize, &str)> = self
            .variants
            .keys()
            .map(|name| (distance(&wrong, &name.to_ascii_lowercase()), name.as_str()))
            .filter(|(d, name)| *d <= 2.max(name.len() / 4))
            .collect();
        scored.sort();
        scored.into_iter().take(3).map(|(_, name)| name).collect()
    }
}

/// Where each vanilla set comes from: (language name, Rust path, noun).
const SOURCES: &[(&str, &str, &str)] = &[
    ("Item", "sand::vanilla::Item", "item"),
    ("Block", "sand::vanilla::Block", "block"),
    ("Biome", "sand::registry::BiomeId", "biome"),
    ("Entity", "sand::vanilla::EntityType", "entity type"),
    ("Sound", "sand::vanilla::SoundEvent", "sound"),
    ("Particle", "sand::vanilla::ParticleType", "particle"),
    ("Attribute", "sand::vanilla::Attribute", "attribute"),
    ("Effect", "sand::registry::EffectId", "status effect"),
    ("Potion", "sand::registry::PotionId", "potion"),
    ("Enchantment", "sand::vanilla::Enchantment", "enchantment"),
    ("DamageType", "sand::registry::DamageTypeId", "damage type"),
];

/// One of Sand's own option sets.
struct OptionSet {
    name: &'static str,
    rust_path: &'static str,
    noun: &'static str,
    /// (value name, serialized form), in presentation order.
    values: &'static [(&'static str, &'static str)],
}

/// Sand's own option sets.
const OPTIONS: &[OptionSet] = &[
    OptionSet {
        name: "Rarity",
        rust_path: "sand::advanced::compiler::Rarity",
        noun: "rarity",
        values: &[
            ("Common", "common"),
            ("Uncommon", "uncommon"),
            ("Rare", "rare"),
            ("Epic", "epic"),
        ],
    },
    OptionSet {
        name: "Sort",
        rust_path: "sand::advanced::compiler::EntitySort",
        noun: "sort order",
        values: &[
            ("Nearest", "nearest"),
            ("Furthest", "furthest"),
            ("Random", "random"),
            ("Arbitrary", "arbitrary"),
        ],
    },
    OptionSet {
        name: "GameMode",
        rust_path: "sand::advanced::compiler::PlayerGameMode",
        noun: "game mode",
        values: &[
            ("Survival", "survival"),
            ("Creative", "creative"),
            ("Adventure", "adventure"),
            ("Spectator", "spectator"),
        ],
    },
    OptionSet {
        name: "Relation",
        rust_path: "sand::entity::Relation",
        noun: "entity relationship",
        values: &[
            ("Owner", Relation::Owner.keyword()),
            ("Leasher", Relation::Leasher.keyword()),
            ("Target", Relation::Target.keyword()),
            ("Vehicle", Relation::Vehicle.keyword()),
            ("Controller", Relation::Controller.keyword()),
            ("Attacker", Relation::Attacker.keyword()),
            ("Origin", Relation::Origin.keyword()),
        ],
    },
    OptionSet {
        name: "Dimension",
        rust_path: "sand::registry::DimensionId",
        noun: "dimension",
        values: &[
            ("Overworld", "minecraft:overworld"),
            ("TheNether", "minecraft:the_nether"),
            ("TheEnd", "minecraft:the_end"),
        ],
    },
    OptionSet {
        name: "DimensionType",
        rust_path: "sand::registry::DimensionTypeId",
        noun: "dimension type",
        values: &[
            ("Overworld", "minecraft:overworld"),
            ("OverworldCaves", "minecraft:overworld_caves"),
            ("TheNether", "minecraft:the_nether"),
            ("TheEnd", "minecraft:the_end"),
        ],
    },
    OptionSet {
        name: "CloneMode",
        rust_path: "sand::command::CloneMode",
        noun: "clone mode",
        values: &[("Normal", "normal"), ("Force", "force"), ("Move", "move")],
    },
    OptionSet {
        name: "DamageScaling",
        rust_path: "sand::component::DamageScaling",
        noun: "damage scaling policy",
        values: &[
            ("Never", "never"),
            (
                "WhenCausedByLivingNonPlayer",
                "when_caused_by_living_non_player",
            ),
            ("Always", "always"),
        ],
    },
    OptionSet {
        name: "DamageEffects",
        rust_path: "sand::component::DamageEffects",
        noun: "damage effect",
        values: &[
            ("Hurt", "hurt"),
            ("Thorns", "thorns"),
            ("Drowning", "drowning"),
            ("Burning", "burning"),
            ("Poking", "poking"),
            ("Freezing", "freezing"),
        ],
    },
    OptionSet {
        name: "DeathMessageType",
        rust_path: "sand::component::DeathMessageType",
        noun: "death message format",
        values: &[
            ("Default", "default"),
            ("FallVariants", "fall_variants"),
            ("IntentionalGameDesign", "intentional_game_design"),
        ],
    },
    OptionSet {
        name: "EquipmentSlot",
        rust_path: "sand::component::EquipmentSlot",
        noun: "equipment slot",
        values: &[
            ("Head", "head"),
            ("Chest", "chest"),
            ("Legs", "legs"),
            ("Feet", "feet"),
            ("Mainhand", "mainhand"),
            ("Offhand", "offhand"),
        ],
    },
    OptionSet {
        name: "Weather",
        rust_path: "sand::advanced::compiler::Weather",
        noun: "weather",
        values: &[("Clear", "clear"), ("Rain", "rain"), ("Thunder", "thunder")],
    },
    OptionSet {
        name: "BossBarColor",
        rust_path: "sand::advanced::compiler::BossBarColor",
        noun: "boss bar colour",
        values: &[
            ("Blue", "blue"),
            ("Green", "green"),
            ("Pink", "pink"),
            ("Purple", "purple"),
            ("Red", "red"),
            ("White", "white"),
            ("Yellow", "yellow"),
        ],
    },
    OptionSet {
        name: "BossBarStyle",
        rust_path: "sand::advanced::compiler::BossBarStyle",
        noun: "boss bar style",
        values: &[
            ("Progress", "progress"),
            ("Notched6", "notched_6"),
            ("Notched10", "notched_10"),
            ("Notched12", "notched_12"),
            ("Notched20", "notched_20"),
        ],
    },
];

/// Every typed registry the language knows.
pub fn registries() -> &'static [Registry] {
    static REGISTRIES: OnceLock<Vec<Registry>> = OnceLock::new();
    REGISTRIES.get_or_init(load)
}

/// A registry by language name.
pub fn registry(name: &str) -> Option<&'static Registry> {
    registries().iter().find(|r| r.name == name)
}

/// The registry that owns an identity such as `minecraft:diamond`, when the
/// identity is one Sand knows for `name`.
pub fn lookup(name: &str, id: &str) -> Option<&'static str> {
    registry(name)?.by_id.get(id).map(String::as_str)
}

/// Registry identity shared by compiler validation and source values.
pub(crate) fn registry_key(name: &str) -> Option<&'static str> {
    Some(match name {
        "Item" => "minecraft:item",
        "Block" => "minecraft:block",
        "Biome" => "minecraft:worldgen/biome",
        "Entity" => "minecraft:entity_type",
        "Sound" => "minecraft:sound_event",
        "Particle" => "minecraft:particle_type",
        "Attribute" => "minecraft:attribute",
        "Effect" => "minecraft:mob_effect",
        "Potion" => "minecraft:potion",
        "Enchantment" => "minecraft:enchantment",
        "DamageType" => "minecraft:damage_type",
        _ => return None,
    })
}

/// Values available to a package's exact Minecraft target, including options.
pub fn for_target(target: &str, name: &str) -> Option<&'static Registry> {
    static TARGETS: OnceLock<BTreeMap<String, Vec<Registry>>> = OnceLock::new();
    let maps = TARGETS.get_or_init(|| {
        sand_version::registry::targets()
            .map(|target| {
                let values = registries()
                    .iter()
                    .map(|registry| {
                        let mut r = registry.clone();
                        if let Some(key) = registry_key(r.name) {
                            r.variants = sand_version::registry::variants(target, key)
                                .expect("generated registry")
                                .clone();
                            r.by_id = r
                                .variants
                                .iter()
                                .map(|(name, id)| (id.clone(), name.clone()))
                                .collect();
                            r.order = r.variants.keys().cloned().collect();
                        }
                        r
                    })
                    .collect();
                (target.to_string(), values)
            })
            .collect()
    });
    if registry_key(name).is_none() {
        return registry(name);
    }
    let target = if target == "latest" {
        sand_version::LATEST_KNOWN
    } else {
        target
    };
    maps.get(target)
        .or_else(|| {
            target
                .strip_suffix(".0")
                .and_then(|target| maps.get(target))
        })?
        .iter()
        .find(|registry| registry.name == name)
}

fn load() -> Vec<Registry> {
    let mut found: BTreeMap<&'static str, BTreeMap<String, String>> = BTreeMap::new();
    for (name, _, _) in SOURCES {
        for target in sand_version::registry::targets() {
            if let Some(values) =
                registry_key(name).and_then(|key| sand_version::registry::variants(target, key))
            {
                found.entry(name).or_default().extend(values.clone());
            }
        }
    }
    let mut registries: Vec<Registry> = SOURCES
        .iter()
        .filter_map(|(name, rust_path, noun)| {
            let variants = found.remove(name)?;
            let by_id = variants
                .iter()
                .map(|(variant, id)| (id.clone(), variant.clone()))
                .collect();
            let order = variants.keys().cloned().collect();
            Some(Registry {
                name,
                rust_path,
                noun,
                vanilla: true,
                variants,
                by_id,
                order,
            })
        })
        .collect();
    for OptionSet {
        name,
        rust_path,
        noun,
        values,
    } in OPTIONS
    {
        let variants: BTreeMap<String, String> = values
            .iter()
            .map(|(variant, value)| (variant.to_string(), value.to_string()))
            .collect();
        let by_id = variants
            .iter()
            .map(|(v, id)| (id.clone(), v.clone()))
            .collect();
        registries.push(Registry {
            name,
            rust_path,
            noun,
            vanilla: false,
            variants,
            by_id,
            order: values
                .iter()
                .map(|(variant, _)| variant.to_string())
                .collect(),
        });
    }
    registries
}

/// Levenshtein distance, for "did you mean" suggestions.
pub fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut previous = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let current = row[j + 1];
            row[j + 1] = if ca == *cb {
                previous
            } else {
                1 + previous.min(row[j]).min(row[j + 1])
            };
            previous = current;
        }
    }
    row[b.len()]
}
