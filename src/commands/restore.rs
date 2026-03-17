use std::fs;

use clap::Args;

use crate::{
    objects::{Ignore, Index, Object, ObjectType},
    utils::path::{ArgListExpander, make_root_relative},
};

/// Discards any working changes for the specified paths
#[derive(Args, Debug)]
pub struct RestoreArg {
    pub files: Vec<String>,
}

impl RestoreArg {
    pub fn run(self) -> anyhow::Result<()> {
        let index = Index::parse_from_disk()?;
        let ignore = Ignore::build_from_disk()?;
        let expanded = ArgListExpander::new_recursive(&self.files, &ignore)?;

        for (path, _) in expanded {
            let normalized = make_root_relative(&path)?;
            if let Some(index_entry) = index.entries.iter().find(|ie| ie.name == normalized) {
                let hex = hex::encode(index_entry.sha);
                let object = Object::<Vec<u8>>::read_from_disk(&hex, ObjectType::Blob)?;
                fs::write(&path, &object.inner)?;
            }
        }

        Ok(())
    }
}
