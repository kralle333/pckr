use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    process::exit,
};

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CollectionConfig {
    pub id: String,
    pub consts: Option<HashMap<String, String>>,
    pub targets: Vec<TargetConfig>,
    pub collections: Option<Vec<CollectionConfig>>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct TargetConfig {
    pub name: Option<String>,
    pub id: String,
    pub list_cmd: String,
    pub select_option_regex: Option<String>,
    pub select_arg_regex: Option<String>,
    pub run_cmd: String,
    pub cwd: Option<String>,
    pub consts: Option<HashMap<String, String>>,
}

fn get_path() -> Result<PathBuf, anyhow::Error> {
    Ok(dirs::config_dir()
        .ok_or(anyhow!("failed to find config dig"))?
        .join("pckr")
        .join("config.yaml"))
}

fn save_config(config: &CollectionConfig) -> Result<(), anyhow::Error> {
    let config_path = get_path()?;
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();

    fs::write(config_path, serde_yaml::to_string(&config)?)?;

    Ok(())
}

pub fn load_config() -> Result<Option<CollectionConfig>, anyhow::Error> {
    let config_path = get_path()?;

    if let Ok(false) = fs::exists(&config_path) {
        return Ok(None);
    }
    let content = fs::read_to_string(&config_path)?;

    let config = match serde_yaml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            println!("{e:?}");
            return Err(anyhow!("Failed to deserialize config {}", e));
        }
    };
    Ok(config)
}

pub fn get_config() -> CollectionConfig {
    match load_config() {
        Ok(Some(config)) => config,
        Ok(None) => {
            match inquire::Confirm::new(
                "Failed to find config file, create config file interactively?",
            )
            .with_default(false)
            .prompt()
            {
                Ok(true) => {}
                Ok(false) => exit(0),
                Err(_) => {}
            };

            // TODO: add template system
            let new_config = CollectionConfig {
                collections: None,
                id: "root".to_string(),
                consts: Some(HashMap::new()),
                targets: vec![],
            };

            save_config(&new_config).unwrap();
            new_config
        }
        Err(e) => {
            println!("Failed to parse pckr config with error: {e}");
            exit(1);
        }
    }
}
