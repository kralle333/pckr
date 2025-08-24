use std::collections::HashMap;

use crate::config::structs::{CollectionConfig, TargetConfig};

pub fn build_consts(
    config: &CollectionConfig,
    target_config: &TargetConfig,
) -> HashMap<String, String> {
    let mut consts = HashMap::new();

    for (name, arg) in config.consts.clone().unwrap_or_default() {
        consts.insert(name.to_string(), arg);
    }
    for (name, arg) in target_config.consts.clone().unwrap_or_default() {
        consts.insert(name, arg);
    }
    consts
}

pub fn replace_consts(string: &str, consts: &HashMap<String, String>) -> String {
    let mut cmd = string.to_string();
    for (key, value) in consts {
        cmd = cmd.replace(format!("{{{{{key}}}}}").as_str(), value);
    }

    shellexpand::full(&cmd).unwrap().to_string()
}
