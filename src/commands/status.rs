use std::{collections::HashMap, fs, os::unix::fs::MetadataExt};

use clap::Args;
use colored::Colorize;

use crate::{
    commands::hash_object::hash_object_from_disk,
    objects::{
        Commit, Ignore, Index, IndexEntry, Object,
        ObjectType::{self},
        Tree,
    },
    utils::{
        bit_dir_walker::BitDirWalker,
        path::relative_path_string,
        repo::{cwd, head_state, repo_root},
    },
};

#[derive(Args, Debug)]
pub struct StatusArg {}

// TODO: Supporting renames would be cool
impl StatusArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;
        let head_state = head_state()?;

        if head_state.detached {
            println!("HEAD detached at {}", head_state.hash);
        } else {
            println!("On branch {}", head_state.name);
        }

        let commit = Object::<Commit>::read_from_disk(&head_state.hash, ObjectType::Commit)?;

        let ignore = Ignore::build_from_disk()?;
        let index = Index::parse_from_disk()?;

        println!("\nChanges to be committed:");
        let changes = get_changes_to_be_committed(&commit.inner.tree, &index)?;
        for change in changes {
            println!("{}", change.green());
        }

        println!("\nChanges not staged for commit:");
        let mut files = BitDirWalker::new(&root, ignore)?
            .map(|entry| -> anyhow::Result<(String, fs::Metadata)> {
                let path = entry.path();
                Ok((
                    path.strip_prefix(&root)?.to_string_lossy().into(),
                    entry.metadata()?,
                ))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        // Compare the index to the file system to see what's modified or deleted
        // These are our unstaged changes
        for entry in index.entries.iter() {
            let relative = relative_path_string(&root.join(&entry.name), cwd()?);

            if !root.join(&entry.name).exists() {
                println!("{}", format!("        deleted:    {}", &relative).red());
                continue;
            }

            let Some((_, meta)) = files
                .iter()
                .position(|(path, _)| path == &entry.name)
                .map(|i| files.remove(i))
            else {
                continue;
            };

            if file_ts_changed(entry, meta) {
                let path = root.join(&entry.name);
                let hash = hash_object_from_disk(path, ObjectType::Blob, false)?;

                if hash != entry.sha {
                    println!("{}", format!("        modified:   {}", &relative).red());
                }
            }
        }

        // Anything that's left over in the files list is untracked
        println!("\nUntracked files:");
        for (path, _) in files {
            let relative = relative_path_string(&root.join(&path), cwd()?);
            println!("        {}", relative.red());
        }

        Ok(())
    }
}

fn file_ts_changed(entry: &IndexEntry, meta: fs::Metadata) -> bool {
    let ctime_equal =
        meta.ctime() == i64::from(entry.ctime.s) && meta.ctime_nsec() == i64::from(entry.ctime.ns);
    let mtime_equal =
        meta.mtime() == i64::from(entry.mtime.s) && meta.mtime_nsec() == i64::from(entry.mtime.ns);
    let files_different = !ctime_equal || !mtime_equal;
    files_different
}

pub fn get_changes_to_be_committed(tree_hash: &str, index: &Index) -> anyhow::Result<Vec<String>> {
    let root = repo_root()?;
    let flattened = flatten_tree(tree_hash, "")?;

    let cwd = cwd()?;
    Ok(index
        .entries
        .iter()
        .filter_map(|entry| {
            let relative = relative_path_string(&root.join(&entry.name), &cwd);

            let result = match flattened.get(&entry.name) {
                None => {
                    format!("        new file:   {}", &relative)
                }
                Some(tree_hash) if *tree_hash != hex::encode(entry.sha) => {
                    format!("        modified:   {}", &relative)
                }
                _ => return None,
            };
            Some(result)
        })
        .collect())
}

fn flatten_tree(tree_hash: &str, prefix_dir: &str) -> anyhow::Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    let tree = Object::<Tree>::read_from_disk(tree_hash, ObjectType::Tree)?;

    for entry in tree.inner.entries {
        let prefixed_path = format!("{prefix_dir}{}", entry.path);

        if entry.get_type()? == "blob" {
            map.insert(prefixed_path, entry.hash);
        } else if entry.get_type()? == "tree" {
            let dir_path = format!("{}/", prefixed_path);
            map.extend(flatten_tree(&entry.hash, &dir_path)?);
        }
    }

    Ok(map)
}
