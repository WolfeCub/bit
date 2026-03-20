use std::{fs, os::unix::fs::MetadataExt};

use crate::{
    commands::hash_object::hash_object_from_disk,
    objects::{Ignore, Index, IndexEntry, ObjectType, flatten_tree_from_disk},
    utils::{bit_dir_walker::BitDirWalker, repo::repo_root},
};

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
