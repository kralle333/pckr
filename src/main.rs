use config::get_config;
use inquire::Select;
use std::env::args;

mod config;
mod selection;

fn main() {
    let config = get_config();

    let selected_target_config = args().nth(1);

    let selected_target_config =
        selected_target_config.and_then(|x| config.targets.iter().find(|c| c.name == x));
    match selected_target_config {
        Some(target_config) => {
            let project_path = target_config
                .project_path
                .as_ref()
                .unwrap_or(&config.default_project_path);

            selection::show_target_selection(target_config, project_path);
        }
        None => {
            let for_selector = config.targets.iter().map(|x| x.name.clone()).collect();
            let ans: String = Select::new("Select Target", for_selector)
                .with_page_size(20)
                .prompt()
                .unwrap();
            let target_config = config.targets.iter().find(|c| c.name == ans).unwrap();

            let project_path = target_config
                .project_path
                .as_ref()
                .unwrap_or(&config.default_project_path);

            selection::show_target_selection(target_config, project_path);
        }
    }
    // if no target show selector
    // if target, find target config and load the specified target in editor
}
