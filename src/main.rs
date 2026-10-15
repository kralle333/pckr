use anyhow::{Result, anyhow, bail};
use inquire::InquireError;

use crate::{
    navigation::{get_collection_and_target, select_target},
    runner::run_target_command,
};
use std::{env::args, process::exit};

use crate::{
    config::{state::get_config, structs::ListerChoice},
    consts::{build_consts, replace_arg, replace_consts},
    selection_input::{
        SelectionInput, create_cmd_input, create_function_input, get_selected_option,
    },
};

mod config;
mod consts;
mod navigation;
mod runner;
mod selection_input;

fn main() {
    match run() {
        Ok(code) => exit(code),
        Err(err) => match err.downcast_ref::<InquireError>() {
            Some(InquireError::OperationCanceled) => exit(0),
            Some(InquireError::OperationInterrupted) => exit(130),
            _ => {
                eprintln!("Error: {err:#}");
                exit(1);
            }
        },
    }
}

fn run() -> Result<i32> {
    let config = get_config();
    let (collections, target_config) = match args().nth(1) {
        Some(path) => get_collection_and_target(&config.root_collection, &path)?,
        None => select_target(&config.root_collection)?,
    };

    let consts = build_consts(&config, &collections, target_config);
    let input = match &target_config.lister {
        ListerChoice::Function { id, args } => {
            let used_function = config
                .functions
                .iter()
                .find(|x| x.id == *id)
                .ok_or_else(|| {
                    anyhow!(
                        "target '{}' uses unknown function '{id}'",
                        target_config.id
                    )
                })?;

            let args_with_consts: Vec<String> = args
                .iter()
                .flatten()
                .map(|x| replace_consts(x, &consts))
                .collect();
            create_function_input(used_function, &args_with_consts, &consts)?
        }
        ListerChoice::List { options, args } => {
            let args = args.clone().unwrap_or_else(|| options.clone());
            if args.len() != options.len() {
                bail!(
                    "target '{}' has {} options but {} args",
                    target_config.id,
                    options.len(),
                    args.len()
                );
            }
            SelectionInput {
                options: options.clone(),
                args,
            }
        }
        ListerChoice::Cmd {
            list_cmd,
            select_option_regex,
            select_arg_regex,
        } => create_cmd_input(
            &replace_consts(list_cmd, &consts),
            select_option_regex.as_deref(),
            select_arg_regex.as_deref(),
        )?,
    };

    let arg = get_selected_option(&input)?;

    let run_cmd = replace_arg(&replace_consts(&target_config.run_cmd, &consts), &arg, true);
    let cwd = target_config
        .cwd
        .as_ref()
        .map(|x| replace_arg(&replace_consts(x, &consts), &arg, false));

    run_target_command(&run_cmd, cwd.as_deref())
}
