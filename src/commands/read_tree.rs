use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
};

use anyhow::Context;
use clap::Args;

use crate::objects::{Index, IndexEntry, Object, ObjectType, TreeEntry, flatten_tree_from_disk};
use hex::FromHex;

/// Extract the contents of a tree object into the working directory
#[derive(Args, Debug)]
pub struct ReadTreeArg {
    #[arg(short)]
    pub update_working_directory: bool,

    #[arg(short, requires = "update_working_directory")]
    pub merge: bool,

    pub tree: String,
}

impl ReadTreeArg {
    pub fn run(self) -> anyhow::Result<()> {
        read_tree(&self.tree, self.update_working_directory)
    }
}

// TODO: Merging is really simple. Eventually let's implement something smarter
pub fn read_tree(tree: &str, update_working_directory: bool) -> anyhow::Result<()> {
    let flat_tree = flatten_tree_from_disk(&tree)?;
    read_flattened_tree_ignorelist(flat_tree, update_working_directory, None)
}

/// Ignored files are not updated in the working directory. Use with caution this only makes sense
/// in certain situations. e.g. You've already manually checked for conflicts like when switching
/// branches
pub fn read_flattened_tree_ignorelist(
    flat_tree: HashMap<String, TreeEntry>,
    update_working_directory: bool,
    ignore_list: Option<HashSet<String>>,
) -> anyhow::Result<()> {
    let mut index = Index::parse_from_disk()?;

    let mut new_entries = HashSet::<String>::new();
    for (path, tree_entry) in flat_tree {
        let hash_bytes = <[u8; 20]>::from_hex(&tree_entry.hash)?;
        let mode = u32::from_str_radix(&tree_entry.mode, 8).context("Mode was not octal")?;

        if let Some(existing) = index.entries.iter_mut().find(|e| e.name == path) {
            existing.sha = hash_bytes;
            existing.mode = mode;
        } else {
            index.entries.push(IndexEntry {
                sha: hash_bytes,
                name: path.clone(),
                mode: mode,
                flags: path.len().min(0xFFF) as u16,
                ..Default::default()
            });
            new_entries.insert(path.clone());
        }
    }
    index.entries.sort_by(|a, b| a.name.cmp(&b.name));

    if update_working_directory {
        for entry in index.entries.iter_mut() {
            // Don't overwrite files in the ignore list
            if ignore_list
                .as_ref()
                .map(|il| il.contains(&entry.name))
                .unwrap_or(false)
            {
                continue;
            }

            let hash = hex::encode(entry.sha);
            let object = Object::<Vec<u8>>::read_from_disk(&hash, ObjectType::Blob)?;

            OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&entry.name)?
                .write_all(&object.inner)?;

            if new_entries.contains(&entry.name) {
                let metadata = fs::metadata(&entry.name).context("Unable to read metadata")?;
                *entry = IndexEntry::build_from_file(entry.sha, &entry.name, metadata)?;
            }
        }
    }

    index.write_to_disk()?;

    Ok(())
}
