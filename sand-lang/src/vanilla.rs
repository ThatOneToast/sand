//! Typed value sets: `Item::Diamond`, `Effect::Speed`, `Rarity::Epic`, ...
//!
//! Vanilla sets are read from the same installed API catalogs that define
//! Rust's `sand::vanilla` and `sand::registry` enums, so the language offers
//! exactly the values (and the Minecraft version) that Rust does. Sand's own
//! option sets, such as `Rarity`, mirror the portable compiler's enums.
//!
//! Every set follows one naming rule: a PascalCase singular noun, with
//! PascalCase values.

use std::collections::BTreeMap;
use std::sync::OnceLock;

/// One typed registry, such as `Item`.
#[derive(Debug)]
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
    ("Entity", "sand::vanilla::EntityType", "entity type"),
    ("Sound", "sand::vanilla::SoundEvent", "sound"),
    ("Particle", "sand::vanilla::ParticleType", "particle"),
    ("Attribute", "sand::vanilla::Attribute", "attribute"),
    ("Effect", "sand::registry::EffectId", "status effect"),
    ("Potion", "sand::registry::PotionId", "potion"),
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

/// The identity inside a contract's `minecraft` text, such as
/// "Serializes as `minecraft:diamond` wherever ..." or
/// "Serializes as minecraft:speed.".
fn serialized_id(text: &str) -> Option<String> {
    let rest = text.strip_prefix("Serializes as ")?;
    let rest = rest.trim_start_matches('`');
    let end = rest
        .find(|c: char| c == '`' || c.is_whitespace())
        .unwrap_or(rest.len());
    let id = rest[..end].trim_end_matches('.');
    id.contains(':').then(|| id.to_string())
}

fn load() -> Vec<Registry> {
    let mut found: BTreeMap<&'static str, BTreeMap<String, String>> = BTreeMap::new();
    let mut add = |path: &str, minecraft: &str| {
        let Some((parent, variant)) = path.rsplit_once("::") else {
            return;
        };
        if let Some((name, _, _)) = SOURCES.iter().find(|(_, rust, _)| *rust == parent)
            && let Some(id) = serialized_id(minecraft)
        {
            found
                .entry(name)
                .or_default()
                .insert(variant.to_string(), id);
        }
    };
    for provider in sand::__private::api_contract::GENERATED_API_PROVIDER_CATALOGS {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(provider) else {
            continue;
        };
        for entry in value["entries"].as_array().into_iter().flatten() {
            let contract = &entry["contract"];
            if contract["kind"] == "variant" {
                add(
                    contract["canonical_path"].as_str().unwrap_or_default(),
                    contract["minecraft"].as_str().unwrap_or_default(),
                );
            }
        }
    }
    for registration in sand_api_contract::inventory::iter::<sand_api_contract::ApiRegistration> {
        if registration.kind == sand_api_contract::ApiKind::Variant {
            add(registration.canonical_path, registration.minecraft);
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
