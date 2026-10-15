use anyhow::{Result, anyhow};
use inquire::Select;

use crate::config::structs::{CollectionConfig, TargetConfig};

/// Resolves a path like `collection/sub_collection/target` and returns the
/// collections walked through (root first) together with the target.
pub fn get_collection_and_target<'a>(
    root: &'a CollectionConfig,
    arg: &str,
) -> Result<(Vec<&'a CollectionConfig>, &'a TargetConfig)> {
    let mut segments: Vec<&str> = arg.split('/').collect();
    let target_id = segments
        .pop()
        .filter(|id| !id.is_empty())
        .ok_or_else(|| anyhow!("invalid target path '{arg}'"))?;

    let mut chain = vec![root];
    for collection_id in segments {
        let current = *chain.last().unwrap();
        let child = current
            .collections
            .iter()
            .flatten()
            .find(|x| x.id == collection_id)
            .ok_or_else(|| {
                anyhow!(
                    "no collection with id '{collection_id}' in '{}'",
                    current.id
                )
            })?;
        chain.push(child);
    }

    let collection = *chain.last().unwrap();
    let target_config = find_target(collection, target_id)?;
    Ok((chain, target_config))
}

/// Lets the user navigate the collection tree and pick a target.
pub fn select_target(
    root: &CollectionConfig,
) -> Result<(Vec<&CollectionConfig>, &TargetConfig)> {
    let mut chain = vec![root];
    loop {
        let collection = *chain.last().unwrap();
        let ans = Select::new("Select", create_all_options(collection)).prompt()?;

        match ans.strip_suffix('/') {
            Some(collection_id) => {
                let child = collection
                    .collections
                    .iter()
                    .flatten()
                    .find(|x| x.id == collection_id)
                    .ok_or_else(|| anyhow!("no collection with id '{collection_id}'"))?;
                chain.push(child);
            }
            None => {
                let target_config = find_target(collection, &ans)?;
                return Ok((chain, target_config));
            }
        }
    }
}

fn find_target<'a>(collection: &'a CollectionConfig, id: &str) -> Result<&'a TargetConfig> {
    collection
        .targets
        .iter()
        .find(|x| x.id == id)
        .ok_or_else(|| anyhow!("no target with id '{id}' in '{}'", collection.id))
}

fn create_all_options(collection: &CollectionConfig) -> Vec<String> {
    let mut target_paths: Vec<_> = collection.targets.iter().map(|x| x.id.clone()).collect();

    for c in collection.collections.iter().flatten() {
        target_paths.push(format!("{}/", c.id));
    }
    target_paths
}
