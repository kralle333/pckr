use std::{
    fs::{self},
    process::exit,
};

use anyhow::anyhow;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub default_project_path: String,
    pub targets: Vec<TargetConfig>,
}

#[derive(Deserialize)]
pub struct TargetConfig {
    pub name: String,
    pub project_path: Option<String>,
    pub file_name_regex: String,
    pub editor: String,
    pub recursive: bool,
}

pub fn load_config() -> Result<Option<Config>, anyhow::Error> {
    let config_path = dirs::config_dir()
        .ok_or(anyhow!("failed to find config dig"))?
        .join("pckr")
        .join("config.toml");

    if let Ok(false) = fs::exists(&config_path) {
        return Ok(None);
    }
    let content = fs::read_to_string(&config_path)?;

    let config = toml::from_str(&content)?;
    Ok(config)
}

pub fn get_config() -> Config {
    let config = match load_config() {
        Ok(Some(config)) => config,
        Ok(None) => {
            // if config missing ask to create config interactively
            // false => exit
            // default project path

            let mut new_config = Config {
                default_project_path: "".to_string(),
                targets: vec![],
            };

            // Add target loop
            {
                // Name
                // filename regrex
                // recursively
                // open in

                // Add another?
            }
            // Save config
            new_config
        }
        Err(e) => {
            println!("Failed to parse pckr config with error: {e}");
            exit(1);
        }
    };
    config
}
