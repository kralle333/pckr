use std::collections::HashMap;

use shellexpand::full_with_context_no_errors;

use crate::config::structs::{CollectionConfig, Config, TargetConfig};

/// Merges consts from the global config, every collection on the path to the
/// target and the target itself. Later layers override earlier ones.
pub fn build_consts(
    config: &Config,
    collections: &[&CollectionConfig],
    target_config: &TargetConfig,
) -> HashMap<String, String> {
    let layers = std::iter::once(&config.consts)
        .chain(collections.iter().map(|collection| &collection.consts))
        .chain(std::iter::once(&target_config.consts));

    let mut consts = HashMap::new();
    for layer in layers.flatten() {
        consts.extend(layer.clone());
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

/// Replaces `{{arg}}` with the selected value, shell quoted when `quote` is set,
/// and `{{arg.raw}}` with the value as is.
pub fn replace_arg(string: &str, arg: &str, quote: bool) -> String {
    let value = if quote {
        shell_quote(arg)
    } else {
        arg.to_string()
    };
    string.replace("{{arg}}", &value).replace("{{arg.raw}}", arg)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}
