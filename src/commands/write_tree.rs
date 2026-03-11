use std::{fs, os::unix::fs::PermissionsExt, path::Path};

use clap::Args;
use itertools::Itertools;

use crate::{
    commands::hash_object::{hash_object, hash_object_from_disk},
    object::ObjectType,
    tree::{Tree, TreeEntry},
    util::{is_file_ignored, repo_root},
};

#[derive(Args, Debug)]
pub struct WriteTreeArg {}

impl WriteTreeArg {
    pub fn run(self) -> anyhow::Result<()> {
        let hash = write_tree(".")?;
        println!("{}", hash);
        Ok(())
    }
}

fn write_tree(dir: &str) -> anyhow::Result<String> {
    let read_dir = fs::read_dir(dir)?;

    let root = repo_root()?.canonicalize()?;
    let canonical_dir = Path::new(dir).canonicalize()?;
    let repo_relative = canonical_dir.strip_prefix(&root)?;

    let entries = read_dir
        .filter_map(|d| {
            let Ok(d) = d else {
                eprintln!("Skipping entry that cannot be read: {:?}", d);
                return None;
            };

            let Some(file_name) = d.file_name().to_str().map(|s| s.to_owned()) else {
                eprintln!("Skipping file with non-UTF-8 name: {:?}", d.file_name());
                return None;
            };

            let Ok(file_type) = d.file_type() else {
                eprintln!("Error reading file type for entry: {:?}", file_name);
                return None;
            };

            let Ok(metadata) = d.metadata() else {
                eprintln!("Error reading metadata for entry: {:?}", file_name);
                return None;
            };

            if file_name == ".bit" || file_name == ".git" {
                return None;
            }

            let full_path = repo_relative.join(&file_name);
            if is_file_ignored(&full_path.to_string_lossy()) {
                return None;
            }

            Some((file_name, file_type, metadata.permissions()))
        })
        .sorted_by_key(|(file_name, file_type, _)| {
            let trailing = if file_type.is_dir() { "/" } else { "" };
            format!("{}{}", file_name, trailing)
        });

    let tree_entries = entries
        .map(
            |(file_name, file_type, file_permissions)| -> anyhow::Result<TreeEntry> {
                let type_ = if file_type.is_dir() {
                    ObjectType::Tree
                } else {
                    ObjectType::Blob
                };

                let path = Path::new(dir)
                    .join(&file_name)
                    .to_str()
                    .expect("Dir and file name should be valid UTF-8")
                    .to_string();

                let hash = if file_type.is_dir() {
                    // TODO: Recursion slow?
                    write_tree(&path)?
                } else {
                    hash_object_from_disk(&path, type_, true)?
                };

                Ok(TreeEntry {
                    mode: format!("{:o}", file_permissions.mode()),
                    path: file_name,
                    hash: hash,
                })
            },
        )
        .collect::<anyhow::Result<Vec<_>>>()?;

    let hash = hash_object(
        ObjectType::Tree,
        Tree {
            entries: tree_entries,
        },
        true,
    )?;

    Ok(hash)
}
