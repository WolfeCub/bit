use std::{
    fs::{self},
};

use anyhow::Context;
use clap::Args;

use crate::{
    commands::{hash_object::hash_object_from_disk, remove},
    objects::{IndexEntry, ObjectType},
    utils::path::make_root_relative,
};

/// Adds a file to the index (creating a blob object for it)
#[derive(Args, Debug)]
pub struct AddArg {
    pub paths: Vec<String>,
}

impl AddArg {
    pub fn run(self) -> anyhow::Result<()> {
        let mut new_index = remove::remove(&self.paths, false)?;

        // TODO: Remove already does this. We're duplicating work here.
        // we can maybe annotate it with #[cached] or pass around the normalized_paths
        let normalized_paths = self
            .paths
            .iter()
            .map(make_root_relative)
            .collect::<anyhow::Result<Vec<_>>>()?;

        for (path, normalized) in self.paths.into_iter().zip(normalized_paths.into_iter()) {
            let metadata = fs::metadata(&path)
                .with_context(|| format!("Failed to read metadata for '{}'", path))?;

            let hash = hash_object_from_disk(&path, ObjectType::Blob, true)?;

            let entry = IndexEntry::build_from_file(hash, &normalized, metadata)?;
            new_index.entries.push(entry);
        }

        new_index.entries.sort_by(|a, b| a.name.cmp(&b.name));

        new_index.write_to_disk()?;

        Ok(())
    }
}
