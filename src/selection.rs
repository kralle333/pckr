use std::{
    fs::{self},
    path::PathBuf,
    process::Command,
};

use inquire::Select;
use regex::Regex;
use walkdir::WalkDir;

use crate::config::TargetConfig;

#[derive(Clone, PartialEq)]
struct SelectableTargets {
    path: PathBuf,
    name: String,
}

fn get_filter(p: &PathBuf, re: &Regex, recursive: bool) -> bool {
    if recursive {
        WalkDir::new(p)
            .into_iter()
            .filter_map(|x| x.ok())
            .filter(|x| x.file_type().is_file())
            .any(|f| match f.file_name().to_str() {
                None => false,
                Some(f) => re.is_match(f),
            })
    } else {
        fs::read_dir(p)
            .expect("failed to read project path")
            .map(|x| x.expect("couldnt map dir"))
            .any(|f| match f.file_name().to_str() {
                None => false,
                Some(f) => re.is_match(f),
            })
    }
}

fn find_targets(
    target_path: &PathBuf,
    file_name_project_regex: &str,
    recursive: bool,
) -> Result<Vec<SelectableTargets>, anyhow::Error> {
    let project_paths: Vec<PathBuf> = fs::read_dir(target_path)?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_dir() { Some(path) } else { None }
            })
        })
        .collect();

    let re = Regex::new(file_name_project_regex)?;

    let selectable_targets: Vec<SelectableTargets> = project_paths
        .into_iter()
        .filter(|p| get_filter(p, &re, recursive))
        .map(|x| SelectableTargets {
            path: x.clone(),
            name: x.file_name().unwrap().to_string_lossy().to_string(),
        })
        .collect();

    Ok(selectable_targets)
}

pub fn show_target_selection(target_config: &TargetConfig, target_folder: &str) {
    let target_path = PathBuf::new().join(target_folder);
    let targets = find_targets(
        &target_path,
        &target_config.file_name_regex,
        target_config.recursive,
    )
    .unwrap_or_default();

    let for_selector = targets.iter().map(|x| x.name.clone()).collect();
    let ans: String = Select::new("Select Rust Project", for_selector)
        .with_page_size(20)
        .prompt()
        .unwrap();

    let selected_target_path = targets
        .into_iter()
        .find(|x| x.path.ends_with(ans.clone()))
        .unwrap()
        .path;

    let status = Command::new(&target_config.editor)
        .arg(selected_target_path)
        .status();

    match status {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("IDE exited with non-zero code: {status}"),
        Err(err) => eprintln!("Failed to launch IDE: {err}"),
    }
}
