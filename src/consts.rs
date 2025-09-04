use std::collections::HashMap;

use shellexpand::full_with_context_no_errors;

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
    let expanded = full_with_context_no_errors(
        &cmd,
        || std::env::var("HOME").ok(), // tilde expansion
        |var| std::env::var(var).ok(), // env expansion (only if present)
    );

    expanded.to_string()
}
