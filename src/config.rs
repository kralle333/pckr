use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    process::exit,
};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub targets: Vec<TargetConfig>,
    pub globals: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct TargetConfig {
    pub name: String,
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

fn save_config(config: &Config) -> Result<(), anyhow::Error> {
    let config_path = get_path()?;
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();

    fs::write(config_path, serde_yaml::to_string(&config)?)?;

    Ok(())
}

pub fn load_config() -> Result<Option<Config>, anyhow::Error> {
    let config_path = get_path()?;

    if let Ok(false) = fs::exists(&config_path) {
        return Ok(None);
    }
    let content = fs::read_to_string(&config_path)?;

    let config = serde_yaml::from_str(&content)?;
    Ok(config)
}

pub fn get_config() -> Config {
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
            // if config missing ask to create config interactively
            // false => exit
            // default project path

            let home_dir = dirs::home_dir().unwrap();
            let default_path = loop {
                let default_path = inquire::Text::new("Path to where to search for projects")
                    .with_default(home_dir.to_str().unwrap())
                    .prompt()
                    .unwrap();

                let path = shellexpand::full(&default_path).unwrap().to_string();
                if !fs::exists(&path).unwrap() {
                    println!("Invalid path entered");
                    continue;
                } else {
                    break path;
                }
            };

            // TODO: add template system
            let new_config = Config {
                targets: vec![],
                globals: Some(HashMap::new()),
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
