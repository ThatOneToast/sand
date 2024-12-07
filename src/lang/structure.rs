use serde::Deserialize;
use tlogger::prelude::*;

#[derive(Deserialize, Debug)]
pub struct SandConfig {
    pub name: String,
    pub version: String,
}

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use toml;

pub struct SandStructure {
    sand_files: Vec<File>,
    project_root: PathBuf,
    config: SandConfig,
}

impl SandStructure {
    pub fn new(project_root: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        let cwd = std::env::current_dir()?;

        debug!("Current Working Directory:", "{:?}", cwd);

        // Read .sand files
        for entry in std::fs::read_dir(&cwd).expect("failed to read dir") {
            let entry = entry.expect("failed to read entry");
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "sand" {
                        files.push(File::open(path).expect("Failed to open file"));
                    }
                }
            }
        }

        // Read and parse TOML config
        let config_path = cwd.join("Sand.toml");

        if !config_path.exists() {
            error!(
                "Config file not found",
                "Please create a Sand.toml file in the root directory of your project."
            );
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Config file not found",
            )));
        }

        let mut config_content = String::new();
        File::open(config_path)?.read_to_string(&mut config_content)?;
        let config: SandConfig = toml::from_str(&config_content)?;

        Ok(Self {
            sand_files: files,
            project_root,
            config,
        })
    }

    pub fn sand_to_grains(&self) -> String {
        let mut grains = String::new();
        info!("Sand Structure", "Combining Sand Files");
        for file in &self.sand_files {
            let mut content = String::new();
            let mut file_clone = file.try_clone().expect("Failed to clone file handle");
            file_clone
                .read_to_string(&mut content)
                .expect("Failed to read file");
            grains.push_str(&content);
            grains.push('\n'); // Add newline between files
        }
        success!("Sand Structure", "Combined Sand Files");
        grains.trim().to_string()
    }

    // Getter for config
    pub fn config(&self) -> &SandConfig {
        &self.config
    }
}
