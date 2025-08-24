use std::{collections::HashMap, process::Command};

use inquire::Select;
use regex::Regex;

use crate::config::structs::Function;

#[derive(Debug)]
pub struct SelectionInput {
    pub options: Vec<String>,
    pub args: Vec<String>,
}

pub(crate) fn create_function_input(function: &Function, args: &[String]) -> SelectionInput {
    let mut command = function.list_cmd.to_string();

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
    // Regex parsing
    let name_regex = function
        .select_option_regex
        .clone()
        .unwrap_or("(.*)".to_string());
    let name_regex = Regex::new(&name_regex).unwrap();

    let arg_regex = function
        .select_arg_regex
        .clone()
        .unwrap_or("(.*)".to_string());
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

pub fn get_selected_option(input: &SelectionInput) -> String {
    let result = Select::new("Select Option", input.options.clone())
        .with_page_size(20)
        .prompt();
    let ans = match result {
        Ok(ans) => ans,
        Err(_) => panic!("Failed to get any options with selected target config"),
    };

    let selected_arg = input
        .args
        .iter()
        .zip::<&[String]>(&input.options)
        .find(|(_, name)| name == &&ans);
    let (arg, _) = selected_arg.unwrap();

    arg.to_string()
}
