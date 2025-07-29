use std::{
    fs::{self},
    path::PathBuf,
    process::Command,
};

use inquire::Select;
use regex::Regex;

use crate::config::TargetConfig;

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct SelectableTargets {
    path: PathBuf,
    name: String,
}

fn recursive_collect(p: &PathBuf, re: &Regex) -> Vec<PathBuf> {
    let found_match = fs::read_dir(p)
        .expect("failed to read project path")
        .map(|x| x.expect("couldnt map dir"))
        .any(|f| match f.file_name().to_str() {
            None => false,
            Some(f) => re.is_match(f),
        });
    if found_match {
        return vec![p.clone()];
    }

    let project_paths: Vec<PathBuf> = fs::read_dir(p)
        .unwrap()
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_dir() { Some(path) } else { None }
            })
        })
        .collect();
    project_paths.iter().fold(vec![], |acc, val| {
        [recursive_collect(val, re), acc].concat()
    })
}

fn collect(p: &PathBuf, re: &Regex) -> Vec<PathBuf> {
    let project_paths: Vec<PathBuf> = fs::read_dir(p)
        .unwrap()
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_dir() { Some(path) } else { None }
            })
        })
        .collect();

    project_paths
        .iter()
        .fold(vec![], |acc, val| {
            let found_match = fs::read_dir(val)
                .expect("failed to read project path")
                .map(|x| x.expect("couldnt map dir"))
                .any(|f| match f.file_name().to_str() {
                    None => false,
                    Some(f) => re.is_match(f),
                });

            match found_match {
                true => [acc, vec![val.clone()]].concat(),
                false => acc,
            }
        })
        .into_iter()
        .collect()
}

pub(crate) fn find_targets(
    target_path: &PathBuf,
    file_name_project_regex: &str,
    recursive: bool,
) -> Result<Vec<SelectableTargets>, anyhow::Error> {
    let re = Regex::new(file_name_project_regex)?;

    let paths = match recursive {
        true => recursive_collect(target_path, &re),
        false => collect(target_path, &re),
    };

    let selectables = paths
        .into_iter()
        .map(|x| SelectableTargets {
            path: x.to_path_buf(),
            name: x.file_name().unwrap().to_str().unwrap().to_string(),
        })
        .collect();

    Ok(selectables)
}

pub fn show_target_selection(target_config: &TargetConfig, target_folder: &str) {
    let target_folder = shellexpand::full(target_folder).unwrap().to_string();
    let target_path = PathBuf::new().join(&target_folder);
    let targets = find_targets(
        &target_path,
        &target_config.file_name_regex,
        target_config.recursive,
    )
    .unwrap_or_default();

    let for_selector: Vec<String> = targets.iter().map(|x| x.name.clone()).collect();
    if for_selector.is_empty() {
        println!("Error: Found no projects");
        return;
    }
    let ans: String = Select::new("Select Project", for_selector)
        .with_page_size(20)
        .prompt()
        .unwrap();

    let selected_target_path = targets
        .into_iter()
        .find(|x| x.path.ends_with(ans.clone()))
        .unwrap()
        .path;

    let selected_target_path = match &target_config.open_in {
        Some(val) => selected_target_path.join(val),
        None => selected_target_path,
    };

    // return;
    let status = Command::new(&target_config.editor)
        .arg(".")
        .current_dir(selected_target_path.as_path())
        .status();

    match status {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("IDE exited with non-zero code: {status}"),
        Err(err) => eprintln!("Failed to launch IDE: {err}"),
    }
}
