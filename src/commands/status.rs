use std::{collections::HashMap, env, fs, os::unix::fs::MetadataExt};

use clap::Args;

use crate::{
    commands::hash_object::{hash_object_from_disk, hash_object_hex_from_disk},
    objects::{
        Commit, Ignore, Index, Object,
        ObjectType::{self},
        Tree,
    },
    utils::{BitDirWalker, find_hash, make_root_relative, repo_root},
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

        println!("Changes to be committed:");

        for entry in index.entries.iter() {
            if let Some(tree_hash) = flattened.get(&entry.name) {
                if *tree_hash != hash {
                    println!("  modified: {}", &entry.name);
                }
            } else {
                println!("  added: {}", &entry.name);
            }
        }


        let files = BitDirWalker::new(&root, ignore)?.collect::<Vec<_>>();


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
