use config::{CollectionConfig, SelectionInput, TargetConfig, get_config};
use inquire::Select;
use std::{collections::HashMap, env::args, process::Command};

mod config;

fn build_consts(
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

fn replace_consts(string: &str, consts: &HashMap<String, String>) -> String {
    let mut cmd = string.to_string();
    for (key, value) in consts {
        cmd = cmd.replace(format!("{{{{{key}}}}}").as_str(), value);
    }

    shellexpand::full(&cmd).unwrap().to_string()
}

fn get_collection_and_target(
    collection: &CollectionConfig,
    arg: &str,
) -> (CollectionConfig, TargetConfig) {
    let path: Vec<String> = arg.split("/").map(|x| x.to_string()).collect();
    if path.is_empty() {
        panic!("invalid arg");
    } else if path.len() == 1 {
        let target_config = collection
            .targets
            .iter()
            .find(|x| x.id == *path.first().unwrap())
            .cloned()
            .unwrap();

        (collection.clone(), target_config)
    } else {
        let this_collection_id = path.first().unwrap();
        if collection.collections.is_none() {
            panic!("expected collection with id {this_collection_id}, but found none!");
        }
        let child = collection
            .collections
            .as_ref()
            .unwrap()
            .iter()
            .find(|x| x.id == *this_collection_id)
            .unwrap();

        get_collection_and_target(child, &path.as_slice()[1..].join("/"))
    }
}

fn create_all_options(collection: &CollectionConfig, path: &str) -> Vec<String> {
    let prepend = if path.is_empty() {
        "".to_string()
    } else {
        format!("{path}/")
    };
    let mut target_paths: Vec<_> = collection
        .targets
        .iter()
        .map(|x| format!("{prepend}{}", x.id))
        .collect();

    if let Some(collections) = &collection.collections {
        for c in collections {
            target_paths.push(format!("{}/", c.id));
        }
    }
    target_paths
}

fn main() {
    let config = get_config();

    let target_config_arg = args().nth(1);

    let root_collection = config.root_collection;

    let (collection_config, target_config) = match target_config_arg {
        Some(target_config) => get_collection_and_target(&root_collection, &target_config),
        None => {
            let mut found_config = root_collection.clone();
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
    };

    let mut consts = build_consts(&collection_config, &target_config);

    let used_function = config
        .functions
        .iter()
        .find(|x| x.id == target_config.function_id)
        .unwrap();

    let args_with_consts: Vec<_> = target_config
        .function_args
        .iter()
        .map(|x| replace_consts(x, &consts))
        .collect();

    let input = used_function.execute(&args_with_consts);

    let arg = get_selected_option(&input);
    println!("arg: {arg}");

    consts.insert("arg".to_string(), arg.to_string());

    println!("{}", target_config.run_cmd);
    let run_cmd = replace_consts(&target_config.run_cmd, &consts);

    println!("RUNCMD: {run_cmd}");
    let mut command = Command::new("sh");

    if let Some(path) = target_config.cwd {
        let cwd = replace_consts(&path, &consts);

        command.current_dir(cwd);
    }

    let status = command.arg("-c").arg(&run_cmd).status();
    match status {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("Command exited with non-zero code: {status}"),
        Err(err) => eprintln!("Failed to run command: {err}"),
    }
}

pub fn get_selected_option(input: &SelectionInput) -> String {
    let ans: String = Select::new("Select Option", input.options.clone())
        .with_page_size(20)
        .prompt()
        .unwrap();

    let selected_arg = input
        .args
        .iter()
        .zip::<&[String]>(&input.options)
        .find(|(_, name)| name == &&ans);
    let (arg, _) = selected_arg.unwrap();

    arg.to_string()
}
