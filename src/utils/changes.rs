use std::{collections::HashMap, fs, os::unix::fs::MetadataExt};

use crate::{
    commands::hash_object::hash_object_from_disk,
    objects::{Ignore, Index, IndexEntry, ObjectType, flatten_tree_from_disk},
    utils::{bit_dir_walker::BitDirWalker, repo::repo_root},
};

#[derive(Debug)]
pub enum StagedChange<'a> {
    Added(&'a IndexEntry),
    Modified {
        entry: &'a IndexEntry,
        head_hash: String,
    },
    Deleted {
        name: String,
        head_hash: String,
    },
}

impl StagedChange<'_> {
    pub fn name(&self) -> &str {
        match self {
            StagedChange::Added(entry) => &entry.name,
            StagedChange::Modified { entry, .. } => &entry.name,
            StagedChange::Deleted { name, .. } => name,
        }
    }

    pub fn head_hash(&self) -> Option<&str> {
        match self {
            StagedChange::Added(_) => None,
            StagedChange::Modified { head_hash, .. } => Some(head_hash),
            StagedChange::Deleted { head_hash, .. } => Some(head_hash),
        }
    }

    pub fn entry(&self) -> Option<&IndexEntry> {
        match self {
            StagedChange::Added(entry) => Some(entry),
            StagedChange::Modified { entry, .. } => Some(entry),
            StagedChange::Deleted { .. } => None,
        }
    }
}

pub fn get_changes_to_be_committed<'a>(
    tree_hash: Option<&str>,
    index: &'a Index,
) -> anyhow::Result<Vec<StagedChange<'a>>> {
    // If we don't have a parent tree hash then we consider everything in the index as new
    let Some(flattened) = tree_hash.map(flatten_tree_from_disk).transpose()? else {
        return Ok(index.entries.iter().map(StagedChange::Added).collect());
    };

    let index_map = index
        .entries
        .iter()
        .map(|entry| (entry.name.clone(), entry))
        .collect::<HashMap<_, _>>();

    let all_names = index
        .entries
        .iter()
        .map(|e| &e.name)
        .chain(flattened.keys())
        .collect::<Vec<_>>();

    Ok(all_names
        .into_iter()
        .filter_map(|name| match (index_map.get(name), flattened.get(name)) {
            // In index but not in HEAD — newly staged file
            (Some(entry), None) => Some(StagedChange::Added(entry)),

            // In both, but hashes differ — file was modified and staged
            (Some(entry), Some(tree_entry)) if *tree_entry.hash != hex::encode(entry.sha) => {
                Some(StagedChange::Modified {
                    entry,
                    head_hash: tree_entry.hash.clone(),
                })
            }

            // In HEAD but not in index — file was staged for deletion
            (None, Some(tree_entry)) => Some(StagedChange::Deleted {
                name: name.clone(),
                head_hash: tree_entry.hash.clone(),
            }),

            // In both with matching hashes — unchanged, or (None, None) which is unreachable
            _ => None,
        })
        .collect())
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnstagedChangeKind {
    Modified,
    Deleted,
}

#[derive(Debug)]
pub struct UnstagedChange {
    pub kind: UnstagedChangeKind,
    pub name: String,
    pub head_hash: String,
}

pub fn get_unstaged_changes(
    index: &Index,
    ignore: &Ignore,
) -> anyhow::Result<(Vec<UnstagedChange>, Vec<String>)> {
    let root = repo_root()?;

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
    let mut unstaged_changes = Vec::with_capacity(index.entries.len());
    for entry in index.entries.iter() {
        if !root.join(&entry.name).exists() {
            unstaged_changes.push(UnstagedChange {
                kind: UnstagedChangeKind::Deleted,
                name: entry.name.clone(),
                head_hash: hex::encode(entry.sha),
            });
            continue;
        }

        let Some((_, meta)) = files
            .iter()
            .position(|(path, _)| path == &entry.name)
            .map(|i| files.remove(i))
        else {
            continue;
        };

        if file_changed_heuristic(entry, meta) {
            let path = root.join(&entry.name);
            let hash = hash_object_from_disk(path, ObjectType::Blob, false)?;

            if hash != entry.sha {
                unstaged_changes.push(UnstagedChange {
                    kind: UnstagedChangeKind::Modified,
                    name: entry.name.clone(),
                    head_hash: hex::encode(entry.sha),
                });
            }
        }
    }

    // Anything left in files we don't know about and it's untracked
    let untracked = files.into_iter().map(|(path, _)| path).collect();
    Ok((unstaged_changes, untracked))
}

pub fn file_changed_heuristic(entry: &IndexEntry, meta: fs::Metadata) -> bool {
    let ctime_equal =
        meta.ctime() == i64::from(entry.ctime.s) && meta.ctime_nsec() == i64::from(entry.ctime.ns);
    let mtime_equal =
        meta.mtime() == i64::from(entry.mtime.s) && meta.mtime_nsec() == i64::from(entry.mtime.ns);
    let size_equal = meta.size() == entry.size as u64;
    let ino_equal = meta.ino() == entry.ino as u64;
    let uid_equal = meta.uid() == entry.uid as u32;
    let gid_equal = meta.gid() == entry.gid as u32;
    let mode_equal = meta.mode() == entry.mode as u32;

    !ctime_equal
        || !mtime_equal
        || !size_equal
        || !ino_equal
        || !uid_equal
        || !gid_equal
        || !mode_equal
}
