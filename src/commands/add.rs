use clap::Args;

use crate::{
    commands::hash_object::hash_object_from_disk,
    objects::{Ignore, Index, IndexEntry, ObjectType},
    utils::path::{ArgListExpander, make_root_relative},
};

/// Adds a file to the index (creating a blob object for it)
#[derive(Args, Debug)]
pub struct AddArg {
    pub paths: Vec<String>,
}

impl AddArg {
    pub fn run(self) -> anyhow::Result<()> {
        let ignore = Ignore::build_from_disk()?;

        let mut index = Index::parse_from_disk()?;

        let mut arg_expander = ArgListExpander::new_recursive(&self.paths, &ignore)?;
        while let Some((path, metadata)) = arg_expander.next() {
            let normalized = make_root_relative(&path)?;

            let hash = hash_object_from_disk(path, ObjectType::Blob, true)?;
            let entry = IndexEntry::build_from_file(hash, &normalized, metadata)?;

            // Update the existing entry if we have one otherwise add a new one.
            if let Some(index_entry) = index.entries.iter_mut().find(|ie| ie.name == normalized) {
                *index_entry = entry;
            } else {
                index.entries.push(entry);
            }
        }

        index.sort();
        index.write_to_disk()?;

        Ok(())
    }
}
