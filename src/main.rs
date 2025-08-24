use inquire::Select;

use crate::{
    config::structs::{CollectionConfig, Config, TargetConfig},
    navigation::{create_all_options, get_collection_and_target},
    runner::run_target_command,
};
use std::env::args;

use crate::{
    config::{state::get_config, structs::ListerChoice},
    consts::{build_consts, replace_consts},
    selection_input::{SelectionInput, create_function_input, get_selected_option},
};

mod config;
mod consts;
mod navigation;
mod runner;
mod selection_input;

fn parse_arg(config: &Config) -> (CollectionConfig, TargetConfig) {
    let target_config_arg = args().nth(1);

    match target_config_arg {
        Some(target_config) => get_collection_and_target(&config.root_collection, &target_config),
        None => {
            let mut found_config = config.root_collection.clone();
            let ans = loop {
                let options = create_all_options(&found_config, "");
                let ans = Select::new("Select", options).prompt().unwrap();

                if ans.ends_with("/") {
                    found_config = found_config
                        .collections
                        .unwrap()
                        .iter()
                        .find(|x| x.id == ans[0..ans.len() - 1])
                        .unwrap()
                        .clone();
                } else {
                    break ans;
                }
            };
            get_collection_and_target(&found_config, &ans)
        }
    }
}

fn main() {
    let config = get_config();
    let (collection_config, target_config) = parse_arg(&config);

    let mut consts = build_consts(&collection_config, &target_config);
    let input = match target_config.lister {
        ListerChoice::Function { id, args } => {
            let used_function = config.functions.iter().find(|x| x.id == id).unwrap();

            let args_with_consts = match args.as_ref() {
                Some(args) => args.iter().map(|x| replace_consts(x, &consts)).collect(),
                None => {
                    vec![]
                }
            };
            create_function_input(used_function, &args_with_consts)
        }

        ListerChoice::List { options, args } => SelectionInput { options, args },
    };

    let arg = get_selected_option(&input);
    consts.insert("arg".to_string(), arg.to_string());

    let run_cmd = replace_consts(&target_config.run_cmd, &consts);
    let cwd = target_config.cwd.map(|x| replace_consts(&x, &consts));

    run_target_command(&run_cmd, cwd);
}
