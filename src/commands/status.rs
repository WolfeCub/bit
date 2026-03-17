use std::{fs, os::unix::fs::MetadataExt};

use clap::Args;
use colored::Colorize;

use crate::{
    commands::hash_object::hash_object_from_disk,
    objects::{
        Commit, Ignore, Index, IndexEntry, Object,
        ObjectType::{self},
        flatten_tree_from_disk,
    },
    utils::{
        bit_dir_walker::BitDirWalker,
        head::HeadState,
        path::relative_path_string,
        repo::{cwd, repo_root},
    },
};

/// Shows the current branch, staged changes, unstaged changes and untracked files
#[derive(Args, Debug)]
pub struct StatusArg {}

// TODO: Supporting renames would be cool
impl StatusArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;
        let head_state = HeadState::read_from_disk()?;

        let head_hash = match head_state {
            HeadState::Detached { hash, .. } => {
                println!("HEAD detached at {}", hash);
                Some(hash)
            }
            HeadState::Attached { branch, hash } => {
                println!("On branch {}", branch);
                Some(hash)
            }
            HeadState::Unborn { branch } => {
                println!("On branch {}\n\nNo commits yet", branch);
                None
            }
        };

        let ignore = Ignore::build_from_disk()?;
        let index = Index::parse_from_disk()?;

        let head_commit = head_hash
            .map(|hh| -> anyhow::Result<String> {
                Ok(Object::<Commit>::read_from_disk(&hh, ObjectType::Commit)?
                    .inner
                    .tree)
            })
            .transpose()?;
        let changes = get_changes_to_be_committed_text(head_commit.as_deref(), &index)?;
        if changes.len() > 0 {
            println!("\nChanges to be committed:");
            for change in changes {
                println!("{}", change.green());
            }
        }

        let mut files = BitDirWalker::new_with_ignore(&root, &ignore)?
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
        let mut unstaged_changes = Vec::<String>::with_capacity(index.entries.len());
        for entry in index.entries.iter() {
            let relative = relative_path_string(&root.join(&entry.name), cwd()?)?;

            if !root.join(&entry.name).exists() {
                unstaged_changes.push(
                    format!("        deleted:    {}", &relative)
                        .red()
                        .to_string(),
                );
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
                    unstaged_changes.push(
                        format!("        modified:   {}", &relative)
                            .red()
                            .to_string(),
                    );
                }
            }
        }

        if unstaged_changes.len() > 0 {
            println!(
                "\nChanges not staged for commit:\n{}",
                unstaged_changes.join("\n")
            );
        }

        if files.len() > 0 {
            println!("\nUntracked files:");
        }
        // Anything that's left over in the files list is untracked
        for (path, _) in files {
            let relative = relative_path_string(&root.join(&path), cwd()?)?;
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

#[derive(Debug)]
pub struct StagedChange<'a> {
    pub new_file: bool,
    pub entry: &'a IndexEntry,
}

pub fn get_changes_to_be_committed<'a>(
    tree_hash: Option<&str>,
    index: &'a Index,
) -> anyhow::Result<Vec<StagedChange<'a>>> {
    let flattened = tree_hash.map(flatten_tree_from_disk).transpose()?;

    Ok(index
        .entries
        .iter()
        .filter_map(|entry| {
            // If we have a parent hash compute modified vs new files, otherwise just show all
            // the files in the index as new (can't have modifications without a parent commit)
            let new_file = if let Some(flattened) = flattened.as_ref() {
                let new_file = match flattened.get(&entry.name) {
                    None => true,
                    Some(tree_entry) if *tree_entry.hash != hex::encode(entry.sha) => false,
                    _ => return None,
                };
                new_file
            } else {
                true
            };
            Some(StagedChange { new_file, entry })
        })
        .collect())
}

pub fn get_changes_to_be_committed_text(
    tree_hash: Option<&str>,
    index: &Index,
) -> anyhow::Result<Vec<String>> {
    let root = repo_root()?;
    let cwd = cwd()?;

    let changes = get_changes_to_be_committed(tree_hash, index)?;

    changes
        .into_iter()
        .map(|change| {
            let relative = relative_path_string(&root.join(&change.entry.name), &cwd)?;

            let result = if change.new_file {
                format!("        new file:   {}", &relative)
            } else {
                format!("        modified:   {}", &relative)
            };
            Ok(result)
        })
        .collect()
}
