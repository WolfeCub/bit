use std::fs;

use anyhow::Context;
use clap::Args;

use crate::{
    objects::{Ignore, Index},
    utils::path::{ArgListExpander, make_root_relative},
};

/// Remove a file from the index and delete it from the filesystem.
#[derive(Args, Debug)]
pub struct RemoveArg {
    pub paths: Vec<String>,

    #[arg(short)]
    pub recursive: bool,
}

impl RemoveArg {
    pub fn run(self) -> anyhow::Result<()> {
        let new_index = remove(&self.paths, true, self.recursive)?;

        new_index.write_to_disk()?;

        Ok(())
    }
}

pub fn remove(paths: &[String], delete_file: bool, recursive: bool) -> anyhow::Result<Index> {
    let ignore = Ignore::build_from_disk()?;

    let mut index = Index::parse_from_disk()?;

    let expanded_args = ArgListExpander::new(paths, &ignore, recursive)?.collect::<Vec<_>>();
    for (path, _) in expanded_args.iter() {
        let normalized = make_root_relative(path)?;
        index.entries.retain(|e| e.name != normalized);
    }

    if delete_file {
        for (path, _) in expanded_args.iter() {
            fs::remove_file(path)
                .with_context(|| format!("Failed to remove file '{}'", path.to_string_lossy()))?;
        }
    }

    Ok(index)
}
