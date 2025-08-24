use crate::config::structs::{CollectionConfig, TargetConfig};

pub fn get_collection_and_target(
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

pub fn create_all_options(collection: &CollectionConfig, path: &str) -> Vec<String> {
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
