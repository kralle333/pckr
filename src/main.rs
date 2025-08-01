use config::{Config, TargetConfig, get_config};
use inquire::Select;
use regex::Regex;
use std::{collections::HashMap, env::args, process::Command};

mod config;

fn build_consts(config: &Config, target_config: &TargetConfig) -> HashMap<String, String> {
    let mut consts = HashMap::new();

    for (name, arg) in config.globals.clone().unwrap_or_default() {
        consts.insert(format!("globals.{name}"), arg);
    }
    for (name, arg) in target_config.consts.clone().unwrap_or_default() {
        consts.insert(name, arg);
    }
    consts
}

fn replace_consts(string: &str, consts: &HashMap<String, String>) -> String {
    let mut cmd = string.to_string();
    for (key, value) in consts {
        cmd = cmd.replace(format!("{{{{{key}}}}}").as_str(), value);
    }
    cmd
}

fn main() {
    let config = get_config();

    let selected_target_config = args().nth(1);

    let selected_target_config =
        selected_target_config.and_then(|x| config.targets.iter().find(|c| c.id == x));
    let target_config = match selected_target_config {
        Some(target_config) => target_config,
        None => {
            let for_selector = config.targets.iter().map(|x| x.name.clone()).collect();
            let ans: String = Select::new("Select Target", for_selector)
                .with_page_size(20)
                .prompt()
                .unwrap();
            config.targets.iter().find(|c| c.name == ans).unwrap()
        }
    };

    let consts = build_consts(&config, target_config);

    let list_cmd = replace_consts(&target_config.list_cmd, &consts);
    let result = Command::new("sh")
        .arg("-c")
        .arg(&list_cmd)
        .output()
        .expect("failed to get list output");

    let list_text = String::from_utf8(result.stdout).unwrap();
    let input = create_selection_input(target_config, &list_text);

    show_options(input, consts); // if no target show selector
    // if target, find target config and load the specified target in editor
}

pub struct SelectionInput {
    pub options: Vec<String>,
    pub args: Vec<String>,
    pub run_cmd: String,
    pub cwd: Option<String>,
}

fn create_selection_input(target_config: &TargetConfig, list_text: &str) -> SelectionInput {
    // Regex parsing
    let name_regex = target_config
        .select_option_regex
        .clone()
        .unwrap_or(".*".to_string());
    let name_regex = Regex::new(&name_regex).unwrap();

    let arg_regex = target_config
        .select_arg_regex
        .clone()
        .unwrap_or(".*".to_string());
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
        run_cmd: target_config.run_cmd.to_string(),
        cwd: target_config.cwd.clone(),
    }
}

pub fn show_options(input: SelectionInput, mut consts: HashMap<String, String>) {
    let ans: String = Select::new("Select Option", input.options.clone())
        .with_page_size(20)
        .prompt()
        .unwrap();

    let selected_arg = input
        .args
        .iter()
        .zip(input.options)
        .find(|(_, name)| *name == ans);
    let (arg, _) = selected_arg.unwrap();

    consts.insert("arg".to_string(), arg.to_string());

    let run_cmd = replace_consts(&input.run_cmd, &consts);

    println!("run cmd: {run_cmd}");

    let mut command = Command::new("sh");

    if let Some(path) = input.cwd {
        let cwd = replace_consts(&path, &consts);
        println!("cwd: {cwd}");
        command.current_dir(cwd);
    }

    let status = command.arg("-c").arg(&run_cmd).status();
    match status {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("Command exited with non-zero code: {status}"),
        Err(err) => eprintln!("Failed to run command: {err}"),
    }
}
