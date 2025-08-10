use anyhow::anyhow;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    process::{Command, exit},
};

fn initial_functions() -> Vec<Function> {
    let files = Function {
        id: "find.files".to_string(),
        list_cmd: r#"fd --type f . {{arg.0}} | grep -E '{{arg.1}}' | sort -u"#.to_string(),
        arg_descriptions: vec!["Root dir".to_string(), "File regex".to_string()],
        select_option_regex: None,
        select_arg_regex: None,
    };

    let dirs = Function {
        id: "list.folders".to_string(),
        arg_descriptions: vec!["Root dir".to_string(), "File regex".to_string()],
        list_cmd: r#"fd --type f . {{arg.0}} | grep -E '{{arg.1}}' | awk -F/ 'NF{NF--; print "/"$0}' OFS=/ | sort -u"#.to_string(),
        select_option_regex: Some("([^/]+)$".to_string()),
        select_arg_regex: None,
    };

    vec![files, dirs]
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Function {
    pub id: String,
    pub arg_descriptions: Vec<String>,
    pub list_cmd: String,
    pub select_option_regex: Option<String>,
    pub select_arg_regex: Option<String>,
}
#[derive(Debug)]
pub struct SelectionInput {
    pub options: Vec<String>,
    pub args: Vec<String>,
}
impl Function {
    pub(crate) fn execute(&self, args: &[String]) -> SelectionInput {
        let mut command = self.list_cmd.to_string();

        for (index, val) in args.iter().enumerate() {
            let from = format!("{{{{arg.{index}}}}}");
            command = command.replace(from.as_str(), val);
        }

        let result = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()
            .expect("failed to get list output");

        let list_text = String::from_utf8(result.stdout).unwrap();
        self.create_selection_input(&list_text)
    }

    fn create_selection_input(&self, list_text: &str) -> SelectionInput {
        // Regex parsing
        let name_regex = self
            .select_option_regex
            .clone()
            .unwrap_or("(.*)".to_string());
        let name_regex = Regex::new(&name_regex).unwrap();

        let arg_regex = self.select_arg_regex.clone().unwrap_or("(.*)".to_string());
        let arg_regex = Regex::new(&arg_regex).unwrap();

        // Build options and args from list command
        let input: (Vec<String>, Vec<String>) = list_text
            .lines()
            .map(|x| {
                let name: String = name_regex
                    .captures_iter(x)
                    .map(|x| x.get(1).unwrap().as_str().to_string())
                    .collect();

                let arg: Vec<String> = arg_regex
                    .captures_iter(x)
                    .map(|x| x.get(1).unwrap().as_str().to_string())
                    .collect();

                match arg {
                    _ if arg.is_empty() => {
                        panic!("unable to extract arg from {x}");
                    }
                    _ if arg.len() > 1 => {
                        panic!(
                            "unable to handle multiple args per option (found {}): {:?}",
                            arg.len(),
                            arg
                        );
                    }
                    _ => (name, arg.first().unwrap().to_string()),
                }
            })
            .fold(HashMap::new(), |mut acc, (name, arg)| {
                acc.insert(name, arg);
                acc
            })
            .iter()
            .fold((vec![], vec![]), |(names, args), (name, arg)| {
                (
                    [names, vec![name.to_string()]].concat(),
                    [args, vec![arg.to_string()]].concat(),
                )
            });

        SelectionInput {
            options: input.0,
            args: input.1,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Config {
    pub functions: Vec<Function>,
    pub consts: Option<HashMap<String, String>>,
    pub root_collection: CollectionConfig,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CollectionConfig {
    pub id: String,
    pub consts: Option<HashMap<String, String>>,
    pub functions: Vec<Function>,
    pub targets: Vec<TargetConfig>,
    pub collections: Option<Vec<CollectionConfig>>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct TargetConfig {
    pub name: Option<String>,
    pub id: String,
    pub function_id: String,
    pub function_args: Vec<String>,
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

    let config = match serde_yaml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            return Err(anyhow!("Failed to deserialize config {}", e));
        }
    };
    Ok(config)
}

pub fn get_config() -> Config {
    match load_config() {
        Ok(Some(config)) => config,
        Ok(None) => {
            match inquire::Confirm::new("Failed to find config file, create default config file?")
                .with_default(false)
                .prompt()
            {
                Ok(true) => {}
                Ok(false) => exit(0),
                Err(_) => {}
            };

            let default_functions = initial_functions();

            // let targets = match inquire::Confirm::new("Add target?").prompt() {
            //     Ok(true) => {
            //         let mut targets = vec![];
            //         loop {
            //             match inquire::Confirm::new("Add another?").prompt() {
            //                 Ok(true) => {
            //                     let id = inquire::Text::new("id").prompt().unwrap();
            //                     let functions = default_functions
            //                         .iter()
            //                         .map(|x| format!("{}({})", x.id.to_string(), x.args.join(",")))
            //                         .collect();
            //                     inquire::Select::new("Select function", functions);
            //                 }

            //                 Ok(false) => break,
            //                 Err(_) => exit(1),
            //             }
            //         }
            //         targets;
            //     }
            //     Ok(false) => {}
            //     Err(_) => exit(1),
            // };

            // TODO: add template system
            let root_collection = CollectionConfig {
                collections: None,
                id: "root".to_string(),
                consts: Some(HashMap::new()),
                targets: vec![TargetConfig {
                    name: Some("Open log file".to_string()),
                    id: "open.log.file".to_string(),
                    function_id: "find.files".to_string(),
                    run_cmd: "$EDITOR {{arg}}".to_string(),
                    cwd: None,
                    consts: None,
                    function_args: vec!["~/".to_string(), ".log$".to_string()],
                }],
                functions: vec![],
            };

            let new_config = Config {
                functions: default_functions,
                consts: Some(HashMap::new()),
                root_collection,
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
