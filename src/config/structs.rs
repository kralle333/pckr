use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Function {
    pub id: String,
    pub arg_descriptions: Option<Vec<String>>,
    pub list_cmd: String,
    pub select_option_regex: Option<String>,
    pub select_arg_regex: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub functions: Vec<Function>,
    pub consts: Option<HashMap<String, String>>,
    pub root_collection: CollectionConfig,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct CollectionConfig {
    pub id: String,
    pub consts: Option<HashMap<String, String>>,
    pub targets: Vec<TargetConfig>,
    pub collections: Option<Vec<CollectionConfig>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ListerChoice {
    Function {
        id: String,
        args: Option<Vec<String>>,
    },
    List {
        options: Vec<String>,
        args: Vec<String>,
    },
    Cmd {
        list_cmd: String,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TargetConfig {
    pub name: Option<String>,
    pub id: String,
    pub lister: ListerChoice,
    pub run_cmd: String,
    pub cwd: Option<String>,
    pub consts: Option<HashMap<String, String>>,
}
