use std::{
    collections::{HashMap, HashSet},
    process::Command,
};

use anyhow::{Context, Result, bail};
use inquire::Select;
use regex::Regex;

use crate::{
    config::structs::Function,
    consts::replace_consts,
};

#[derive(Debug)]
pub struct SelectionInput {
    pub options: Vec<String>,
    pub args: Vec<String>,
}

pub(crate) fn create_function_input(
    function: &Function,
    args: &[String],
    consts: &HashMap<String, String>,
) -> Result<SelectionInput> {
    let mut command = function.list_cmd.to_string();

    for (index, val) in args.iter().enumerate() {
        let from = format!("{{{{arg.{index}}}}}");
        command = command.replace(from.as_str(), val);
    }
    if let Some(missing) = Regex::new(r"\{\{arg\.\d+\}\}").unwrap().find(&command) {
        bail!(
            "function '{}' expects {} but it was not given",
            function.id,
            missing.as_str()
        );
    }
    let command = replace_consts(&command, consts);

    create_cmd_input(
        &command,
        function.select_option_regex.as_deref(),
        function.select_arg_regex.as_deref(),
    )
}

pub(crate) fn create_cmd_input(
    command: &str,
    select_option_regex: Option<&str>,
    select_arg_regex: Option<&str>,
) -> Result<SelectionInput> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .with_context(|| format!("failed to run list command: {command}"))?;

    let list_text =
        String::from_utf8(output.stdout).context("list command output is not valid UTF-8")?;
    let input = parse_list_output(&list_text, select_option_regex, select_arg_regex)?;

    if input.options.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "list command returned no options: {command}\n{}",
            stderr.trim()
        );
    }
    Ok(input)
}

fn parse_list_output(
    list_text: &str,
    select_option_regex: Option<&str>,
    select_arg_regex: Option<&str>,
) -> Result<SelectionInput> {
    let name_regex = Regex::new(select_option_regex.unwrap_or("(.*)"))
        .context("invalid select_option_regex")?;
    let arg_regex =
        Regex::new(select_arg_regex.unwrap_or("(.*)")).context("invalid select_arg_regex")?;

    // Build options and args from list command, keeping the output order
    let mut seen = HashSet::new();
    let mut entries = vec![];
    for line in list_text.lines().filter(|x| !x.trim().is_empty()) {
        let name = captures(&name_regex, line).concat();
        let arg = match captures(&arg_regex, line).as_slice() {
            [] => bail!("unable to extract arg from {line}"),
            [arg] => arg.clone(),
            args => bail!(
                "unable to handle multiple args per option (found {}): {:?}",
                args.len(),
                args
            ),
        };
        let name = if name.is_empty() { line.to_string() } else { name };

        if seen.insert((name.clone(), arg.clone())) {
            entries.push((name, arg));
        }
    }

    // Options sharing a name get their arg appended so they can be told apart
    let mut name_counts: HashMap<&str, usize> = HashMap::new();
    for (name, _) in &entries {
        *name_counts.entry(name).or_default() += 1;
    }
    let options = entries
        .iter()
        .map(|(name, arg)| match name_counts[name.as_str()] {
            1 => name.clone(),
            _ => format!("{name} ({arg})"),
        })
        .collect();
    let args = entries.into_iter().map(|(_, arg)| arg).collect();

    Ok(SelectionInput { options, args })
}

/// Returns capture group 1 of every match, or the whole match if the regex has no group.
fn captures(regex: &Regex, line: &str) -> Vec<String> {
    regex
        .captures_iter(line)
        .filter_map(|x| x.get(1).or_else(|| x.get(0)))
        .map(|x| x.as_str().to_string())
        .collect()
}

pub fn get_selected_option(input: &SelectionInput) -> Result<String> {
    if input.options.is_empty() {
        bail!("no options to select from");
    }
    let selected = Select::new("Select Option", input.options.clone())
        .with_page_size(20)
        .raw_prompt()?;

    Ok(input.args[selected.index].clone())
}
