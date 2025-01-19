use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::PathBuf,
};

use sand_commands::prelude::MinecraftCommand;

pub type Function = Vec<Box<dyn MinecraftCommand>>;

fn get_pack_format(version: &str) -> u32 {
    match version {
        "1.13" | "1.14" | "1.14.4" => 4,
        "1.15" | "1.16" | "1.16.1" => 5,
        "1.16.2" | "1.16.5" => 6,
        "1.17" | "1.17.1" => 7,
        "1.18" | "1.18.1" => 8,
        "1.18.2" => 9,
        "1.19" | "1.19.3" => 10,
        "1.19.4" => 12,
        "1.20" | "1.20.1" => 15,
        "1.20.2" => 18,
        "1.20.3" | "1.20.4" => 26,
        "1.20.5" | "1.20.6" => 41,
        "1.21" | "1.21.1" => 48,
        "1.21.2" | "1.21.3" => 57,
        "1.21.4" => 61,
        _ => {
            println!(
                "Warning: Unsupported version {}. Defaulting to pack_format 61.",
                version
            );
            61
        }
    }
}

pub struct Datapack {
    name: String,
    namespace: Option<String>,
    description: String,
    version: String,
    functions: HashMap<String, Function>,
    tick_functions: HashMap<String, Function>,

    output_to: PathBuf,
}

impl Datapack {
    pub fn new(name: &str, description: &str, version: &str, output_to: &PathBuf) -> Self {
        Self {
            name: name.to_string(),
            namespace: None,
            description: description.to_string(),
            version: version.to_string(),
            functions: HashMap::new(),
            tick_functions: HashMap::new(),
            output_to: output_to.to_path_buf(),
        }
    }

    pub fn set_namespace(&mut self, namespace: &str) {
        self.namespace = Some(namespace.to_string().to_lowercase());
    }

    pub fn add_function(&mut self, name: &str, statements: Vec<Box<dyn MinecraftCommand>>) {
        self.functions.insert(name.to_string(), statements);
    }

    pub fn add_tick_function(&mut self, name: &str, statements: Vec<Box<dyn MinecraftCommand>>) {
        self.tick_functions
            .insert(name.to_string(), statements);
    }

    pub fn prepare_directories(&self) -> io::Result<()> {
        let root_path = self.output_to.join(&self.name);
        let data_path = root_path.join("data");
        let ns_path = data_path.join(self.namespace.as_ref().unwrap_or(&self.name.to_lowercase()));

        // Create the root, data, and namespace folders
        fs::create_dir_all(&ns_path)?;

        // Create all required subdirectories
        let paths = [
            ns_path.join("function"),
            ns_path.join("structure"),
            ns_path.join("tags"),
            ns_path.join("tags/function"),
            ns_path.join("advancement"),
            ns_path.join("banner_pattern"),
            ns_path.join("chat_type"),
            ns_path.join("damage_type"),
            ns_path.join("dimensions"),
            ns_path.join("dimension_type"),
            ns_path.join("enchantment"),
            ns_path.join("enchantment_provider"),
            ns_path.join("instrument"),
            ns_path.join("item_modifier"),
            ns_path.join("jukebox_song"),
            ns_path.join("loot_table"),
            ns_path.join("painting_variant"),
            ns_path.join("predicate"),
            ns_path.join("recipe"),
            ns_path.join("trim_material"),
            ns_path.join("trim_pattern"),
            ns_path.join("wolf_variant"),
            ns_path.join("worldgen/biome"),
            ns_path.join("worldgen/configured_carver"),
            ns_path.join("worldgen/configured_feature"),
            ns_path.join("worldgen/density_function"),
            ns_path.join("worldgen/noise"),
            ns_path.join("worldgen/noise_settings"),
            ns_path.join("worldgen/placed_feature"),
            ns_path.join("worldgen/processor_list"),
            ns_path.join("worldgen/structure"),
            ns_path.join("worldgen/structure_set"),
            ns_path.join("worldgen/template_pool"),
            ns_path.join("worldgen/world_preset"),
            ns_path.join("worldgen/flat_level_generator_preset"),
            ns_path.join("worldgen/multi_noise_biome_source_parameter_list"),
        ];

        for path in paths {
            fs::create_dir_all(path)?;
        }

        Ok(())
    }

    fn new_function_file(
        &self,
        name: &str,
        statements: &Vec<Box<dyn MinecraftCommand>>,
    ) -> Result<(), std::io::Error> {
        let functions_dir = self.functions_dir();
        let func_file_path = functions_dir.join(format!("{}.mcfunction", name));

        if !func_file_path.exists() {
            fs::create_dir_all(func_file_path.parent().unwrap())?;
        }

        let mut func_file = fs::File::create(func_file_path)?;

        // Convert all statements to strings and join with newlines
        let func_content = statements
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        // Write content and ensure there's a trailing newline
        writeln!(func_file, "{}", func_content)?;

        Ok(())
    }

    pub fn build(&mut self) -> io::Result<()> {
        self.prepare_directories().unwrap();

        let pack_meta = format!(
            r#"{{"pack": {{
    "pack_format": {},
    "description": "{}"
}}}}"#,
            get_pack_format(&self.version),
            self.description
        );
        let pack_path = self.output_to.join(&self.name).join("pack.mcmeta");
        if !pack_path.exists() {
            fs::create_dir_all(pack_path.parent().unwrap())
                .expect("Failed to create pack.mcmeta directory");
        }
        let mut pack_meta_file =
            fs::File::create(&pack_path).expect("Failed to create pack.mcmeta");
        pack_meta_file
            .write_all(pack_meta.as_bytes())
            .expect("Failed to write to pack.mcmeta");

        if !self.functions.is_empty() {
            for function in self.functions.iter() {
                let func_name = function.0;
                let func_statements= function.1;
                self.new_function_file(func_name.as_str(), func_statements)
                    .unwrap();
            }
        }

        Ok(())
    }

    fn functions_dir(&self) -> PathBuf {
        self.output_to
            .join(&self.name)
            .join("data")
            .join(
                &self
                    .namespace
                    .clone()
                    .unwrap_or(self.name.to_lowercase().to_string()),
            )
            .join("function")
    }
}
