use std::{env, fs};

use anyhow::Context;
use clap::Args;

use crate::{objects::Index, util::repo_root};

#[derive(Args, Debug)]
pub struct RemoveArg {
    pub paths: Vec<String>,
}

impl RemoveArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;

        let index = Index::parse_from_disk()?;

        let cwd = env::current_dir()?;
        let normalized_paths = self
            .paths
            .iter()
            .map(|path| -> anyhow::Result<String> {
                let absolute_path = cwd.join(path).canonicalize()?;
                let repo_relative_path = absolute_path.strip_prefix(&root).with_context(|| {
                    format!("Path {absolute_path:?} is not within the repository")
                })?;

                Ok(repo_relative_path.to_string_lossy().into())
            })
            .collect::<anyhow::Result<Vec<String>>>()?;

        // TODO: This is inefficient, we should be able to do this in one pass
        let new_entries = index
            .entries
            .into_iter()
            .filter(|entry| !normalized_paths.contains(&entry.name))
            .collect::<Vec<_>>();

        let bytes = Index::from_entries(new_entries).serialize()?;

        for p in self.paths.iter() {
            fs::remove_file(p).with_context(|| format!("Failed to remove file '{}'", p))?;
        }

        fs::write(root.join(".bit/index"), bytes).context("Failed to write new index to disk")?;

        Ok(())
    }
}
