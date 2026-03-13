use std::{collections::HashMap, env, fs, os::unix::fs::MetadataExt};

use clap::Args;
use colored::Colorize;

use crate::{
    commands::hash_object::hash_object_from_disk,
    objects::{
        Commit, Ignore, Index, Object,
        ObjectType::{self},
        Tree,
    },
    utils::{BitDirWalker, find_hash, relative_path_string, repo_root},
};

#[derive(Args, Debug)]
pub struct StatusArg {}

impl StatusArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;
        let head_content = fs::read_to_string(root.join(".bit/HEAD"))?;
        let (head, attached) = head_content
            .trim()
            .strip_prefix("ref: refs/heads/")
            .map_or((head_content.as_ref(), false), |s| (s, true));

        if attached {
            println!("On branch {head}");
        } else {
            println!("HEAD detached at {head}");
        }

        // TODO: Not sure this is correct? We should probably use the refs/heads/ prefix we stripped
        let hash = find_hash(head)?;
        let commit = Object::<Commit>::read_from_disk(&hash, ObjectType::Commit)?;

        let flattened = flatten_tree(&commit.inner.tree, "")?;

        let ignore = Ignore::build_from_disk()?;
        let index = Index::parse_from_disk()?;
        let cwd = env::current_dir()?;

        println!("\nChanges to be committed:");
        // Compare HEAD hashes to the index to see what's modified
        for entry in index.entries.iter() {
            let relative = relative_path_string(&root.join(&entry.name), &cwd);

            if let Some(tree_hash) = flattened.get(&entry.name) {
                if *tree_hash != hex::encode(entry.sha) {
                    println!("{}", format!("        modified:   {}", &relative).green());
                }
            } else {
                println!("{}", format!("        new file:   {}", &relative).green());
            }
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

        for entry in index.entries.iter() {
            let relative = relative_path_string(&root.join(&entry.name), &cwd);

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

            let ctime_equal = meta.ctime() == i64::from(entry.ctime.s)
                && meta.ctime_nsec() == i64::from(entry.ctime.ns);
            let mtime_equal = meta.mtime() == i64::from(entry.mtime.s)
                && meta.mtime_nsec() == i64::from(entry.mtime.ns);

            if !ctime_equal || !mtime_equal {
                let path = root.join(&entry.name);
                let hash = hash_object_from_disk(path, ObjectType::Blob, false)?;

                if hash != entry.sha {
                    println!("{}", format!("        modified:   {}", &relative).red());
                }
            }
        }

        println!("\nUntracked files:");
        for (path, _) in files {
            let relative = relative_path_string(&root.join(&path), &cwd);
            println!("        {}", relative.red());
        }

        Ok(())
    }
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
