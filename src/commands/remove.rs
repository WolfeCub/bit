use std::fs;

use anyhow::Context;
use clap::Args;

use crate::{objects::Index, utils::make_root_relative};

#[derive(Args, Debug)]
pub struct RemoveArg {
    pub paths: Vec<String>,
}

impl RemoveArg {
    pub fn run(self) -> anyhow::Result<()> {
        let new_index = remove(&self.paths, true)?;

        new_index.write_to_disk()?;

        Ok(())
    }
}

pub fn remove(paths: &[String], delete_file: bool) -> anyhow::Result<Index> {
    let index = Index::parse_from_disk()?;

    let normalized_paths = paths
        .iter()
        .map(make_root_relative)
        .collect::<anyhow::Result<Vec<_>>>()?;

    // TODO: This is inefficient, we should be able to do this in one pass
    let new_entries = index
        .entries
        .into_iter()
        .filter(|entry| !normalized_paths.contains(&entry.name))
        .collect::<Vec<_>>();

    let new_index = Index::from_entries(new_entries);

    if delete_file {
        for p in paths.iter() {
            fs::remove_file(p).with_context(|| format!("Failed to remove file '{}'", p))?;
        }
    }

    Ok(new_index)
}
