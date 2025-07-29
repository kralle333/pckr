use std::{
    fs::{self},
    path::PathBuf,
    process::exit,
};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use toml::to_string_pretty;

use crate::selection::find_targets;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub default_project_path: String,
    pub targets: Vec<TargetConfig>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct TargetConfig {
    pub name: String,
    pub project_path: Option<String>,
    pub file_name_regex: String,
    pub editor: String,
    pub recursive: bool,
    pub open_in: Option<String>,
}

fn get_path() -> Result<PathBuf, anyhow::Error> {
    Ok(dirs::config_dir()
        .ok_or(anyhow!("failed to find config dig"))?
        .join("pckr")
        .join("config.toml"))
}

fn save_config(config: &Config) -> Result<(), anyhow::Error> {
    let config_path = get_path()?;
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();

    fs::write(config_path, to_string_pretty(&config)?)?;

    Ok(())
}

pub fn load_config() -> Result<Option<Config>, anyhow::Error> {
    let config_path = get_path()?;

    if let Ok(false) = fs::exists(&config_path) {
        return Ok(None);
    }
    let content = fs::read_to_string(&config_path)?;

    let config = toml::from_str(&content)?;
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

            println!("Adding new target");
            let mut targets = vec![];

            loop {
                let ans = inquire::Text::new("Enter name of your target")
                    .prompt()
                    .unwrap();

                let path = loop {
                    let path = inquire::Text::new("Path to where to search for projects")
                        .with_default("")
                        .with_help_message(&format!(
                            "Leave empty to fall back to using config project path {default_path}"
                        ))
                        .prompt()
                        .unwrap();

                    if path.is_empty() {
                        break path;
                    }
                    let path = shellexpand::full(&path).unwrap().to_string();
                    if !fs::exists(&path).unwrap() {
                        println!("Invalid path entered");
                        continue;
                    } else {
                        break path;
                    }
                };
                let path = (!path.is_empty()).then_some(path);

                let regex = inquire::Text::new("File regex for finding projects")
                    .prompt()
                    .unwrap();

                let editor = inquire::Text::new("Editor to open projects in")
                    .with_help_message("Command to open your edtior from your shell")
                    .prompt()
                    .unwrap();

                let recursive = inquire::Confirm::new("Recursive search for projects?")
                    .with_default(false)
                    .prompt()
                    .unwrap();

                let open_in = inquire::Text::new("File or dir to optionally open in")
                    .with_default("")
                    .prompt()
                    .map(Some)
                    .unwrap_or(None);

                println!("Looking for projects with this config...");
                let to_search = match path.as_ref() {
                    Some(path) => path,
                    None => default_path.as_str(),
                };

                let projects_with_this_config =
                    find_targets(&PathBuf::new().join(to_search), &regex, recursive).unwrap();

                println!("Found {} projects", projects_with_this_config.len());

                targets.push(TargetConfig {
                    name: ans.to_string(),
                    project_path: path,
                    file_name_regex: regex,
                    editor,
                    recursive,
                    open_in,
                });
                let add_new = inquire::Confirm::new("Add another?").prompt().unwrap();
                if !add_new {
                    break;
                }
            }
            let new_config = Config {
                default_project_path: default_path,
                targets,
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
